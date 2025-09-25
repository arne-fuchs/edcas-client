use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Fssbodysignals {
    #[serde(rename = "SystemAddress")]
    system_address: i64,

    #[serde(rename = "StarPos")]
    star_pos: Vec<f64>,

    #[serde(rename = "BodyID")]
    body_id: i64,

    #[serde(rename = "StarSystem")]
    star_system: String,

    #[serde(rename = "Signals")]
    signals: Vec<Signal>,

    #[serde(rename = "event")]
    event: String,

    #[serde(rename = "BodyName")]
    body_name: String,

    #[serde(rename = "odyssey")]
    odyssey: bool,

    #[serde(rename = "horizons")]
    horizons: bool,

    #[serde(rename = "timestamp")]
    timestamp: String,
}

#[derive(Serialize, Deserialize)]
pub struct Signal {
    #[serde(rename = "Type")]
    signal_type: String,

    #[serde(rename = "Count")]
    count: i64,
}
