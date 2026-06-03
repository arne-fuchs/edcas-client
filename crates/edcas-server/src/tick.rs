use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

const MIN_TICK_INTERVAL_SECS: i64 = 20 * 3600; // ticks cannot be closer than 20 h
const FALLBACK_INTERVAL_SECS: i64 = 24 * 3600;

// Minimum rows in faction_influence_history within the last 48 h before we trust
// it for tick detection. Below this threshold the table is too new and we fall back
// to the slower JSONB scan of journal_events.
const MIN_HISTORY_ROWS_FOR_DETECTION: i64 = 500;

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

    // Choose detection method based on how much influence history we have. The
    // journal_events fallback only measures player *traffic* peaks, so it is a last
    // resort used while faction_influence_history is still filling up.
    let history_rows: i64 = client
        .query_one(
            "SELECT COUNT(*) FROM faction_influence_history WHERE event_timestamp > NOW() - INTERVAL '48 hours'",
            &[],
        )
        .await?
        .get(0);

    let rows = if history_rows >= MIN_HISTORY_ROWS_FOR_DETECTION {
        info!("tick detector: using faction_influence_history ({history_rows} rows in 48 h)");
        detect_from_influence_history(&client).await?
    } else {
        info!(
            "tick detector: history table has only {history_rows} rows (need {MIN_HISTORY_ROWS_FOR_DETECTION}), using journal_events fallback"
        );
        detect_from_journal_events(&client).await?
    };

    // Upsert one row per UTC day. We re-scan the last ~72 h every run and refine each
    // day's estimate as more post-tick reports arrive — the unique key is the calendar
    // day, so a day can never accumulate multiple "tick" rows.
    let mut stored = 0usize;
    for row in &rows {
        let tick_time: DateTime<Utc> = row.get(0);
        let system_count: i32 = row.get(1);
        let tick_date: chrono::NaiveDate = row.get(2);

        client
            .execute(
                "INSERT INTO server_ticks (tick_time, tick_date, system_count)
                 VALUES ($1, $2, $3)
                 ON CONFLICT (tick_date) DO UPDATE SET
                     tick_time    = EXCLUDED.tick_time,
                     system_count = EXCLUDED.system_count",
                &[&tick_time, &tick_date, &system_count],
            )
            .await?;
        stored += 1;
        info!("server tick {tick_date} at {tick_time} ({system_count} systems with influence changes)");
    }

    if stored == 0 {
        info!("tick detector: no tick candidates in the last 72 h");
    }

    Ok(())
}

/// Detect ticks from ACTUAL influence changes in faction_influence_history.
///
/// After a BGS tick a faction's influence changes; the change shows up as a non-zero
/// delta versus that faction-system's previous reading — i.e. exactly on the *first*
/// observation after the tick. Between ticks the same value is re-reported (delta 0)
/// and does not count, so this isolates real BGS activity from mere player traffic.
///
/// Because commanders visit systems at scattered times, those first-change observations
/// are spread over the hours *following* the tick. The raw peak therefore lags the tick.
/// We instead take the **leading edge**: the earliest 15-min window of the day that
/// already reaches at least half the day's peak change count. `system_count` is the
/// day's total of distinct systems that changed (a measure of tick coverage, not of a
/// single window). Returns (tick_time, system_count, tick_date), newest first.
///
/// Uses the indexed faction_influence_history table (fast, no JSONB).
async fn detect_from_influence_history(
    client: &tokio_postgres::Client,
) -> anyhow::Result<Vec<tokio_postgres::Row>> {
    Ok(client
        .query(
            r#"
            WITH influence_changes AS (
                -- Per-faction-system influence delta vs the previous reading. We look
                -- slightly beyond 72 h so LAG can see a "previous" value for entries
                -- near the 72 h boundary.
                SELECT
                    system_address,
                    event_timestamp,
                    ABS(
                        influence - LAG(influence) OVER (
                            PARTITION BY faction_name, system_address
                            ORDER BY event_timestamp
                        )
                    ) AS delta
                FROM faction_influence_history
                WHERE event_timestamp > NOW() - INTERVAL '74 hours'
            ),
            changed AS (
                -- Each system that changed, bucketed into the 15-min window of its
                -- first post-tick observation. delta > 0.001 = > 0.1 pp in 0–1 scale.
                SELECT DISTINCT
                    system_address,
                    date_trunc('hour', event_timestamp) +
                        make_interval(mins => (EXTRACT(MINUTE FROM event_timestamp)::int / 15) * 15)
                        AS window_start
                FROM influence_changes
                WHERE delta IS NOT NULL
                  AND delta > 0.001
                  AND event_timestamp > NOW() - INTERVAL '72 hours'
            ),
            windows AS (
                SELECT
                    window_start,
                    (window_start AT TIME ZONE 'UTC')::date AS tick_date,
                    COUNT(DISTINCT system_address)::int     AS win_count
                FROM changed
                GROUP BY window_start
            ),
            day_peaks AS (
                SELECT tick_date, MAX(win_count) AS peak
                FROM windows
                GROUP BY tick_date
            ),
            day_totals AS (
                SELECT
                    (window_start AT TIME ZONE 'UTC')::date AS tick_date,
                    COUNT(DISTINCT system_address)::int     AS total_systems
                FROM changed
                GROUP BY 1
            ),
            edge AS (
                -- Leading edge: earliest window of the day reaching >= 50% of the day's
                -- peak. Require a meaningful peak (>= 10 systems) to avoid noise days.
                SELECT DISTINCT ON (w.tick_date)
                    w.tick_date,
                    w.window_start
                FROM windows w
                JOIN day_peaks d USING (tick_date)
                WHERE d.peak >= 10
                  AND w.win_count >= GREATEST(5, d.peak / 2)
                ORDER BY w.tick_date, w.window_start
            )
            SELECT e.window_start AS tick_time, t.total_systems AS system_count, e.tick_date
            FROM edge e
            JOIN day_totals t USING (tick_date)
            ORDER BY e.window_start DESC
            "#,
            &[],
        )
        .await?)
}

/// Legacy detection: scan raw journal_events JSONB for 15-min windows with many
/// distinct systems reporting faction data.  Used as a fallback while the
/// faction_influence_history table is still new and sparse.
async fn detect_from_journal_events(
    client: &tokio_postgres::Client,
) -> anyhow::Result<Vec<tokio_postgres::Row>> {
    Ok(client
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
                    system_count,
                    (window_start AT TIME ZONE 'UTC')::date AS tick_date
                FROM windows
                WHERE system_count >= 20
                ORDER BY DATE(window_start AT TIME ZONE 'UTC'), system_count DESC
            )
            SELECT window_start AS tick_time, system_count, tick_date FROM day_peaks
            ORDER BY window_start DESC
            "#,
            &[],
        )
        .await?)
}

/// Returns all recorded server ticks, newest first, as (tick_time, system_count, detected_at).
pub async fn get_all_ticks(
    pool: &Pool,
) -> anyhow::Result<Vec<(DateTime<Utc>, i32, DateTime<Utc>)>> {
    let client = pool.get().await?;
    let rows = client
        .query(
            "SELECT tick_time, system_count, detected_at FROM server_ticks ORDER BY tick_time DESC",
            &[],
        )
        .await?;
    Ok(rows
        .iter()
        .map(|r| (r.get(0), r.get(1), r.get(2)))
        .collect())
}
