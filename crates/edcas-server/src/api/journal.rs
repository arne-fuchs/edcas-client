use std::sync::atomic::Ordering;

use deadpool_postgres::Pool;
use edcas_common::journal::JournalEvent;
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use tracing::error;

use crate::db;
use crate::stats;

/// Same wire format as EDDN so parsing is identical to the EDDN listener.
#[derive(serde::Deserialize)]
pub struct UploadMessage {
    #[serde(rename = "$schemaRef")]
    schema_ref: String,
    message: serde_json::Value,
}

async fn process_message(pool: &Pool, msg: UploadMessage) {
    stats::CLIENT_RECEIVED.fetch_add(1, Ordering::Relaxed);
    let event_type = msg.message.get("event").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
    let (journal_id, event_ts) = match db::insert_raw_event(pool, &msg.schema_ref, &msg.message).await {
        Ok(v) => v,
        Err(e) => {
            stats::CLIENT_ERRORS.fetch_add(1, Ordering::Relaxed);
            error!(event_type = %event_type, "insert_raw_event failed: {e:#}");
            return;
        }
    };
    match JournalEvent::from_eddn_message(msg.message) {
        Some(event) => {
            match db::dispatch_event(pool, journal_id, event_ts, event).await {
                Ok(_) => { stats::CLIENT_DISPATCHED.fetch_add(1, Ordering::Relaxed); }
                Err(e) => {
                    stats::CLIENT_ERRORS.fetch_add(1, Ordering::Relaxed);
                    error!(event_type = %event_type, journal_id, "dispatch_event failed: {e:#}");
                }
            }
        }
        None => {
            stats::CLIENT_SKIPPED.fetch_add(1, Ordering::Relaxed);
        }
    }
}

#[rocket::post("/api/v1/journal/event", data = "<body>")]
pub async fn ingest_event(pool: &State<Pool>, body: Json<UploadMessage>) -> Status {
    process_message(pool, body.into_inner()).await;
    Status::NoContent
}

/// Batch variant: accepts up to 500 events in one request.
/// Returns 204 on success; individual event errors are logged and skipped.
#[rocket::post("/api/v1/journal/events", data = "<body>")]
pub async fn ingest_events(pool: &State<Pool>, body: Json<Vec<UploadMessage>>) -> Status {
    for msg in body.into_inner() {
        process_message(pool, msg).await;
    }
    Status::NoContent
}
