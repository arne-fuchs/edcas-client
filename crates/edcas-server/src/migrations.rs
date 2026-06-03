//! Minimal embedded database migration runner.
//!
//! Migrations are applied automatically on server startup, tracked in the
//! `schema_migrations` table so each runs exactly once, in order. The SQL is embedded
//! into the binary with `include_str!`, so a deploy is self-contained — no manual
//! `psql -f` step and no schema drift between the code and the database.
//!
//! ## Adding a migration
//! Append a new `(version, include_str!(...))` entry to [`MIGRATIONS`]. **Never edit or
//! reorder existing entries** — they may already be recorded as applied in production.
//! `0001_initial_schema` is the full canonical schema (`schema.sql`); it is written with
//! `CREATE TABLE/INDEX IF NOT EXISTS`, so it also safely "adopts" a pre-existing database
//! by creating only the objects that are missing.

use deadpool_postgres::Pool;
use tracing::info;

/// Ordered list of `(version, sql)`. Append-only.
const MIGRATIONS: &[(&str, &str)] = &[("0001_initial_schema", include_str!("../schema.sql"))];

/// Applies any not-yet-applied migrations. Idempotent: safe to run on every startup.
pub async fn run(pool: &Pool) -> anyhow::Result<()> {
    let mut client = pool.get().await?;

    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                 version    TEXT PRIMARY KEY,
                 applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
             )",
        )
        .await?;

    for (version, sql) in MIGRATIONS {
        let already_applied: bool = client
            .query_one(
                "SELECT EXISTS (SELECT 1 FROM schema_migrations WHERE version = $1)",
                &[version],
            )
            .await?
            .get(0);
        if already_applied {
            continue;
        }

        info!("Applying migration {version}");
        let tx = client.transaction().await?;
        tx.batch_execute(sql).await?;
        tx.execute(
            "INSERT INTO schema_migrations (version) VALUES ($1)",
            &[version],
        )
        .await?;
        tx.commit().await?;
        info!("Migration {version} applied");
    }

    Ok(())
}
