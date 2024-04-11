use std::sync::Arc;

use eframe::egui::Ui;
use num_format::{Locale, ToFormattedString};

use crate::app::explorer::body::{Parent, Signal};
use crate::app::settings::Settings;
#[derive(Clone)]
pub struct Ring {
    pub timestamp: String,
    pub event: String,
    pub scan_type: String,
    pub body_name: String,
    pub body_id: i64,
    pub parents: Vec<Parent>,
    pub star_system: String,
    pub system_address: i64,
    pub distance_from_arrival_ls: f64,
    pub semi_major_axis: f64,
    pub eccentricity: f64,
    pub orbital_inclination: f64,
    pub periapsis: f64,
    pub orbital_period: f64,
    pub ascending_node: f64,
    pub mean_anomaly: f64,
    pub was_discovered: bool,
    pub was_mapped: bool,
    pub ring_signals: Vec<Signal>,
    pub settings: Arc<Settings>,
}