use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

const MIN_TICK_INTERVAL_SECS: i64 = 20 * 3600; // ticks cannot be closer than 20 h
const FALLBACK_INTERVAL_SECS: i64 = 24 * 3600;

/// Fetches the last 14 ticks and returns (last_tick, next_predicted_tick, system_count).
/// Intervals shorter than 20 h are treated as false positives and excluded from the average.
/// Falls back to +24 h if no valid intervals exist.
pub async fn get_tick_prediction(
    pool: &Pool,
) -> anyhow::Result<Option<(DateTime<Utc>, DateTime<Utc>, i32)>> {
    let client = pool.get().await?;
    let rows = client
        .query(
            "SELECT tick_time, system_count FROM server_ticks ORDER BY tick_time DESC LIMIT 14",
            &[],
        )
        .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let last_tick: DateTime<Utc> = rows[0].get(0);
    let system_count: i32 = rows[0].get(1);

    let avg_secs = if rows.len() >= 2 {
        let mut valid_intervals: Vec<i64> = Vec::new();
        for i in 0..rows.len() - 1 {
            let newer: DateTime<Utc> = rows[i].get(0);
            let older: DateTime<Utc> = rows[i + 1].get(0);
            let secs = (newer - older).num_seconds();
            if secs >= MIN_TICK_INTERVAL_SECS {
                valid_intervals.push(secs);
            } else {
                warn!(
                    "tick prediction: ignoring suspiciously short interval {:.1} h between {} and {}",
                    secs as f64 / 3600.0,
                    older,
                    newer
                );
            }
        }
        if valid_intervals.is_empty() {
            warn!("tick prediction: no valid intervals found, falling back to 24 h");
            FALLBACK_INTERVAL_SECS
        } else {
            let avg = valid_intervals.iter().sum::<i64>() / valid_intervals.len() as i64;
            info!(
                "tick prediction: {} valid intervals averaged, interval {:.1} h",
                valid_intervals.len(),
                avg as f64 / 3600.0
            );
            avg
        }
    } else {
        FALLBACK_INTERVAL_SECS
    };

    let mut next_predicted = last_tick + chrono::Duration::seconds(avg_secs);
    let now = Utc::now();
    while next_predicted <= now {
        next_predicted += chrono::Duration::seconds(avg_secs);
    }

    Ok(Some((last_tick, next_predicted, system_count)))
}

pub fn spawn_tick_detector(pool: Pool) {
    tokio::spawn(async move {
        if let Err(e) = detect_and_store(&pool).await {
            error!("tick detection error: {e:#}");
        }
        let mut ticker = interval(Duration::from_secs(30 * 60));
        ticker.tick().await;
        loop {
            ticker.tick().await;
            if let Err(e) = detect_and_store(&pool).await {
                error!("tick detection error: {e:#}");
            }
        }
    });
}

async fn detect_and_store(pool: &Pool) -> anyhow::Result<()> {
    let client = pool.get().await?;

    // Skip detection entirely if a tick was stored within the last 20 hours to avoid
    // false positives caused by calendar-day boundary artefacts in the peak query.
    let recent: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM server_ticks WHERE tick_time > NOW() - INTERVAL '20 hours'",
            &[],
        )
        .await?
        .get(0);
    if recent > 0 {
        info!("tick detector: tick already stored within last 20 h, skipping");
        return Ok(());
    }

    let rows = client
        .query(
            r#"
            WITH windows AS (
                SELECT
                    date_trunc('hour', event_timestamp) +
                        make_interval(mins => (EXTRACT(MINUTE FROM event_timestamp)::int / 15) * 15)
                        AS window_start,
                    COUNT(DISTINCT data->>'SystemAddress')::int AS system_count
                FROM journal_events
                WHERE event_type IN ('FSDJump', 'Location', 'CarrierJump')
                  AND event_timestamp > NOW() - INTERVAL '72 hours'
                  AND data ? 'Factions'
                GROUP BY 1
            ),
            day_peaks AS (
                SELECT DISTINCT ON (DATE(window_start AT TIME ZONE 'UTC'))
                    window_start,
                    system_count
                FROM windows
                WHERE system_count >= 20
                ORDER BY DATE(window_start AT TIME ZONE 'UTC'), system_count DESC
            )
            SELECT window_start, system_count FROM day_peaks
            ORDER BY window_start DESC
            "#,
            &[],
        )
        .await?;

    let mut inserted = 0usize;
    for row in &rows {
        let tick_time: DateTime<Utc> = row.get(0);
        let system_count: i32 = row.get(1);

        let tick_hour: i64 = tick_time.timestamp() / 3600;
        let affected = client
            .execute(
                "INSERT INTO server_ticks (tick_time, system_count, tick_hour)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (tick_hour) DO NOTHING",
                &[&tick_time, &system_count, &tick_hour],
            )
            .await?;
        if affected > 0 {
            inserted += 1;
            info!("detected server tick at {tick_time} ({system_count} systems)");
        }
    }

    if inserted == 0 {
        info!("tick detector: no new ticks found");
    }

    Ok(())
}
