use deadpool_postgres::Pool;
use edcas_common::api::{LandingPadsResponse, StationEconomyResponse, StationResponse};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, State};

#[get("/api/v1/stations?<name>&<system_name>&<market_id>&<limit>")]
pub async fn search_stations(
    pool: &State<Pool>,
    name: Option<String>,
    system_name: Option<String>,
    market_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Json<Vec<StationResponse>>, Status> {
    query_stations(pool, name, system_name, market_id, limit, false).await
}

pub(crate) async fn query_stations(
    pool: &State<Pool>,
    name: Option<String>,
    system_name: Option<String>,
    market_id: Option<i64>,
    limit: Option<i64>,
    carriers_only: bool,
) -> Result<Json<Vec<StationResponse>>, Status> {
    let client = pool.get().await.map_err(|_| Status::ServiceUnavailable)?;
    let limit = limit.unwrap_or(50).min(200);

    let name_pattern = name.as_ref().map(|n| format!("%{}%", n.to_lowercase()));
    let system_pattern = system_name.as_ref().map(|s| format!("%{}%", s.to_lowercase()));

    let rows = client
        .query(
            &format!(
                "SELECT s.market_id, s.system_address, ss.name as system_name,
                        s.name, st.value as station_type,
                        s.faction_name, g.value as government, e.value as economy
                 FROM stations s
                 LEFT JOIN star_systems ss ON s.system_address = ss.system_address
                 LEFT JOIN station_type st ON s.type = st.id
                 LEFT JOIN government g ON s.government = g.id
                 LEFT JOIN economy_type e ON s.economy = e.id
                 WHERE ($1::text IS NULL OR LOWER(s.name) LIKE $1)
                   AND ($2::text IS NULL OR LOWER(ss.name) LIKE $2)
                   AND ($3::bigint IS NULL OR s.market_id = $3)
                   AND (NOT $4 OR st.value = 'FleetCarrier')
                   AND ($4 OR st.value IS NULL OR st.value != 'FleetCarrier')
                 ORDER BY s.name
                 LIMIT {limit}"
            ),
            &[&name_pattern, &system_pattern, &market_id, &carriers_only],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    let mut results = Vec::new();
    for row in &rows {
        let mid: i64 = row.get("market_id");
        let system_address: i64 = row.get("system_address");

        let economies = fetch_economies(&client, mid).await?;
        let services = fetch_services(&client, mid).await?;
        let landing_pads = fetch_landing_pads(&client, mid).await?;

        results.push(StationResponse {
            market_id: mid,
            system_address,
            system_name: row.get::<_, Option<String>>("system_name").unwrap_or_default(),
            name: row.get("name"),
            station_type: row.get("station_type"),
            faction_name: row.get("faction_name"),
            government: row.get("government"),
            economy: row.get("economy"),
            economies,
            services,
            landing_pads,
            dist_from_star_ls: None,
        });
    }

    Ok(Json(results))
}

async fn fetch_economies(
    client: &tokio_postgres::Client,
    market_id: i64,
) -> Result<Vec<StationEconomyResponse>, Status> {
    let rows = client
        .query(
            "SELECT et.value as name, se.proportion
             FROM station_economies se
             LEFT JOIN economy_type et ON se.economy_type = et.id
             WHERE se.market_id = $1",
            &[&market_id],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(rows
        .iter()
        .map(|r| StationEconomyResponse {
            name: r.get::<_, Option<String>>("name").unwrap_or_default(),
            proportion: r.get("proportion"),
        })
        .collect())
}

async fn fetch_services(
    client: &tokio_postgres::Client,
    market_id: i64,
) -> Result<Vec<String>, Status> {
    let rows = client
        .query(
            "SELECT sst.value as service
             FROM station_services ss
             LEFT JOIN station_services_types sst ON ss.id = sst.id
             WHERE ss.market_id = $1",
            &[&market_id],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(rows
        .iter()
        .filter_map(|r| r.get::<_, Option<String>>("service"))
        .collect())
}

async fn fetch_landing_pads(
    client: &tokio_postgres::Client,
    market_id: i64,
) -> Result<Option<LandingPadsResponse>, Status> {
    let row = client
        .query_opt(
            "SELECT small, medium, large FROM station_landing_pads WHERE market_id = $1",
            &[&market_id],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(row.map(|r| LandingPadsResponse {
        small: r.get("small"),
        medium: r.get("medium"),
        large: r.get("large"),
    }))
}
