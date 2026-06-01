use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::{File, OpenOptions};
#[cfg(not(target_arch = "wasm32"))]
use std::io::{BufRead, BufReader, Seek, SeekFrom};
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc;
#[cfg(not(target_arch = "wasm32"))]
use std::thread;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, SystemTime};

use edcas_common::api::{ConstructionDepotSubmission, ConstructionResourceSubmission};
use edcas_common::journal::{
    CarrierJump, FsdJump, FssSignalDiscovered, JournalEvent, Location, Scan,
};
use edcas_common::journal::types::Conflict;
use tracing::{debug, error, info, warn};
use serde_json;

#[derive(Clone)]
pub struct ConflictData {
    pub war_type: String,
    pub status: String,
    pub opponent: String,
    pub our_won_days: i32,
    pub opponent_won_days: i32,
    pub our_stake: String,
    pub opponent_stake: String,
}

#[derive(Clone)]
pub struct FactionInfo {
    pub name: String,
    pub influence: f32,
    pub government: String,
    pub allegiance: String,
    pub happiness: String,
    pub active_states: Vec<String>,
    pub pending_states: Vec<String>,
    pub recovering_states: Vec<String>,
    pub conflict: Option<ConflictData>,
}

#[derive(Clone)]
pub struct SystemData {
    pub name: String,
    pub system_address: i64,
    pub coords: (f32, f32, f32),
    pub economy: String,
    pub second_economy: String,
    pub government: String,
    pub allegiance: String,
    pub security: String,
    pub population: i64,
    pub body: String,
    pub body_id: i32,
    pub body_type: String,
    pub factions: Vec<FactionInfo>,
    pub system_faction: String,
    pub controlling_power: Option<String>,
    pub powers: Vec<String>,
}

#[derive(Clone)]
pub struct BodyScan {
    pub body_id: i32,
    pub body_name: String,
    pub planet_class: String,
    pub landable: bool,
    pub scan_type: String,
    pub distance_from_arrival_ls: f32,
    pub radius: f32,
    pub mass_em: f32,
    pub surface_temperature: f32,
    pub surface_gravity: f32,
    pub tidal_lock: bool,
    pub volcanism: String,
    pub atmosphere: String,
    pub terraform_state: String,
    pub star_type: String,
    pub parents: Vec<BodyParent>,
    pub rings: Vec<BodyRing>,
    pub materials: Vec<BodyMaterial>,
    pub estimated_value: i64,
    pub composition: Option<BodyComposition>,
}

#[derive(Clone)]
pub struct BodyComposition {
    pub ice: f32,
    pub rock: f32,
    pub metal: f32,
}

#[derive(Clone)]
pub struct BodyParent {
    pub body_id: i32,
    pub parent_type: ParentType,
}

#[derive(Clone)]
pub enum ParentType {
    Star,
    Planet,
    Ring,
    Null,
}

#[derive(Clone)]
pub struct BodyRing {
    pub name: String,
    pub ring_class: String,
    pub mass_mt: f64,
    pub inner_rad: f64,
    pub outer_rad: f64,
}

#[derive(Clone)]
pub struct BodyMaterial {
    pub name: String,
    pub percent: f64,
}

#[derive(Clone)]
pub struct BodySignal {
    pub signal_type: String,
    pub signal_type_localised: Option<String>,
    pub count: i32,
}

#[derive(Clone)]
pub struct DiscoveredSignal {
    pub display_name: String,
    /// Raw signal type string from the journal (e.g. "FleetCarrier", "StationAsteroid", USS types).
    pub signal_type: Option<String>,
    pub uss_type: Option<String>,
    pub spawning_state: Option<String>,
    pub spawning_faction: Option<String>,
    pub threat_level: Option<i32>,
    pub time_remaining: Option<f32>,
    pub is_station: bool,
    /// Set when the journal includes `BodyID` — lets us attach this signal to a body in the tree.
    pub body_id: Option<i32>,
}

impl DiscoveredSignal {
    pub fn from_event(e: &FssSignalDiscovered) -> Self {
        let display_name = e.signal_name_localised
            .as_deref()
            .unwrap_or_else(|| clean_signal_name(&e.signal_name))
            .to_string();
        let uss_type = e.uss_type_localised.clone().or_else(|| e.uss_type.clone());
        let spawning_state = e.spawning_state_localised.clone().or_else(|| e.spawning_state.clone());
        Self {
            display_name,
            signal_type: e.signal_type.clone(),
            uss_type,
            spawning_state,
            spawning_faction: e.spawning_faction.clone(),
            threat_level: e.threat_level,
            time_remaining: e.time_remaining,
            is_station: e.is_station,
            body_id: e.body_id,
        }
    }
}

fn clean_signal_name(raw: &str) -> &str {
    raw.trim_start_matches('$').trim_end_matches(';')
}

/// Return `localised` if non-empty, otherwise strip `$..._name;` / `$...;` wrappers from `raw`.
fn localised_or_clean(localised: &str, raw: &str) -> String {
    if !localised.is_empty() {
        return localised.to_owned();
    }
    raw.trim_start_matches('$')
        .trim_end_matches("_name;")
        .trim_end_matches(';')
        .to_owned()
}

impl BodySignal {
    pub fn is_biological(&self) -> bool {
        self.signal_type.contains("Biological")
    }
    pub fn is_geological(&self) -> bool {
        self.signal_type.contains("Geological")
    }
    pub fn display_type(&self) -> &str {
        self.signal_type_localised.as_deref().unwrap_or(&self.signal_type)
    }
}

#[derive(Clone)]
pub struct SaaBodyData {
    pub signals: Vec<BodySignal>,
    pub genuses: Vec<String>,
}

#[derive(Clone)]
pub struct StationData {
    pub name: String,
    pub station_type: String,
    pub system_name: String,
    pub dist_from_star_ls: f32,
    pub services: Vec<String>,
    pub economy: String,
    pub secondary_economies: Vec<(String, f32)>,
    pub faction: String,
    pub government: String,
    pub allegiance: String,
    pub landing_pads: Option<(i32, i32, i32)>,
    pub market_id: i64,
    /// Body the station orbits, taken from the SupercruiseExit or Location BodyID.
    pub host_body_id: Option<i32>,
    /// Locally-parsed commodities from Market.json at the time of docking.
    /// Empty until Market.json is detected for this station's market_id.
    pub commodities: Vec<edcas_common::api::CommodityResponse>,
}

#[derive(Clone, Default)]
pub struct InventoryItem {
    pub name: String,
    pub localised: String,
    pub count: i32,
}

#[derive(Clone, Default)]
pub struct CargoItem {
    pub name: String,
    pub localised: String,
    pub count: i32,
    pub stolen: i32,
}

#[derive(Clone, Default)]
pub struct OnFootInventory {
    pub items: Vec<InventoryItem>,
    pub components: Vec<InventoryItem>,
    pub consumables: Vec<InventoryItem>,
    pub data: Vec<InventoryItem>,
}

#[derive(Clone)]
pub struct EngineeringModifier {
    pub label: String,
    pub value: f32,
    pub original_value: f32,
    pub less_is_good: bool,
}

#[derive(Clone)]
pub struct EngineeringData {
    pub blueprint: String,
    pub level: u8,
    pub quality: f32,
    pub engineer: String,
    pub experimental: String,
    pub modifiers: Vec<EngineeringModifier>,
}

#[derive(Clone)]
pub struct ShipModule {
    pub slot: String,
    pub item: String,
    pub power: f32,
    pub priority: u8,
    pub health: Option<f32>,
    pub value: Option<i64>,
    pub engineering: Option<EngineeringData>,
}

#[derive(Clone)]
pub struct SuitWeapon {
    pub slot: String,
    pub name: String,
    pub class: u8,
    pub mods: Vec<String>,
}

#[derive(Clone)]
pub struct SuitData {
    pub suit_type: String,
    pub grade: u8,
    pub loadout_name: String,
    pub mods: Vec<String>,
    pub weapons: Vec<SuitWeapon>,
}

#[derive(Clone)]
pub struct PilotData {
    pub name: String,
    pub credits: i64,
    pub ship_type: String,
    pub ship_name: String,
    pub ship_ident: String,
    /// Minimum landing pad size required by the current ship: 'S', 'M', or 'L'.
    /// Defaults to 'S' (no restriction) until a Loadout event is processed.
    pub ship_pad_size: char,
    pub fuel_level: f32,
    pub fuel_capacity: f32,
    pub reserve_fuel_capacity: f32,
    pub hull_health: f32,
    pub max_jump_range: f32,
    pub unladen_mass: f32,
    pub cargo_capacity: i32,
    pub modules_value: i64,
    pub rebuy: i64,
    pub game_mode: String,
    pub horizons: bool,
    pub odyssey: bool,
    pub rank_combat: u8,
    pub rank_trade: u8,
    pub rank_explore: u8,
    pub rank_soldier: u8,
    pub rank_exobiologist: u8,
    pub rank_empire: u8,
    pub rank_federation: u8,
    pub rank_cqc: u8,
    pub progress_combat: u8,
    pub progress_trade: u8,
    pub progress_explore: u8,
    pub progress_soldier: u8,
    pub progress_exobiologist: u8,
    pub progress_empire: u8,
    pub progress_federation: u8,
    pub progress_cqc: u8,
    pub reputation_empire: f32,
    pub reputation_federation: f32,
    pub reputation_alliance: f32,
    pub power: String,
    pub power_merits: i64,
    pub suit: Option<SuitData>,
}

impl Default for PilotData {
    fn default() -> Self {
        Self {
            name: String::new(),
            credits: 0,
            ship_type: String::new(),
            ship_name: String::new(),
            ship_ident: String::new(),
            ship_pad_size: 'S',
            fuel_level: 0.0,
            fuel_capacity: 0.0,
            reserve_fuel_capacity: 0.0,
            hull_health: 0.0,
            max_jump_range: 0.0,
            unladen_mass: 0.0,
            cargo_capacity: 0,
            modules_value: 0,
            rebuy: 0,
            game_mode: String::new(),
            horizons: false,
            odyssey: false,
            rank_combat: 0,
            rank_trade: 0,
            rank_explore: 0,
            rank_soldier: 0,
            rank_exobiologist: 0,
            rank_empire: 0,
            rank_federation: 0,
            rank_cqc: 0,
            progress_combat: 0,
            progress_trade: 0,
            progress_explore: 0,
            progress_soldier: 0,
            progress_exobiologist: 0,
            progress_empire: 0,
            progress_federation: 0,
            progress_cqc: 0,
            reputation_empire: 0.0,
            reputation_federation: 0.0,
            reputation_alliance: 0.0,
            power: String::new(),
            power_merits: 0,
            suit: None,
        }
    }
}

