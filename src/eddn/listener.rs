use log::{error, warn};
use postgres::{Client, NoTls};
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

use crate::eddn::eddn_adapter::EddnAdapter;

pub async fn run() {
    let (bus_writer, bus_reader) = channel::<String>();
    let eddn = EddnAdapter { bus_writer };
    thread::spawn(move || {
        let user = std::env::var("DB_USER").expect("No DB_USER set in environment variables");
        let password =
            std::env::var("DB_PASSWORD").expect("No DB_PASSWORD set in environment variables");
        let host = std::env::var("DB_HOST").unwrap_or("localhost".to_string());
        let db_url = format!("host={host} user={user} password={password}");
        let mut client =
            Client::connect(db_url.as_str(), NoTls).expect("Couldn't connect to postgres");
        loop {
            match bus_reader.recv() {
                Ok(string) => match serde_json::Value::from_str(string.as_str()) {
                    Ok(json) => {
                        match json.get("message") {
                            Some(json) => {
                                while client.is_closed() || client.is_valid(Duration::from_secs(1)).is_err()
                                {
                                    warn!("Postgres client closed -> reconnecting");
                                    thread::sleep(Duration::from_millis(100));
                                    if let Ok(conn) = Client::connect(db_url.as_str(), NoTls) {
                                        client = conn;
                                    }
                                }
                                let now = Instant::now();
                                let mut event = json["event"].as_str();
                                if event.is_none() {
                                    event = if json.get("commodities").is_some() {
                                        Some("commodities")
                                    } else if json.get("modules").is_some() {
                                        Some("modules")
                                    } else if json.get("ships").is_some() {
                                        Some("ships")
                                    } else {
                                        Some("unknown")
                                    };
                                }
                                let event = event.unwrap();
                                let current_timestamp = chrono::Utc::now();

                                let journal_id: Option<i64> = match client.query_one(
                                    // language=postgresql
                                    "INSERT INTO journal_events (timestamp, event_type, data) VALUES ($1,$2,$3) RETURNING journal_events.id;",
                                    &[&current_timestamp, &event, &json],
                                ){
                                    Ok(row) => Some(row.get(0)),
                                    Err(e) => {
                                        error!("Unable to insert json blob: {}", e);
                                        None
                                    }
                                };
                                if now.elapsed().as_secs() >= 1 {
                                    warn!(
                                        "Event {} took {} second(s)",
                                        journal_id.unwrap_or_default(),
                                        now.elapsed().as_secs(),
                                    );
                                }
                            },
                            None => todo!(),
                        }
                    }
                    Err(error) => {
                        error!("Error parsing json: {}", error);
                    }
                },
                Err(err) => {
                    error!("{}", err);
                }
            }
        }
    });
    println!("Ready!");
    eddn.subscribe_to_eddn().await;
}
