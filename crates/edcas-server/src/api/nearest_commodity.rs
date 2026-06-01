use deadpool_postgres::Pool;
use edcas_common::api::{MultiCommodityQuery, MultiCommodityResult, NearestCommodityResult};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, State};
use tracing::error;

#[get("/api/v1/nearest-commodity?<commodity>&<reference_system>&<limit>")]
pub async fn nearest_commodity(
    pool: &State<Pool>,
    commodity: String,
    reference_system: String,
    limit: Option<i64>,
) -> Result<Json<Vec<NearestCommodityResult>>, Status> {
    let client = pool.get().await.map_err(|e| {
        error!("nearest_commodity: pool error: {e}");
        Status::ServiceUnavailable
    })?;
    let limit = limit.unwrap_or(10).clamp(1, 50);

    tracing::info!("nearest_commodity: commodity={commodity:?} reference_system={reference_system:?} limit={limit}");

    let sys_row = client
        .query_opt(
            "SELECT x, y, z FROM star_systems WHERE LOWER(name) = LOWER($1) LIMIT 1",
            &[&reference_system],
        )
        .await
        .map_err(|e| {
            error!("nearest_commodity: system lookup error: {e}");
            Status::InternalServerError
        })?;

    let (ref_x, ref_y, ref_z): (f32, f32, f32) = match sys_row {
        Some(r) => (r.get("x"), r.get("y"), r.get("z")),
        None => {
            tracing::warn!("nearest_commodity: reference system {reference_system:?} not found");
            return Ok(Json(vec![]));
        }
    };

    let rows = client
        .query(
            &format!(
                "SELECT s.market_id, s.system_address,
                        ss.name  AS system_name,
                        s.name   AS station_name,
                        s.station_type,
                        cl.name  AS commodity_name,
                        cl.buy_price, cl.stock, cl.sell_price,
                        COALESCE(lp.large  > 0, false) AS has_large_pad,
                        COALESCE(lp.medium > 0, false) AS has_medium_pad,
                        SQRT(
                            POW(ss.x - $1::real, 2) +
                            POW(ss.y - $2::real, 2) +
                            POW(ss.z - $3::real, 2)
                        )::float4 AS distance_ly
                 FROM commodity_listening cl
                 JOIN stations s      ON cl.market_id = s.market_id
                 JOIN star_systems ss ON s.system_address = ss.system_address
                 LEFT JOIN station_landing_pads lp ON s.market_id = lp.market_id
                 WHERE LOWER(REPLACE(REPLACE(cl.name, ' ', ''), '-', '')) = LOWER(REPLACE(REPLACE($4::text, ' ', ''), '-', ''))
                   AND cl.stock > 0
                   AND ss.x IS NOT NULL
                   AND (s.station_type IS NULL OR s.station_type != 'FleetCarrier')
                 ORDER BY distance_ly ASC
                 LIMIT {limit}"
            ),
            &[&ref_x, &ref_y, &ref_z, &commodity],
        )
        .await
        .map_err(|e| {
            error!("nearest_commodity: query error: {e}");
            Status::InternalServerError
        })?;

    tracing::info!("nearest_commodity: returning {} results", rows.len());

    Ok(Json(
        rows.iter()
            .map(|r| NearestCommodityResult {
                market_id:      r.get("market_id"),
                system_address: r.get("system_address"),
                system_name:    r.get("system_name"),
                station_name:   r.get("station_name"),
                station_type:   r.get("station_type"),
                distance_ly:    r.get("distance_ly"),
                commodity_name: r.get("commodity_name"),
                buy_price:      r.get("buy_price"),
                stock:          r.get("stock"),
                sell_price:     r.get("sell_price"),
                has_large_pad:  r.get("has_large_pad"),
                has_medium_pad: r.get("has_medium_pad"),
            })
            .collect(),
    ))
}

