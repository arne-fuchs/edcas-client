use deadpool_postgres::Pool;
use std::time::Duration;
use tokio::time;
use tracing::{error, info};

const REFRESH_INTERVAL_SECS: u64 = 15 * 60;
const RADIUS: f32 = 500.0;

pub fn spawn_cache_refresher(pool: Pool) {
    tokio::spawn(async move {
        refresh_all(&pool).await;
        let mut interval = time::interval(Duration::from_secs(REFRESH_INTERVAL_SECS));
        interval.tick().await; // skip the immediate first tick (already ran above)
        loop {
            interval.tick().await;
            refresh_all(&pool).await;
        }
    });
}

async fn refresh_all(pool: &Pool) {
    info!("Trade cache refresh starting (radius={RADIUS} Ly, large pads only)");
    match refresh_routes(pool, "L").await {
        Ok(n) => info!("  routes [L]: {n} rows"),
        Err(e) => error!("  routes [L] failed: {e:#}"),
    }
    match refresh_loops(pool, "L").await {
        Ok(n) => info!("  loops  [L]: {n} rows"),
        Err(e) => error!("  loops  [L] failed: {e:#}"),
    }
    info!("Trade cache refresh complete");
}

fn pad_nearby_clause(pad: &str) -> &'static str {
    match pad {
        "L" => " AND COALESCE(slp.large, 0) > 0",
        "M" => " AND (COALESCE(slp.large, 0) > 0 OR COALESCE(slp.medium, 0) > 0)",
        _ => "",
    }
}

