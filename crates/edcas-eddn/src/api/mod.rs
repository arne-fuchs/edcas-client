pub mod carriers;
pub mod stations;
pub mod systems;

use axum::{routing::get, Router};
use deadpool_postgres::Pool;
use tower_http::cors::CorsLayer;

pub fn build_router(pool: Pool) -> Router {
    Router::new()
        .route("/api/v1/systems/:address", get(systems::get_system))
        .route("/api/v1/systems/:address/bodies", get(systems::get_system_bodies))
        .route("/api/v1/stations", get(stations::search_stations))
        .route("/api/v1/carriers", get(carriers::search_carriers))
        .layer(CorsLayer::permissive())
        .with_state(pool)
}
