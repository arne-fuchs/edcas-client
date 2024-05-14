use std::sync::{Arc, Mutex};

use crate::edcas::settings::Settings;
use chrono::{DateTime, Utc};

pub struct CarrierState {
    pub carriers: Vec<Carrier>,
    pub search: String,
    pub settings: Arc<Mutex<Settings>>,
}

#[derive(Clone)]
pub struct Carrier {
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub callsign: String,
    pub services: String,
    pub docking_access: String,
    pub allow_notorious: bool,
    pub current_system: String,
    pub current_body: String,
    pub next_system: String,
    pub next_body: String,
    pub departure: DateTime<Utc>,
}