async fn refresh_routes(pool: &Pool, pad: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    let pad_clause = pad_nearby_clause(pad);
    let r = RADIUS;

    let sql = format!(
        r#"
        WITH nearby AS MATERIALIZED (
            SELECT s.market_id, ss.x, ss.y, ss.z,
                   ss.name AS system_name, s.name AS station_name,
                   COALESCE(slp.large,  0) AS large,
                   COALESCE(slp.medium, 0) AS medium,
                   alg.value AS allegiance
            FROM stations s
            JOIN star_systems ss ON s.system_address = ss.system_address
            LEFT JOIN station_landing_pads slp ON slp.market_id = s.market_id
            LEFT JOIN allegiance alg ON alg.id = ss.allegiance
            WHERE ss.x BETWEEN -{r} AND {r}
              AND ss.y BETWEEN -{r} AND {r}
              AND ss.z BETWEEN -{r} AND {r}
            {pad_clause}
        ),
        best_buys AS MATERIALIZED (
            SELECT DISTINCT ON (cl.name)
                cl.market_id, cl.name, cl.buy_price, cl.stock
            FROM commodity_listening cl
            JOIN nearby n ON n.market_id = cl.market_id
            WHERE cl.buy_price > 0 AND cl.stock >= 10000 AND cl.stock_bracket > 0
            ORDER BY cl.name, cl.buy_price ASC
        ),
        sells AS MATERIALIZED (
            SELECT cl.market_id, cl.name, cl.sell_price, cl.demand
            FROM commodity_listening cl
            JOIN nearby n ON n.market_id = cl.market_id
            WHERE cl.demand > 0 AND cl.sell_price > 0 AND cl.demand_bracket > 0
        ),
        results AS (
            SELECT
                ROW_NUMBER() OVER (ORDER BY (s.sell_price - b.buy_price) DESC)::integer AS rank,
                b.market_id              AS from_market_id,
                s.market_id              AS to_market_id,
                b.name                   AS commodity,
                b.buy_price, s.sell_price,
                (s.sell_price - b.buy_price) AS profit,
                b.stock                  AS supply,
                s.demand,
                SQRT(
                    POWER((fm.x - tm.x)::double precision, 2) +
                    POWER((fm.y - tm.y)::double precision, 2) +
                    POWER((fm.z - tm.z)::double precision, 2)
                )::real                  AS distance_ly,
                fm.station_name          AS from_station_name,
                tm.station_name          AS to_station_name,
                fm.system_name           AS from_system_name,
                tm.system_name           AS to_system_name,
                CASE WHEN fm.large  > 0 THEN 'L'
                     WHEN fm.medium > 0 THEN 'M'
                     ELSE 'S' END        AS from_max_pad,
                CASE WHEN tm.large  > 0 THEN 'L'
                     WHEN tm.medium > 0 THEN 'M'
                     ELSE 'S' END        AS to_max_pad,
                fm.allegiance            AS from_allegiance,
                tm.allegiance            AS to_allegiance
            FROM best_buys b
            JOIN sells s ON s.name = b.name
                AND s.market_id != b.market_id
                AND s.sell_price > b.buy_price
            JOIN nearby fm ON fm.market_id = b.market_id
            JOIN nearby tm ON tm.market_id = s.market_id
            WHERE (s.sell_price - b.buy_price) >= 1000
              AND POWER((fm.x - tm.x)::double precision, 2) +
                  POWER((fm.y - tm.y)::double precision, 2) +
                  POWER((fm.z - tm.z)::double precision, 2) <= POWER({r}::double precision, 2)
            ORDER BY profit DESC
            LIMIT 50
        )
        INSERT INTO cached_trade_routes (
            pad_filter, rank,
            from_market_id, to_market_id, commodity,
            buy_price, sell_price, profit, supply, demand,
            distance_ly,
            from_station_name, to_station_name,
            from_system_name,  to_system_name,
            from_max_pad, to_max_pad,
            from_allegiance, to_allegiance,
            cached_at
        )
        SELECT
            '{pad}', rank,
            from_market_id, to_market_id, commodity,
            buy_price, sell_price, profit, supply, demand,
            distance_ly,
            from_station_name, to_station_name,
            from_system_name,  to_system_name,
            from_max_pad, to_max_pad,
            from_allegiance, to_allegiance,
            NOW()
        FROM results
        ON CONFLICT (pad_filter, rank) DO UPDATE SET
            from_market_id    = EXCLUDED.from_market_id,
            to_market_id      = EXCLUDED.to_market_id,
            commodity         = EXCLUDED.commodity,
            buy_price         = EXCLUDED.buy_price,
            sell_price        = EXCLUDED.sell_price,
            profit            = EXCLUDED.profit,
            supply            = EXCLUDED.supply,
            demand            = EXCLUDED.demand,
            distance_ly       = EXCLUDED.distance_ly,
            from_station_name = EXCLUDED.from_station_name,
            to_station_name   = EXCLUDED.to_station_name,
            from_system_name  = EXCLUDED.from_system_name,
            to_system_name    = EXCLUDED.to_system_name,
            from_max_pad      = EXCLUDED.from_max_pad,
            to_max_pad        = EXCLUDED.to_max_pad,
            from_allegiance   = EXCLUDED.from_allegiance,
            to_allegiance     = EXCLUDED.to_allegiance,
            cached_at         = EXCLUDED.cached_at
        "#
    );

    let n = client.execute(&sql, &[]).await?;
    Ok(n)
}

