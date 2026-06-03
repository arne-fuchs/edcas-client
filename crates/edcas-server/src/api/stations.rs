use chrono::Utc;
use deadpool_postgres::Pool;
use edcas_common::api::{
    CommodityPricePoint, CommodityResponse, LandingPadsResponse, ModuleResponse, ShipResponse,
    StationEconomyResponse, StationResponse,
};
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
                        s.name, s.carrier_name, s.station_type,
                        s.faction_name, s.government, s.economy,
                        s.event_timestamp,
                        (SELECT MAX(cl.event_timestamp) FROM commodity_listening cl
                         WHERE cl.market_id = s.market_id) AS market_updated_at
                 FROM stations s
                 LEFT JOIN star_systems ss ON s.system_address = ss.system_address
                 WHERE ($1::text IS NULL OR LOWER(s.name) LIKE $1)
                   AND ($2::text IS NULL OR LOWER(ss.name) LIKE $2)
                   AND ($3::bigint IS NULL OR s.market_id = $3)
                   AND (NOT $4 OR s.station_type = 'FleetCarrier')
                   AND ($4 OR s.station_type IS NULL OR s.station_type != 'FleetCarrier')
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
        let commodities = fetch_commodities(&client, mid).await?;
        let modules = fetch_modules(&client, mid).await?;
        let ships = fetch_ships(&client, mid).await?;

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
            carrier_name: row.get("carrier_name"),
            updated_at: row.get("event_timestamp"),
            market_updated_at: row.get("market_updated_at"),
            commodities,
            modules,
            ships,
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
            "SELECT economy_type as name, proportion
             FROM station_economies
             WHERE market_id = $1",
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
            "SELECT service_type FROM station_services WHERE market_id = $1",
            &[&market_id],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(rows
        .iter()
        .filter_map(|r| r.get::<_, Option<String>>("service_type"))
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

async fn fetch_commodities(
    client: &tokio_postgres::Client,
    market_id: i64,
) -> Result<Vec<CommodityResponse>, Status> {
    let rows = client
        .query(
            "SELECT name, mean_price, buy_price, stock, sell_price, demand
             FROM commodity_listening
             WHERE market_id = $1
             ORDER BY name",
            &[&market_id],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(rows
        .iter()
        .map(|r| CommodityResponse {
            name: r.get("name"),
            mean_price: r.get("mean_price"),
            buy_price: r.get("buy_price"),
            stock: r.get("stock"),
            sell_price: r.get("sell_price"),
            demand: r.get("demand"),
        })
        .collect())
}

async fn fetch_modules(
    client: &tokio_postgres::Client,
    market_id: i64,
) -> Result<Vec<ModuleResponse>, Status> {
    let rows = client
        .query(
            "SELECT id, name, category, cost, ship
             FROM modul_listening
             WHERE market_id = $1
             ORDER BY category, name",
            &[&market_id],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(rows
        .iter()
        .map(|r| ModuleResponse {
            id: r.get("id"),
            name: r.get("name"),
            category: r.get("category"),
            cost: r.get("cost"),
            ship: r.get("ship"),
        })
        .collect())
}

async fn fetch_ships(
    client: &tokio_postgres::Client,
    market_id: i64,
) -> Result<Vec<ShipResponse>, Status> {
    let rows = client
        .query(
            "SELECT id, name, basevalue
             FROM ship_listening
             WHERE market_id = $1
             ORDER BY name",
            &[&market_id],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(rows
        .iter()
        .map(|r| ShipResponse {
            id: r.get("id"),
            name: r.get("name"),
            basevalue: r.get("basevalue"),
        })
        .collect())
}

#[get("/api/v1/commodity-price-history?<market_id>&<commodity>&<days>")]
pub async fn commodity_price_history(
    pool: &State<Pool>,
    market_id: i64,
    commodity: String,
    days: Option<i32>,
) -> Result<Json<Vec<CommodityPricePoint>>, Status> {
    let client = pool.get().await.map_err(|_| Status::ServiceUnavailable)?;
    let days = days.unwrap_or(30).clamp(1, 90) as i64;
    let cutoff = Utc::now() - chrono::Duration::days(days);
    let rows = client
        .query(
            "SELECT buy_price, sell_price, stock, demand, event_timestamp
             FROM commodity_price_history
             WHERE market_id = $1 AND name = $2 AND event_timestamp >= $3
             ORDER BY event_timestamp ASC",
            &[&market_id, &commodity, &cutoff],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;
    let points = rows
        .iter()
        .map(|r| CommodityPricePoint {
            buy_price: r.get("buy_price"),
            sell_price: r.get("sell_price"),
            stock: r.get("stock"),
            demand: r.get("demand"),
            timestamp: r.get("event_timestamp"),
        })
        .collect();
    Ok(Json(points))
}
