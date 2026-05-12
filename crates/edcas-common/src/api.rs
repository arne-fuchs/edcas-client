use serde::{Deserialize, Serialize};

/// GET /api/v1/systems/:address
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemResponse {
    pub system_address: i64,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub allegiance: Option<String>,
    pub economy: Option<String>,
    pub second_economy: Option<String>,
    pub government: Option<String>,
    pub security: Option<String>,
    pub population: Option<i64>,
    pub controlling_power: Option<String>,
    pub factions: Vec<SystemFactionInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemFactionInfo {
    pub name: String,
    pub influence: f32,
    pub government: Option<String>,
    pub allegiance: Option<String>,
    pub happiness: Option<String>,
    pub active_states: Vec<String>,
    pub pending_states: Vec<String>,
    pub recovering_states: Vec<String>,
}

/// GET /api/v1/systems/:address/bodies
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BodyResponse {
    pub id: i32,
    pub system_address: i64,
    pub name: String,
    pub is_star: bool,
    /// star_type or planet_class
    pub body_class: Option<String>,
    pub distance_from_arrival_ls: Option<f32>,
    pub radius: Option<f32>,
    pub mass_em: Option<f32>,
    pub surface_temperature: Option<f32>,
    pub surface_gravity: Option<f32>,
    pub landable: bool,
    pub atmosphere: Option<String>,
    pub volcanism: Option<String>,
    pub terraform_state: Option<String>,
    pub tidal_lock: bool,
    pub was_discovered: bool,
    pub was_mapped: bool,
    pub estimated_value: Option<i64>,
    pub rings: Vec<RingResponse>,
    pub materials: Vec<MaterialResponse>,
    pub parents: Vec<ParentResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RingResponse {
    pub name: String,
    pub ring_class: String,
    pub mass_mt: f64,
    pub inner_rad: f64,
    pub outer_rad: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MaterialResponse {
    pub name: String,
    pub percent: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ParentResponse {
    pub parent_type: String,
    pub parent_id: i32,
}

/// GET /api/v1/stations — query params: name, system_name, market_id
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StationResponse {
    pub market_id: i64,
    pub system_address: i64,
    pub system_name: String,
    pub name: String,
    pub station_type: Option<String>,
    pub faction_name: Option<String>,
    pub government: Option<String>,
    pub economy: Option<String>,
    pub economies: Vec<StationEconomyResponse>,
    pub services: Vec<String>,
    pub landing_pads: Option<LandingPadsResponse>,
    pub dist_from_star_ls: Option<f32>,
    pub commodities: Vec<CommodityResponse>,
    pub modules: Vec<ModuleResponse>,
    pub ships: Vec<ShipResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommodityResponse {
    pub name: String,
    pub mean_price: i32,
    pub buy_price: i32,
    pub stock: i32,
    pub sell_price: i32,
    pub demand: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModuleResponse {
    pub id: String,
    pub name: Option<String>,
    pub category: Option<String>,
    pub cost: i32,
    pub ship: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShipResponse {
    pub id: String,
    pub name: Option<String>,
    pub basevalue: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StationEconomyResponse {
    pub name: String,
    pub proportion: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LandingPadsResponse {
    pub small: i32,
    pub medium: i32,
    pub large: i32,
}

/// GET /api/v1/carriers — query params: name, callsign
/// Fleet carriers are stored in the stations table (type = "FleetCarrier").
pub type CarrierResponse = StationResponse;

/// Query parameters for station/carrier search
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct StationQuery {
    pub name: Option<String>,
    pub system_name: Option<String>,
    pub market_id: Option<i64>,
    pub limit: Option<i64>,
}

/// Query parameters for carrier search
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CarrierQuery {
    pub name: Option<String>,
    pub callsign: Option<String>,
    pub system_name: Option<String>,
    pub limit: Option<i64>,
}

/// One entry per unique faction; presences lists every system the faction inhabits.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FactionResponse {
    pub name: String,
    pub government: Option<String>,
    pub allegiance: Option<String>,
    pub presences: Vec<FactionPresence>,
}

/// The faction's presence in one specific system.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FactionPresence {
    pub system_address: i64,
    pub system_name: String,
    pub influence: f32,
    pub happiness: Option<String>,
    pub active_states: Vec<String>,
    pub pending_states: Vec<String>,
    pub recovering_states: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FactionQuery {
    pub name: Option<String>,
    pub limit: Option<i64>,
}
