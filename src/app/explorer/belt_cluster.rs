use std::sync::Arc;

use crate::app::explorer::body::Parent;
use crate::app::settings::Settings;

#[derive(Clone)]
pub struct BeltCluster {
    pub timestamp: String,
    pub event: String,
    pub scan_type: String,
    pub body_name: String,
    pub body_id: i64,
    pub parents: Vec<Parent>,
    pub star_system: String,
    pub system_address: i64,
    pub distance_from_arrival_ls: f64,
    pub was_discovered: bool,
    pub was_mapped: bool,
    pub settings: Arc<Settings>,
}
