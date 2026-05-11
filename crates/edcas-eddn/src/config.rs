use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub db_host: String,
    pub db_user: String,
    pub db_password: String,
    pub db_name: String,
    pub db_port: u16,
    pub api_port: u16,
    pub eddn_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            db_host: env::var("DB_HOST").unwrap_or_else(|_| "localhost".into()),
            db_user: env::var("DB_USER")
                .map_err(|_| anyhow::anyhow!("DB_USER environment variable not set"))?,
            db_password: env::var("DB_PASSWORD")
                .map_err(|_| anyhow::anyhow!("DB_PASSWORD environment variable not set"))?,
            db_name: env::var("DB_NAME").unwrap_or_else(|_| "edcas".into()),
            db_port: env::var("DB_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(5432),
            api_port: env::var("API_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            eddn_url: env::var("EDDN_URL")
                .unwrap_or_else(|_| "tcp://eddn.edcd.io:9500".into()),
        })
    }

    pub fn db_url(&self) -> String {
        format!(
            "host={} port={} user={} password={} dbname={}",
            self.db_host, self.db_port, self.db_user, self.db_password, self.db_name
        )
    }
}
