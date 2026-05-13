use deadpool_postgres::Pool;
use edcas_common::api::TradeRouteResponse;
use rocket::{get, http::Status, serde::json::Json, State};
use tracing::error;

#[get("/api/v1/trade-routes?<system_address>&<max_distance>&<pad_size>&<min_profit>&<limit>")]
pub async fn search_trade_routes(
    pool: &State<Pool>,
    system_address: Option<i64>,
    max_distance: Option<f32>,
    pad_size: Option<String>,
    min_profit: Option<i32>,
    limit: Option<i64>,
) -> Result<Json<Vec<TradeRouteResponse>>, Status> {
    let addr = match system_address {
        Some(a) => a,
        None => return Ok(Json(vec![])),
    };

    let max_dist = max_distance.unwrap_or(200.0_f32);
    let min_profit_val = min_profit.unwrap_or(1000_i32);
    let limit_val = limit.unwrap_or(50_i64).min(200);

    let client = pool.get().await.map_err(|e| {
        error!("DB pool error: {e:#}");
        Status::ServiceUnavailable
    })?;

    let center_row = client
        .query_opt(
            "SELECT x, y, z FROM star_systems WHERE system_address = $1",
            &[&addr],
        )
        .await
        .map_err(|e| {
            error!("Center lookup failed: {e:#}");
            Status::InternalServerError
        })?;

    let (cx, cy, cz): (f32, f32, f32) = match center_row {
        Some(r) => (r.get(0), r.get(1), r.get(2)),
        None => return Ok(Json(vec![])),
    };

    let pad_clause = match pad_size.as_deref() {
        Some("L") => " AND fm.large > 0 AND tm.large > 0",
        Some("M") => " AND (fm.large > 0 OR fm.medium > 0) AND (tm.large > 0 OR tm.medium > 0)",
        _ => "",
    };

    let sql = format!(
        r#"
        WITH nearby AS (
            SELECT
                s.market_id,
                ss.x, ss.y, ss.z,
                ss.name   AS system_name,
                s.name    AS station_name,
                COALESCE(slp.large,  0) AS large,
                COALESCE(slp.medium, 0) AS medium
            FROM stations s
            JOIN star_systems ss ON s.system_address = ss.system_address
            LEFT JOIN station_landing_pads slp ON slp.market_id = s.market_id
            WHERE ABS(ss.x - $1::real) <= $4::real
              AND ABS(ss.y - $2::real) <= $4::real
              AND ABS(ss.z - $3::real) <= $4::real
        )
        SELECT
            fc.market_id              AS from_market_id,
            tc.market_id              AS to_market_id,
            fc.name                   AS commodity,
            fc.buy_price,
            tc.sell_price,
            (tc.sell_price - fc.buy_price) AS profit,
            fc.stock                  AS supply,
            tc.demand,
            SQRT(
                POWER((fm.x - tm.x)::double precision, 2) +
                POWER((fm.y - tm.y)::double precision, 2) +
                POWER((fm.z - tm.z)::double precision, 2)
            )::real                   AS distance_ly,
            fm.station_name           AS from_station_name,
            tm.station_name           AS to_station_name,
            fm.system_name            AS from_system_name,
            tm.system_name            AS to_system_name,
            CASE WHEN fm.large  > 0 THEN 'L'
                 WHEN fm.medium > 0 THEN 'M'
                 ELSE 'S' END         AS from_max_pad,
            CASE WHEN tm.large  > 0 THEN 'L'
                 WHEN tm.medium > 0 THEN 'M'
                 ELSE 'S' END         AS to_max_pad
        FROM nearby fm
        JOIN commodity_listening fc ON fc.market_id = fm.market_id
            AND fc.buy_price > 0 AND fc.stock > 0
        JOIN commodity_listening tc ON tc.name = fc.name
            AND tc.demand > 0 AND tc.sell_price > fc.buy_price
        JOIN nearby tm ON tm.market_id = tc.market_id
            AND tm.market_id != fm.market_id
        WHERE (tc.sell_price - fc.buy_price) >= $5
          AND SQRT(
                  POWER((fm.x - tm.x)::double precision, 2) +
                  POWER((fm.y - tm.y)::double precision, 2) +
                  POWER((fm.z - tm.z)::double precision, 2)
              ) <= $4::double precision
          {pad_clause}
        ORDER BY profit DESC
        LIMIT {limit_val}
        "#
    );

    let rows = client
        .query(&sql, &[&cx, &cy, &cz, &max_dist, &min_profit_val])
        .await
        .map_err(|e| {
            error!("Trade routes query failed: {e:#}");
            Status::InternalServerError
        })?;

    let results = rows
        .iter()
        .map(|r| TradeRouteResponse {
            from_market_id: r.get("from_market_id"),
            to_market_id: r.get("to_market_id"),
            commodity: r.get("commodity"),
            buy_price: r.get("buy_price"),
            sell_price: r.get("sell_price"),
            profit: r.get("profit"),
            supply: r.get("supply"),
            demand: r.get("demand"),
            distance_ly: r.get("distance_ly"),
            from_station_name: r.get("from_station_name"),
            to_station_name: r.get("to_station_name"),
            from_system_name: r.get("from_system_name"),
            to_system_name: r.get("to_system_name"),
            from_max_pad: r.get("from_max_pad"),
            to_max_pad: r.get("to_max_pad"),
        })
        .collect();

    Ok(Json(results))
}
