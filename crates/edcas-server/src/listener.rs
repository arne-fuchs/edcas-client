use std::io::Read;
use std::thread;
use std::time::Duration;

use anyhow::Context;
use deadpool_postgres::Pool;
use edcas_common::journal::JournalEvent;
use flate2::read::ZlibDecoder;
use tracing::{error, info, warn};

use crate::db;
use crate::stats;

/// EDDN message wrapper — the `message` field contains the actual journal event.
#[derive(serde::Deserialize)]
struct EddnMessage {
    #[serde(rename = "$schemaRef")]
    schema_ref: String,
    message: serde_json::Value,
}

pub fn spawn_listener(eddn_url: String, pool: Pool) {
    // Capture the handle here, while we are still inside the Tokio runtime.
    // thread::spawn creates a plain OS thread with no Tokio context, so
    // Handle::current() would panic if called from inside the closure.
    let handle = tokio::runtime::Handle::current();
    thread::spawn(move || {
        let context = zmq::Context::new();

        loop {
            // (Re-)create a fresh socket on every connection attempt so that
            // stale state from a previous broken connection doesn't linger.
            let subscriber = match context.socket(zmq::SUB) {
                Ok(s) => s,
                Err(e) => {
                    error!("failed to create ZMQ socket: {e} — retrying in 5 s");
                    thread::sleep(Duration::from_secs(5));
                    continue;
                }
            };

            if let Err(e) = subscriber.connect(&eddn_url) {
                error!("failed to connect to EDDN ({eddn_url}): {e} — retrying in 5 s");
                thread::sleep(Duration::from_secs(5));
                continue;
            }
            if let Err(e) = subscriber.set_subscribe(b"") {
                error!("failed to subscribe on ZMQ socket: {e} — retrying in 5 s");
                thread::sleep(Duration::from_secs(5));
                continue;
            }

            info!("EDDN listener connected to {eddn_url}");

            loop {
                match subscriber.recv_bytes(0) {
                    Ok(compressed) => {
                        let json_str = match decompress(&compressed) {
                            Ok(s) => s,
                            Err(e) => {
                                warn!("decompression error: {e}");
                                continue;
                            }
                        };

                        stats::EDDN_RECEIVED.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let pool_clone = pool.clone();
                        handle.spawn(async move {
                            match handle_message(&json_str, &pool_clone).await {
                                Ok(dispatched) => {
                                    if dispatched {
                                        stats::EDDN_DISPATCHED
                                            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    }
                                }
                                Err(e) => {
                                    stats::EDDN_ERRORS
                                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                    error!("failed to handle EDDN message: {e:#}");
                                }
                            }
                        });
                    }
                    Err(e) => {
                        error!("ZMQ recv error: {e} — reconnecting in 5 s");
                        break; // drop subscriber, outer loop will reconnect
                    }
                }
            }

            thread::sleep(Duration::from_secs(5));
        }
    });
}

/// Returns `true` if the event was recognised and dispatched to the DB.
async fn handle_message(json_str: &str, pool: &Pool) -> anyhow::Result<bool> {
    let wrapper: EddnMessage =
        serde_json::from_str(json_str).context("parsing EDDN wrapper")?;

    let (journal_id, event_ts) =
        db::insert_raw_event(pool, &wrapper.schema_ref, &wrapper.message).await?;

    let event = match JournalEvent::from_eddn_message(wrapper.message) {
        Some(e) => e,
        None => return Ok(false),
    };

    db::dispatch_event(pool, journal_id, event_ts, event).await?;

    Ok(true)
}

fn decompress(bytes: &[u8]) -> anyhow::Result<String> {
    let mut decoder = ZlibDecoder::new(bytes);
    let mut out = String::new();
    decoder.read_to_string(&mut out)?;
    Ok(out)
}
