use deadpool_postgres::Pool;
use edcas_common::api::{FactionPresence, FactionResponse};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, State};

#[get("/api/v1/factions?<name>&<limit>")]
pub async fn search_factions(
    pool: &State<Pool>,
    name: Option<String>,
    limit: Option<i64>,
) -> Result<Json<Vec<FactionResponse>>, Status> {
    let client = pool.get().await.map_err(|_| Status::ServiceUnavailable)?;
    let limit = limit.unwrap_or(50).min(200);

    let name_pattern = name.as_ref().map(|n| format!("%{}%", n.to_lowercase()));

    // One row per unique faction name.
    let rows = client
        .query(
            &format!(
                "SELECT DISTINCT ON (f.name) f.name, g.value as government, a.value as allegiance
                 FROM factions f
                 LEFT JOIN government g ON f.government = g.id
                 LEFT JOIN allegiance a ON f.allegiance = a.id
                 WHERE ($1::text IS NULL OR LOWER(f.name) LIKE $1)
                 ORDER BY f.name
                 LIMIT {limit}"
            ),
            &[&name_pattern],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    let mut results = Vec::new();
    for row in &rows {
        let fname: String = row.get("name");
        let presences = fetch_presences(&client, &fname).await?;
        results.push(FactionResponse {
            name: fname,
            government: row.get("government"),
            allegiance: row.get("allegiance"),
            presences,
        });
    }

    Ok(Json(results))
}

async fn fetch_presences(
    client: &tokio_postgres::Client,
    faction_name: &str,
) -> Result<Vec<FactionPresence>, Status> {
    let rows = client
        .query(
            "SELECT f.system_address, COALESCE(ss.name, '') as system_name,
                    COALESCE(f.influence, 0.0) as influence, h.value as happiness
             FROM factions f
             LEFT JOIN star_systems ss ON f.system_address = ss.system_address
             LEFT JOIN happiness h ON f.happiness = h.id
             WHERE f.name = $1
             ORDER BY f.influence DESC NULLS LAST",
            &[&faction_name],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    let mut presences = Vec::new();
    for row in &rows {
        let saddr: i64 = row.get("system_address");

        let state_rows = client
            .query(
                "SELECT fsn.value as state, fs.status
                 FROM faction_states fs
                 LEFT JOIN faction_state_name fsn ON fs.state = fsn.id
                 WHERE fs.faction_name = $1 AND fs.system_address = $2",
                &[&faction_name, &saddr],
            )
            .await
            .map_err(|_| Status::InternalServerError)?;

        let mut active = Vec::new();
        let mut pending = Vec::new();
        let mut recovering = Vec::new();
        for sr in &state_rows {
            let state = sr.get::<_, Option<String>>("state").unwrap_or_default();
            let status: String = sr.get("status");
            match status.as_str() {
                "Pending"    => pending.push(state),
                "Recovering" => recovering.push(state),
                _            => active.push(state),
            }
        }

        presences.push(FactionPresence {
            system_address: saddr,
            system_name: row.get("system_name"),
            influence: row.get("influence"),
            happiness: row.get("happiness"),
            active_states: active,
            pending_states: pending,
            recovering_states: recovering,
        });
    }

    Ok(presences)
}
