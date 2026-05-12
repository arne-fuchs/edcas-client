use std::io::Read;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::Context;
use chrono::Utc;
use deadpool_postgres::Pool;
use edcas_common::journal::JournalEvent;
use flate2::read::ZlibDecoder;
use tracing::{error, info, warn};

use crate::db;

/// EDDN message wrapper — the `message` field contains the actual journal event.
#[derive(serde::Deserialize)]
struct EddnMessage {
    #[serde(rename = "$schemaRef")]
    schema_ref: String,
    message: serde_json::Value,
}

pub fn spawn_listener(eddn_url: String, pool: Pool) {
    thread::spawn(move || {
        let rt = tokio::runtime::Handle::current();
        let context = zmq::Context::new();
        let subscriber = context
            .socket(zmq::SUB)
            .expect("failed to create ZMQ socket");
        subscriber
            .connect(&eddn_url)
            .expect("failed to connect to EDDN");
        subscriber
            .set_subscribe(b"")
            .expect("failed to subscribe");

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

                    let pool_clone = pool.clone();
                    rt.spawn(async move {
                        if let Err(e) = handle_message(&json_str, &pool_clone).await {
                            error!("failed to handle EDDN message: {e:#}");
                        }
                    });
                }
                Err(e) => {
                    error!("ZMQ recv error: {e}");
                    thread::sleep(Duration::from_secs(5));
                }
            }
        }
    });
}

async fn handle_message(json_str: &str, pool: &Pool) -> anyhow::Result<()> {
    let wrapper: EddnMessage =
        serde_json::from_str(json_str).context("parsing EDDN wrapper")?;

    // Record the raw event for auditing
    let journal_id = insert_raw_event(pool, &wrapper.schema_ref, &wrapper.message).await?;

    // Dispatch to the appropriate DB inserter
    let event = match JournalEvent::from_eddn_message(wrapper.message) {
        Some(e) => e,
        None => return Ok(()), // unhandled event type — not an error
    };

    match event {
        JournalEvent::FsdJump(ref e) => db::travel::insert_fsd_jump(pool, journal_id, e).await?,
        JournalEvent::Location(ref e) => db::travel::insert_location(pool, journal_id, e).await?,
        JournalEvent::CarrierJump(ref e) => {
            db::travel::insert_carrier_jump(pool, journal_id, e).await?
        }
        JournalEvent::Scan(ref e) => db::scan::insert_scan(pool, journal_id, e).await?,
        JournalEvent::Docked(ref e) => db::station::insert_docked(pool, journal_id, e).await?,
        JournalEvent::Commodities(ref e) => {
            db::station::insert_commodities(pool, journal_id, e).await?
        }
        JournalEvent::Outfitting(ref e) => {
            db::station::insert_outfitting(pool, journal_id, e).await?
        }
        JournalEvent::Shipyard(ref e) => db::station::insert_shipyard(pool, journal_id, e).await?,
        JournalEvent::SaaSignalsFound(ref e) => {
            db::scan::insert_saa_signals(pool, journal_id, e).await?
        }
        JournalEvent::FssBodySignals(ref e) => {
            db::scan::insert_fss_body_signals(pool, journal_id, e).await?
        }
        // Raw-events only — no typed table
        JournalEvent::ScanBaryCentre(_) | JournalEvent::FssSignalDiscovered(_) => {}
    }

    Ok(())
}

async fn insert_raw_event(
    pool: &Pool,
    schema_ref: &str,
    message: &serde_json::Value,
) -> anyhow::Result<i64> {
    let client = pool.get().await?;
    let event_type = message
        .get("event")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let now = Utc::now();
    let row = client
        .query_one(
            "INSERT INTO journal_events (timestamp, event_type, schema_ref, data)
             VALUES ($1, $2, $3, $4)
             RETURNING id",
            &[&now, &event_type, &schema_ref, message],
        )
        .await?;
    Ok(row.get(0))
}

fn decompress(bytes: &[u8]) -> anyhow::Result<String> {
    let mut decoder = ZlibDecoder::new(bytes);
    let mut out = String::new();
    decoder.read_to_string(&mut out)?;
    Ok(out)
}
