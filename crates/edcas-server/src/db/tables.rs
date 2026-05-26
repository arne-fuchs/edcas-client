use tokio_postgres::{Client, Transaction};

const ALLOWED_LOOKUP_TABLES: &[&str] = &[
    "allegiance",
    "atmosphere",
    "atmosphere_type",
    "economy_type",
    "faction_state_name",
    "government",
    "happiness",
    "material_type",
    "planet_class",
    "planet_composition_type",
    "power",
    "ring_class",
    "security",
    "signal_type",
    "star_type",
    "station_services_types",
    "station_type",
    "terraform_state",
    "volcanism",
    "war_type",
];

fn assert_allowed_table(table: &str) {
    assert!(
        ALLOWED_LOOKUP_TABLES.contains(&table),
        "lookup_or_insert called with unexpected table name: {table:?}"
    );
}

/// Ensures a value exists in a lookup table (INSERT ... ON CONFLICT DO NOTHING)
/// and returns its integer id.
pub async fn lookup_or_insert(
    tx: &Transaction<'_>,
    table: &str,
    value: &str,
    journal_id: i64,
) -> anyhow::Result<i32> {
    assert_allowed_table(table);
    let insert_sql = format!(
        "INSERT INTO {table} (value, journal_id) VALUES ($1, $2) ON CONFLICT (value) DO NOTHING"
    );
    tx.execute(insert_sql.as_str(), &[&value, &journal_id])
        .await?;

    let select_sql = format!("SELECT id FROM {table} WHERE value = $1");
    let row = tx.query_one(select_sql.as_str(), &[&value]).await?;
    Ok(row.get(0))
}

/// Like `lookup_or_insert` but operates on a plain `Client` (no transaction).
pub async fn lookup_or_insert_client(
    client: &Client,
    table: &str,
    value: &str,
    journal_id: i64,
) -> anyhow::Result<i32> {
    assert_allowed_table(table);
    let insert_sql = format!(
        "INSERT INTO {table} (value, journal_id) VALUES ($1, $2) ON CONFLICT (value) DO NOTHING"
    );
    client
        .execute(insert_sql.as_str(), &[&value, &journal_id])
        .await?;

    let select_sql = format!("SELECT id FROM {table} WHERE value = $1");
    let row = client.query_one(select_sql.as_str(), &[&value]).await?;
    Ok(row.get(0))
}
