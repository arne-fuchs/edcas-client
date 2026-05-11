use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use deadpool_postgres::Pool;
use edcas_common::api::{LandingPadsResponse, StationEconomyResponse, StationQuery, StationResponse};

pub async fn search_stations(
    State(pool): State<Pool>,
    Query(params): Query<StationQuery>,
) -> Result<Json<Vec<StationResponse>>, StatusCode> {
    query_stations(&pool, &params, false).await
}

pub(crate) async fn query_stations(
    pool: &Pool,
    params: &StationQuery,
    carriers_only: bool,
) -> Result<Json<Vec<StationResponse>>, StatusCode> {
    let client = pool.get().await.map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let limit = params.limit.unwrap_or(50).min(200);

    let name_pattern = params.name.as_ref().map(|n| format!("%{}%", n.to_lowercase()));
    let system_pattern = params.system_name.as_ref().map(|s| format!("%{}%", s.to_lowercase()));

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
            &[&name_pattern, &system_pattern, &params.market_id, &carriers_only],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut results = Vec::new();
    for row in &rows {
        let market_id: i64 = row.get("market_id");
        let system_address: i64 = row.get("system_address");

        let economies = fetch_economies(&client, market_id).await?;
        let services = fetch_services(&client, market_id).await?;
        let landing_pads = fetch_landing_pads(&client, market_id).await?;

        results.push(StationResponse {
            market_id,
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
) -> Result<Vec<StationEconomyResponse>, StatusCode> {
    let rows = client
        .query(
            "SELECT et.value as name, se.proportion
             FROM station_economies se
             LEFT JOIN economy_type et ON se.economy_type = et.id
             WHERE se.market_id = $1",
            &[&market_id],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

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
) -> Result<Vec<String>, StatusCode> {
    let rows = client
        .query(
            "SELECT sst.value as service
             FROM station_services ss
             LEFT JOIN station_services_types sst ON ss.id = sst.id
             WHERE ss.market_id = $1",
            &[&market_id],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(rows
        .iter()
        .filter_map(|r| r.get::<_, Option<String>>("service"))
        .collect())
}

async fn fetch_landing_pads(
    client: &tokio_postgres::Client,
    market_id: i64,
) -> Result<Option<LandingPadsResponse>, StatusCode> {
    let row = client
        .query_opt(
            "SELECT small, medium, large FROM station_landing_pads WHERE market_id = $1",
            &[&market_id],
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(row.map(|r| LandingPadsResponse {
        small: r.get("small"),
        medium: r.get("medium"),
        large: r.get("large"),
    }))
}
