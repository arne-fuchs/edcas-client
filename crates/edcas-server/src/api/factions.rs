use deadpool_postgres::Pool;
use edcas_common::api::{ConflictInfo, FactionPresence, FactionResponse};
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
                    COALESCE(f.influence, 0.0) as influence, h.value as happiness,
                    COALESCE(je.event_timestamp, je.timestamp) as updated_at
             FROM factions f
             LEFT JOIN star_systems ss ON f.system_address = ss.system_address
             LEFT JOIN happiness h ON f.happiness = h.id
             LEFT JOIN journal_events je ON je.id = f.journal_id
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

        let conflict = fetch_conflict(&client, faction_name, saddr).await?;

        presences.push(FactionPresence {
            system_address: saddr,
            system_name: row.get("system_name"),
            influence: row.get("influence"),
            happiness: row.get("happiness"),
            active_states: active,
            pending_states: pending,
            recovering_states: recovering,
            conflict,
            updated_at: row.get("updated_at"),
        });
    }

    Ok(presences)
}

async fn fetch_conflict(
    client: &tokio_postgres::Client,
    faction_name: &str,
    system_address: i64,
) -> Result<Option<ConflictInfo>, Status> {
    let rows = client
        .query(
            "SELECT wt.value as war_type, c.status,
                    CASE WHEN c.faction1_name = $1 THEN c.faction1_won_days ELSE c.faction2_won_days END as our_won_days,
                    CASE WHEN c.faction1_name = $1 THEN c.faction2_won_days ELSE c.faction1_won_days END as opp_won_days,
                    CASE WHEN c.faction1_name = $1 THEN c.faction2_name    ELSE c.faction1_name    END as opponent_name,
                    CASE WHEN c.faction1_name = $1 THEN c.faction1_stake   ELSE c.faction2_stake   END as our_stake,
                    CASE WHEN c.faction1_name = $1 THEN c.faction2_stake   ELSE c.faction1_stake   END as opp_stake
             FROM conflicts c
             LEFT JOIN war_type wt ON c.war_type = wt.id
             WHERE c.system_address = $2
               AND (c.faction1_name = $1 OR c.faction2_name = $1)
             LIMIT 1",
            &[&faction_name, &system_address],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    if let Some(row) = rows.first() {
        Ok(Some(ConflictInfo {
            war_type: row.get::<_, Option<String>>("war_type").unwrap_or_default(),
            status: row.get::<_, Option<String>>("status").unwrap_or_default(),
            opponent_name: row.get::<_, Option<String>>("opponent_name").unwrap_or_default(),
            our_won_days: row.get::<_, Option<i32>>("our_won_days").unwrap_or(0),
            opponent_won_days: row.get::<_, Option<i32>>("opp_won_days").unwrap_or(0),
            our_stake: row.get("our_stake"),
            opponent_stake: row.get("opp_stake"),
        }))
    } else {
        Ok(None)
    }
}
