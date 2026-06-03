use deadpool_postgres::Pool;
use edcas_common::api::{ServerTickEntry, ServerTickResponse};
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

/// Full list of recorded server ticks, newest first.
#[get("/api/v1/server-ticks")]
pub async fn get_server_ticks(pool: &State<Pool>) -> Json<Vec<ServerTickEntry>> {
    match tick::get_all_ticks(pool).await {
        Ok(ticks) => Json(
            ticks
                .into_iter()
                .map(|(tick_time, system_count, detected_at)| ServerTickEntry {
                    tick_time,
                    system_count,
                    detected_at,
                })
                .collect(),
        ),
        Err(_) => Json(Vec::new()),
    }
}
