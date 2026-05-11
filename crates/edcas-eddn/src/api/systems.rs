use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use deadpool_postgres::Pool;
use edcas_common::api::{BodyResponse, MaterialResponse, ParentResponse, RingResponse, SystemResponse};

pub async fn get_system(
    State(pool): State<Pool>,
    Path(system_address): Path<i64>,
) -> Result<Json<SystemResponse>, StatusCode> {
    let client = pool.get().await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let row = client
        .query_opt(
            "SELECT ss.system_address, ss.name, ss.x, ss.y, ss.z,
                    a.value as allegiance, e.value as economy, e2.value as second_economy,
                    g.value as government, sec.value as security, ss.population, p.value as controlling_power
             FROM star_systems ss
             LEFT JOIN allegiance a ON ss.allegiance = a.id
             LEFT JOIN economy_type e ON ss.economy = e.id
             LEFT JOIN economy_type e2 ON ss.second_economy = e2.id
             LEFT JOIN government g ON ss.government = g.id
             LEFT JOIN security sec ON ss.security = sec.id
             LEFT JOIN power p ON ss.controlling_power = p.id
             WHERE ss.system_address = $1",
            &[&system_address],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        None => Err(StatusCode::NOT_FOUND),
        Some(r) => Ok(Json(SystemResponse {
            system_address: r.get("system_address"),
            name: r.get("name"),
            x: r.get::<_, f32>("x"),
            y: r.get::<_, f32>("y"),
            z: r.get::<_, f32>("z"),
            allegiance: r.get("allegiance"),
            economy: r.get("economy"),
            second_economy: r.get("second_economy"),
            government: r.get("government"),
            security: r.get("security"),
            population: r.get("population"),
            controlling_power: r.get("controlling_power"),
        })),
    }
}

pub async fn get_system_bodies(
    State(pool): State<Pool>,
    Path(system_address): Path<i64>,
) -> Result<Json<Vec<BodyResponse>>, StatusCode> {
    let client = pool.get().await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;

    let body_rows = client
        .query(
            "SELECT b.id, b.system_address, b.name, b.mass_em, b.radius, b.landable,
                    b.axial_tilt, b.tidal_lock, b.mapped, b.mean_anomaly,
                    b.eccentricity, b.ascending_node, b.orbital_period,
                    b.semi_major_axis, b.rotation_period, b.surface_gravity,
                    b.surface_pressure, b.orbital_inclination, b.surface_temperature,
                    b.distance,
                    pc.value as planet_class, v.value as volcanism,
                    a.value as atmosphere, at.value as atmosphere_type,
                    ts.value as terraform_state,
                    false as is_star
             FROM body b
             LEFT JOIN planet_class pc ON b.planet_class = pc.id
             LEFT JOIN volcanism v ON b.volcanism = v.id
             LEFT JOIN atmosphere a ON b.atmosphere = a.id
             LEFT JOIN atmosphere_type at ON b.atmosphere_type = at.id
             LEFT JOIN terraform_state ts ON b.terraform_state = ts.id
             WHERE b.system_address = $1",
            &[&system_address],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let star_rows = client
        .query(
            "SELECT s.id, s.system_address, s.name, NULL::real as mass_em, s.radius, false as landable,
                    NULL::real as axial_tilt, false as tidal_lock, false as mapped, NULL::real as mean_anomaly,
                    NULL::real as eccentricity, NULL::real as ascending_node, NULL::real as orbital_period,
                    NULL::real as semi_major_axis, NULL::real as rotation_period, NULL::real as surface_gravity,
                    NULL::real as surface_pressure, NULL::real as orbital_inclination, s.surface_temperature,
                    NULL::real as distance,
                    st.value as planet_class, NULL::text as volcanism,
                    NULL::text as atmosphere, NULL::text as atmosphere_type, NULL::text as terraform_state,
                    true as is_star
             FROM star s
             LEFT JOIN star_type st ON s.star_type = st.id
             WHERE s.system_address = $1",
            &[&system_address],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut bodies = Vec::new();

    for row in body_rows.iter().chain(star_rows.iter()) {
        let body_id: i32 = row.get("id");
        let is_star: bool = row.get("is_star");

        let rings = fetch_rings(&client, body_id, system_address).await?;
        let materials = if !is_star {
            fetch_materials(&client, body_id, system_address).await?
        } else {
            vec![]
        };
        let parents = fetch_parents(&client, body_id, system_address).await?;

        bodies.push(BodyResponse {
            id: body_id,
            system_address: row.get("system_address"),
            name: row.get("name"),
            is_star,
            body_class: row.get("planet_class"),
            distance_from_arrival_ls: row.get("distance"),
            radius: row.get("radius"),
            mass_em: row.get("mass_em"),
            surface_temperature: row.get("surface_temperature"),
            surface_gravity: row.get("surface_gravity"),
            landable: row.get("landable"),
            atmosphere: row.get("atmosphere"),
            volcanism: row.get("volcanism"),
            terraform_state: row.get("terraform_state"),
            tidal_lock: row.get("tidal_lock"),
            was_discovered: false,
            was_mapped: row.try_get("mapped").unwrap_or(false),
            estimated_value: None,
            rings,
            materials,
            parents,
        });
    }

    bodies.sort_by_key(|b| b.distance_from_arrival_ls.map(|d| (d * 100.0) as i64).unwrap_or(0));
    Ok(Json(bodies))
}

async fn fetch_rings(
    client: &tokio_postgres::Client,
    body_id: i32,
    system_address: i64,
) -> Result<Vec<RingResponse>, StatusCode> {
    let rows = client
        .query(
            "SELECT r.name, rc.value as ring_class, r.inner_rad, r.outer_rad, r.mass_mt
             FROM ring r
             LEFT JOIN ring_class rc ON r.ring_class = rc.id
             WHERE r.body_id = $1 AND r.system_address = $2",
            &[&body_id, &system_address],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(rows
        .iter()
        .map(|r| RingResponse {
            name: r.get("name"),
            ring_class: r.get::<_, Option<String>>("ring_class").unwrap_or_default(),
            inner_rad: r.get::<_, f32>("inner_rad") as f64,
            outer_rad: r.get::<_, f32>("outer_rad") as f64,
            mass_mt: r.get::<_, f32>("mass_mt") as f64,
        })
        .collect())
}

async fn fetch_materials(
    client: &tokio_postgres::Client,
    body_id: i32,
    system_address: i64,
) -> Result<Vec<MaterialResponse>, StatusCode> {
    let rows = client
        .query(
            "SELECT mt.value as name, pm.percent
             FROM planet_material pm
             LEFT JOIN material_type mt ON pm.material_type = mt.id
             WHERE pm.body_id = $1 AND pm.system_address = $2
             ORDER BY pm.percent DESC",
            &[&body_id, &system_address],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(rows
        .iter()
        .map(|r| MaterialResponse {
            name: r.get::<_, Option<String>>("name").unwrap_or_default(),
            percent: r.get::<_, f32>("percent") as f64,
        })
        .collect())
}

async fn fetch_parents(
    client: &tokio_postgres::Client,
    body_id: i32,
    system_address: i64,
) -> Result<Vec<ParentResponse>, StatusCode> {
    let rows = client
        .query(
            "SELECT type, parent_id FROM parents
             WHERE body_id = $1 AND system_address = $2",
            &[&body_id, &system_address],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(rows
        .iter()
        .map(|r| ParentResponse {
            parent_type: r.get("type"),
            parent_id: r.get("parent_id"),
        })
        .collect())
}
