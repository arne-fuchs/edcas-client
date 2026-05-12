pub mod construction;
pub mod scan;
pub mod station;
pub mod tables;
pub mod travel;

use deadpool_postgres::{Config as PoolConfig, Pool, Runtime};
use tokio_postgres::NoTls;

pub use tables::{lookup_or_insert, lookup_or_insert_client};

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
