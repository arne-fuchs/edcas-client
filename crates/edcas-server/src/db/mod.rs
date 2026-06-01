pub mod construction;
pub mod scan;
pub mod station;
pub mod tables;
pub mod travel;

use chrono::{DateTime, Utc};
use deadpool_postgres::{Config as PoolConfig, Pool, Runtime};
use edcas_common::api::{ConstructionDepotSubmission, ConstructionResourceSubmission};
use edcas_common::journal::JournalEvent;
use tokio_postgres::NoTls;

/// Stores the raw message JSON in `journal_events`.
/// Returns `(journal_id, event_timestamp)` where `event_timestamp` is
/// parsed from `message['timestamp']`, falling back to `Utc::now()`.
pub async fn insert_raw_event(
    pool: &Pool,
    schema_ref: &str,
    message: &serde_json::Value,
) -> anyhow::Result<(i64, DateTime<Utc>)> {
    let event_type = message.get("event").and_then(|v| v.as_str()).unwrap_or("unknown");
    let event_timestamp = message
        .get("timestamp")
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(Utc::now);
    let received_at = Utc::now();
    let client = pool.get().await?;
    let row = client
        .query_one(
            "INSERT INTO journal_events (timestamp, event_timestamp, event_type, schema_ref, data)
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
            &[&received_at, &event_timestamp, &event_type, &schema_ref, message],
        )
        .await?;
    Ok((row.get(0), event_timestamp))
}

/// Dispatches a typed journal event to the appropriate DB inserter.
/// `event_timestamp` is the actual in-game time from the journal event's
/// `timestamp` field; it guards against old data overwriting newer rows.
pub async fn dispatch_event(
    pool: &Pool,
    journal_id: i64,
    event_timestamp: DateTime<Utc>,
    event: JournalEvent,
) -> anyhow::Result<()> {
    match event {
        JournalEvent::FsdJump(ref e) => {
            travel::insert_fsd_jump(pool, journal_id, event_timestamp, e).await?
        }
        JournalEvent::Location(ref e) => {
            travel::insert_location(pool, journal_id, event_timestamp, e).await?
        }
        JournalEvent::CarrierJump(ref e) => {
            travel::insert_carrier_jump(pool, journal_id, event_timestamp, e).await?
        }
        JournalEvent::Scan(ref e) => {
            scan::insert_scan(pool, journal_id, event_timestamp, e).await?
        }
        JournalEvent::Docked(ref e) => {
            station::insert_docked(pool, journal_id, event_timestamp, e).await?
        }
        JournalEvent::Commodities(ref e) => {
            station::insert_commodities(pool, journal_id, event_timestamp, e).await?
        }
        JournalEvent::Outfitting(ref e) => {
            station::insert_outfitting(pool, journal_id, event_timestamp, e).await?
        }
        JournalEvent::Shipyard(ref e) => {
            station::insert_shipyard(pool, journal_id, event_timestamp, e).await?
        }
        JournalEvent::CarrierStats(ref e) => {
            station::update_carrier_name(pool, e).await?
        }
        JournalEvent::SaaSignalsFound(ref e) => {
            scan::insert_saa_signals(pool, journal_id, e).await?
        }
        JournalEvent::FssBodySignals(ref e) => {
            scan::insert_fss_body_signals(pool, journal_id, e).await?
        }
        JournalEvent::ColonisationConstructionDepot(ref e) => {
            let submission = ConstructionDepotSubmission {
                market_id: e.market_id,
                system_address: e.system_address,
                station_name: String::new(),
                progress: e.construction_progress,
                construction_complete: e.construction_complete,
                construction_failed: e.construction_failed,
                resources: e
                    .resources
                    .iter()
                    .map(|r| ConstructionResourceSubmission {
                        name: r.name.clone(),
                        display_name: r.display_name().to_string(),
                        required_amount: r.required_amount,
                        provided_amount: r.provided_amount,
                        payment: r.payment,
                    })
                    .collect(),
            };
            construction::upsert_depot(pool, event_timestamp, &submission).await?
        }
        JournalEvent::ScanBaryCentre(_)
        | JournalEvent::FssSignalDiscovered(_)
        | JournalEvent::FssDiscoveryScan(_)
        | JournalEvent::FssAllBodiesFound(_)
        | JournalEvent::NavBeaconScan(_)
        | JournalEvent::ScanOrganic(_)
        | JournalEvent::SupercruiseExit(_) => {}
    }
    Ok(())
}

pub fn build_pool(db_url: &str) -> anyhow::Result<Pool> {
    // Parse "host=... port=... user=... password=... dbname=..." into PoolConfig
    let mut cfg = PoolConfig::new();
    for pair in db_url.split_whitespace() {
        if let Some((key, val)) = pair.split_once('=') {
            match key {
                "host" => cfg.host = Some(val.into()),
                "port" => cfg.port = Some(val.parse()?),
                "user" => cfg.user = Some(val.into()),
                "password" => cfg.password = Some(val.into()),
                "dbname" => cfg.dbname = Some(val.into()),
                _ => {}
            }
        }
    }
    let pool = cfg
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .map_err(|e| anyhow::anyhow!("failed to create db pool: {e}"))?;
    Ok(pool)
}
