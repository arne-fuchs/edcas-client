use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use deadpool_postgres::Pool;
use edcas_common::api::{CarrierQuery, CarrierResponse, StationQuery};

use super::stations::query_stations;

pub async fn search_carriers(
    State(pool): State<Pool>,
    Query(params): Query<CarrierQuery>,
) -> Result<Json<Vec<CarrierResponse>>, StatusCode> {
    let station_params = StationQuery {
        name: params.name.or(params.callsign),
        system_name: params.system_name,
        market_id: None,
        limit: params.limit,
    };
    query_stations(&pool, &station_params, true).await
}
