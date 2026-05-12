use chrono::Utc;
use deadpool_postgres::Pool;
use edcas_common::api::{ConstructionDepotResponse, ConstructionDepotSubmission};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, post, State};

use crate::db;

#[get("/api/v1/construction-depots?<name>&<system_name>&<market_id>&<limit>")]
pub async fn search_construction_depots(
    pool: &State<Pool>,
    name: Option<String>,
    system_name: Option<String>,
    market_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Json<Vec<ConstructionDepotResponse>>, Status> {
    let limit = limit.unwrap_or(50).min(200);
    db::construction::query_depots(pool, name.as_deref(), system_name.as_deref(), market_id, limit)
        .await
        .map(Json)
        .map_err(|_| Status::InternalServerError)
}

#[post("/api/v1/construction-depots", data = "<submission>")]
pub async fn submit_construction_depot(
    pool: &State<Pool>,
    submission: Json<ConstructionDepotSubmission>,
) -> Result<Status, Status> {
    db::construction::upsert_depot(pool, Utc::now(), &submission)
        .await
        .map(|_| Status::NoContent)
        .map_err(|_| Status::InternalServerError)
}
