use std::collections::HashMap;
use deadpool_postgres::Pool;
use edcas_common::api::{BodyResponse, MaterialResponse, ParentResponse, RingResponse, SystemFactionInfo, SystemResponse};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, State};

#[get("/api/v1/systems/<system_address>")]
pub async fn get_system(
    pool: &State<Pool>,
    system_address: i64,
) -> Result<Json<SystemResponse>, Status> {
    let client = pool.get().await.map_err(|_| Status::ServiceUnavailable)?;
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
        .map_err(|_| Status::InternalServerError)?;

    match row {
        None => Err(Status::NotFound),
        Some(r) => {
            let addr: i64 = r.get("system_address");
            let factions = fetch_system_factions(&client, addr).await?;
            Ok(Json(SystemResponse {
                system_address: addr,
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
                factions,
            }))
        }
    }
}

#[get("/api/v1/systems/<system_address>/bodies")]
pub async fn get_system_bodies(
    pool: &State<Pool>,
    system_address: i64,
) -> Result<Json<Vec<BodyResponse>>, Status> {
    let client = pool.get().await.map_err(|_| Status::ServiceUnavailable)?;

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
        .map_err(|_| Status::InternalServerError)?;

    let star_rows = client
        .query(
            "SELECT s.id, s.system_address, s.name,
                    NULL::real as mass_em, s.radius, false as landable,
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
        .map_err(|_| Status::InternalServerError)?;

    // Fetch rings, materials, and parents for the whole system in 3 bulk queries
    // instead of 3 per body, avoiding the N+1 problem.
    let ring_rows = client
        .query(
            "SELECT r.body_id, r.name, rc.value as ring_class, r.inner_rad, r.outer_rad, r.mass_mt
             FROM ring r
             LEFT JOIN ring_class rc ON r.ring_class = rc.id
             WHERE r.system_address = $1",
            &[&system_address],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    let material_rows = client
        .query(
            "SELECT pm.body_id, mt.value as name, pm.percent
             FROM planet_material pm
             LEFT JOIN material_type mt ON pm.material_type = mt.id
             WHERE pm.system_address = $1
             ORDER BY pm.percent DESC",
            &[&system_address],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    let parent_rows = client
        .query(
            "SELECT type, parent_id, body_id FROM parents WHERE system_address = $1",
            &[&system_address],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    let mut rings_map: HashMap<i32, Vec<RingResponse>> = HashMap::new();
    for row in &ring_rows {
        let body_id: i32 = row.get("body_id");
        rings_map.entry(body_id).or_default().push(RingResponse {
            name: row.get("name"),
            ring_class: row.get::<_, Option<String>>("ring_class").unwrap_or_default(),
            inner_rad: row.get::<_, f32>("inner_rad") as f64,
            outer_rad: row.get::<_, f32>("outer_rad") as f64,
            mass_mt: row.get::<_, f32>("mass_mt") as f64,
        });
    }

    let mut materials_map: HashMap<i32, Vec<MaterialResponse>> = HashMap::new();
    for row in &material_rows {
        let body_id: i32 = row.get("body_id");
        materials_map.entry(body_id).or_default().push(MaterialResponse {
            name: row.get::<_, Option<String>>("name").unwrap_or_default(),
            percent: row.get::<_, f32>("percent") as f64,
        });
    }

    let mut parents_map: HashMap<i32, Vec<ParentResponse>> = HashMap::new();
    for row in &parent_rows {
        let body_id: i32 = row.get("body_id");
        parents_map.entry(body_id).or_default().push(ParentResponse {
            parent_type: row.get("type"),
            parent_id: row.get("parent_id"),
        });
    }

    let mut bodies = Vec::new();

    for row in body_rows.iter().chain(star_rows.iter()) {
        let body_id: i32 = row.get("id");
        let is_star: bool = row.get("is_star");

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
            rings: rings_map.remove(&body_id).unwrap_or_default(),
            materials: if is_star { vec![] } else { materials_map.remove(&body_id).unwrap_or_default() },
            parents: parents_map.remove(&body_id).unwrap_or_default(),
        });
    }

    bodies.sort_by_key(|b| b.distance_from_arrival_ls.map(|d| (d * 100.0) as i64).unwrap_or(0));
    Ok(Json(bodies))
}

async fn fetch_system_factions(
    client: &tokio_postgres::Client,
    system_address: i64,
) -> Result<Vec<SystemFactionInfo>, Status> {
    let rows = client
        .query(
            "SELECT f.name, f.influence, g.value as government, a.value as allegiance, h.value as happiness
             FROM factions f
             LEFT JOIN government g ON f.government = g.id
             LEFT JOIN allegiance a ON f.allegiance = a.id
             LEFT JOIN happiness h ON f.happiness = h.id
             WHERE f.system_address = $1
             ORDER BY f.influence DESC NULLS LAST",
            &[&system_address],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    let mut result = Vec::new();
    for row in &rows {
        let name: String = row.get("name");
        let state_rows = client
            .query(
                "SELECT fsn.value as state, fs.status
                 FROM faction_states fs
                 LEFT JOIN faction_state_name fsn ON fs.state = fsn.id
                 WHERE fs.faction_name = $1 AND fs.system_address = $2",
                &[&name, &system_address],
            )
            .await
            .map_err(|_| Status::InternalServerError)?;

        let mut active = Vec::new();
        let mut pending = Vec::new();
        let mut recovering = Vec::new();
        for sr in &state_rows {
            let state: String = sr.get::<_, Option<String>>("state").unwrap_or_default();
            let status: String = sr.get("status");
            match status.as_str() {
                "Active" => active.push(state),
                "Pending" => pending.push(state),
                "Recovering" => recovering.push(state),
                _ => active.push(state),
            }
        }

        result.push(SystemFactionInfo {
            influence: row.get::<_, Option<f32>>("influence").unwrap_or(0.0),
            government: row.get("government"),
            allegiance: row.get("allegiance"),
            happiness: row.get("happiness"),
            active_states: active,
            pending_states: pending,
            recovering_states: recovering,
            name,
        });
    }

    Ok(result)
}