#[derive(Clone)]
pub struct OrganicScan {
    pub body_id: Option<i32>,
    pub genus: String,
    pub species: String,
    pub scan_phase: String,
}

/// A construction depot the player has visited, ready to submit to the server.
#[derive(Clone)]
pub struct ConstructionDepotData {
    pub submission: ConstructionDepotSubmission,
    pub system_name: String,
}

#[derive(Clone)]
pub struct JournalData {
    pub current_system: Option<SystemData>,
    pub bodies: Vec<BodyScan>,
    pub fss_signals: HashMap<i32, Vec<BodySignal>>,
    pub saa_data: HashMap<i32, SaaBodyData>,
    pub stations: Vec<StationData>,
    pub discovered_signals: Vec<DiscoveredSignal>,
    pub materials_raw: Vec<InventoryItem>,
    pub materials_manufactured: Vec<InventoryItem>,
    pub materials_encoded: Vec<InventoryItem>,
    pub cargo: Vec<CargoItem>,
    pub backpack: OnFootInventory,
    pub shiplocker: OnFootInventory,
    pub modules: Vec<ShipModule>,
    pub loadout_health: HashMap<String, f32>,
    pub loadout_engineering: HashMap<String, Option<EngineeringData>>,
    pub pilot: PilotData,
    /// Most recently visited construction depots, keyed by market_id.
    pub construction_depots: HashMap<i64, ConstructionDepotData>,
    /// Systems claimed by the player via ColonisationSystemClaim, keyed by system_address.
    pub claimed_systems: HashMap<i64, String>,
    /// (market_id, station_name, system_name, system_address) of the last Docked event.
    pub last_docked: Option<(i64, String, String, i64)>,
    /// Running tally of cargo transferred to each fleet carrier this session,
    /// keyed by carrier market_id → commodity name (lowercase) → count.
    /// Populated live from CargoTransfer events; no snapshot file exists so it
    /// starts at zero each session.
    pub carrier_cargo: HashMap<i64, HashMap<String, i32>>,
    /// Best-known localised display name for a normalised commodity key, built up from any
    /// event that carries a `Type_Localised` field alongside `Type`.
    pub commodity_names: HashMap<String, String>,
    /// On-foot materials stored on each fleet carrier, loaded from FCMaterials.json.
    /// Keyed by carrier market_id → (localised_name, stock).
    pub carrier_fc_materials: HashMap<i64, Vec<(String, i32)>>,
    /// BodyID from the most recent SupercruiseExit where BodyType != "Station".
    /// Cleared after consumption by the next Docked event or system jump.
    pending_host_body_id: Option<i32>,
    /// All visited systems this session, most recent first.
    pub visited_systems: Vec<SystemData>,
    /// All docked non-carrier stations this session, most recent first.
    pub visited_stations: Vec<StationData>,
    /// All docked fleet carriers this session, most recent first.
    pub visited_carriers: Vec<StationData>,
    pub fss_body_count: Option<u32>,
    pub fss_non_body_count: Option<u32>,
    pub fss_all_bodies_found: bool,
    pub nav_beacon_bodies: Option<u32>,
    pub organic_scans: Vec<OrganicScan>,
    /// Timestamp of the most recently processed journal event (ISO 8601 string).
    pub latest_event_timestamp: String,
    /// When set, CargoTransfer events for seeded carriers with timestamp ≤ this value
    /// are skipped to avoid double-counting after an in-session restart.
    /// Cleared after the initial file load so live events are never skipped.
    pub carrier_cargo_skip_before: Option<String>,
    /// Market IDs that were seeded from my_carriers.json on this startup.
    pub carrier_cargo_seeded: std::collections::HashSet<i64>,
    /// Locally-parsed commodity data from Market.json, as (market_id, commodities).
    /// Set by the watcher whenever Market.json changes; used to show market data
    /// immediately without waiting for the API round-trip.
    pub local_market: Option<(i64, Vec<edcas_common::api::CommodityResponse>)>,
}

impl JournalData {
    pub fn new() -> Self {
        Self {
            current_system: None,
            bodies: Vec::new(),
            fss_signals: HashMap::new(),
            saa_data: HashMap::new(),
            stations: Vec::new(),
            discovered_signals: Vec::new(),
            materials_raw: Vec::new(),
            materials_manufactured: Vec::new(),
            materials_encoded: Vec::new(),
            cargo: Vec::new(),
            backpack: OnFootInventory::default(),
            shiplocker: OnFootInventory::default(),
            modules: Vec::new(),
            loadout_health: HashMap::new(),
            loadout_engineering: HashMap::new(),
            pilot: PilotData::default(),
            construction_depots: HashMap::new(),
            claimed_systems: HashMap::new(),
            last_docked: None,
            carrier_cargo: HashMap::new(),
            commodity_names: HashMap::new(),
            carrier_fc_materials: HashMap::new(),
            pending_host_body_id: None,
            visited_systems: Vec::new(),
            visited_stations: Vec::new(),
            visited_carriers: Vec::new(),
            fss_body_count: None,
            fss_non_body_count: None,
            fss_all_bodies_found: false,
            nav_beacon_bodies: None,
            organic_scans: Vec::new(),
            latest_event_timestamp: String::new(),
            carrier_cargo_skip_before: None,
            carrier_cargo_seeded: std::collections::HashSet::new(),
            local_market: None,
        }
    }

