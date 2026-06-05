use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

const MIN_TICK_INTERVAL_SECS: i64 = 20 * 3600; // ticks cannot be closer than 20 h
const FALLBACK_INTERVAL_SECS: i64 = 24 * 3600;

/// Once the predicted tick time has passed we keep returning it (rather than rolling
/// straight to the next cycle) for this long, so clients display "Tick Imminent" for a
/// full window instead of a flicker. The real tick lands somewhere inside it.
const IMMINENT_WINDOW_SECS: i64 = 15 * 60;

// Minimum rows in faction_influence_history within the last 48 h before we trust
// it for tick detection. Below this threshold the table is too new and we fall back
// to the slower JSONB scan of journal_events.
const MIN_HISTORY_ROWS_FOR_DETECTION: i64 = 500;

/// Fetches the last 14 ticks and returns (last_tick, next_predicted_tick, system_count).
/// Intervals shorter than 20 h are treated as false positives and excluded from the average.
/// Falls back to +24 h if no valid intervals exist. Once the predicted time passes it is held
/// (kept <= now) for `IMMINENT_WINDOW_SECS` so clients show "Tick Imminent" for a full window
/// rather than a flicker, then rolls forward to the next cycle.
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
    // Hold the predicted time in the past for IMMINENT_WINDOW_SECS after it passes — that
    // keeps next_predicted_tick <= now, which is the signal the client renders as "Tick
    // Imminent". Only once the window has fully elapsed do we roll forward to the next cycle.
    while next_predicted + chrono::Duration::seconds(IMMINENT_WINDOW_SECS) <= now {
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

/// Detect the BGS tick as the moment the **first systems start reporting changed influence**.
///
/// A tick is a single global instant once per day; in the data it shows up as a sharp lift
/// off a flat lull — before the tick almost nothing is changing, then a wave of systems begin
/// reporting new influence values. We detect that leading edge directly. (Empirically the tick
/// is ~18:45–19:00 UTC; the bulk of changes that follow, peaking ~20:00–22:00, are just the
/// evening player-activity smear as commanders re-scan systems — NOT the tick.)
///
/// ## Why a *stable* reference, not the previous row
/// `faction_influence_history` is contaminated: the same (faction, system) gets OLD and NEW
/// values reported within seconds of each other for *hours* after a tick (commanders with
/// lagged / cached / replayed uploads). So `influence - LAG(influence)` over consecutive rows
/// flip-flops constantly and just tracks player traffic — the failure mode of every earlier
/// version. Instead we compare each reading to that system's **own average 2–4 h earlier**
/// (`ref`), a stable pre-window baseline. A reading only counts as "changed" if it departs
/// from `ref` by > 0.005 (0.5 pp). During the pre-tick lull current ≈ ref (both old) ⇒ ~0
/// changes; right after the tick the first fresh values diverge ⇒ the count lifts.
///
/// ## The detection rule (simple, leading-edge, self-deduping)
/// Per 15-min window: `rate = (distinct systems changed vs ref) / (distinct systems observed)`.
/// A window is the tick when its `rate` clears a small floor (≥ 0.007) **and** the preceding
/// 3 h stayed quiet (`max rate < 0.005`). The quiet-precondition is what makes this the
/// *leading edge*: only the lull→activity transition fires, so it self-limits to one tick per
/// ~24 h cycle with no calendar-day reset (the post-tick smear and the next day's pre-dawn tail
/// never satisfy "preceding 3 h was quiet"). `following >= 6` requires ~1.5 h of data after the
/// candidate before emitting it (so the current day's tick lands with a ~1.5–2 h confirmation
/// lag). Windows with < 50 observed systems are dropped to ignore ingestion gaps.
///
/// `system_count` = distinct systems that genuinely changed (vs `ref`) within 24 h of the tick
/// (real tick coverage). Returns (tick_time, system_count, tick_date), newest first. Uses the
/// indexed faction_influence_history table (no JSONB).
async fn detect_from_influence_history(
    client: &tokio_postgres::Client,
) -> anyhow::Result<Vec<tokio_postgres::Row>> {
    Ok(client
        .query(
            r#"
            WITH refd AS (
                -- Each reading paired with this faction-system's own average influence
                -- 2-4 h earlier: a stable pre-window baseline that is immune to the
                -- old/new flip-flop of interleaved lagged uploads.
                SELECT
                    system_address,
                    event_timestamp,
                    influence,
                    AVG(influence) OVER (
                        PARTITION BY faction_name, system_address
                        ORDER BY event_timestamp
                        RANGE BETWEEN INTERVAL '4 hours' PRECEDING
                                  AND INTERVAL '2 hours' PRECEDING
                    ) AS ref
                FROM faction_influence_history
                WHERE event_timestamp > NOW() - INTERVAL '98 hours'
            ),
            windows AS (
                -- Per 15-min window: distinct systems observed, and distinct systems whose
                -- influence has departed from its 2-4 h baseline by > 0.005 (a genuine change,
                -- not the per-row noise). Drop windows with < 50 observed (ingestion gaps).
                SELECT
                    date_trunc('hour', event_timestamp) +
                        make_interval(mins => (EXTRACT(MINUTE FROM event_timestamp)::int / 15) * 15)
                        AS window_start,
                    COUNT(DISTINCT system_address) AS observed,
                    COUNT(DISTINCT system_address)
                        FILTER (WHERE ref IS NOT NULL AND ABS(influence - ref) > 0.005) AS changed
                FROM refd
                GROUP BY 1
                HAVING COUNT(DISTINCT system_address) >= 50
            ),
            rated AS (
                SELECT window_start, changed::float / observed AS rate
                FROM windows
            ),
            flagged AS (
                -- For each window: the highest change-rate in the preceding 3 h (the lull
                -- level), and how many windows of data follow (confirmation guard).
                SELECT
                    window_start,
                    rate,
                    MAX(rate) OVER (
                        ORDER BY window_start
                        RANGE BETWEEN INTERVAL '3 hours' PRECEDING
                                  AND INTERVAL '15 minutes' PRECEDING
                    ) AS prior_max_rate,
                    COUNT(*) OVER (
                        ORDER BY window_start
                        RANGE BETWEEN INTERVAL '15 minutes' FOLLOWING
                                  AND INTERVAL '2 hours' FOLLOWING
                    ) AS following
                FROM rated
            ),
            ticks AS (
                -- The tick is the leading edge: the change-rate clears the floor while the
                -- preceding 3 h was quiet. That quiet-precondition fires only on the
                -- lull->activity transition, so it self-limits to one per ~24 h cycle.
                SELECT window_start AS tick_time,
                       (window_start AT TIME ZONE 'UTC')::date AS tick_date
                FROM flagged
                WHERE rate >= 0.007
                  AND prior_max_rate <= 0.005
                  AND following >= 6
            )
            SELECT
                t.tick_time,
                (
                    SELECT COUNT(DISTINCT r.system_address)::int
                    FROM refd r
                    WHERE r.ref IS NOT NULL
                      AND ABS(r.influence - r.ref) > 0.005
                      AND r.event_timestamp >= t.tick_time
                      AND r.event_timestamp <  t.tick_time + INTERVAL '24 hours'
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
