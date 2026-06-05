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

    let mut rows = if history_rows >= MIN_HISTORY_ROWS_FOR_DETECTION {
        info!("tick detector: using faction_influence_history ({history_rows} rows in 48 h)");
        detect_from_influence_history(&client).await?
    } else {
        info!(
            "tick detector: history table has only {history_rows} rows (need {MIN_HISTORY_ROWS_FOR_DETECTION}), using journal_events fallback"
        );
        detect_from_journal_events(&client).await?
    };

    // Bootstrap bridge: the influence-history method can only flag a change when a
    // faction-system has a *previous* reading to diff against. Right after the history
    // table starts filling up, almost every row is a first-seen value, so it yields no
    // candidates even though there is plenty of journal data. In that case fall back to
    // the journal_events traffic-peak method so we still record a (rougher) tick; this
    // self-upgrades to the accurate influence method once two dense days exist.
    if rows.is_empty() && history_rows >= MIN_HISTORY_ROWS_FOR_DETECTION {
        info!("tick detector: influence history too shallow to diff yet, falling back to journal_events");
        rows = detect_from_journal_events(&client).await?;
    }

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
        info!("server tick {tick_date} at {tick_time} ({system_count} systems)");
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
/// observation after the tick. Between ticks the same value is re-reported (delta 0).
///
/// The hard part: with dense, global data a single tick's first-observations are smeared
/// across the whole *following* day, because commanders visit ~thousands of systems at
/// scattered times. The raw count of changing systems therefore tracks the diurnal
/// player-activity curve (evening peak, early-morning trough), *not* the tick — so any
/// "leading edge of raw counts" locks onto the daily player ramp, not the BGS tick. (The
/// old version used the earliest window reaching 50 % of the day's peak and, once the
/// table filled up 24/7, degenerated to the first window after 00:00 UTC every day.)
///
/// The signal that cancels player activity is the **normalized change rate** =
/// (systems that changed) / (systems observed) per 15-min window. Right after a tick
/// nearly every visited system reports a fresh value, so the rate is high; as the cycle
/// ages, repeat visits dominate and the rate decays to a trough just before the next
/// tick. We smooth that rate and find the **deepest minimum of each ~24 h cycle** — the
/// single pre-tick lull — then place the tick one window after it. The minimum is taken
/// over the preceding ~11 h (not a narrow +/- 3 h window, which shallow mid-decline noise
/// dips would satisfy), so only the genuine cycle-wide lull qualifies. This is
/// calendar-day-agnostic (no 00:00 reset), robust to the diurnal curve, and yields one
/// tick per ~24 h cycle.
///
/// Windows with < 30 observed systems are dropped so ingestion gaps (e.g. a server
/// restart) can't masquerade as a change lull. The most recent trough is only emitted
/// once it has >= 2 h of following data to confirm it as a real minimum (the cost is a
/// ~2-3 h confirmation lag on the current day's tick). `system_count` is the distinct
/// systems that changed in the 24 h following the tick (tick coverage). Returns
/// (tick_time, system_count, tick_date), newest first.
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
                -- back a few days so several tick cycles are visible at once.
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
                WHERE event_timestamp > NOW() - INTERVAL '98 hours'
            ),
            windows AS (
                -- Per 15-min window: how many distinct systems were observed at all, and
                -- how many showed an influence change (delta > 0.001 = > 0.1 pp in 0-1
                -- scale). Drop windows with < 30 observed systems so ingestion gaps don't
                -- look like change lulls.
                SELECT
                    date_trunc('hour', event_timestamp) +
                        make_interval(mins => (EXTRACT(MINUTE FROM event_timestamp)::int / 15) * 15)
                        AS window_start,
                    COUNT(DISTINCT system_address)                                  AS observed,
                    COUNT(DISTINCT system_address) FILTER (WHERE delta > 0.001)     AS changed
                FROM influence_changes
                GROUP BY 1
                HAVING COUNT(DISTINCT system_address) >= 30
            ),
            smoothed AS (
                -- Normalized change rate, smoothed with a centered 9-window moving average
                -- to suppress 15-min noise.
                SELECT
                    window_start,
                    AVG(changed::float / observed) OVER (
                        ORDER BY window_start ROWS BETWEEN 4 PRECEDING AND 4 FOLLOWING
                    ) AS rate_ma
                FROM windows
            ),
            troughs AS (
                -- A window is *the* pre-tick lull when its smoothed change-rate is the lowest
                -- across the preceding ~11 h (most of a full tick cycle) AND the next 2 h, i.e.
                -- the rate has already bottomed out and turned back up.
                --
                -- The old test only compared against +/- 3 h, so during the long, gentle decline
                -- from the post-tick peak any shallow noise dip was trivially "the minimum within
                -- 3 h" and got mistaken for the lull — pinning the tick to a random daytime window
                -- (e.g. a bogus 10:45 when the real trough/tick is ~17:45). Requiring the deepest
                -- point of the whole cycle isolates the single genuine lull that precedes a tick;
                -- the 8 FOLLOWING (rate has risen again) is what makes it a trough rather than just
                -- the lowest-so-far point on the way down.
                SELECT
                    window_start,
                    rate_ma,
                    MIN(rate_ma) OVER (
                        ORDER BY window_start ROWS BETWEEN 44 PRECEDING AND 8 FOLLOWING
                    ) AS local_min,
                    COUNT(*) OVER (
                        ORDER BY window_start ROWS BETWEEN 1 FOLLOWING AND 12 FOLLOWING
                    ) AS following
                FROM smoothed
            ),
            ticks AS (
                -- The tick onset is the window just after the lull bottom. One per UTC day
                -- (the 24 h cadence keeps troughs > 6 h apart, so this only de-dupes ties).
                SELECT DISTINCT ON ((window_start AT TIME ZONE 'UTC')::date)
                    window_start + INTERVAL '15 minutes'   AS tick_time,
                    (window_start AT TIME ZONE 'UTC')::date AS tick_date
                FROM troughs
                WHERE rate_ma = local_min
                  AND following >= 8
                ORDER BY (window_start AT TIME ZONE 'UTC')::date, window_start
            )
            SELECT
                t.tick_time,
                (
                    SELECT COUNT(DISTINCT ic.system_address)::int
                    FROM influence_changes ic
                    WHERE ic.delta > 0.001
                      AND ic.event_timestamp >= t.tick_time
                      AND ic.event_timestamp <  t.tick_time + INTERVAL '24 hours'
                ) AS system_count,
                t.tick_date
            FROM ticks t
            ORDER BY t.tick_time DESC
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
