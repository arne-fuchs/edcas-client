use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use crate::edcas::cargo_reader::CargoReader;

pub struct Mining {
    pub prospectors: VecDeque<Prospector>,
    pub cargo: Arc<Mutex<CargoReader>>,
}

pub struct MiningMaterial {
    pub name: String,
    pub name_localised: String,
    pub proportion: f64,
    pub buy_price: f64,
}

pub struct Prospector {
    pub timestamp: String,
    pub event: String,
    pub materials: Vec<MiningMaterial>,
    pub content: String,
    pub content_localised: String,
    pub remaining: f64,
}
