use deadpool_postgres::Pool;
use edcas_common::api::{TradeLoopResponse, TradeRouteResponse};
use rocket::{get, http::Status, serde::json::Json, State};
use tracing::error;

#[get("/api/v1/trade-routes")]
pub async fn get_trade_routes(
    pool: &State<Pool>,
) -> Result<Json<Vec<TradeRouteResponse>>, Status> {
    let client = pool.get().await.map_err(|e| {
        error!("DB pool error: {e:#}");
        Status::ServiceUnavailable
    })?;

    let rows = client
        .query(
            "SELECT from_market_id, to_market_id, commodity,
                    buy_price, sell_price, profit, supply, demand,
                    distance_ly,
                    from_station_name, to_station_name,
                    from_system_name,  to_system_name,
                    from_max_pad, to_max_pad,
                    from_allegiance, to_allegiance,
                    cached_at
             FROM cached_trade_routes
             WHERE pad_filter = 'L'
             ORDER BY rank",
            &[],
        )
        .await
        .map_err(|e| {
            error!("Trade routes cache query failed: {e:#}");
            Status::InternalServerError
        })?;

    let results = rows
        .iter()
        .map(|r| TradeRouteResponse {
            from_market_id:    r.get("from_market_id"),
            to_market_id:      r.get("to_market_id"),
            commodity:         r.get("commodity"),
            buy_price:         r.get("buy_price"),
            sell_price:        r.get("sell_price"),
            profit:            r.get("profit"),
            supply:            r.get("supply"),
            demand:            r.get("demand"),
            distance_ly:       r.get("distance_ly"),
            from_station_name: r.get("from_station_name"),
            to_station_name:   r.get("to_station_name"),
            from_system_name:  r.get("from_system_name"),
            to_system_name:    r.get("to_system_name"),
            from_max_pad:      r.get("from_max_pad"),
            to_max_pad:        r.get("to_max_pad"),
            from_allegiance:   r.get("from_allegiance"),
            to_allegiance:     r.get("to_allegiance"),
            cached_at:         r.get("cached_at"),
        })
        .collect();

    Ok(Json(results))
}

#[get("/api/v1/trade-loops")]
pub async fn get_trade_loops(
    pool: &State<Pool>,
) -> Result<Json<Vec<TradeLoopResponse>>, Status> {
    let client = pool.get().await.map_err(|e| {
        error!("DB pool error: {e:#}");
        Status::ServiceUnavailable
    })?;

    let rows = client
        .query(
            "SELECT market_id_a, market_id_b,
                    commodity_out, buy_price_out, sell_price_out, profit_out,
                    commodity_back, buy_price_back, sell_price_back, profit_back,
                    total_profit, distance_ly,
                    station_name_a, station_name_b,
                    system_name_a,  system_name_b,
                    max_pad, allegiance_a, allegiance_b, supply_out, supply_back, demand_out, demand_back,
                    cached_at
             FROM cached_trade_loops
             WHERE pad_filter = 'L'
             ORDER BY rank",
            &[],
        )
        .await
        .map_err(|e| {
            error!("Trade loops cache query failed: {e:#}");
            Status::InternalServerError
        })?;

    let results = rows
        .iter()
        .map(|r| TradeLoopResponse {
            market_id_a:     r.get("market_id_a"),
            market_id_b:     r.get("market_id_b"),
            commodity_out:   r.get("commodity_out"),
            buy_price_out:   r.get("buy_price_out"),
            sell_price_out:  r.get("sell_price_out"),
            profit_out:      r.get("profit_out"),
            commodity_back:  r.get("commodity_back"),
            buy_price_back:  r.get("buy_price_back"),
            sell_price_back: r.get("sell_price_back"),
            profit_back:     r.get("profit_back"),
            total_profit:    r.get("total_profit"),
            distance_ly:     r.get("distance_ly"),
            station_name_a:  r.get("station_name_a"),
            station_name_b:  r.get("station_name_b"),
            system_name_a:   r.get("system_name_a"),
            system_name_b:   r.get("system_name_b"),
            max_pad:         r.get("max_pad"),
            allegiance_a:    r.get("allegiance_a"),
            allegiance_b:    r.get("allegiance_b"),
            supply_out:      r.get("supply_out"),
            supply_back:     r.get("supply_back"),
            demand_out:      r.get("demand_out"),
            demand_back:     r.get("demand_back"),
            cached_at:       r.get("cached_at"),
        })
        .collect();

    Ok(Json(results))
}