    pub fn process_line(&mut self, line: &str) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return;
        }

        let value: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => return,
        };

        if let Some(ts) = value.get("timestamp").and_then(|v| v.as_str()) {
            self.latest_event_timestamp = ts.to_string();
        }

        // Loadout carries per-module health and detailed ship stats not in the typed event system.
        // Extract it before from_json consumes value.
        if value.get("event").and_then(|e| e.as_str()) == Some("Loadout") {
            let ship_internal = value["Ship"].as_str().unwrap_or("");
            self.pilot.ship_type = value["Ship_Localised"].as_str()
                .or_else(|| value["Ship"].as_str())
                .unwrap_or("").to_string();
            self.pilot.ship_pad_size = ship_pad_size(ship_internal);
            self.pilot.ship_name  = value["ShipName"].as_str().unwrap_or("").to_string();
            self.pilot.ship_ident = value["ShipIdent"].as_str().unwrap_or("").to_string();
            self.pilot.hull_health    = value["HullHealth"].as_f64().unwrap_or(0.0) as f32;
            self.pilot.max_jump_range = value["MaxJumpRange"].as_f64().unwrap_or(0.0) as f32;
            self.pilot.unladen_mass   = value["UnladenMass"].as_f64().unwrap_or(0.0) as f32;
            self.pilot.cargo_capacity = value["CargoCapacity"].as_i64().unwrap_or(0) as i32;
            self.pilot.modules_value  = value["ModulesValue"].as_i64().unwrap_or(0);
            self.pilot.rebuy          = value["Rebuy"].as_i64().unwrap_or(0);
            self.pilot.fuel_capacity  = value["FuelCapacity"]["Main"].as_f64().unwrap_or(0.0) as f32;
            self.pilot.reserve_fuel_capacity = value["FuelCapacity"]["Reserve"].as_f64().unwrap_or(0.0) as f32;

            if let Some(mods) = value["Modules"].as_array() {
                self.loadout_health.clear();
                self.loadout_engineering.clear();
                for m in mods {
                    let slot = m["Slot"].as_str().unwrap_or("").to_string();
                    if let Some(h) = m["Health"].as_f64() {
                        self.loadout_health.insert(slot.clone(), h as f32);
                    }
                    let eng = parse_engineering(&m["Engineering"]);
                    self.loadout_engineering.insert(slot, eng);
                }
                for i in 0..self.modules.len() {
                    let slot = self.modules[i].slot.clone();
                    self.modules[i].health = self.loadout_health.get(&slot).copied();
                    self.modules[i].engineering = self.loadout_engineering.get(&slot).cloned().flatten();
                }
            }
        }

        match JournalEvent::from_json(value) {
            Some(JournalEvent::FsdJump(e)) => {
                debug!("FSDJump to {}", e.star_system);
                self.pending_host_body_id = None;
                let sys = system_from_fsdjump(&e);
                let addr = sys.system_address;
                self.visited_systems.retain(|s| s.system_address != addr);
                self.visited_systems.insert(0, sys.clone());
                self.current_system = Some(sys);
                self.clear_scan_data();
                self.fss_body_count = None;
                self.fss_non_body_count = None;
                self.fss_all_bodies_found = false;
                self.nav_beacon_bodies = None;
                self.organic_scans.clear();
            }
            Some(JournalEvent::Location(e)) => {
                debug!("Location: {}", e.star_system);
                let sys = system_from_location(&e);
                let addr = sys.system_address;
                self.visited_systems.retain(|s| s.system_address != addr);
                self.visited_systems.insert(0, sys.clone());
                self.current_system = Some(sys);
                self.clear_scan_data();
                self.fss_body_count = None;
                self.fss_non_body_count = None;
                self.fss_all_bodies_found = false;
                self.nav_beacon_bodies = None;
                self.organic_scans.clear();
                // When the game starts while docked, Location carries the station data.
                if e.docked {
                    if let (Some(name), Some(market_id)) = (e.station_name, e.market_id) {
                        let station_type = e.station_type.unwrap_or_default();
                        self.last_docked = Some((market_id, name.clone(), e.star_system.clone(), e.system_address));
                        let raw_economy = e.station_economy.unwrap_or_default();
                        let raw_government = e.station_government.unwrap_or_default();
                        let secondary_economies = e.station_economies.as_ref()
                            .map(|v| v.iter().skip(1)
                                .map(|se| (localised_or_clean("", &se.name), se.proportion))
                                .collect())
                            .unwrap_or_default();
                        // Location body_id is the body the station sits on (when BodyType != "Star").
                        let host_body_id = if e.body_type != "Star" && e.body_id != 0 {
                            Some(e.body_id as i32)
                        } else {
                            None
                        };
                        let station = StationData {
                            name: name.clone(),
                            station_type: station_type.clone(),
                            system_name: e.star_system.clone(),
                            dist_from_star_ls: e.dist_from_star_ls.unwrap_or(0.0),
                            services: e.station_services.unwrap_or_default(),
                            economy: localised_or_clean(&e.station_economy_localised, &raw_economy),
                            secondary_economies,
                            faction: e.station_faction.as_ref().map(|f| f.name.clone()).unwrap_or_default(),
                            government: localised_or_clean(&e.station_government_localised, &raw_government),
                            allegiance: e.station_allegiance.unwrap_or_default(),
                            landing_pads: e.landing_pads.as_ref().map(|lp| (lp.small, lp.medium, lp.large)),
                            market_id,
                            host_body_id,
                            commodities: Vec::new(),
                        };
                        if !self.stations.iter().any(|s| s.market_id == market_id) {
                            self.stations.push(station.clone());
                        }
                        if station_type == "FleetCarrier" {
                            self.visited_carriers.retain(|s| s.market_id != market_id);
                            self.visited_carriers.insert(0, station);
                        } else {
                            let prior_commodities = self.visited_stations.iter()
                                .find(|s| s.market_id == market_id)
                                .map(|s| s.commodities.clone())
                                .unwrap_or_default();
                            self.visited_stations.retain(|s| s.market_id != market_id);
                            let mut station = station;
                            if station.commodities.is_empty() {
                                station.commodities = prior_commodities;
                            }
                            self.visited_stations.insert(0, station);
                        }
                    }
                }
            }
            Some(JournalEvent::CarrierJump(e)) => {
                debug!("CarrierJump to {}", e.star_system);
                let sys = system_from_carrier_jump(&e);
                let addr = sys.system_address;
                self.visited_systems.retain(|s| s.system_address != addr);
                self.visited_systems.insert(0, sys.clone());
                self.current_system = Some(sys);
                self.clear_scan_data();
                self.fss_body_count = None;
                self.fss_non_body_count = None;
                self.fss_all_bodies_found = false;
                self.nav_beacon_bodies = None;
                self.organic_scans.clear();
            }
            Some(JournalEvent::Scan(e)) => {
                debug!("Scan: {}", e.body_name);
                self.bodies.push(body_from_scan(&e));
            }
            Some(JournalEvent::ScanBaryCentre(e)) => {
                let parents = e.parents.as_ref()
                    .map(|pv| pv.iter()
                        .filter_map(|p| p.parent_id().map(|pid| BodyParent {
                            body_id: pid,
                            parent_type: parent_type_from_str(p.parent_type()),
                        }))
                        .collect())
                    .unwrap_or_default();
                self.bodies.push(BodyScan {
                    body_id: e.body_id,
                    body_name: format!("{} Barycentre", e.star_system),
                    planet_class: String::new(),
                    landable: false,
                    scan_type: "AutoScan".into(),
                    distance_from_arrival_ls: e.distance_from_arrival_ls,
                    radius: 0.0,
                    mass_em: 0.0,
                    surface_temperature: 0.0,
                    surface_gravity: 0.0,
                    tidal_lock: false,
                    volcanism: String::new(),
                    atmosphere: String::new(),
                    terraform_state: String::new(),
                    star_type: String::new(),
                    parents,
                    rings: vec![],
                    materials: vec![],
                    estimated_value: 0,
                    composition: None,
                });
            }
            Some(JournalEvent::FssBodySignals(e)) => {
                debug!("FSSBodySignals for body {}", e.body_id);
                let signals = e.signals.iter().map(|s| BodySignal {
                    signal_type: s.signal_type.clone(),
                    signal_type_localised: s.signal_type_localised.clone(),
                    count: s.count,
                }).collect();
                self.fss_signals.insert(e.body_id, signals);
            }
            Some(JournalEvent::SaaSignalsFound(e)) => {
                debug!("SAASignalsFound for body {}", e.body_id);
                let signals = e.signals.iter().map(|s| BodySignal {
                    signal_type: s.signal_type.clone(),
                    signal_type_localised: s.signal_type_localised.clone(),
                    count: s.count,
                }).collect();
                let genuses = e.genuses.as_ref()
                    .map(|gv| gv.iter()
                        .map(|g| g.genus_localised.clone().unwrap_or_else(|| {
                            g.genus.trim_end_matches(';').replace('_', " ")
                        }))
                        .collect())
                    .unwrap_or_default();
                self.saa_data.insert(e.body_id, SaaBodyData { signals, genuses });
            }
            Some(JournalEvent::FssSignalDiscovered(e)) => {
                debug!("FSSSignalDiscovered: {}", e.signal_name);
                self.discovered_signals.push(DiscoveredSignal::from_event(&e));
            }
            Some(JournalEvent::FssDiscoveryScan(e)) => {
                debug!("FSSDiscoveryScan: {} bodies", e.body_count);
                self.fss_body_count = Some(e.body_count);
                self.fss_non_body_count = Some(e.non_body_count);
                self.fss_all_bodies_found = false;
            }
            Some(JournalEvent::FssAllBodiesFound(_)) => {
                debug!("FSSAllBodiesFound");
                self.fss_all_bodies_found = true;
            }
            Some(JournalEvent::NavBeaconScan(e)) => {
                debug!("NavBeaconScan: {} bodies", e.num_bodies);
                self.nav_beacon_bodies = Some(e.num_bodies);
            }
            Some(JournalEvent::ScanOrganic(e)) => {
                debug!("ScanOrganic: {} {}", e.genus, e.scan_type);
                let genus = e.genus_localised.clone().unwrap_or_else(|| {
                    e.genus.trim_matches('$').trim_end_matches(';').replace('_', " ")
                });
                let species = e.species_localised.clone().unwrap_or_else(|| {
                    e.species.trim_matches('$').trim_end_matches(';').replace('_', " ")
                });
                if let Some(existing) = self.organic_scans.iter_mut()
                    .find(|s| s.body_id == e.body_id && s.genus == genus)
                {
                    existing.scan_phase = e.scan_type.clone();
                } else {
                    self.organic_scans.push(OrganicScan { body_id: e.body_id, genus, species, scan_phase: e.scan_type });
                }
            }
            None => {
                // Handle events not in the typed enum
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    match v.get("event").and_then(|e| e.as_str()) {
                        Some("Materials") => {
                            self.materials_raw = parse_inventory_array(&v["Raw"]);
                            self.materials_manufactured = parse_inventory_array(&v["Manufactured"]);
                            self.materials_encoded = parse_inventory_array(&v["Encoded"]);
                        }
                        Some("Commander") => {
                            self.pilot.name = v["Name"].as_str().unwrap_or("").to_string();
                        }
                        Some("LoadGame") => {
                            if let Some(name) = v["Commander"].as_str() {
                                if !name.is_empty() { self.pilot.name = name.to_string(); }
                            }
                            self.pilot.credits = v["Credits"].as_i64().unwrap_or(0);
                            let ship_internal = v["Ship"].as_str().unwrap_or("");
                            self.pilot.ship_type = v["Ship_Localised"].as_str()
                                .or_else(|| v["Ship"].as_str())
                                .unwrap_or("").to_string();
                            self.pilot.ship_pad_size = ship_pad_size(ship_internal);
                            self.pilot.ship_name = v["ShipName"].as_str().unwrap_or("").to_string();
                            self.pilot.ship_ident = v["ShipIdent"].as_str().unwrap_or("").to_string();
                            self.pilot.fuel_level = v["FuelLevel"].as_f64().unwrap_or(0.0) as f32;
                            self.pilot.game_mode = v["GameMode"].as_str().unwrap_or("").to_string();
                            self.pilot.horizons = v["Horizons"].as_bool().unwrap_or(false);
                            self.pilot.odyssey = v["Odyssey"].as_bool().unwrap_or(false);
                        }
                        Some("Rank") => {
                            self.pilot.rank_combat = v["Combat"].as_u64().unwrap_or(0) as u8;
                            self.pilot.rank_trade = v["Trade"].as_u64().unwrap_or(0) as u8;
                            self.pilot.rank_explore = v["Explore"].as_u64().unwrap_or(0) as u8;
                            self.pilot.rank_soldier = v["Soldier"].as_u64().unwrap_or(0) as u8;
                            self.pilot.rank_exobiologist = v["Exobiologist"].as_u64().unwrap_or(0) as u8;
                            self.pilot.rank_empire = v["Empire"].as_u64().unwrap_or(0) as u8;
                            self.pilot.rank_federation = v["Federation"].as_u64().unwrap_or(0) as u8;
                            self.pilot.rank_cqc = v["CQC"].as_u64().unwrap_or(0) as u8;
                        }
                        Some("Progress") => {
                            self.pilot.progress_combat = v["Combat"].as_u64().unwrap_or(0) as u8;
                            self.pilot.progress_trade = v["Trade"].as_u64().unwrap_or(0) as u8;
                            self.pilot.progress_explore = v["Explore"].as_u64().unwrap_or(0) as u8;
                            self.pilot.progress_soldier = v["Soldier"].as_u64().unwrap_or(0) as u8;
                            self.pilot.progress_exobiologist = v["Exobiologist"].as_u64().unwrap_or(0) as u8;
                            self.pilot.progress_empire = v["Empire"].as_u64().unwrap_or(0) as u8;
                            self.pilot.progress_federation = v["Federation"].as_u64().unwrap_or(0) as u8;
                            self.pilot.progress_cqc = v["CQC"].as_u64().unwrap_or(0) as u8;
                        }
                        Some("Reputation") => {
                            self.pilot.reputation_empire = v["Empire"].as_f64().unwrap_or(0.0) as f32;
                            self.pilot.reputation_federation = v["Federation"].as_f64().unwrap_or(0.0) as f32;
                            self.pilot.reputation_alliance = v["Alliance"].as_f64().unwrap_or(0.0) as f32;
                        }
                        Some("Powerplay") => {
                            self.pilot.power = v["Power"].as_str().unwrap_or("").to_string();
                            self.pilot.power_merits = v["Merits"].as_i64().unwrap_or(0);
                        }
                        Some("CargoTransfer") => {
                            // Track cargo moved to/from the carrier we're currently docked at.
                            if let Some((market_id, _, _, _)) = self.last_docked {
                                let is_carrier = self.visited_carriers.iter().any(|c| c.market_id == market_id);
                                if is_carrier {
                                    // Skip events already captured in the snapshot to avoid
                                    // double-counting when EDCAS restarts mid-session.
                                    let skip = self.carrier_cargo_skip_before.as_deref()
                                        .map(|skip_ts| {
                                            self.carrier_cargo_seeded.contains(&market_id)
                                                && v["timestamp"].as_str()
                                                    .map_or(false, |ts| ts <= skip_ts)
                                        })
                                        .unwrap_or(false);
                                    if !skip {
                                        if let Some(transfers) = v["Transfers"].as_array() {
                                            let cargo = self.carrier_cargo.entry(market_id).or_default();
                                            for t in transfers {
                                                let name = t["Type"].as_str().unwrap_or("").to_lowercase();
                                                if name.is_empty() { continue; }
                                                if let Some(loc) = t["Type_Localised"].as_str().filter(|s| !s.is_empty()) {
                                                    self.commodity_names.entry(name.clone()).or_insert_with(|| loc.to_string());
                                                }
                                                let count = t["Count"].as_i64().unwrap_or(0) as i32;
                                                match t["Direction"].as_str().unwrap_or("") {
                                                    "tocarrier" => *cargo.entry(name).or_insert(0) += count,
                                                    "toship" => {
                                                        let e = cargo.entry(name).or_insert(0);
                                                        *e = (*e - count).max(0);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Some("ColonisationSystemClaim") => {
                            if let (Some(addr), Some(name)) = (
                                v["SystemAddress"].as_i64(),
                                v["StarSystem"].as_str(),
                            ) {
                                self.claimed_systems.insert(addr, name.to_string());
                            }
                        }
                        Some("SuitLoadout") => {
                            let raw_name = v["SuitName"].as_str().unwrap_or("");
                            let (suit_type, grade) = parse_suit_name(raw_name);
                            let mods = v["SuitMods"].as_array()
                                .map(|arr| arr.iter()
                                    .filter_map(|m| m.as_str())
                                    .map(format_suit_mod)
                                    .collect())
                                .unwrap_or_default();
                            let weapons = v["Modules"].as_array()
                                .map(|arr| arr.iter().map(|m| SuitWeapon {
                                    slot: m["SlotName"].as_str().unwrap_or("").to_string(),
                                    name: m["ModuleName_Localised"].as_str()
                                        .or_else(|| m["ModuleName"].as_str())
                                        .unwrap_or("").to_string(),
                                    class: m["Class"].as_u64().unwrap_or(0) as u8,
                                    mods: m["WeaponMods"].as_array()
                                        .map(|wm| wm.iter()
                                            .filter_map(|w| w.as_str())
                                            .map(format_suit_mod)
                                            .collect())
                                        .unwrap_or_default(),
                                }).collect())
                                .unwrap_or_default();
                            self.pilot.suit = Some(SuitData {
                                suit_type,
                                grade,
                                loadout_name: v["LoadoutName"].as_str().unwrap_or("").to_string(),
                                mods,
                                weapons,
                            });
                        }
                        _ => {}
                    }
                }
            }
            Some(JournalEvent::SupercruiseExit(e)) => {
                // Track the body approached; used as host_body_id for the next Docked event.
                if e.body_type != "Station" && e.body_id != 0 {
                    self.pending_host_body_id = Some(e.body_id);
                } else {
                    self.pending_host_body_id = None;
                }
            }
            Some(JournalEvent::Docked(e)) => {
                debug!("Docked at {} (market_id={})", e.station_name, e.market_id);
                self.last_docked = Some((
                    e.market_id,
                    e.station_name.clone(),
                    e.star_system.clone(),
                    e.system_address,
                ));
                let secondary_economies = e.station_economies.as_ref()
                    .map(|v| v.iter().skip(1)
                        .map(|se| (localised_or_clean("", &se.name), se.proportion))
                        .collect())
                    .unwrap_or_default();
                let host_body_id = self.pending_host_body_id.take();
                let station = StationData {
                    name: e.station_name.clone(),
                    station_type: e.station_type.clone(),
                    system_name: e.star_system.clone(),
                    dist_from_star_ls: e.dist_from_star_ls.unwrap_or(0.0),
                    services: e.station_services.clone().unwrap_or_default(),
                    economy: localised_or_clean(&e.station_economy_localised, &e.station_economy),
                    secondary_economies,
                    faction: e.station_faction.as_ref().map(|f| f.name.clone()).unwrap_or_default(),
                    government: localised_or_clean(&e.station_government_localised, &e.station_government),
                    allegiance: e.station_allegiance.clone(),
                    landing_pads: e.landing_pads.as_ref().map(|lp| (lp.small, lp.medium, lp.large)),
                    market_id: e.market_id,
                    host_body_id,
                    commodities: Vec::new(),
                };
                if !self.stations.iter().any(|s| s.market_id == station.market_id) {
                    self.stations.push(station.clone());
                }
                if e.station_type == "FleetCarrier" {
                    self.visited_carriers.retain(|s| s.market_id != station.market_id);
                    self.visited_carriers.insert(0, station);
                } else {
                    // Preserve commodities from a prior visit so they survive re-dock
                    // until Market.json is re-parsed (which may take up to 500 ms).
                    let prior_commodities = self.visited_stations.iter()
                        .find(|s| s.market_id == station.market_id)
                        .map(|s| s.commodities.clone())
                        .unwrap_or_default();
                    self.visited_stations.retain(|s| s.market_id != station.market_id);
                    let mut station = station;
                    if station.commodities.is_empty() {
                        station.commodities = prior_commodities;
                    }
                    self.visited_stations.insert(0, station);
                }
            }
            Some(JournalEvent::ColonisationConstructionDepot(e)) => {
                debug!("ColonisationConstructionDepot for market_id={}", e.market_id);
                let (station_name, system_address, system_name) = self
                    .last_docked
                    .as_ref()
                    .filter(|(mid, _, _, _)| *mid == e.market_id)
                    .map(|(_, name, sys, addr)| (name.clone(), *addr, sys.clone()))
                    .unwrap_or_else(|| (format!("Depot {}", e.market_id), e.system_address, String::new()));
                let submission = ConstructionDepotSubmission {
                    market_id: e.market_id,
                    system_address,
                    station_name,
                    progress: e.construction_progress,
                    construction_complete: e.construction_complete,
                    construction_failed: e.construction_failed,
                    resources: e.resources.iter().map(|r| ConstructionResourceSubmission {
                        name: r.name.clone(),
                        display_name: r.display_name().to_string(),
                        required_amount: r.required_amount,
                        provided_amount: r.provided_amount,
                        payment: r.payment,
                    }).collect(),
                };
                self.construction_depots.insert(e.market_id, ConstructionDepotData { submission, system_name });
            }
            _ => {}
        }
    }

    fn clear_scan_data(&mut self) {
        self.bodies.clear();
        self.fss_signals.clear();
        self.saa_data.clear();
        self.stations.clear();
        self.discovered_signals.clear();
    }

    pub fn clear(&mut self) {
        self.current_system = None;
        self.clear_scan_data();
        self.carrier_cargo.clear();
        self.commodity_names.clear();
        self.carrier_cargo_seeded.clear();
        self.carrier_cargo_skip_before = None;
        // carrier_fc_materials is file-based, not event-based — cleared by the watcher when FCMaterials.json changes
    }
}

/// Map the internal `Ship` field from the journal Loadout event to the minimum
/// landing pad size the ship requires: 'L', 'M', or 'S'.
fn ship_pad_size(internal_name: &str) -> char {
    match internal_name.to_lowercase().as_str() {
        // Large-pad-only ships
        "anaconda" | "corvette" | "cutter" | "belugaliner" | "orca"
        | "type7" | "type7_multipurpose" | "type9" | "type9_military"
        | "type10" => 'L',
        // Small-pad ships
        "sidewinder" | "hauler" | "eagle" | "eagle_mkii" | "adder"
        | "viper" | "viper_mkiii" | "viper_mkiv"
        | "imperialeargle" | "imperial_eagle" => 'S',
        // Everything else defaults to medium (M or L pad accepted)
        _ => 'M',
    }
}

fn faction_info_from_journal(
    f: &edcas_common::journal::types::Faction,
    conflicts: Option<&Vec<Conflict>>,
) -> FactionInfo {
    let conflict = conflicts.and_then(|cs| {
        cs.iter()
            .find(|c| {
                c.status.to_lowercase() != "completed"
                    && c.faction1.won_days < 4
                    && c.faction2.won_days < 4
                    && (c.faction1.name == f.name || c.faction2.name == f.name)
            })
            .map(|c| {
                let (ours, theirs) = if c.faction1.name == f.name {
                    (&c.faction1, &c.faction2)
                } else {
                    (&c.faction2, &c.faction1)
                };
                ConflictData {
                    war_type: c.war_type.clone(),
                    status: c.status.clone(),
                    opponent: theirs.name.clone(),
                    our_won_days: ours.won_days,
                    opponent_won_days: theirs.won_days,
                    our_stake: ours.stake.clone(),
                    opponent_stake: theirs.stake.clone(),
                }
            })
    });
    FactionInfo {
        name: f.name.clone(),
        influence: f.influence,
        government: f.government.clone(),
        allegiance: f.allegiance.clone(),
        happiness: f.happiness.clone(),
        active_states: f
            .active_states
            .as_ref()
            .map(|s| s.iter().map(|x| x.state.clone()).collect())
            .unwrap_or_default(),
        pending_states: f
            .pending_states
            .as_ref()
            .map(|s| s.iter().map(|x| x.state.clone()).collect())
            .unwrap_or_default(),
        recovering_states: f
            .recovering_states
            .as_ref()
            .map(|s| s.iter().map(|x| x.state.clone()).collect())
            .unwrap_or_default(),
        conflict,
    }
}

fn parent_type_from_str(s: &str) -> ParentType {
    match s {
        "Star" => ParentType::Star,
        "Planet" => ParentType::Planet,
        "Ring" => ParentType::Ring,
        _ => ParentType::Null,
    }
}

fn system_from_fsdjump(e: &FsdJump) -> SystemData {
    let coords = coords_from_star_pos(&e.star_pos);
    SystemData {
        name: e.star_system.clone(),
        system_address: e.system_address,
        coords,
        economy: e.system_economy.clone(),
        second_economy: e.system_second_economy.clone(),
        government: e.system_government.clone(),
        allegiance: e.system_allegiance.clone(),
        security: e.system_security.clone(),
        population: e.population,
        body: e.body.clone(),
        body_id: e.body_id,
        body_type: e.body_type.clone(),
        factions: e
            .factions
            .as_ref()
            .map(|f| f.iter().map(|faction| faction_info_from_journal(faction, e.conflicts.as_ref())).collect())
            .unwrap_or_default(),
        system_faction: e
            .system_faction
            .as_ref()
            .map(|f| f.name.clone())
            .unwrap_or_default(),
        controlling_power: e.controlling_power.clone(),
        powers: e.powers.clone().unwrap_or_default(),
    }
}

fn system_from_location(e: &Location) -> SystemData {
    let coords = coords_from_star_pos(&e.star_pos);
    SystemData {
        name: e.star_system.clone(),
        system_address: e.system_address,
        coords,
        economy: e.system_economy.clone(),
        second_economy: e.system_second_economy.clone(),
        government: e.system_government.clone(),
        allegiance: e.system_allegiance.clone(),
        security: e.system_security.clone(),
        population: e.population,
        body: e.body.clone(),
        body_id: e.body_id as i32,
        body_type: e.body_type.clone(),
        factions: e
            .factions
            .as_ref()
            .map(|f| f.iter().map(|faction| faction_info_from_journal(faction, e.conflicts.as_ref())).collect())
            .unwrap_or_default(),
        system_faction: e
            .system_faction
            .as_ref()
            .map(|f| f.name.clone())
            .unwrap_or_default(),
        controlling_power: e.controlling_power.clone(),
        powers: e.powers.clone().unwrap_or_default(),
    }
}

fn system_from_carrier_jump(e: &CarrierJump) -> SystemData {
    let coords = coords_from_star_pos(&e.star_pos);
    SystemData {
        name: e.star_system.clone(),
        system_address: e.system_address,
        coords,
        economy: e.system_economy.clone(),
        second_economy: e.system_second_economy.clone(),
        government: e.system_government.clone(),
        allegiance: e.system_allegiance.clone(),
        security: e.system_security.clone(),
        population: e.population,
        body: e.body.clone(),
        body_id: e.body_id,
        body_type: e.body_type.clone(),
        factions: e
            .factions
            .as_ref()
            .map(|f| f.iter().map(|faction| faction_info_from_journal(faction, e.conflicts.as_ref())).collect())
            .unwrap_or_default(),
        system_faction: e
            .system_faction
            .as_ref()
            .map(|f| f.name.clone())
            .unwrap_or_default(),
        controlling_power: e.controlling_power.clone(),
        powers: e.powers.clone().unwrap_or_default(),
    }
}

fn coords_from_star_pos(star_pos: &[f32]) -> (f32, f32, f32) {
    (
        star_pos.first().copied().unwrap_or(0.0),
        star_pos.get(1).copied().unwrap_or(0.0),
        star_pos.get(2).copied().unwrap_or(0.0),
    )
}

fn body_from_scan(e: &Scan) -> BodyScan {
    let parents = e
        .parents
        .as_ref()
        .map(|pv| {
            pv.iter()
                .filter_map(|p| {
                    p.parent_id().map(|pid| BodyParent {
                        body_id: pid,
                        parent_type: parent_type_from_str(p.parent_type()),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let rings = e
        .rings
        .as_ref()
        .map(|rv| {
            rv.iter()
                .map(|r| BodyRing {
                    name: r.name.clone(),
                    ring_class: r.ring_class.clone(),
                    mass_mt: r.mass_mt,
                    inner_rad: r.inner_rad,
                    outer_rad: r.outer_rad,
                })
                .collect()
        })
        .unwrap_or_default();

    let materials = e
        .materials
        .as_ref()
        .map(|mv| {
            mv.iter()
                .map(|m| BodyMaterial {
                    name: m.name.clone(),
                    percent: m.percent,
                })
                .collect()
        })
        .unwrap_or_default();

    BodyScan {
        body_id: e.body_id,
        body_name: e.body_name.clone(),
        planet_class: e.planet_class.clone().unwrap_or_default(),
        landable: e.landable,
        scan_type: e.scan_type.clone().unwrap_or_default(),
        distance_from_arrival_ls: e.distance_from_arrival_ls.unwrap_or(0.0),
        radius: e.radius.unwrap_or(0.0),
        mass_em: e.mass_em.unwrap_or(0.0),
        surface_temperature: e.surface_temperature.unwrap_or(0.0),
        surface_gravity: e.surface_gravity.unwrap_or(0.0),
        tidal_lock: e.tidal_lock,
        volcanism: e.volcanism.clone().unwrap_or_default(),
        atmosphere: e.atmosphere.clone().unwrap_or_default(),
        terraform_state: e.terraform_state.clone().unwrap_or_default(),
        star_type: e.star_type.clone().unwrap_or_default(),
        parents,
        rings,
        materials,
        estimated_value: e.estimated_value.unwrap_or(0),
        composition: e.composition.as_ref().map(|c| BodyComposition {
            ice: c.ice,
            rock: c.rock,
            metal: c.metal,
        }),
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug)]
pub struct BulkUploadProgress {
    pub current_file: usize,
    pub total_files: usize,
    pub lines_done: u64,
    pub done: bool,
    pub error: Option<String>,
}

#[cfg(not(target_arch = "wasm32"))]
pub struct JournalReader {
    handle: Option<thread::JoinHandle<()>>,
    should_stop: std::sync::Arc<AtomicBool>,
    receiver: mpsc::Receiver<JournalData>,
}

#[cfg(not(target_arch = "wasm32"))]
impl JournalReader {
    pub fn start(journal_dir: PathBuf, api_url: Option<String>) -> Self {
        info!("Initializing journal reader for: {}", journal_dir.display());
        let (tx, rx) = mpsc::channel();
        let should_stop = std::sync::Arc::new(AtomicBool::new(false));
        let stop_flag = should_stop.clone();

        let upload_tx_opt = api_url.filter(|u| !u.is_empty()).map(|url| {
            let (upload_tx, upload_rx) = mpsc::channel::<String>();
            thread::spawn(move || {
                let client = reqwest::blocking::Client::builder()
                    .timeout(Duration::from_secs(5))
                    .build()
                    .unwrap();
                let endpoint = format!("{}/api/v1/journal/event", url);
                for raw_line in upload_rx {
                    if let Ok(message) = serde_json::from_str::<serde_json::Value>(&raw_line) {
                        let wrapper = serde_json::json!({
                            "$schemaRef": "edcas-client-upload/v1",
                            "message": message
                        });
                        let _ = client.post(&endpoint).json(&wrapper).send();
                    }
                }
            });
            upload_tx
        });

        let handle = thread::spawn(move || {
            let mut journal_data = JournalData::new();

            info!("Loading existing journal files from: {}", journal_dir.display());
            load_existing_files(&journal_dir, &mut journal_data, upload_tx_opt.as_ref());
            load_cargo_file(&journal_dir.join("Cargo.json"), &mut journal_data);
            journal_data.backpack = load_onfoot_file(&journal_dir.join("Backpack.json"));
            journal_data.shiplocker = load_onfoot_file(&journal_dir.join("ShipLocker.json"));
            if let Some((mid, items)) = load_fc_materials_file(&journal_dir.join("FCMaterials.json")) {
                journal_data.carrier_fc_materials.insert(mid, items);
            }
            load_modules_file(&journal_dir.join("ModulesInfo.json"), &mut journal_data);
            info!("Loaded {} bodies from existing files", journal_data.bodies.len());
            let _ = tx.send(journal_data.clone());

            watch_latest_file(&journal_dir, &mut journal_data, &tx, &stop_flag, &upload_tx_opt);
        });

        Self {
            handle: Some(handle),
            should_stop,
            receiver: rx,
        }
    }

    pub fn try_recv(&self) -> Option<JournalData> {
        self.receiver.try_recv().ok()
    }

    pub fn stop(&mut self) {
        info!("Stopping journal reader");
        self.should_stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    pub fn restart(&mut self, journal_dir: PathBuf, api_url: Option<String>) {
        self.stop();
        *self = Self::start(journal_dir, api_url);
    }
}

#[cfg(not(target_arch = "wasm32"))]
// Shorter initial delay in tests so retry-path tests don't take seconds.
#[cfg(not(test))]
const RETRY_INITIAL_DELAY: Duration = Duration::from_millis(500);
#[cfg(test)]
const RETRY_INITIAL_DELAY: Duration = Duration::from_millis(1);

#[cfg(not(target_arch = "wasm32"))]
fn flush_batch(
    client: &reqwest::blocking::Client,
    endpoint: &str,
    batch: &[serde_json::Value],
    send_errors: &mut u64,
) {
    const MAX_RETRIES: u32 = 10;
    let mut delay = RETRY_INITIAL_DELAY;

    for attempt in 0..=MAX_RETRIES {
        let result = client.post(endpoint).json(batch).send();
        match result {
            Err(e) => {
                if attempt == MAX_RETRIES {
                    error!("Batch send failed ({} events) after {} attempts: {e:#}", batch.len(), attempt + 1);
                    *send_errors += 1;
                    return;
                }
                // Network / connect error — retry after backoff
            }
            Ok(r) => {
                let status = r.status().as_u16();
                match status {
                    200 | 202 | 204 => return, // success
                    503 | 504 | 429 => {
                        if attempt == MAX_RETRIES {
                            error!("Batch send failed ({} events) after {} attempts: HTTP {}", batch.len(), attempt + 1, status);
                            eprintln!("Upload error: HTTP {status} after {} retries ({} events in batch)", attempt + 1, batch.len());
                            *send_errors += 1;
                            return;
                        }
                        // Transient server overload — retry after backoff
                    }
                    other => {
                        error!("Batch send failed ({} events): HTTP {}", batch.len(), other);
                        eprintln!("Upload error: HTTP {other} ({} events in batch)", batch.len());
                        *send_errors += 1;
                        return;
                    }
                }
            }
        }
        thread::sleep(delay);
        delay = (delay * 2).min(Duration::from_secs(30));
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// Starts a background thread that uploads all journal files (oldest first) to the server.
/// Returns a receiver that yields progress updates.
pub fn start_bulk_upload(
    journal_dir: PathBuf,
    api_url: String,
) -> mpsc::Receiver<BulkUploadProgress> {
    let (progress_tx, progress_rx) = mpsc::channel();
    thread::spawn(move || {
        let mut files = find_all_journal_files(&journal_dir);
        files.reverse(); // oldest first
        let total = files.len();
        if total == 0 {
            let _ = progress_tx.send(BulkUploadProgress {
                current_file: 0,
                total_files: 0,
                lines_done: 0,
                done: true,
                error: Some("No journal files found".into()),
            });
            return;
        }

        let client = match reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                let _ = progress_tx.send(BulkUploadProgress {
                    current_file: 0,
                    total_files: total,
                    lines_done: 0,
                    done: true,
                    error: Some(format!("Failed to create HTTP client: {e}")),
                });
                return;
            }
        };
        let endpoint = format!("{}/api/v1/journal/events", api_url);
        info!("Starting batched upload to {}", endpoint);
        let mut lines_done: u64 = 0;
        let mut send_errors: u64 = 0;
        const BATCH_SIZE: usize = 50;

        for (i, file) in files.iter().enumerate() {
            let mut batch: Vec<serde_json::Value> = Vec::with_capacity(BATCH_SIZE);

            if let Ok(content) = std::fs::read_to_string(file) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.len() > 65_536 {
                        continue;
                    }
                    if let Ok(message) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        batch.push(serde_json::json!({
                            "$schemaRef": "edcas-client-upload/v1",
                            "message": message
                        }));
                        lines_done += 1;
                        if batch.len() >= BATCH_SIZE {
                            flush_batch(&client, &endpoint, &batch, &mut send_errors);
                            batch.clear();
                        }
                    }
                }
            }
            if !batch.is_empty() {
                flush_batch(&client, &endpoint, &batch, &mut send_errors);
            }

            let _ = progress_tx.send(BulkUploadProgress {
                current_file: i + 1,
                total_files: total,
                lines_done,
                done: i + 1 == total,
                error: if send_errors > 0 {
                    Some(format!("{send_errors} send error(s) — check log for details"))
                } else {
                    None
                },
            });
        }
    });
    progress_rx
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for JournalReader {
    fn drop(&mut self) {
        self.should_stop.store(true, Ordering::SeqCst);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_inventory_array(arr: &serde_json::Value) -> Vec<InventoryItem> {
    arr.as_array()
        .map(|items| {
            items.iter().filter_map(|item| {
                let name = item.get("Name")?.as_str()?.to_owned();
                let localised = item.get("Name_Localised")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&name)
                    .to_owned();
                let count = item.get("Count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                Some(InventoryItem { name, localised, count })
            }).collect()
        })
        .unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_cargo_file(path: &Path, data: &mut JournalData) {
    let Ok(content) = std::fs::read_to_string(path) else { return };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) else { return };
    data.cargo = v["Inventory"].as_array()
        .map(|items| items.iter().filter_map(|item| {
            let name = item.get("Name")?.as_str()?.to_owned();
            let localised = item.get("Name_Localised")
                .and_then(|v| v.as_str())
                .unwrap_or(&name)
                .to_owned();
            let count = item.get("Count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let stolen = item.get("Stolen").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            Some(CargoItem { name, localised, count, stolen })
        }).collect())
        .unwrap_or_default();
}

#[cfg(not(target_arch = "wasm32"))]
fn load_modules_file(path: &Path, data: &mut JournalData) {
    let Ok(text) = std::fs::read_to_string(path) else { return };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else { return };
    let Some(arr) = v["Modules"].as_array() else { return };
    let modules: Vec<ShipModule> = arr.iter().map(|m| {
        let slot = m["Slot"].as_str().unwrap_or("").to_string();
        let health      = data.loadout_health.get(&slot).copied();
        let engineering = data.loadout_engineering.get(&slot).cloned().flatten();
        ShipModule {
            slot,
            item:     m["Item"].as_str().unwrap_or("").to_string(),
            power:    m["Power"].as_f64().unwrap_or(0.0) as f32,
            priority: m["Priority"].as_u64().unwrap_or(0) as u8,
            health,
            value: None,
            engineering,
        }
    }).collect();
    data.modules = modules;
}

fn parse_suit_name(raw: &str) -> (String, u8) {
    let grade = raw.find("_class")
        .and_then(|i| raw[i + 6..].parse::<u8>().ok())
        .unwrap_or(1);
    let base = raw.find("_class").map(|i| &raw[..i]).unwrap_or(raw);
    let suit_type = match base {
        "explorationsuit" => "Artemis Suit",
        "utilitysuit"     => "Maverick Suit",
        "tacticalsuit"    => "Dominator Suit",
        other             => other,
    };
    (suit_type.to_string(), grade)
}

fn format_suit_mod(raw: &str) -> String {
    let stripped = raw.strip_prefix("suit_")
        .or_else(|| raw.strip_prefix("wpn_mod_"))
        .unwrap_or(raw);
    stripped.split('_')
        .map(|w| {
            let mut c = w.chars();
            c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_engineering(v: &serde_json::Value) -> Option<EngineeringData> {
    if !v.is_object() {
        return None;
    }
    let raw_exp = v["ExperimentalEffect_Localised"].as_str()
        .filter(|s| !s.is_empty() && !s.starts_with('$'))
        .or_else(|| v["ExperimentalEffect"].as_str().filter(|s| !s.is_empty()))
        .unwrap_or("")
        .to_string();
    let experimental = if raw_exp.starts_with("special_") {
        // e.g. "special_weapon_efficient" -> "Weapon Efficient"
        let body = raw_exp.trim_start_matches("special_").trim_end_matches("_name");
        body.split('_')
            .map(|w| { let mut c = w.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        raw_exp
    };

    let modifiers = v["Modifiers"].as_array()
        .map(|arr| arr.iter().map(|m| EngineeringModifier {
            label:          camel_to_words(m["Label"].as_str().unwrap_or("")),
            value:          m["Value"].as_f64().unwrap_or(0.0) as f32,
            original_value: m["OriginalValue"].as_f64().unwrap_or(0.0) as f32,
            less_is_good:   m["LessIsGood"].as_i64().unwrap_or(0) != 0,
        }).collect())
        .unwrap_or_default();

    Some(EngineeringData {
        blueprint:    format_blueprint(v["BlueprintName"].as_str().unwrap_or("")),
        level:        v["Level"].as_u64().unwrap_or(0) as u8,
        quality:      v["Quality"].as_f64().unwrap_or(0.0) as f32,
        engineer:     v["Engineer"].as_str().unwrap_or("").to_string(),
        experimental,
        modifiers,
    })
}

fn format_blueprint(raw: &str) -> String {
    let name = raw.find('_').map(|i| &raw[i + 1..]).unwrap_or(raw);
    camel_to_words(name)
}

fn camel_to_words(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.char_indices() {
        if i > 0 && ch.is_uppercase() {
            // insert space before a capital that follows a lowercase letter
            let prev = s[..i].chars().last().unwrap_or(' ');
            if prev.is_lowercase() {
                out.push(' ');
            }
        }
        out.push(ch);
    }
    out
}

#[cfg(not(target_arch = "wasm32"))]
fn load_fc_materials_file(path: &Path) -> Option<(i64, Vec<(String, i32)>)> {
    let content = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let market_id = v["MarketID"].as_i64()?;
    let items: Vec<(String, i32)> = v["Items"].as_array()?
        .iter()
        .filter_map(|item| {
            let stock = item["Stock"].as_i64().unwrap_or(0) as i32;
            if stock <= 0 { return None; }
            let name = item["Name_Localised"].as_str()
                .or_else(|| item["Name"].as_str())
                .map(|s| s.trim_start_matches('$').trim_end_matches("_name;").trim_end_matches(';').to_string())
                .unwrap_or_default();
            if name.is_empty() { return None; }
            Some((name, stock))
        })
        .collect();
    Some((market_id, items))
}

#[cfg(not(target_arch = "wasm32"))]
fn load_market_file(path: &Path) -> Option<(i64, Vec<edcas_common::api::CommodityResponse>)> {
    let content = std::fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&content).ok()?;
    let market_id = v["MarketID"].as_i64()?;
    let items = v["Items"].as_array()?;
    let commodities: Vec<edcas_common::api::CommodityResponse> = items.iter().filter_map(|item| {
        let name = item["Name"].as_str()
            .map(|s| s.trim_start_matches('$').trim_end_matches("_name;").to_string())
            .filter(|s| !s.is_empty())
            .or_else(|| item["Name_Localised"].as_str().filter(|s| !s.is_empty()).map(|s| s.to_string()))
            .unwrap_or_default();
        if name.is_empty() { return None; }
        Some(edcas_common::api::CommodityResponse {
            name,
            mean_price: item["MeanPrice"].as_i64().unwrap_or(0) as i32,
            buy_price:  item["BuyPrice"].as_i64().unwrap_or(0) as i32,
            stock:      item["Stock"].as_i64().unwrap_or(0) as i32,
            sell_price: item["SellPrice"].as_i64().unwrap_or(0) as i32,
            demand:     item["Demand"].as_i64().unwrap_or(0) as i32,
        })
    }).collect();
    if commodities.is_empty() { return None; }
    Some((market_id, commodities))
}

#[cfg(not(target_arch = "wasm32"))]
fn load_onfoot_file(path: &Path) -> OnFootInventory {
    let Ok(content) = std::fs::read_to_string(path) else { return OnFootInventory::default() };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) else { return OnFootInventory::default() };
    OnFootInventory {
        items: parse_inventory_array(&v["Items"]),
        components: parse_inventory_array(&v["Components"]),
        consumables: parse_inventory_array(&v["Consumables"]),
        data: parse_inventory_array(&v["Data"]),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn load_existing_files(
    dir: &Path,
    data: &mut JournalData,
    upload_tx: Option<&mpsc::Sender<String>>,
) {
    if !dir.exists() || !dir.is_dir() {
        warn!("Journal directory does not exist: {}", dir.display());
        return;
    }
    let files = find_all_journal_files(dir);
    if files.is_empty() {
        warn!("No journal file found in: {}", dir.display());
        return;
    }

    // Seed carrier cargo from the persisted snapshot; these carriers are skipped
    // during backfill to avoid double-counting events already included in the snapshot.
    let seeded_ids = seed_my_carrier_cargo(data);

    // Backfill carrier cargo from previous sessions before processing the current
    // file so that chronological order is preserved for correct running totals.
    let prev_files: Vec<PathBuf> = files.iter().skip(1).take(15).rev().cloned().collect();
    if !prev_files.is_empty() {
        info!("Backfilling carrier cargo from {} previous journal files", prev_files.len());
        backfill_carrier_cargo(&prev_files, data, &seeded_ids);
    }

    info!("Loading journal file: {}", files[0].display());
    read_file_lines(&files[0], data, None);
    // After replaying the journal the skip-window is no longer needed; live events must
    // always be applied unconditionally.
    data.carrier_cargo_skip_before = None;

    // When the latest file starts with a Location (same system, new game session),
    // the scans from the previous session are in older files — backfill them.
    if data.current_system.is_some() && data.bodies.is_empty() {
        let system_address = data.current_system.as_ref().unwrap().system_address;
        for prev_file in files.iter().skip(1).take(3) {
            info!("Backfilling scans from previous journal: {}", prev_file.display());
            load_previous_scans_for_system(prev_file, system_address, data, None);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn seed_my_carrier_cargo(data: &mut JournalData) -> std::collections::HashSet<i64> {
    let mc = crate::my_carriers::MyCarriersData::load();
    let seeded: std::collections::HashSet<i64> = mc.carriers.keys().copied().collect();
    // If the snapshot was saved mid-session, skip CargoTransfer events that are already
    // captured in the snapshot (timestamp ≤ snapshot_timestamp) to prevent double-counting
    // when EDCAS is restarted without closing the game.
    if !mc.snapshot_timestamp.is_empty() {
        data.carrier_cargo_skip_before = Some(mc.snapshot_timestamp);
    }
    data.carrier_cargo_seeded = seeded.clone();
    for (market_id, cargo) in mc.carriers {
        data.carrier_cargo.insert(market_id, cargo);
    }
    seeded
}

#[cfg(not(target_arch = "wasm32"))]
fn backfill_carrier_cargo(
    files: &[PathBuf],
    data: &mut JournalData,
    skip_ids: &std::collections::HashSet<i64>,
) {
    for path in files {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => continue,
        };
        let reader = BufReader::new(file);
        let mut docked_carrier_market_id: Option<i64> = None;

        for line in reader.lines().flatten() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let v: serde_json::Value = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(_) => continue,
            };
            match v["event"].as_str().unwrap_or("") {
                "Docked" => {
                    if v["StationType"].as_str() == Some("FleetCarrier") {
                        docked_carrier_market_id = v["MarketID"].as_i64();
                    } else {
                        docked_carrier_market_id = None;
                    }
                }
                "Location" => {
                    if v["Docked"].as_bool() == Some(true)
                        && v["StationType"].as_str() == Some("FleetCarrier")
                    {
                        docked_carrier_market_id = v["MarketID"].as_i64();
                    } else {
                        docked_carrier_market_id = None;
                    }
                }
                "Undocked" => {
                    docked_carrier_market_id = None;
                }
                "CargoTransfer" => {
                    if let Some(market_id) = docked_carrier_market_id {
                        // Skip carriers already seeded from the persisted snapshot
                        if skip_ids.contains(&market_id) {
                            continue;
                        }
                        if let Some(transfers) = v["Transfers"].as_array() {
                            let cargo = data.carrier_cargo.entry(market_id).or_default();
                            for t in transfers {
                                let name = t["Type"].as_str().unwrap_or("").to_lowercase();
                                if name.is_empty() {
                                    continue;
                                }
                                if let Some(loc) =
                                    t["Type_Localised"].as_str().filter(|s| !s.is_empty())
                                {
                                    data.commodity_names
                                        .entry(name.clone())
                                        .or_insert_with(|| loc.to_string());
                                }
                                let count = t["Count"].as_i64().unwrap_or(0) as i32;
                                match t["Direction"].as_str().unwrap_or("") {
                                    "tocarrier" => *cargo.entry(name).or_insert(0) += count,
                                    "toship" => {
                                        let e = cargo.entry(name).or_insert(0);
                                        *e = (*e - count).max(0);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn watch_latest_file(
    dir: &Path,
    data: &mut JournalData,
    tx: &mpsc::Sender<JournalData>,
    stop_flag: &AtomicBool,
    upload_tx: &Option<mpsc::Sender<String>>,
) {
    // Initialise to the file already loaded by load_existing_files so the first loop
    // iteration does an incremental tail rather than a redundant full re-read.
    let mut last_file: Option<PathBuf> = find_latest_journal_file(dir);
    let mut last_position: u64 = last_file.as_ref()
        .and_then(|f| f.metadata().ok())
        .map(|m| m.len())
        .unwrap_or(0);
    let mut last_cargo_mtime: Option<SystemTime> = None;
    let mut last_backpack_mtime: Option<SystemTime> = None;
    let mut last_shiplocker_mtime: Option<SystemTime> = None;
    let mut last_fc_materials_mtime: Option<SystemTime> = None;
    let mut last_modules_mtime: Option<SystemTime> = None;
    let mut last_market_mtime: Option<SystemTime> = None;
    let mut last_outfitting_mtime: Option<SystemTime> = None;
    let mut last_shipyard_mtime: Option<SystemTime> = None;

    loop {
        if stop_flag.load(Ordering::SeqCst) {
            return;
        }

        let mut changed = false;

        // ── Journal file ──────────────────────────────────────────
        if let Some(active) = find_latest_journal_file(dir) {
            let file_changed = last_file.as_ref() != Some(&active);
            if file_changed {
                info!("Journal file changed to: {}", active.display());
                last_file = Some(active.clone());
                data.clear();
                // Re-seed persisted carrier cargo so it survives new-session clear.
                // A new journal file always means a fresh game session, so any snapshot
                // timestamp is from before this session — no skip needed.
                seed_my_carrier_cargo(data);
                data.carrier_cargo_skip_before = None;
                read_file_lines(&active, data, upload_tx.as_ref());
                last_position = active.metadata().map(|m| m.len()).unwrap_or(0);
                changed = true;
            } else if let Ok(metadata) = active.metadata() {
                let file_size = metadata.len();
                if file_size > last_position {
                    if let Ok(mut file) = OpenOptions::new().read(true).open(&active) {
                        let _ = file.seek(SeekFrom::Start(last_position));
                        let reader = BufReader::new(file);
                        for line in reader.lines().flatten() {
                            let trimmed = line.trim().to_owned();
                            if !trimmed.is_empty() {
                                data.process_line(&trimmed);
                                if let Some(ref up_tx) = upload_tx {
                                    let _ = up_tx.send(trimmed);
                                }
                                changed = true;
                            }
                        }
                        last_position = file_size;
                    }
                }
            }
        }

        // ── Inventory JSON files ──────────────────────────────────
        let cargo_path = dir.join("Cargo.json");
        if let Ok(mtime) = cargo_path.metadata().and_then(|m| m.modified()) {
            if Some(mtime) != last_cargo_mtime {
                last_cargo_mtime = Some(mtime);
                load_cargo_file(&cargo_path, data);
                changed = true;
            }
        }

        let backpack_path = dir.join("Backpack.json");
        if let Ok(mtime) = backpack_path.metadata().and_then(|m| m.modified()) {
            if Some(mtime) != last_backpack_mtime {
                last_backpack_mtime = Some(mtime);
                data.backpack = load_onfoot_file(&backpack_path);
                changed = true;
            }
        }

        let shiplocker_path = dir.join("ShipLocker.json");
        if let Ok(mtime) = shiplocker_path.metadata().and_then(|m| m.modified()) {
            if Some(mtime) != last_shiplocker_mtime {
                last_shiplocker_mtime = Some(mtime);
                data.shiplocker = load_onfoot_file(&shiplocker_path);
                changed = true;
            }
        }

        let fc_materials_path = dir.join("FCMaterials.json");
        if let Ok(mtime) = fc_materials_path.metadata().and_then(|m| m.modified()) {
            if Some(mtime) != last_fc_materials_mtime {
                last_fc_materials_mtime = Some(mtime);
                if let Some((mid, items)) = load_fc_materials_file(&fc_materials_path) {
                    data.carrier_fc_materials.insert(mid, items);
                }
                changed = true;
            }
        }

        let modules_path = dir.join("ModulesInfo.json");
        if let Ok(mtime) = modules_path.metadata().and_then(|m| m.modified()) {
            if Some(mtime) != last_modules_mtime {
                last_modules_mtime = Some(mtime);
                load_modules_file(&modules_path, data);
                changed = true;
            }
        }

        // ── Station companion files (uploaded to server when an API URL is set) ──
        let market_path = dir.join("Market.json");
        if let Ok(mtime) = market_path.metadata().and_then(|m| m.modified()) {
            if Some(mtime) != last_market_mtime {
                last_market_mtime = Some(mtime);
                if let Some(ref up_tx) = upload_tx {
                    upload_companion_file(&market_path, up_tx);
                }
                if let Some((mid, commodities)) = load_market_file(&market_path) {
                    if let Some(station) = data.visited_stations.iter_mut().find(|s| s.market_id == mid) {
                        station.commodities = commodities.clone();
                    }
                    data.local_market = Some((mid, commodities));
                    changed = true;
                }
            }
        }

        let outfitting_path = dir.join("Outfitting.json");
        if let Ok(mtime) = outfitting_path.metadata().and_then(|m| m.modified()) {
            if Some(mtime) != last_outfitting_mtime {
                last_outfitting_mtime = Some(mtime);
                if let Some(ref up_tx) = upload_tx {
                    upload_companion_file(&outfitting_path, up_tx);
                }
            }
        }

        let shipyard_path = dir.join("Shipyard.json");
        if let Ok(mtime) = shipyard_path.metadata().and_then(|m| m.modified()) {
            if Some(mtime) != last_shipyard_mtime {
                last_shipyard_mtime = Some(mtime);
                if let Some(ref up_tx) = upload_tx {
                    upload_companion_file(&shipyard_path, up_tx);
                }
            }
        }

        if changed {
            let _ = tx.send(data.clone());
        }

        thread::sleep(Duration::from_millis(500));
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn upload_companion_file(path: &Path, tx: &std::sync::mpsc::Sender<String>) {
    match std::fs::read_to_string(path) {
        Ok(content) => {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Ok(compact) = serde_json::to_string(&value) {
                    let _ = tx.send(compact);
                }
            }
        }
        Err(e) => error!("Failed to read companion file {}: {}", path.display(), e),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn read_file_lines(
    path: &Path,
    data: &mut JournalData,
    upload_tx: Option<&mpsc::Sender<String>>,
) {
    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let trimmed = line.trim().to_owned();
                if trimmed.is_empty() {
                    continue;
                }
                data.process_line(&trimmed);
                if let Some(tx) = upload_tx {
                    // Skip large lines (NavRoute, ShipLocker snapshots) that have no server handler.
                    if trimmed.len() <= 65_536 {
                        let _ = tx.send(trimmed);
                    }
                }
            }
        }
        Err(e) => error!("Failed to open journal file {}: {}", path.display(), e),
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn find_all_journal_files(dir: &Path) -> Vec<PathBuf> {
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<_> = read_dir
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "log")
                .unwrap_or(false)
        })
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            parse_journal_timestamp(&name).map(|ts| (e.path(), ts))
        })
        .collect();
    files.sort_by(|a, b| b.1.cmp(&a.1));
    files.into_iter().map(|(path, _)| path).collect()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn find_latest_journal_file(dir: &Path) -> Option<PathBuf> {
    find_all_journal_files(dir).into_iter().next()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_previous_scans_for_system(
    path: &Path,
    system_address: i64,
    data: &mut JournalData,
    upload_tx: Option<&mpsc::Sender<String>>,
) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to open previous journal file {}: {}", path.display(), e);
            return;
        }
    };

    let mut current_sys_addr: Option<i64> = None;
    let reader = BufReader::new(file);
    for line in reader.lines().flatten() {
        let trimmed = line.trim().to_owned();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(tx) = upload_tx {
            if trimmed.len() <= 65_536 {
                let _ = tx.send(trimmed.clone());
            }
        }
        let value: serde_json::Value = match serde_json::from_str(&trimmed) {
            Ok(v) => v,
            Err(_) => continue,
        };
        match JournalEvent::from_json(value) {
            Some(JournalEvent::FsdJump(e)) => {
                current_sys_addr = Some(e.system_address);
            }
            Some(JournalEvent::Location(e)) => {
                current_sys_addr = Some(e.system_address);
            }
            Some(JournalEvent::CarrierJump(e)) => {
                current_sys_addr = Some(e.system_address);
            }
            Some(JournalEvent::Scan(e)) if current_sys_addr == Some(system_address) => {
                if !data.bodies.iter().any(|b| b.body_id == e.body_id) {
                    data.bodies.push(body_from_scan(&e));
                }
            }
            Some(JournalEvent::ScanBaryCentre(e)) if current_sys_addr == Some(system_address) => {
                if !data.bodies.iter().any(|b| b.body_id == e.body_id) {
                    let parents = e.parents.as_ref()
                        .map(|pv| pv.iter()
                            .filter_map(|p| p.parent_id().map(|pid| BodyParent {
                                body_id: pid,
                                parent_type: parent_type_from_str(p.parent_type()),
                            }))
                            .collect())
                        .unwrap_or_default();
                    data.bodies.push(BodyScan {
                        body_id: e.body_id,
                        body_name: format!("{} Barycentre", e.star_system),
                        planet_class: String::new(),
                        landable: false,
                        scan_type: "AutoScan".into(),
                        distance_from_arrival_ls: e.distance_from_arrival_ls,
                        radius: 0.0,
                        mass_em: 0.0,
                        surface_temperature: 0.0,
                        surface_gravity: 0.0,
                        tidal_lock: false,
                        volcanism: String::new(),
                        atmosphere: String::new(),
                        terraform_state: String::new(),
                        star_type: String::new(),
                        parents,
                        rings: vec![],
                        materials: vec![],
                        estimated_value: 0,
                        composition: None,
                    });
                }
            }
            Some(JournalEvent::FssBodySignals(e)) if current_sys_addr == Some(system_address) => {
                data.fss_signals.entry(e.body_id).or_insert_with(|| {
                    e.signals.iter().map(|s| BodySignal {
                        signal_type: s.signal_type.clone(),
                        signal_type_localised: s.signal_type_localised.clone(),
                        count: s.count,
                    }).collect()
                });
            }
            Some(JournalEvent::SaaSignalsFound(e)) if current_sys_addr == Some(system_address) => {
                data.saa_data.entry(e.body_id).or_insert_with(|| {
                    let signals = e.signals.iter().map(|s| BodySignal {
                        signal_type: s.signal_type.clone(),
                        signal_type_localised: s.signal_type_localised.clone(),
                        count: s.count,
                    }).collect();
                    let genuses = e.genuses.as_ref()
                        .map(|gv| gv.iter()
                            .map(|g| g.genus_localised.clone().unwrap_or_else(|| {
                                g.genus.trim_end_matches(';').replace('_', " ")
                            }))
                            .collect())
                        .unwrap_or_default();
                    SaaBodyData { signals, genuses }
                });
            }
            _ => {}
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn parse_journal_timestamp(filename: &str) -> Option<u64> {
    // Journal.YYYY-MM-DDTHHMMSS.NN.log
    let parts: Vec<&str> = filename.split('.').collect();
    if parts.len() >= 3 && parts[0] == "Journal" && *parts.last()? == "log" {
        let d = parts[1]; // "YYYY-MM-DDTHHMMSS" (17 chars)
        if d.len() != 17 { return None; }
        let year: u64 = d[0..4].parse().ok()?;
        let month: u64 = d[5..7].parse().ok()?;
        let day: u64 = d[8..10].parse().ok()?;
        let hour: u64 = d[11..13].parse().ok()?;
        let min: u64 = d[13..15].parse().ok()?;
        let sec: u64 = d[15..17].parse().ok()?;
        Some(year * 10_000_000_000 + month * 100_000_000 + day * 1_000_000 + hour * 10_000 + min * 100 + sec)
    } else {
        None
    }
}

pub fn build_body_tree(bodies: &[BodyScan]) -> Vec<TreeNode> {
    use std::collections::{HashMap, HashSet};

    let mut body_map: HashMap<i32, &BodyScan> = HashMap::new();
    for body in bodies {
        body_map.insert(body.body_id, body);
    }

    let mut children_of: HashMap<i32, Vec<i32>> = HashMap::new();
    let mut has_known_parent: HashSet<i32> = HashSet::new();

    for (&body_id, body) in &body_map {
        if let Some(first_parent) = body.parents.first() {
            let parent_id = first_parent.body_id;
            if body_map.contains_key(&parent_id) {
                children_of.entry(parent_id).or_default().push(body_id);
                has_known_parent.insert(body_id);
            }
        }
    }

    let mut root_ids: Vec<i32> = body_map
        .keys()
        .filter(|&&id| !has_known_parent.contains(&id))
        .copied()
        .collect();
    root_ids.sort();

    root_ids
        .iter()
        .map(|&id| build_tree_node(id, &body_map, &children_of))
        .collect()
}

fn build_tree_node(
    body_id: i32,
    body_map: &std::collections::HashMap<i32, &BodyScan>,
    children_of: &std::collections::HashMap<i32, Vec<i32>>,
) -> TreeNode {
    let body = body_map[&body_id];
    let mut child_ids = children_of.get(&body_id).cloned().unwrap_or_default();
    child_ids.sort();
    TreeNode {
        name: body.body_name.clone(),
        body_id,
        children: child_ids
            .iter()
            .map(|&cid| build_tree_node(cid, body_map, children_of))
            .collect(),
        data: Some((*body).clone()),
    }
}

#[derive(Clone)]
pub struct TreeNode {
    pub name: String,
    pub body_id: i32,
    pub children: Vec<TreeNode>,
    pub data: Option<BodyScan>,
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod upload_tests {
    use super::*;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{Arc, Mutex};

    /// Spawns a minimal HTTP server on a random port.
    ///
    /// `status_fn` is called with the 0-based request index and returns the
    /// HTTP status code to send back.  Returns `(port, request_count)`.
    fn spawn_mock_server(
        status_fn: impl Fn(usize) -> u16 + Send + 'static,
    ) -> (u16, Arc<Mutex<usize>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let count = Arc::new(Mutex::new(0usize));
        let count_clone = count.clone();

        std::thread::spawn(move || {
            let mut idx = 0;
            for stream in listener.incoming() {
                let Ok(stream) = stream else { break };
                let status = status_fn(idx);
                serve_request(stream, status);
                *count_clone.lock().unwrap() += 1;
                idx += 1;
            }
        });

        (port, count)
    }

    fn serve_request(stream: TcpStream, status: u16) {
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut content_length = 0usize;

        // Read HTTP headers
        let mut line = String::new();
        loop {
            line.clear();
            if reader.read_line(&mut line).is_err() {
                return;
            }
            if line == "\r\n" || line.is_empty() {
                break;
            }
            let lower = line.to_lowercase();
            if lower.starts_with("content-length:") {
                content_length = line[16..].trim().parse().unwrap_or(0);
            }
        }
        // Drain the request body so the client doesn't get a broken-pipe error
        let mut body = vec![0u8; content_length];
        let _ = reader.read_exact(&mut body);

        let status_text = match status {
            200 => "OK",
            202 => "Accepted",
            204 => "No Content",
            504 => "Gateway Timeout",
            _ => "Internal Server Error",
        };
        let mut stream = reader.into_inner();
        let _ = write!(
            stream,
            "HTTP/1.1 {} {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            status, status_text
        );
    }

    /// Creates a temp directory containing a single journal file with the given lines.
    fn make_journal_dir(lines: &[&str]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "edcas_upload_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let content = lines.join("\n");
        std::fs::write(dir.join("Journal.2022-01-01T000000.01.log"), content).unwrap();
        dir
    }

    fn sample_fsd_jump() -> &'static str {
        r#"{ "timestamp":"2022-06-28T15:16:44Z", "event":"FSDJump", "Taxi":false, "Multicrew":false, "StarSystem":"Test", "SystemAddress":1, "StarPos":[0.0,0.0,0.0], "SystemAllegiance":"", "SystemEconomy":"$economy_None;", "SystemSecondEconomy":"$economy_None;", "SystemGovernment":"$government_None;", "SystemSecurity":"$GAlAXY_MAP_INFO_state_anarchy;", "Population":0, "Body":"Test", "BodyID":0, "BodyType":"Star" }"#
    }

    #[test]
    fn upload_succeeds_and_reports_correct_counts() {
        // 55 valid events → BATCH_SIZE=50 means 2 batches (50+5)
        let events: Vec<&str> = std::iter::repeat(sample_fsd_jump()).take(55).collect();
        let dir = make_journal_dir(&events);

        let (port, req_count) = spawn_mock_server(|_| 202);
        let rx = start_bulk_upload(dir.clone(), format!("http://127.0.0.1:{}", port));

        let last = rx.into_iter().last().unwrap();
        assert!(last.done, "upload should be marked done");
        assert_eq!(last.lines_done, 55, "all 55 events should be counted");
        assert!(last.error.is_none(), "no errors expected: {:?}", last.error);
        assert_eq!(*req_count.lock().unwrap(), 2, "expected 2 batch requests");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn upload_counts_504_as_send_error() {
        let events: Vec<&str> = std::iter::repeat(sample_fsd_jump()).take(5).collect();
        let dir = make_journal_dir(&events);

        let (port, _) = spawn_mock_server(|_| 504);
        let rx = start_bulk_upload(dir.clone(), format!("http://127.0.0.1:{}", port));

        let last = rx.into_iter().last().unwrap();
        assert!(last.done);
        assert!(
            last.error.is_some(),
            "504 response should be recorded as a send error"
        );
        assert!(
            last.error.as_deref().unwrap_or("").contains("send error"),
            "error message should mention 'send error', got: {:?}",
            last.error
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn upload_skips_empty_and_malformed_lines() {
        let lines = [
            sample_fsd_jump(),
            "",
            "   ",
            "not json",
            sample_fsd_jump(),
        ];
        let dir = make_journal_dir(&lines);

        let (port, req_count) = spawn_mock_server(|_| 202);
        let rx = start_bulk_upload(dir.clone(), format!("http://127.0.0.1:{}", port));

        let last = rx.into_iter().last().unwrap();
        assert!(last.done);
        assert_eq!(last.lines_done, 2, "only valid JSON lines count");
        assert!(last.error.is_none());
        // 2 valid events fit in a single batch
        assert_eq!(*req_count.lock().unwrap(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn upload_empty_directory_reports_no_files_error() {
        let dir = std::env::temp_dir().join(format!(
            "edcas_upload_test_empty_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();

        // No server needed — upload fails immediately
        let rx = start_bulk_upload(dir.clone(), "http://127.0.0.1:1".into());

        let last = rx.into_iter().last().unwrap();
        assert!(last.done);
        assert!(last.error.is_some(), "empty dir should report an error");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn upload_mixed_success_and_error_accumulates_count() {
        // 55 events → 2 batches (50+5).  First batch (idx=0) succeeds; all
        // subsequent requests (retries of batch 1) return 503 so that batch
        // exhausts its retries and counts as exactly 1 send error.
        let events: Vec<&str> = std::iter::repeat(sample_fsd_jump()).take(55).collect();
        let dir = make_journal_dir(&events);

        let (port, _) = spawn_mock_server(|idx| if idx == 0 { 202 } else { 503 });
        let rx = start_bulk_upload(dir.clone(), format!("http://127.0.0.1:{}", port));

        let last = rx.into_iter().last().unwrap();
        assert!(last.done);
        assert_eq!(last.lines_done, 55);
        assert!(last.error.as_deref().unwrap_or("").contains("1 send error"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
