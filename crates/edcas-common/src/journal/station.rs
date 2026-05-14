use serde::{Deserialize, Serialize};

use crate::journal::types::{LandingPads, StationEconomy, StationFaction};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Docked {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "StationName")]
    pub station_name: String,
    #[serde(rename = "StationType")]
    pub station_type: String,
    #[serde(rename = "MarketID")]
    pub market_id: i64,
    #[serde(rename = "SystemAddress")]
    pub system_address: i64,
    #[serde(rename = "StarSystem")]
    pub star_system: String,
    #[serde(rename = "StationFaction")]
    pub station_faction: Option<StationFaction>,
    #[serde(rename = "StationGovernment", default)]
    pub station_government: String,
    #[serde(rename = "StationAllegiance", default)]
    pub station_allegiance: String,
    #[serde(rename = "StationServices")]
    pub station_services: Option<Vec<String>>,
    #[serde(rename = "StationEconomy", default)]
    pub station_economy: String,
    #[serde(rename = "StationEconomies")]
    pub station_economies: Option<Vec<StationEconomy>>,
    #[serde(rename = "LandingPads")]
    pub landing_pads: Option<LandingPads>,
    #[serde(rename = "DistFromStarLS")]
    pub dist_from_star_ls: Option<f32>,
    #[serde(rename = "Wanted", default)]
    pub wanted: bool,
    #[serde(rename = "ActiveFine", default)]
    pub active_fine: bool,
    #[serde(rename = "Taxi", default)]
    pub taxi: bool,
    #[serde(rename = "horizons", default)]
    pub horizons: bool,
    #[serde(rename = "odyssey", default)]
    pub odyssey: bool,
}

/// Emitted when the player views their fleet carrier stats panel.
/// Contains the custom owner-given name separate from the callsign.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CarrierStats {
    #[serde(rename = "CarrierID")]
    pub carrier_id: i64,
    #[serde(rename = "Callsign")]
    pub callsign: String,
    #[serde(rename = "Name")]
    pub name: String,
}

/// EDDN commodities schema (not a journal event; sent separately by the game)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Commodities {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "marketId")]
    pub market_id: i64,
    #[serde(rename = "stationName")]
    pub station_name: String,
    #[serde(rename = "systemName")]
    pub system_name: String,
    #[serde(rename = "horizons", default)]
    pub horizons: bool,
    #[serde(rename = "odyssey", default)]
    pub odyssey: bool,
    #[serde(rename = "commodities")]
    pub commodities: Vec<Commodity>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Commodity {
    pub name: String,
    #[serde(rename = "meanPrice")]
    pub mean_price: i32,
    #[serde(rename = "buyPrice")]
    pub buy_price: i32,
    #[serde(rename = "stock")]
    pub stock: i32,
    #[serde(rename = "stockBracket")]
    pub stock_bracket: i32,
    #[serde(rename = "sellPrice")]
    pub sell_price: i32,
    #[serde(rename = "demand")]
    pub demand: i32,
    #[serde(rename = "demandBracket")]
    pub demand_bracket: i32,
    #[serde(rename = "statusFlags")]
    pub status_flags: Option<Vec<String>>,
}

/// EDDN outfitting / modules schema
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Outfitting {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "marketId")]
    pub market_id: i64,
    #[serde(rename = "stationName")]
    pub station_name: String,
    #[serde(rename = "systemName")]
    pub system_name: String,
    #[serde(rename = "horizons", default)]
    pub horizons: bool,
    #[serde(rename = "odyssey", default)]
    pub odyssey: bool,
    pub modules: Vec<OutfittingModule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutfittingModule {
    pub id: i64,
    pub category: String,
    pub name: String,
    pub cost: Option<i64>,
    pub ship: Option<String>,
}

/// EDDN shipyard schema
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Shipyard {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "marketId")]
    pub market_id: i64,
    #[serde(rename = "stationName")]
    pub station_name: String,
    #[serde(rename = "systemName")]
    pub system_name: String,
    #[serde(rename = "horizons", default)]
    pub horizons: bool,
    #[serde(rename = "odyssey", default)]
    pub odyssey: bool,
    pub ships: Vec<ShipyardShip>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShipyardShip {
    pub id: i64,
    pub name: String,
    #[serde(rename = "basevalue")]
    pub base_value: Option<i64>,
}
