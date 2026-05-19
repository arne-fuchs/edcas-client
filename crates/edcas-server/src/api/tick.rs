use deadpool_postgres::Pool;
use edcas_common::api::ServerTickResponse;
use rocket::{get, State};
use rocket::serde::json::Json;

use crate::tick;

#[get("/api/v1/server-tick")]
pub async fn get_server_tick(pool: &State<Pool>) -> Json<ServerTickResponse> {
    match tick::get_tick_prediction(pool).await {
        Ok(Some((last_tick, next_predicted_tick, system_count))) => Json(ServerTickResponse {
            last_tick: Some(last_tick),
            next_predicted_tick: Some(next_predicted_tick),
            system_count: Some(system_count),
        }),
        _ => Json(ServerTickResponse {
            last_tick: None,
            next_predicted_tick: None,
            system_count: None,
        }),
    }
}
