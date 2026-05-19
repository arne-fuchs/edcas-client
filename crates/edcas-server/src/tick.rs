use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use tokio::time::{interval, Duration};
use tracing::{error, info};

/// Fetches the last 7 ticks and returns (last_tick, next_predicted_tick, system_count).
/// The prediction is the last tick plus the average interval between the last 7 ticks.
/// Falls back to +24 h if fewer than 2 ticks are recorded.
pub async fn get_tick_prediction(
    pool: &Pool,
) -> anyhow::Result<Option<(DateTime<Utc>, DateTime<Utc>, i32)>> {
    let client = pool.get().await?;
    let rows = client
        .query(
            "SELECT tick_time, system_count FROM server_ticks ORDER BY tick_time DESC LIMIT 7",
            &[],
        )
        .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let last_tick: DateTime<Utc> = rows[0].get(0);
    let system_count: i32 = rows[0].get(1);

    let next_predicted = if rows.len() >= 2 {
        let mut total_secs = 0i64;
        for i in 0..rows.len() - 1 {
            let newer: DateTime<Utc> = rows[i].get(0);
            let older: DateTime<Utc> = rows[i + 1].get(0);
            total_secs += (newer - older).num_seconds();
        }
        let avg_secs = total_secs / (rows.len() - 1) as i64;
        info!(
            "tick prediction: {} ticks averaged, interval {:.1} h",
            rows.len(),
            avg_secs as f64 / 3600.0
        );
        last_tick + chrono::Duration::seconds(avg_secs)
    } else {
        last_tick + chrono::Duration::hours(24)
    };

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

        let affected = client
            .execute(
                r#"
                INSERT INTO server_ticks (tick_time, system_count)
                VALUES ($1, $2)
                ON CONFLICT (date_trunc('hour', tick_time)) DO NOTHING
                "#,
                &[&tick_time, &system_count],
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
