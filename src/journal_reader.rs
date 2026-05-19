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
    CarrierJump, FsdJump, FssSignalDiscovered, JournalEvent, Location, Scan, SupercruiseExit,
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

#[derive(Clone, Default)]
pub struct PilotData {
    pub name: String,
    pub credits: i64,
    pub ship_type: String,
    pub ship_name: String,
    pub ship_ident: String,
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

impl ConstructionDepotData {
    pub fn progress(&self) -> f32 {
        self.submission.progress
    }
    pub fn station_name(&self) -> &str {
        &self.submission.station_name
    }
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
            pending_host_body_id: None,
            visited_systems: Vec::new(),
            visited_stations: Vec::new(),
            visited_carriers: Vec::new(),
            fss_body_count: None,
            fss_non_body_count: None,
            fss_all_bodies_found: false,
            nav_beacon_bodies: None,
            organic_scans: Vec::new(),
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

        // Loadout carries per-module health and detailed ship stats not in the typed event system.
        // Extract it before from_json consumes value.
        if value.get("event").and_then(|e| e.as_str()) == Some("Loadout") {
            self.pilot.ship_type = value["Ship_Localised"].as_str()
                .or_else(|| value["Ship"].as_str())
                .unwrap_or("").to_string();
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
                        };
                        if !self.stations.iter().any(|s| s.market_id == market_id) {
                            self.stations.push(station.clone());
                        }
                        if station_type == "FleetCarrier" {
                            self.visited_carriers.retain(|s| s.market_id != market_id);
                            self.visited_carriers.insert(0, station);
                        } else {
                            self.visited_stations.retain(|s| s.market_id != market_id);
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
                            self.pilot.ship_type = v["Ship_Localised"].as_str()
                                .or_else(|| v["Ship"].as_str())
                                .unwrap_or("").to_string();
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
                };
                if !self.stations.iter().any(|s| s.market_id == station.market_id) {
                    self.stations.push(station.clone());
                }
                if e.station_type == "FleetCarrier" {
                    self.visited_carriers.retain(|s| s.market_id != station.market_id);
                    self.visited_carriers.insert(0, station);
                } else {
                    self.visited_stations.retain(|s| s.market_id != station.market_id);
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
    }
}

fn faction_info_from_journal(
    f: &edcas_common::journal::types::Faction,
    conflicts: Option<&Vec<Conflict>>,
) -> FactionInfo {
    let conflict = conflicts.and_then(|cs| {
        cs.iter()
            .find(|c| c.faction1.name == f.name || c.faction2.name == f.name)
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
            load_existing_files(&journal_dir, &mut journal_data);
            load_cargo_file(&journal_dir.join("Cargo.json"), &mut journal_data);
            journal_data.backpack = load_onfoot_file(&journal_dir.join("Backpack.json"));
            journal_data.shiplocker = load_onfoot_file(&journal_dir.join("ShipLocker.json"));
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
            .timeout(Duration::from_secs(10))
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
        let endpoint = format!("{}/api/v1/journal/event", api_url);
        info!("Starting single-event upload to {}", endpoint);
        let mut lines_done: u64 = 0;
        let mut send_errors: u64 = 0;

        for (i, file) in files.iter().enumerate() {
            if let Ok(content) = std::fs::read_to_string(file) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    // Skip events over 64 KB — these are large inventory snapshots
                    // (ShipLocker, Backpack, NavRoute) that have no server handler.
                    if trimmed.len() > 65_536 {
                        continue;
                    }
                    if let Ok(message) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        let event_type = message.get("event")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string();
                        let body = serde_json::json!({
                            "$schemaRef": "edcas-client-upload/v1",
                            "message": message
                        });
                        let result = client.post(&endpoint).json(&body).send();
                        // On transport error, retry once — stale keep-alive connections
                        // get closed by nginx between requests.
                        let result = match result {
                            Err(ref e) if e.is_request() || e.is_connect() => {
                                client.post(&endpoint).json(&body).send()
                            }
                            other => other,
                        };
                        match result {
                            Err(e) => {
                                error!("Send failed ({event_type}): {e:#}");
                                send_errors += 1;
                            }
                            Ok(r) if !r.status().is_success() && r.status().as_u16() != 204 => {
                                error!("Send failed ({event_type}): HTTP {}", r.status());
                                send_errors += 1;
                            }
                            Ok(_) => {}
                        }
                        lines_done += 1;
                    }
                }
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
fn load_existing_files(dir: &Path, data: &mut JournalData) {
    if !dir.exists() || !dir.is_dir() {
        warn!("Journal directory does not exist: {}", dir.display());
        return;
    }
    let files = find_all_journal_files(dir);
    if files.is_empty() {
        warn!("No journal file found in: {}", dir.display());
        return;
    }
    info!("Loading journal file: {}", files[0].display());
    read_file_lines(&files[0], data);

    // When the latest file starts with a Location (same system, new game session),
    // the scans from the previous session are in older files — backfill them.
    if data.current_system.is_some() && data.bodies.is_empty() {
        let system_address = data.current_system.as_ref().unwrap().system_address;
        for prev_file in files.iter().skip(1).take(3) {
            info!("Backfilling scans from previous journal: {}", prev_file.display());
            load_previous_scans_for_system(prev_file, system_address, data);
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
    let mut last_file: Option<PathBuf> = None;
    let mut last_position: u64 = 0;
    let mut last_cargo_mtime: Option<SystemTime> = None;
    let mut last_backpack_mtime: Option<SystemTime> = None;
    let mut last_shiplocker_mtime: Option<SystemTime> = None;
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
                last_position = 0;
                data.clear();
                read_file_lines(&active, data);
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
fn read_file_lines(path: &Path, data: &mut JournalData) {
    match File::open(path) {
        Ok(file) => {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                data.process_line(&line);
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
fn load_previous_scans_for_system(path: &Path, system_address: i64, data: &mut JournalData) {
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
