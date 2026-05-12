use deadpool_postgres::Pool;
use edcas_common::api::{FactionResponse, FactionStateResponse};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{get, State};

#[get("/api/v1/factions?<name>&<system_name>&<limit>")]
pub async fn search_factions(
    pool: &State<Pool>,
    name: Option<String>,
    system_name: Option<String>,
    limit: Option<i64>,
) -> Result<Json<Vec<FactionResponse>>, Status> {
    let client = pool.get().await.map_err(|_| Status::ServiceUnavailable)?;
    let limit = limit.unwrap_or(100).min(500);

    let name_pattern = name.as_ref().map(|n| format!("%{}%", n.to_lowercase()));
    let system_pattern = system_name.as_ref().map(|s| format!("%{}%", s.to_lowercase()));

    let rows = client
        .query(
            &format!(
                "SELECT f.name, f.system_address, ss.name as system_name,
                        g.value as government, a.value as allegiance, h.value as happiness,
                        f.influence
                 FROM factions f
                 LEFT JOIN star_systems ss ON f.system_address = ss.system_address
                 LEFT JOIN government g ON f.government = g.id
                 LEFT JOIN allegiance a ON f.allegiance = a.id
                 LEFT JOIN happiness h ON f.happiness = h.id
                 WHERE ($1::text IS NULL OR LOWER(f.name) LIKE $1)
                   AND ($2::text IS NULL OR LOWER(ss.name) LIKE $2)
                 ORDER BY f.name, ss.name
                 LIMIT {limit}"
            ),
            &[&name_pattern, &system_pattern],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    let mut results = Vec::new();
    for row in &rows {
        let fname: String = row.get("name");
        let saddr: i64 = row.get("system_address");
        let states = fetch_states(&client, &fname, saddr).await?;
        results.push(FactionResponse {
            name: fname,
            system_address: saddr,
            system_name: row.get::<_, Option<String>>("system_name").unwrap_or_default(),
            government: row.get("government"),
            allegiance: row.get("allegiance"),
            happiness: row.get("happiness"),
            influence: row.get("influence"),
            states,
        });
    }

    Ok(Json(results))
}

async fn fetch_states(
    client: &tokio_postgres::Client,
    faction_name: &str,
    system_address: i64,
) -> Result<Vec<FactionStateResponse>, Status> {
    let rows = client
        .query(
            "SELECT fsn.value as state, fs.status
             FROM faction_states fs
             LEFT JOIN faction_state_name fsn ON fs.state = fsn.id
             WHERE fs.faction_name = $1 AND fs.system_address = $2",
            &[&faction_name, &system_address],
        )
        .await
        .map_err(|_| Status::InternalServerError)?;

    Ok(rows
        .iter()
        .map(|r| FactionStateResponse {
            state: r.get::<_, Option<String>>("state").unwrap_or_default(),
            status: r.get("status"),
        })
        .collect())
}
