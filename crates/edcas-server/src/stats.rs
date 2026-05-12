use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tracing::info;

pub static EDDN_RECEIVED: AtomicU64 = AtomicU64::new(0);
pub static EDDN_DISPATCHED: AtomicU64 = AtomicU64::new(0);
pub static EDDN_ERRORS: AtomicU64 = AtomicU64::new(0);
pub static CLIENT_RECEIVED: AtomicU64 = AtomicU64::new(0);
pub static CLIENT_DISPATCHED: AtomicU64 = AtomicU64::new(0);
pub static CLIENT_SKIPPED: AtomicU64 = AtomicU64::new(0);
pub static CLIENT_ERRORS: AtomicU64 = AtomicU64::new(0);

/// Spawns a Tokio task that logs ingestion statistics every 60 seconds.
/// Counters are reset after each log line so values represent the last interval.
pub fn spawn_stats_logger() {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        interval.tick().await; // skip the immediate first tick
        loop {
            interval.tick().await;
            let eddn_recv = EDDN_RECEIVED.swap(0, Ordering::Relaxed);
            let eddn_disp = EDDN_DISPATCHED.swap(0, Ordering::Relaxed);
            let eddn_err  = EDDN_ERRORS.swap(0, Ordering::Relaxed);
            let cli_recv  = CLIENT_RECEIVED.swap(0, Ordering::Relaxed);
            let cli_disp  = CLIENT_DISPATCHED.swap(0, Ordering::Relaxed);
            let cli_skip  = CLIENT_SKIPPED.swap(0, Ordering::Relaxed);
            let cli_err   = CLIENT_ERRORS.swap(0, Ordering::Relaxed);
            info!(
                "stats [1 min] \
                 EDDN: recv={eddn_recv} dispatched={eddn_disp} errors={eddn_err} | \
                 client-upload: recv={cli_recv} dispatched={cli_disp} skipped={cli_skip} errors={cli_err}"
            );
        }
    });
}