async fn refresh_loops(pool: &Pool, pad: &str) -> Result<u64, Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    let pad_clause = pad_nearby_clause(pad);
    let r = RADIUS;

    let sql = format!(
        r#"
        WITH nearby AS MATERIALIZED (
            SELECT s.market_id, ss.x, ss.y, ss.z,
                   ss.name AS system_name, s.name AS station_name,
                   COALESCE(slp.large,  0) AS large,
                   COALESCE(slp.medium, 0) AS medium,
                   alg.value AS allegiance
            FROM stations s
            JOIN star_systems ss ON s.system_address = ss.system_address
            LEFT JOIN station_landing_pads slp ON slp.market_id = s.market_id
            LEFT JOIN allegiance alg ON alg.id = ss.allegiance
            WHERE ss.x BETWEEN -{r} AND {r}
              AND ss.y BETWEEN -{r} AND {r}
              AND ss.z BETWEEN -{r} AND {r}
            {pad_clause}
        ),
        best_buys AS MATERIALIZED (
            SELECT DISTINCT ON (cl.name)
                cl.market_id, cl.name, cl.buy_price, cl.stock
            FROM commodity_listening cl
            JOIN nearby n ON n.market_id = cl.market_id
            WHERE cl.buy_price > 0 AND cl.stock >= 10000 AND cl.stock_bracket > 0
            ORDER BY cl.name, cl.buy_price ASC
        ),
        sells AS MATERIALIZED (
            SELECT cl.market_id, cl.name, cl.sell_price, cl.demand
            FROM commodity_listening cl
            JOIN nearby n ON n.market_id = cl.market_id
            WHERE cl.demand > 0 AND cl.sell_price > 0 AND cl.demand_bracket > 0
        ),
        -- Best outbound route per (A, B) station pair
        outbound AS MATERIALIZED (
            SELECT DISTINCT ON (b.market_id, s.market_id)
                b.market_id    AS a_mkt,
                s.market_id    AS b_mkt,
                b.name         AS commodity,
                b.buy_price,
                s.sell_price,
                (s.sell_price - b.buy_price) AS profit,
                b.stock        AS supply_out,
                s.demand       AS demand_out,
                fm.x AS ax, fm.y AS ay, fm.z AS az,
                tm.x AS bx, tm.y AS by, tm.z AS bz,
                fm.station_name  AS sta_a,
                tm.station_name  AS sta_b,
                fm.system_name   AS sys_a,
                tm.system_name   AS sys_b,
                fm.allegiance    AS allegiance_a,
                tm.allegiance    AS allegiance_b,
                CASE WHEN fm.large  > 0 THEN 'L'
                     WHEN fm.medium > 0 THEN 'M'
                     ELSE 'S' END AS pad_a,
                CASE WHEN tm.large  > 0 THEN 'L'
                     WHEN tm.medium > 0 THEN 'M'
                     ELSE 'S' END AS pad_b
            FROM best_buys b
            JOIN sells s ON s.name = b.name
                AND s.market_id != b.market_id
                AND s.sell_price > b.buy_price
            JOIN nearby fm ON fm.market_id = b.market_id
            JOIN nearby tm ON tm.market_id = s.market_id
            WHERE (s.sell_price - b.buy_price) >= 1000
              AND POWER((fm.x - tm.x)::double precision, 2) +
                  POWER((fm.y - tm.y)::double precision, 2) +
                  POWER((fm.z - tm.z)::double precision, 2) <= POWER({r}::double precision, 2)
            ORDER BY b.market_id, s.market_id, profit DESC
        ),
        -- Best return commodity for each (A, B) pair: buy at B, sell at A
        return_legs AS MATERIALIZED (
            SELECT DISTINCT ON (o.a_mkt, o.b_mkt)
                o.a_mkt,
                o.b_mkt,
                cl_buy.name         AS commodity,
                cl_buy.buy_price,
                cl_sell.sell_price,
                (cl_sell.sell_price - cl_buy.buy_price) AS profit,
                cl_buy.stock        AS supply_back,
                cl_sell.demand      AS demand_back
            FROM (SELECT DISTINCT a_mkt, b_mkt FROM outbound) o
            JOIN commodity_listening cl_buy
                ON cl_buy.market_id = o.b_mkt
               AND cl_buy.buy_price > 0
               AND cl_buy.stock > 0
               AND cl_buy.stock_bracket > 0
            JOIN commodity_listening cl_sell
                ON cl_sell.market_id = o.a_mkt
               AND cl_sell.name = cl_buy.name
               AND cl_sell.demand > 0
               AND cl_sell.sell_price > cl_buy.buy_price
               AND cl_sell.demand_bracket > 0
            WHERE (cl_sell.sell_price - cl_buy.buy_price) >= 1000
            ORDER BY o.a_mkt, o.b_mkt,
                     (cl_sell.sell_price - cl_buy.buy_price) DESC
        ),
        results AS (
            SELECT
                ROW_NUMBER() OVER (
                    ORDER BY (o.profit + r.profit) DESC
                )::integer AS rank,
                o.a_mkt                 AS market_id_a,
                o.b_mkt                 AS market_id_b,
                o.commodity             AS commodity_out,
                o.buy_price             AS buy_price_out,
                o.sell_price            AS sell_price_out,
                o.profit                AS profit_out,
                r.commodity             AS commodity_back,
                r.buy_price             AS buy_price_back,
                r.sell_price            AS sell_price_back,
                r.profit                AS profit_back,
                (o.profit + r.profit)   AS total_profit,
                SQRT(
                    POWER((o.ax - o.bx)::double precision, 2) +
                    POWER((o.ay - o.by)::double precision, 2) +
                    POWER((o.az - o.bz)::double precision, 2)
                )::real                 AS distance_ly,
                o.sta_a                 AS station_name_a,
                o.sta_b                 AS station_name_b,
                o.sys_a                 AS system_name_a,
                o.sys_b                 AS system_name_b,
                o.allegiance_a,
                o.allegiance_b,
                o.supply_out,
                r.supply_back,
                o.demand_out,
                r.demand_back,
                CASE WHEN o.pad_a = 'S' OR o.pad_b = 'S' THEN 'S'
                     WHEN o.pad_a = 'M' OR o.pad_b = 'M' THEN 'M'
                     ELSE 'L' END       AS max_pad
            FROM outbound o
            JOIN return_legs r
                ON r.a_mkt = o.a_mkt
               AND r.b_mkt = o.b_mkt
            ORDER BY total_profit DESC
            LIMIT 50
        )
        INSERT INTO cached_trade_loops (
            pad_filter, rank,
            market_id_a, market_id_b,
            commodity_out, buy_price_out, sell_price_out, profit_out,
            commodity_back, buy_price_back, sell_price_back, profit_back,
            total_profit, distance_ly,
            station_name_a, station_name_b,
            system_name_a,  system_name_b,
            allegiance_a, allegiance_b,
            supply_out, supply_back,
            demand_out, demand_back,
            max_pad, cached_at
        )
        SELECT
            '{pad}', rank,
            market_id_a, market_id_b,
            commodity_out, buy_price_out, sell_price_out, profit_out,
            commodity_back, buy_price_back, sell_price_back, profit_back,
            total_profit, distance_ly,
            station_name_a, station_name_b,
            system_name_a,  system_name_b,
            allegiance_a, allegiance_b,
            supply_out, supply_back,
            demand_out, demand_back,
            max_pad, NOW()
        FROM results
        ON CONFLICT (pad_filter, rank) DO UPDATE SET
            market_id_a     = EXCLUDED.market_id_a,
            market_id_b     = EXCLUDED.market_id_b,
            commodity_out   = EXCLUDED.commodity_out,
            buy_price_out   = EXCLUDED.buy_price_out,
            sell_price_out  = EXCLUDED.sell_price_out,
            profit_out      = EXCLUDED.profit_out,
            commodity_back  = EXCLUDED.commodity_back,
            buy_price_back  = EXCLUDED.buy_price_back,
            sell_price_back = EXCLUDED.sell_price_back,
            profit_back     = EXCLUDED.profit_back,
            total_profit    = EXCLUDED.total_profit,
            distance_ly     = EXCLUDED.distance_ly,
            station_name_a  = EXCLUDED.station_name_a,
            station_name_b  = EXCLUDED.station_name_b,
            system_name_a   = EXCLUDED.system_name_a,
            system_name_b   = EXCLUDED.system_name_b,
            allegiance_a    = EXCLUDED.allegiance_a,
            allegiance_b    = EXCLUDED.allegiance_b,
            supply_out      = EXCLUDED.supply_out,
            supply_back     = EXCLUDED.supply_back,
            demand_out      = EXCLUDED.demand_out,
            demand_back     = EXCLUDED.demand_back,
            max_pad         = EXCLUDED.max_pad,
            cached_at       = EXCLUDED.cached_at
        "#
    );

    let n = client.execute(&sql, &[]).await?;
    Ok(n)
}
