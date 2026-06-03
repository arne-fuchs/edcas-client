use chrono::Utc;
use deadpool_postgres::Pool;
use edcas_common::api::{ConflictInfo, FactionPresence, FactionResponse, InfluencePoint};
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

    let rows = client
        .query(
            &format!(
                "SELECT DISTINCT ON (name) name, government, allegiance
                 FROM factions
                 WHERE ($1::text IS NULL OR LOWER(name) LIKE $1)
                 ORDER BY name
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
                    COALESCE(f.influence, 0.0) as influence, f.happiness,
                    COALESCE(je.event_timestamp, je.timestamp) as updated_at
             FROM factions f
             LEFT JOIN star_systems ss ON f.system_address = ss.system_address
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
                "SELECT state, status FROM faction_states
                 WHERE faction_name = $1 AND system_address = $2",
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

        let conflict = fetch_conflict(client, faction_name, saddr).await?;

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
            "SELECT war_type, status,
                    CASE WHEN faction1_name = $1 THEN faction1_won_days ELSE faction2_won_days END as our_won_days,
                    CASE WHEN faction1_name = $1 THEN faction2_won_days ELSE faction1_won_days END as opp_won_days,
                    CASE WHEN faction1_name = $1 THEN faction2_name    ELSE faction1_name    END as opponent_name,
                    CASE WHEN faction1_name = $1 THEN faction1_stake   ELSE faction2_stake   END as our_stake,
                    CASE WHEN faction1_name = $1 THEN faction2_stake   ELSE faction1_stake   END as opp_stake
             FROM conflicts
             WHERE system_address = $2
               AND (faction1_name = $1 OR faction2_name = $1)
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

#[get("/api/v1/faction-influence-history?<name>&<system_address>&<days>")]
pub async fn faction_influence_history(
    pool: &State<Pool>,
    name: String,
    system_address: i64,
    days: Option<i32>,
) -> Result<Json<Vec<InfluencePoint>>, Status> {
    let client = pool.get().await.map_err(|_| Status::ServiceUnavailable)?;
    let days = days.unwrap_or(90).clamp(1, 365) as i64;
    let cutoff = Utc::now() - chrono::Duration::days(days);
    let rows = client
        .query(
            "SELECT influence, event_timestamp
             FROM faction_influence_history
             WHERE faction_name = $1 AND system_address = $2 AND event_timestamp >= $3
             ORDER BY event_timestamp ASC",
            &[&name, &system_address, &cutoff],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;
    let points = rows
        .iter()
        .map(|r| InfluencePoint {
            influence: r.get("influence"),
            timestamp: r.get("event_timestamp"),
        })
        .collect();
    Ok(Json(points))
}
