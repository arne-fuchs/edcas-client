mod api;
mod config;
mod db;
mod listener;

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

    let rocket_cfg = rocket::Config {
        port: cfg.api_port,
        address: std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
        ..rocket::Config::default()
    };

    info!("API server starting on port {}", cfg.api_port);

    rocket::build()
        .configure(rocket_cfg)
        .manage(pool)
        .mount(
            "/",
            rocket::routes![
                api::systems::get_system,
                api::systems::get_system_bodies,
                api::stations::search_stations,
                api::carriers::search_carriers,
                api::factions::search_factions,
            ],
        )
        .launch()
        .await
        .map_err(|e| anyhow::anyhow!("Rocket launch error: {e}"))?;

    Ok(())
}
