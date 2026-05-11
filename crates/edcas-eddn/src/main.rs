mod api;
mod config;
mod db;
mod listener;

use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("edcas_eddn=info".parse()?))
        .init();

    let cfg = config::Config::from_env()?;
    info!("Connecting to database at {}", cfg.db_host);

    let pool = db::build_pool(&cfg.db_url())?;

    // Verify DB connection on startup
    {
        let client = pool.get().await?;
        client.query_one("SELECT 1", &[]).await?;
    }
    info!("Database connection OK");

    // Start EDDN listener in a blocking thread (ZMQ is synchronous)
    listener::spawn_listener(cfg.eddn_url.clone(), pool.clone());
    info!("EDDN listener started, connecting to {}", cfg.eddn_url);

    // Start REST API server
    let addr = SocketAddr::from(([0, 0, 0, 0], cfg.api_port));
    let router = api::build_router(pool);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("API server listening on http://{addr}");

    axum::serve(listener, router).await?;
    Ok(())
}
