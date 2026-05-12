use deadpool_postgres::Pool;
use edcas_common::api::{ConstructionDepotResponse, ConstructionDepotSubmission, ConstructionResourceResponse};

pub async fn upsert_depot(pool: &Pool, submission: &ConstructionDepotSubmission) -> anyhow::Result<()> {
    let mut client = pool.get().await?;
    let tx = client.build_transaction().start().await?;

    tx.execute(
        "INSERT INTO construction_depots
            (market_id, system_address, station_name, progress,
             construction_complete, construction_failed, last_updated)
         VALUES ($1,$2,$3,$4,$5,$6,NOW())
         ON CONFLICT (market_id) DO UPDATE SET
            system_address=$2, station_name=$3, progress=$4,
            construction_complete=$5, construction_failed=$6,
            last_updated=NOW()",
        &[
            &submission.market_id,
            &submission.system_address,
            &submission.station_name,
            &submission.progress,
            &submission.construction_complete,
            &submission.construction_failed,
        ],
    )
    .await?;

    // Replace resources atomically
    tx.execute(
        "DELETE FROM construction_resources WHERE market_id=$1",
        &[&submission.market_id],
    )
    .await?;

    for res in &submission.resources {
        tx.execute(
            "INSERT INTO construction_resources
                (market_id, name, display_name, required_amount, provided_amount, payment)
             VALUES ($1,$2,$3,$4,$5,$6)",
            &[
                &submission.market_id,
                &res.name,
                &res.display_name,
                &res.required_amount,
                &res.provided_amount,
                &res.payment,
            ],
        )
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn query_depots(
    pool: &Pool,
    name: Option<&str>,
    system_name: Option<&str>,
    market_id: Option<i64>,
    limit: i64,
) -> anyhow::Result<Vec<ConstructionDepotResponse>> {
    let client = pool.get().await?;
    let name_pattern = name.map(|n| format!("%{}%", n.to_lowercase()));
    let system_pattern = system_name.map(|s| format!("%{}%", s.to_lowercase()));

    let rows = client
        .query(
            &format!(
                "SELECT cd.market_id, cd.system_address,
                        COALESCE(ss.name, cd.system_address::text) as system_name,
                        cd.station_name, cd.progress,
                        cd.construction_complete, cd.construction_failed,
                        cd.last_updated::text
                 FROM construction_depots cd
                 LEFT JOIN star_systems ss ON cd.system_address = ss.system_address
                 WHERE ($1::text IS NULL OR LOWER(cd.station_name) LIKE $1)
                   AND ($2::text IS NULL OR LOWER(COALESCE(ss.name,'')) LIKE $2)
                   AND ($3::bigint IS NULL OR cd.market_id = $3)
                 ORDER BY cd.last_updated DESC
                 LIMIT {limit}"
            ),
            &[&name_pattern, &system_pattern, &market_id],
        )
        .await?;

    let mut depots = Vec::new();
    for row in &rows {
        let mid: i64 = row.get("market_id");
        let resources = fetch_resources(&client, mid).await?;
        depots.push(ConstructionDepotResponse {
            market_id: mid,
            system_address: row.get("system_address"),
            system_name: row.get("system_name"),
            station_name: row.get("station_name"),
            progress: row.get("progress"),
            construction_complete: row.get("construction_complete"),
            construction_failed: row.get("construction_failed"),
            last_updated: row.get("last_updated"),
            resources,
        });
    }
    Ok(depots)
}

async fn fetch_resources(
    client: &tokio_postgres::Client,
    market_id: i64,
) -> anyhow::Result<Vec<ConstructionResourceResponse>> {
    let rows = client
        .query(
            "SELECT name, display_name, required_amount, provided_amount, payment
             FROM construction_resources
             WHERE market_id = $1
             ORDER BY display_name",
            &[&market_id],
        )
        .await?;

    Ok(rows
        .iter()
        .map(|r| ConstructionResourceResponse {
            name: r.get("name"),
            display_name: r.get("display_name"),
            required_amount: r.get("required_amount"),
            provided_amount: r.get("provided_amount"),
            payment: r.get("payment"),
        })
        .collect())
}
