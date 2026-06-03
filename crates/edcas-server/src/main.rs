mod api;
mod cache;
mod config;
mod db;
mod listener;
mod request_logger;
mod stats;
mod tick;

use tracing::info;
use tracing_subscriber::EnvFilter;

#[rocket::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("edcas_server=info".parse()?))
        .init();

    let cfg = config::Config::from_env()?;
    info!("Connecting to database at {}", cfg.db_host);

    let pool = db::build_pool(&cfg.db_url())?;

    {
        let client = pool.get().await?;
        client.query_one("SELECT 1", &[]).await?;
    }
    info!("Database connection OK");

    listener::spawn_listener(cfg.eddn_url.clone(), pool.clone());
    info!("EDDN listener started, connecting to {}", cfg.eddn_url);

    stats::spawn_stats_logger();
    info!("Stats logger started (logs every 60 s)");

    cache::spawn_cache_refresher(pool.clone());
    info!("Trade cache refresher started (15 min interval)");

    tick::spawn_tick_detector(pool.clone());
    info!("Server tick detector started (30 min interval)");

    let rocket_cfg = rocket::Config {
        port: cfg.api_port,
        address: std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
        limits: rocket::data::Limits::default()
            .limit("json", rocket::data::ByteUnit::Mebibyte(50)),
        ..rocket::Config::default()
    };

    info!("API server starting on port {}", cfg.api_port);

    rocket::build()
        .configure(rocket_cfg)
        .attach(request_logger::RequestLogger)
        .manage(pool)
        .mount(
            "/",
            rocket::routes![
                api::systems::get_system,
                api::systems::get_system_bodies,
                api::systems::system_population_history,
                api::stations::search_stations,
                api::carriers::search_carriers,
                api::factions::search_factions,
                api::factions::faction_influence_history,
                api::stations::commodity_price_history,
                api::construction::search_construction_depots,
                api::construction::submit_construction_depot,
                api::journal::ingest_event,
                api::journal::ingest_events,
                api::nearest_commodity::nearest_commodity,
                api::nearest_commodity::nearest_multi_commodity,
                api::trade_routes::get_trade_routes,
                api::trade_routes::get_trade_loops,
                api::tick::get_server_tick,
            ],
        )
        .launch()
        .await
        .map_err(|e| anyhow::anyhow!("Rocket launch error: {e}"))?;

    Ok(())
}
