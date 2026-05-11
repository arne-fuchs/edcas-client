use serde::{Deserialize, Serialize};

use crate::journal::types::{Conflict, Faction, SystemFaction};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FsdJump {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "StarSystem")]
    pub star_system: String,
    #[serde(rename = "SystemAddress")]
    pub system_address: i64,
    #[serde(rename = "StarPos")]
    pub star_pos: Vec<f32>,
    #[serde(rename = "Body", default)]
    pub body: String,
    #[serde(rename = "BodyID", default)]
    pub body_id: i32,
    #[serde(rename = "BodyType", default)]
    pub body_type: String,
    #[serde(rename = "Population", default)]
    pub population: i64,
    #[serde(rename = "SystemEconomy", default)]
    pub system_economy: String,
    #[serde(rename = "SystemSecondEconomy", default)]
    pub system_second_economy: String,
    #[serde(rename = "SystemGovernment", default)]
    pub system_government: String,
    #[serde(rename = "SystemAllegiance", default)]
    pub system_allegiance: String,
    #[serde(rename = "SystemSecurity", default)]
    pub system_security: String,
    #[serde(rename = "Factions")]
    pub factions: Option<Vec<Faction>>,
    #[serde(rename = "SystemFaction")]
    pub system_faction: Option<SystemFaction>,
    #[serde(rename = "Conflicts")]
    pub conflicts: Option<Vec<Conflict>>,
    #[serde(rename = "ControllingPower")]
    pub controlling_power: Option<String>,
    #[serde(rename = "Powers")]
    pub powers: Option<Vec<String>>,
    #[serde(rename = "Multicrew", default)]
    pub multicrew: bool,
    #[serde(rename = "JumpDist")]
    pub jump_dist: Option<f32>,
    #[serde(rename = "FuelUsed")]
    pub fuel_used: Option<f32>,
    #[serde(rename = "FuelLevel")]
    pub fuel_level: Option<f32>,
    #[serde(rename = "horizons", default)]
    pub horizons: bool,
    #[serde(rename = "odyssey", default)]
    pub odyssey: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Location {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "StarSystem")]
    pub star_system: String,
    #[serde(rename = "SystemAddress")]
    pub system_address: i64,
    #[serde(rename = "StarPos")]
    pub star_pos: Vec<f32>,
    #[serde(rename = "Body", default)]
    pub body: String,
    #[serde(rename = "BodyID", default)]
    pub body_id: i64,
    #[serde(rename = "BodyType", default)]
    pub body_type: String,
    #[serde(rename = "Population", default)]
    pub population: i64,
    #[serde(rename = "SystemEconomy", default)]
    pub system_economy: String,
    #[serde(rename = "SystemSecondEconomy", default)]
    pub system_second_economy: String,
    #[serde(rename = "SystemGovernment", default)]
    pub system_government: String,
    #[serde(rename = "SystemAllegiance", default)]
    pub system_allegiance: String,
    #[serde(rename = "SystemSecurity", default)]
    pub system_security: String,
    #[serde(rename = "Docked", default)]
    pub docked: bool,
    #[serde(rename = "Factions")]
    pub factions: Option<Vec<Faction>>,
    #[serde(rename = "SystemFaction")]
    pub system_faction: Option<SystemFaction>,
    #[serde(rename = "Conflicts")]
    pub conflicts: Option<Vec<Conflict>>,
    #[serde(rename = "ControllingPower")]
    pub controlling_power: Option<String>,
    #[serde(rename = "Powers")]
    pub powers: Option<Vec<String>>,
    // Station fields (present when Docked = true)
    #[serde(rename = "StationName")]
    pub station_name: Option<String>,
    #[serde(rename = "MarketID")]
    pub market_id: Option<i64>,
    #[serde(rename = "StationType")]
    pub station_type: Option<String>,
    #[serde(rename = "StationGovernment")]
    pub station_government: Option<String>,
    #[serde(rename = "StationAllegiance")]
    pub station_allegiance: Option<String>,
    #[serde(rename = "StationEconomy")]
    pub station_economy: Option<String>,
    #[serde(rename = "StationEconomies")]
    pub station_economies: Option<Vec<crate::journal::types::StationEconomy>>,
    #[serde(rename = "StationFaction")]
    pub station_faction: Option<crate::journal::types::StationFaction>,
    #[serde(rename = "StationServices")]
    pub station_services: Option<Vec<String>>,
    #[serde(rename = "DistFromStarLS")]
    pub dist_from_star_ls: Option<f32>,
    #[serde(rename = "Taxi", default)]
    pub taxi: bool,
    #[serde(rename = "Multicrew", default)]
    pub multicrew: bool,
    #[serde(rename = "horizons", default)]
    pub horizons: bool,
    #[serde(rename = "odyssey", default)]
    pub odyssey: bool,
}

/// Fleet carrier jump. MarketID is absent when the player is on foot inside the carrier.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CarrierJump {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "StarSystem")]
    pub star_system: String,
    #[serde(rename = "SystemAddress")]
    pub system_address: i64,
    #[serde(rename = "StarPos")]
    pub star_pos: Vec<f32>,
    #[serde(rename = "Body", default)]
    pub body: String,
    #[serde(rename = "BodyID", default)]
    pub body_id: i32,
    #[serde(rename = "BodyType", default)]
    pub body_type: String,
    #[serde(rename = "Population", default)]
    pub population: i64,
    #[serde(rename = "SystemEconomy", default)]
    pub system_economy: String,
    #[serde(rename = "SystemSecondEconomy", default)]
    pub system_second_economy: String,
    #[serde(rename = "SystemGovernment", default)]
    pub system_government: String,
    #[serde(rename = "SystemAllegiance", default)]
    pub system_allegiance: String,
    #[serde(rename = "SystemSecurity", default)]
    pub system_security: String,
    #[serde(rename = "MarketID")]
    pub market_id: Option<i64>,
    #[serde(rename = "StationName")]
    pub station_name: Option<String>,
    #[serde(rename = "StationType")]
    pub station_type: Option<String>,
    #[serde(rename = "StationEconomy")]
    pub station_economy: Option<String>,
    #[serde(rename = "StationEconomies")]
    pub station_economies: Option<Vec<crate::journal::types::StationEconomy>>,
    #[serde(rename = "StationFaction")]
    pub station_faction: Option<crate::journal::types::StationFaction>,
    #[serde(rename = "StationServices")]
    pub station_services: Option<Vec<String>>,
    #[serde(rename = "Factions")]
    pub factions: Option<Vec<Faction>>,
    #[serde(rename = "SystemFaction")]
    pub system_faction: Option<SystemFaction>,
    #[serde(rename = "Conflicts")]
    pub conflicts: Option<Vec<Conflict>>,
    #[serde(rename = "ControllingPower")]
    pub controlling_power: Option<String>,
    #[serde(rename = "Powers")]
    pub powers: Option<Vec<String>>,
    #[serde(rename = "Docked", default)]
    pub docked: bool,
    #[serde(rename = "horizons", default)]
    pub horizons: bool,
    #[serde(rename = "odyssey", default)]
    pub odyssey: bool,
}
