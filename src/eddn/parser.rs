use log::{error, info, warn};
use postgres::{Client, NoTls};
use std::thread;
use std::time::{Duration, Instant};

use crate::eddn::interpreter;

pub fn run() {
    let user = std::env::var("DB_USER").expect("No DB_USER set in environment variables");
    let password =
        std::env::var("DB_PASSWORD").expect("No DB_PASSWORD set in environment variables");
    let host = std::env::var("DB_HOST").unwrap_or("localhost".to_string());
    let db_url = format!("host={host} user={user} password={password}");
    let mut client = Client::connect(db_url.as_str(), NoTls).expect("Couldn't connect to postgres");

    //language=postgresql
    let sql = "SELECT id,event_type,data FROM journal_events WHERE parsed=false ORDER BY timestamp ASC LIMIT 100;";

    loop {
        while client.is_closed() || client.is_valid(Duration::from_secs(1)).is_err() {
            warn!("Postgres client closed -> reconnecting");
            thread::sleep(Duration::from_millis(100));
            if let Ok(conn) = Client::connect(db_url.as_str(), NoTls) {
                client = conn;
            }
        }

        let rows = match client.query(sql, &[]) {
            Ok(rows) => {
                rows
            },
            Err(err) => {
                error!("Couldn't get journal logs: {}",err);
                std::thread::sleep(Duration::from_secs(1));
                continue;
            },
        };

        for row in rows{
            let journal_id:i64 = row.get(0);
            let event:&str = row.get(1);
            let json: serde_json::value::Value = row.get(2);

            let now = Instant::now();
            if let Err(err) = interpreter::interpret_json(journal_id, event,json::parse(json.to_string().as_str()).unwrap() , &mut client){
                error!("Interpreter resulted in error: {}",err);
            }
            if now.elapsed().as_secs() >= 1 {
                warn!(
                    "Event {} took {} second(s)",
                    journal_id,
                    now.elapsed().as_secs(),
                );
            }
            let sql = "UPDATE journal_events SET parsed=true WHERE id=$1;";
            match client.execute(sql, &[&journal_id]) {
                Ok(_) => {},
                Err(err) => {error!("Interpreter update journal resulted in error: {}",err);}
            }
        }
    }
}
