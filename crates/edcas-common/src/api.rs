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
    pub market_id: Option<i64>,
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
    pub conflict: Option<ConflictInfo>,
}

/// Conflict data for war-like states (War, CivilWar, Election).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConflictInfo {
    pub war_type: String,
    pub status: String,
    pub opponent_name: String,
    pub our_won_days: i32,
    pub opponent_won_days: i32,
    pub our_stake: Option<String>,
    pub opponent_stake: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct FactionQuery {
    pub name: Option<String>,
    pub limit: Option<i64>,
}

/// GET /api/v1/construction-depots — query params: name, system_name, market_id, limit
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConstructionDepotResponse {
    pub market_id: i64,
    pub system_address: i64,
    pub system_name: String,
    pub station_name: String,
    pub progress: f32,
    pub construction_complete: bool,
    pub construction_failed: bool,
    pub last_updated: String,
    pub resources: Vec<ConstructionResourceResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConstructionResourceResponse {
    pub name: String,
    pub display_name: String,
    pub required_amount: i32,
    pub provided_amount: i32,
    pub payment: i64,
}

/// Query parameters for construction depot search
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ConstructionQuery {
    pub name: Option<String>,
    pub system_name: Option<String>,
    pub market_id: Option<i64>,
    pub limit: Option<i64>,
}

/// GET /api/v1/trade-routes
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TradeRouteResponse {
    pub from_market_id: i64,
    pub to_market_id: i64,
    pub commodity: String,
    pub buy_price: i32,
    pub sell_price: i32,
    pub profit: i32,
    pub supply: i32,
    pub demand: i32,
    pub distance_ly: f32,
    pub from_station_name: String,
    pub to_station_name: String,
    pub from_system_name: String,
    pub to_system_name: String,
    pub from_max_pad: Option<String>,
    pub to_max_pad: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TradeRouteQuery {
    pub system_address: Option<i64>,
    pub max_distance: Option<f32>,
    pub pad_size: Option<String>,
    pub min_profit: Option<i32>,
    pub limit: Option<i64>,
}

/// POST /api/v1/construction-depots — client submits depot data from journal
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConstructionDepotSubmission {
    pub market_id: i64,
    pub system_address: i64,
    pub station_name: String,
    pub progress: f32,
    pub construction_complete: bool,
    pub construction_failed: bool,
    pub resources: Vec<ConstructionResourceSubmission>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConstructionResourceSubmission {
    pub name: String,
    pub display_name: String,
    pub required_amount: i32,
    pub provided_amount: i32,
    pub payment: i64,
}