#[post("/api/v1/nearest-multi-commodity", data = "<body>", format = "json")]
pub async fn nearest_multi_commodity(
    pool: &State<Pool>,
    body: Json<MultiCommodityQuery>,
) -> Result<Json<Vec<MultiCommodityResult>>, Status> {
    let client = pool.get().await.map_err(|e| {
        error!("nearest_multi_commodity: pool error: {e}");
        Status::ServiceUnavailable
    })?;
    let query = body.into_inner();

    if query.commodities.is_empty() {
        return Ok(Json(vec![]));
    }

    let limit = query.limit.unwrap_or(15).clamp(1, 50);

    tracing::info!(
        "nearest_multi_commodity: {} commodities, reference_system={:?}, limit={limit}",
        query.commodities.len(),
        query.reference_system,
    );

    let sys_row = client
        .query_opt(
            "SELECT x, y, z FROM star_systems WHERE LOWER(name) = LOWER($1) LIMIT 1",
            &[&query.reference_system],
        )
        .await
        .map_err(|e| {
            error!("nearest_multi_commodity: system lookup error: {e}");
            Status::InternalServerError
        })?;

    let (ref_x, ref_y, ref_z): (f32, f32, f32) = match sys_row {
        Some(r) => (r.get("x"), r.get("y"), r.get("z")),
        None => {
            tracing::warn!("nearest_multi_commodity: reference system {:?} not found", query.reference_system);
            return Ok(Json(vec![]));
        }
    };

    let normalized: Vec<String> = query
        .commodities
        .iter()
        .map(|c| c.to_lowercase().replace(' ', "").replace('-', ""))
        .collect();

    let sql = format!(
        "SELECT s.market_id,
                ss.name  AS system_name,
                s.name   AS station_name,
                s.station_type,
                COALESCE(lp.large  > 0, false) AS has_large_pad,
                COALESCE(lp.medium > 0, false) AS has_medium_pad,
                COUNT(DISTINCT LOWER(REPLACE(REPLACE(cl.name, ' ', ''), '-', '')))::bigint AS matched_count,
                ARRAY_AGG(DISTINCT cl.name ORDER BY cl.name) AS matched_commodities,
                SQRT(
                    POW(ss.x - $1::real, 2) +
                    POW(ss.y - $2::real, 2) +
                    POW(ss.z - $3::real, 2)
                )::float4 AS distance_ly
         FROM commodity_listening cl
         JOIN stations s      ON cl.market_id = s.market_id
         JOIN star_systems ss ON s.system_address = ss.system_address
         LEFT JOIN station_landing_pads lp ON s.market_id = lp.market_id
         WHERE LOWER(REPLACE(REPLACE(cl.name, ' ', ''), '-', '')) = ANY($4::text[])
           AND cl.stock > 0
           AND ss.x IS NOT NULL
           AND (s.station_type IS NULL OR s.station_type != 'FleetCarrier')
         GROUP BY s.market_id, ss.name, s.name, s.station_type, lp.large, lp.medium, ss.x, ss.y, ss.z
         ORDER BY matched_count DESC, distance_ly ASC
         LIMIT {limit}"
    );

    let rows = client
        .query(&sql, &[&ref_x, &ref_y, &ref_z, &normalized])
        .await
        .map_err(|e| {
            error!("nearest_multi_commodity: query error: {e}");
            Status::InternalServerError
        })?;

    tracing::info!("nearest_multi_commodity: returning {} results", rows.len());

    Ok(Json(
        rows.iter()
            .map(|r| MultiCommodityResult {
                market_id:             r.get("market_id"),
                system_name:           r.get("system_name"),
                station_name:          r.get("station_name"),
                station_type:          r.get("station_type"),
                distance_ly:           r.get("distance_ly"),
                has_large_pad:         r.get("has_large_pad"),
                has_medium_pad:        r.get("has_medium_pad"),
                matched_commodities:   r.get("matched_commodities"),
                matched_count:         r.get("matched_count"),
            })
            .collect(),
    ))
}
