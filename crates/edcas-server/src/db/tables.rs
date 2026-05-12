use tokio_postgres::{Client, Transaction};

/// Ensures a value exists in a lookup table (INSERT ... ON CONFLICT DO NOTHING)
/// and returns its integer id.
pub async fn lookup_or_insert(
    tx: &Transaction<'_>,
    table: &str,
    value: &str,
    journal_id: i64,
) -> anyhow::Result<i32> {
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
