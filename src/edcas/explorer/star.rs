use std::sync::Arc;

use crate::edcas::explorer::body::Parent;
use crate::edcas::explorer::planet::AsteroidRing;
use crate::edcas::settings::Settings;
#[derive(Clone)]
pub struct Star {
    pub timestamp: String,
    pub event: String,
    pub scan_type: String,
    pub body_name: String,
    pub body_id: i64,
    pub parents: Vec<Parent>,
    pub star_system: String,
    pub system_address: i64,
    pub distance_from_arrival_ls: f64,
    pub star_type: String,
    pub subclass: i64,
    pub stellar_mass: f64,
    pub radius: f64,
    pub absolute_magnitude: f64,
    pub age_my: i64,
    pub surface_temperature: f64,
    pub luminosity: String,
    pub semi_major_axis: Option<f64>,
    pub eccentricity: Option<f64>,
    pub orbital_inclination: Option<f64>,
    pub periapsis: Option<f64>,
    pub orbital_period: Option<f64>,
    pub ascending_node: Option<f64>,
    pub mean_anomaly: Option<f64>,
    pub rotation_period: f64,
    pub axial_tilt: f64,
    pub was_discovered: bool,
    pub was_mapped: bool,
    pub asteroid_rings: Vec<AsteroidRing>,
    pub settings: Arc<Settings>,
}
