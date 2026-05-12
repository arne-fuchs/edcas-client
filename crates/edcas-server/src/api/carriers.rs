use edcas_common::api::CarrierResponse;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, State};
use deadpool_postgres::Pool;

use super::stations::query_stations;

#[get("/api/v1/carriers?<name>&<callsign>&<system_name>&<limit>")]
pub async fn search_carriers(
    pool: &State<Pool>,
    name: Option<String>,
    callsign: Option<String>,
    system_name: Option<String>,
    limit: Option<i64>,
) -> Result<Json<Vec<CarrierResponse>>, Status> {
    let effective_name = name.or(callsign);
    query_stations(pool, effective_name, system_name, None, limit, true).await
}
