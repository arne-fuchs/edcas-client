//! EDDN (Elite Dangerous Data Network) uploader.
//!
//! Mirrors the edcas-API upload thread in [`crate::journal_reader`], but instead of
//! forwarding raw journal lines to the edcas server it converts them into the public
//! EDDN schemas and POSTs them to the EDDN gateway, the same way EDMC / EDDiscovery do.
//!
//! Only a curated, sanitized subset of data is sent. See the EDDN developer docs:
//! <https://github.com/EDCD/EDDN/blob/live/docs/Developers.md>
#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashSet;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde_json::{json, Map, Value};
use tracing::*;

const SOFTWARE_NAME: &str = "EDCAS";
const JOURNAL_SCHEMA: &str = "https://eddn.edcd.io/schemas/journal/1";
const COMMODITY_SCHEMA: &str = "https://eddn.edcd.io/schemas/commodity/3";
const OUTFITTING_SCHEMA: &str = "https://eddn.edcd.io/schemas/outfitting/2";
const SHIPYARD_SCHEMA: &str = "https://eddn.edcd.io/schemas/shipyard/2";
const FSS_DISCOVERY_SCAN_SCHEMA: &str = "https://eddn.edcd.io/schemas/fssdiscoveryscan/1";
const FSS_ALL_BODIES_FOUND_SCHEMA: &str = "https://eddn.edcd.io/schemas/fssallbodiesfound/1";
const FSS_BODY_SIGNALS_SCHEMA: &str = "https://eddn.edcd.io/schemas/fssbodysignals/1";
const FSS_SIGNAL_DISCOVERED_SCHEMA: &str = "https://eddn.edcd.io/schemas/fsssignaldiscovered/1";
const NAV_BEACON_SCAN_SCHEMA: &str = "https://eddn.edcd.io/schemas/navbeaconscan/1";
const SCAN_BARYCENTRE_SCHEMA: &str = "https://eddn.edcd.io/schemas/scanbarycentre/1";
const CODEX_ENTRY_SCHEMA: &str = "https://eddn.edcd.io/schemas/codexentry/1";
const NAV_ROUTE_SCHEMA: &str = "https://eddn.edcd.io/schemas/navroute/1";
const APPROACH_SETTLEMENT_SCHEMA: &str = "https://eddn.edcd.io/schemas/approachsettlement/1";
const FC_MATERIALS_SCHEMA: &str = "https://eddn.edcd.io/schemas/fcmaterials_journal/1";
const DOCKING_GRANTED_SCHEMA: &str = "https://eddn.edcd.io/schemas/dockinggranted/1";
const DOCKING_DENIED_SCHEMA: &str = "https://eddn.edcd.io/schemas/dockingdenied/1";

/// Journal events that the EDDN `journal/1` schema accepts. Everything else is dropped.
const ALLOWED_JOURNAL_EVENTS: &[&str] =
    &["Docked", "FSDJump", "Scan", "Location", "SAASignalsFound", "CarrierJump"];

/// Faction sub-object keys that must be stripped from FSDJump / Location events.
const FACTION_PRIVATE_KEYS: &[&str] =
    &["HappiestSystem", "HomeSystem", "MyReputation", "SquadronFaction"];

#[derive(Clone, Copy, Debug)]
pub enum CompanionKind {
    Market,
    Outfitting,
    Shipyard,
    /// `NavRoute.json` — the plotted multi-jump route. Not station-bound, so it has no
    /// docked-station gating like the other companions.
    NavRoute,
}

/// Input fed to the uploader thread.
pub enum EddnInput {
    /// Update augmentation state only; do **not** upload. Used to seed state from the
    /// current session file at startup without re-sending historical events.
    Seed(String),
    /// A live journal line: update state and upload it if it is an eligible event.
    Line(String),
    /// A station companion file (Market/Outfitting/Shipyard.json) to convert and upload.
    Companion { kind: CompanionKind, content: String },
}

/// Configuration for the EDDN uploader.
pub struct EddnConfig {
    /// Upload gateway URL, e.g. `https://eddn.edcd.io:4430/upload/`.
    pub url: String,
    /// When true, `/test` is appended to every `$schemaRef` so messages hit the EDDN
    /// test pipeline (validated but not relayed to consumers).
    pub test_mode: bool,
    /// MarketIDs of fleet carriers the commander owns (from persisted state). At an
    /// owned carrier the game writes the player's *own* parked fleet into `Shipyard.json`
    /// (priced at resale value), so its shipyard must not be uploaded.
    pub owned_carriers: HashSet<i64>,
}

/// Augmentation state accumulated from the journal line stream. EDDN requires several
/// fields (system position, game version, horizons/odyssey flags) that individual events
/// don't carry, so we track them from the events that do.
#[derive(Default, Clone)]
pub struct EddnState {
    pub uploader_id: String,
    pub gameversion: String,
    pub gamebuild: String,
    pub horizons: Option<bool>,
    pub odyssey: Option<bool>,
    pub star_system: Option<String>,
    pub system_address: Option<i64>,
    pub star_pos: Option<[f64; 3]>,
    /// MarketID of the station we are currently docked at, if any. Used to ensure that
    /// market/outfitting/shipyard companion files are only uploaded for the live station
    /// (never a stale file, nor the player's own stored-ship list).
    pub docked_market_id: Option<i64>,
    /// MarketIDs of fleet carriers the commander owns. Seeded from persisted state and
    /// extended live from `CarrierStats` events.
    pub owned_carriers: HashSet<i64>,
}

/// Spawns the uploader thread and returns the sender used to feed it.
pub fn start_uploader(config: EddnConfig) -> mpsc::Sender<EddnInput> {
    let (tx, rx) = mpsc::channel::<EddnInput>();
    thread::spawn(move || {
        let client = match reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                error!("EDDN: failed to build HTTP client: {e} — uploads disabled");
                return;
            }
        };
        info!(
            "EDDN uploader started (url={}, test_mode={})",
            config.url, config.test_mode
        );

        let mut state = EddnState {
            owned_carriers: config.owned_carriers.clone(),
            ..Default::default()
        };
        for input in rx {
            match input {
                EddnInput::Seed(line) => {
                    if let Ok(v) = serde_json::from_str::<Value>(&line) {
                        update_state(&mut state, &v);
                    }
                }
                EddnInput::Line(line) => {
                    if let Ok(v) = serde_json::from_str::<Value>(&line) {
                        update_state(&mut state, &v);
                        if let Some(envelope) = build_line_message(&v, &state, config.test_mode) {
                            post(&client, &config.url, &envelope);
                        }
                    }
                }
                EddnInput::Companion { kind, content } => {
                    if let Ok(v) = serde_json::from_str::<Value>(&content) {
                        let envelope = match kind {
                            CompanionKind::Market => {
                                build_commodity_message(&v, &state, config.test_mode)
                            }
                            CompanionKind::Outfitting => {
                                build_outfitting_message(&v, &state, config.test_mode)
                            }
                            CompanionKind::Shipyard => {
                                build_shipyard_message(&v, &state, config.test_mode)
                            }
                            CompanionKind::NavRoute => {
                                build_navroute_message(&v, &state, config.test_mode)
                            }
                        };
                        match envelope {
                            Some(envelope) => post(&client, &config.url, &envelope),
                            None => debug!(
                                "EDDN: {kind:?} companion not uploaded \
                                 (not docked at its market, or it's your own carrier)"
                            ),
                        }
                    }
                }
            }
        }
        info!("EDDN uploader stopped");
    });
    tx
}

/// Updates augmentation state from a journal event.
fn update_state(state: &mut EddnState, event: &Value) {
    let evt = event.get("event").and_then(|v| v.as_str()).unwrap_or("");
    match evt {
        "Fileheader" => {
            if let Some(gv) = event.get("gameversion").and_then(|v| v.as_str()) {
                state.gameversion = gv.to_string();
            }
            if let Some(b) = event.get("build").and_then(|v| v.as_str()) {
                state.gamebuild = b.trim().to_string();
            }
        }
        "LoadGame" => {
            if let Some(name) = event.get("Commander").and_then(|v| v.as_str()) {
                state.uploader_id = name.to_string();
            }
            // Only record horizons/odyssey when actually present — EDDN forbids
            // sending these keys with a guessed/false value.
            if let Some(h) = event.get("Horizons").and_then(|v| v.as_bool()) {
                state.horizons = Some(h);
            }
            if let Some(o) = event.get("Odyssey").and_then(|v| v.as_bool()) {
                state.odyssey = Some(o);
            }
            if state.gameversion.is_empty() {
                if let Some(gv) = event.get("gameversion").and_then(|v| v.as_str()) {
                    state.gameversion = gv.to_string();
                }
            }
            if state.gamebuild.is_empty() {
                if let Some(b) = event.get("build").and_then(|v| v.as_str()) {
                    state.gamebuild = b.trim().to_string();
                }
            }
        }
        "Commander" => {
            if let Some(name) = event.get("Name").and_then(|v| v.as_str()) {
                state.uploader_id = name.to_string();
            }
        }
        "Location" | "FSDJump" | "CarrierJump" => {
            if let Some(s) = event.get("StarSystem").and_then(|v| v.as_str()) {
                state.star_system = Some(s.to_string());
            }
            if let Some(a) = event.get("SystemAddress").and_then(|v| v.as_i64()) {
                state.system_address = Some(a);
            }
            if let Some(p) = event.get("StarPos").and_then(parse_star_pos) {
                state.star_pos = Some(p);
            }
            // An FSDJump means we left any station; a Location/CarrierJump may report
            // being docked, so honour its `Docked` flag.
            if evt == "FSDJump" {
                state.docked_market_id = None;
            } else if event.get("Docked").and_then(|v| v.as_bool()) == Some(true) {
                state.docked_market_id = event.get("MarketID").and_then(|v| v.as_i64());
            } else if event.get("Docked").and_then(|v| v.as_bool()) == Some(false) {
                state.docked_market_id = None;
            }
        }
        "Docked" => {
            state.docked_market_id = event.get("MarketID").and_then(|v| v.as_i64());
        }
        "Undocked" => {
            state.docked_market_id = None;
        }
        // These events only ever fire for a carrier the commander owns; the CarrierID is
        // that carrier's MarketID.
        "CarrierStats" | "CarrierBuy" => {
            if let Some(id) = event.get("CarrierID").and_then(|v| v.as_i64()) {
                state.owned_carriers.insert(id);
            }
        }
        _ => {}
    }
}

fn parse_star_pos(v: &Value) -> Option<[f64; 3]> {
    let arr = v.as_array()?;
    if arr.len() != 3 {
        return None;
    }
    Some([arr[0].as_f64()?, arr[1].as_f64()?, arr[2].as_f64()?])
}

/// Recursively removes every key ending in `_Localised`.
fn strip_localised(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|k, _| !k.ends_with("_Localised"));
            for v in map.values_mut() {
                strip_localised(v);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                strip_localised(v);
            }
        }
        _ => {}
    }
}

/// Builds a wrapped EDDN `journal/1` envelope, or `None` if the event is not eligible.
pub fn build_journal_message(event: &Value, state: &EddnState, test_mode: bool) -> Option<Value> {
    let evt = event.get("event")?.as_str()?;
    if !ALLOWED_JOURNAL_EVENTS.contains(&evt) {
        return None;
    }

    let mut msg = event.as_object()?.clone();

    // 1. Strip all localised display strings.
    let mut wrapped = Value::Object(msg);
    strip_localised(&mut wrapped);
    msg = match wrapped {
        Value::Object(m) => m,
        _ => return None,
    };

    // 2. Per-event removal of personal / volatile fields.
    match evt {
        "Docked" => {
            for k in ["Wanted", "ActiveFine", "CockpitBreach"] {
                msg.remove(k);
            }
        }
        "FSDJump" => {
            for k in ["Wanted", "BoostUsed", "FuelLevel", "FuelUsed", "JumpDist"] {
                msg.remove(k);
            }
            strip_faction_private(&mut msg);
        }
        "Location" => {
            for k in ["Wanted", "Latitude", "Longitude"] {
                msg.remove(k);
            }
            strip_faction_private(&mut msg);
        }
        _ => {}
    }

    // 3. Augment with system identity / position from tracked state.
    augment_system(&mut msg, state)?;

    // 4. horizons / odyssey flags (only when known).
    if let Some(h) = state.horizons {
        msg.insert("horizons".to_string(), json!(h));
    }
    if let Some(o) = state.odyssey {
        msg.insert("odyssey".to_string(), json!(o));
    }

    Some(wrap(JOURNAL_SCHEMA, Value::Object(msg), state, test_mode))
}

fn strip_faction_private(msg: &mut Map<String, Value>) {
    if let Some(Value::Array(factions)) = msg.get_mut("Factions") {
        for faction in factions {
            if let Value::Object(f) = faction {
                for k in FACTION_PRIVATE_KEYS {
                    f.remove(*k);
                }
            }
        }
    }
}

/// Ensures the message carries `StarSystem`, `SystemAddress` and `StarPos`, filling them
/// from tracked state when absent. Returns `None` if a value can't be supplied or if the
/// event's own system identity contradicts our tracked position (stale data).
fn augment_system(msg: &mut Map<String, Value>, state: &EddnState) -> Option<()> {
    augment_system_keyed(msg, state, "StarSystem")
}

/// Like [`augment_system`], but the system-name property key varies between schemas
/// (`StarSystem`, `SystemName`, or `System`). `SystemAddress`/`StarPos` are constant.
fn augment_system_keyed(
    msg: &mut Map<String, Value>,
    state: &EddnState,
    sys_key: &str,
) -> Option<()> {
    // Cross-check: if the event names a system that differs from our tracked one, the
    // augmented StarPos/system name would be wrong — drop the message.
    if let (Some(ev_addr), Some(state_addr)) = (
        msg.get("SystemAddress").and_then(|v| v.as_i64()),
        state.system_address,
    ) {
        if ev_addr != state_addr {
            return None;
        }
    }

    if !msg.contains_key(sys_key) {
        msg.insert(sys_key.to_string(), json!(state.star_system.clone()?));
    }
    if !msg.contains_key("SystemAddress") {
        msg.insert("SystemAddress".to_string(), json!(state.system_address?));
    }
    if !msg.contains_key("StarPos") {
        msg.insert("StarPos".to_string(), json!(state.star_pos?));
    }
    Some(())
}

/// Strips the journal `$<name>_name;` wrapping from a commodity symbol.
fn clean_commodity_name(raw: &str) -> String {
    let n = raw.trim_start_matches('$').trim_end_matches(';');
    let n = n.strip_suffix("_name").unwrap_or(n);
    n.to_lowercase()
}

/// Lowercases the symbol at `key` for each item, preserving order and dropping duplicates.
/// The EDDN outfitting/shipyard schemas require the module/ship arrays to be unique, and
/// the game's files can list the same entry more than once.
fn unique_symbols(items: &[Value], key: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    items
        .iter()
        .filter_map(|it| it.get(key).and_then(|v| v.as_str()))
        .map(|s| s.to_lowercase())
        .filter(|s| seen.insert(s.clone()))
        .collect()
}

/// Returns `true` if a station companion file (market/outfitting/shipyard) for `market_id`
/// may be uploaded — i.e. we are currently docked at exactly that station.
///
/// Every persistent-file uploader **must** gate on this. Companion files sit on disk
/// between sessions and are read by mtime, so without this check a stale file (or the
/// player's own stored-ship list written to `Shipyard.json`) could be uploaded as if it
/// were the live station's data. Streamed journal events don't need it — they're only
/// uploaded live and are already system-cross-checked in [`augment_system`].
fn companion_for_docked_station(state: &EddnState, market_id: i64) -> bool {
    state.docked_market_id == Some(market_id)
}

fn num(item: &Value, key: &str) -> Value {
    item.get(key)
        .filter(|v| v.is_number())
        .cloned()
        .unwrap_or_else(|| json!(0))
}

/// Builds an EDDN `commodity/3` envelope from a `Market.json` payload.
pub fn build_commodity_message(market: &Value, state: &EddnState, test_mode: bool) -> Option<Value> {
    let system = market.get("StarSystem").and_then(|v| v.as_str())?;
    let station = market.get("StationName").and_then(|v| v.as_str())?;
    let market_id = market.get("MarketID").and_then(|v| v.as_i64())?;
    if !companion_for_docked_station(state, market_id) {
        return None;
    }
    let timestamp = market.get("timestamp").and_then(|v| v.as_str())?;
    let items = market.get("Items").and_then(|v| v.as_array())?;

    let mut commodities = Vec::new();
    for item in items {
        let name_raw = match item.get("Name").and_then(|v| v.as_str()) {
            Some(n) => n,
            None => continue,
        };
        // Skip commodities that aren't really tradable on the open market.
        if item.get("legality").and_then(|v| v.as_str()).is_some_and(|s| !s.is_empty()) {
            continue;
        }
        commodities.push(json!({
            "name": clean_commodity_name(name_raw),
            "meanPrice": num(item, "MeanPrice"),
            "buyPrice": num(item, "BuyPrice"),
            "stock": num(item, "Stock"),
            "stockBracket": num(item, "StockBracket"),
            "sellPrice": num(item, "SellPrice"),
            "demand": num(item, "Demand"),
            "demandBracket": num(item, "DemandBracket"),
        }));
    }
    if commodities.is_empty() {
        return None;
    }

    let mut message = json!({
        "systemName": system,
        "stationName": station,
        "marketId": market_id,
        "timestamp": timestamp,
        "commodities": commodities,
    });
    add_flags(&mut message, state);
    Some(wrap(COMMODITY_SCHEMA, message, state, test_mode))
}

/// Builds an EDDN `outfitting/2` envelope from an `Outfitting.json` payload.
pub fn build_outfitting_message(
    outfitting: &Value,
    state: &EddnState,
    test_mode: bool,
) -> Option<Value> {
    let system = outfitting.get("StarSystem").and_then(|v| v.as_str())?;
    let station = outfitting.get("StationName").and_then(|v| v.as_str())?;
    let market_id = outfitting.get("MarketID").and_then(|v| v.as_i64())?;
    if !companion_for_docked_station(state, market_id) {
        return None;
    }
    let timestamp = outfitting.get("timestamp").and_then(|v| v.as_str())?;
    let items = outfitting.get("Items").and_then(|v| v.as_array())?;

    let modules = unique_symbols(items, "Name");
    if modules.is_empty() {
        return None;
    }

    let mut message = json!({
        "systemName": system,
        "stationName": station,
        "marketId": market_id,
        "timestamp": timestamp,
        "modules": modules,
    });
    add_flags(&mut message, state);
    Some(wrap(OUTFITTING_SCHEMA, message, state, test_mode))
}

/// Builds an EDDN `shipyard/2` envelope from a `Shipyard.json` payload.
pub fn build_shipyard_message(shipyard: &Value, state: &EddnState, test_mode: bool) -> Option<Value> {
    let system = shipyard.get("StarSystem").and_then(|v| v.as_str())?;
    let station = shipyard.get("StationName").and_then(|v| v.as_str())?;
    let market_id = shipyard.get("MarketID").and_then(|v| v.as_i64())?;
    if !companion_for_docked_station(state, market_id) {
        return None;
    }
    // At a carrier the commander owns, `Shipyard.json` lists the player's own parked fleet
    // (priced at resale value), not a public for-sale list — never upload it. Other
    // commanders' carriers (and stations) still report a genuine for-sale list.
    if state.owned_carriers.contains(&market_id) {
        return None;
    }
    let timestamp = shipyard.get("timestamp").and_then(|v| v.as_str())?;
    let price_list = shipyard.get("PriceList").and_then(|v| v.as_array())?;

    let ships = unique_symbols(price_list, "ShipType");
    if ships.is_empty() {
        return None;
    }

    let mut message = json!({
        "systemName": system,
        "stationName": station,
        "marketId": market_id,
        "timestamp": timestamp,
        "ships": ships,
    });
    add_flags(&mut message, state);
    Some(wrap(SHIPYARD_SCHEMA, message, state, test_mode))
}

/// Adds the optional `horizons` / `odyssey` flags to a commodity/outfitting/shipyard message.
fn add_flags(message: &mut Value, state: &EddnState) {
    if let Value::Object(map) = message {
        if let Some(h) = state.horizons {
            map.insert("horizons".to_string(), json!(h));
        }
        if let Some(o) = state.odyssey {
            map.insert("odyssey".to_string(), json!(o));
        }
    }
}

/// Clones a journal event, strips every `_Localised` string, and keeps only the keys in
/// `allowed`. The EDDN exploration/codex/docking schemas are all `additionalProperties:
/// false`, so a stray key (or a personal field like `Progress`/`IsNewEntry`) would get the
/// whole message rejected — this whitelist guarantees we only emit permitted keys. Returns
/// `None` if the event isn't a JSON object.
fn whitelist_event(event: &Value, allowed: &[&str]) -> Option<Map<String, Value>> {
    let mut cloned = event.clone();
    strip_localised(&mut cloned);
    let mut map = match cloned {
        Value::Object(m) => m,
        _ => return None,
    };
    map.retain(|k, _| allowed.contains(&k.as_str()));
    Some(map)
}

/// Whitelists the keys of every object in the array at `key`, in place. Array item objects
/// in these schemas are also `additionalProperties: false`.
fn whitelist_array_items(msg: &mut Map<String, Value>, key: &str, allowed: &[&str]) {
    if let Some(Value::Array(arr)) = msg.get_mut(key) {
        for item in arr.iter_mut() {
            if let Value::Object(m) = item {
                m.retain(|k, _| allowed.contains(&k.as_str()));
            }
        }
    }
}

/// Adds horizons/odyssey flags and wraps a finished message map in its EDDN envelope.
fn finish(
    msg: Map<String, Value>,
    schema: &str,
    state: &EddnState,
    test_mode: bool,
) -> Option<Value> {
    let mut message = Value::Object(msg);
    add_flags(&mut message, state);
    Some(wrap(schema, message, state, test_mode))
}

/// Routes a live journal line to the matching EDDN schema builder. The six `journal/1`
/// events keep going through [`build_journal_message`]; the richer exploration, codex and
/// docking events each have their own dedicated schema. Anything unrecognised yields `None`.
pub fn build_line_message(event: &Value, state: &EddnState, test_mode: bool) -> Option<Value> {
    let evt = event.get("event")?.as_str()?;
    match evt {
        "Docked" | "FSDJump" | "Scan" | "Location" | "SAASignalsFound" | "CarrierJump" => {
            build_journal_message(event, state, test_mode)
        }
        "FSSDiscoveryScan" => {
            // `Progress` is personal data and forbidden by the schema; the whitelist drops it.
            let mut msg = whitelist_event(
                event,
                &[
                    "timestamp",
                    "event",
                    "SystemName",
                    "StarPos",
                    "SystemAddress",
                    "BodyCount",
                    "NonBodyCount",
                ],
            )?;
            augment_system_keyed(&mut msg, state, "SystemName")?;
            finish(msg, FSS_DISCOVERY_SCAN_SCHEMA, state, test_mode)
        }
        "FSSAllBodiesFound" => {
            let mut msg = whitelist_event(
                event,
                &[
                    "timestamp",
                    "event",
                    "SystemName",
                    "StarPos",
                    "SystemAddress",
                    "Count",
                ],
            )?;
            augment_system_keyed(&mut msg, state, "SystemName")?;
            finish(msg, FSS_ALL_BODIES_FOUND_SCHEMA, state, test_mode)
        }
        "FSSBodySignals" => {
            let mut msg = whitelist_event(
                event,
                &[
                    "timestamp",
                    "event",
                    "StarSystem",
                    "StarPos",
                    "SystemAddress",
                    "BodyID",
                    "BodyName",
                    "Signals",
                ],
            )?;
            whitelist_array_items(&mut msg, "Signals", &["Type", "Count"]);
            augment_system_keyed(&mut msg, state, "StarSystem")?;
            finish(msg, FSS_BODY_SIGNALS_SCHEMA, state, test_mode)
        }
        "FSSSignalDiscovered" => build_fsssignaldiscovered_message(event, state, test_mode),
        "NavBeaconScan" => {
            let mut msg = whitelist_event(
                event,
                &[
                    "timestamp",
                    "event",
                    "StarSystem",
                    "StarPos",
                    "SystemAddress",
                    "NumBodies",
                ],
            )?;
            augment_system_keyed(&mut msg, state, "StarSystem")?;
            finish(msg, NAV_BEACON_SCAN_SCHEMA, state, test_mode)
        }
        "ScanBaryCentre" => {
            let mut msg = whitelist_event(
                event,
                &[
                    "timestamp",
                    "event",
                    "StarSystem",
                    "StarPos",
                    "SystemAddress",
                    "BodyID",
                    "SemiMajorAxis",
                    "Eccentricity",
                    "OrbitalInclination",
                    "Periapsis",
                    "OrbitalPeriod",
                    "AscendingNode",
                    "MeanAnomaly",
                ],
            )?;
            augment_system_keyed(&mut msg, state, "StarSystem")?;
            finish(msg, SCAN_BARYCENTRE_SCHEMA, state, test_mode)
        }
        "CodexEntry" => {
            // `IsNewEntry` / `NewTraitsDiscovered` are personal data and forbidden — dropped
            // by the whitelist. The system-name key for this schema is `System`.
            let mut msg = whitelist_event(
                event,
                &[
                    "timestamp",
                    "event",
                    "System",
                    "StarPos",
                    "SystemAddress",
                    "EntryID",
                    "Name",
                    "Region",
                    "Category",
                    "SubCategory",
                    "Latitude",
                    "Longitude",
                    "NearestDestination",
                    "VoucherAmount",
                    "Traits",
                    "BodyID",
                    "BodyName",
                ],
            )?;
            augment_system_keyed(&mut msg, state, "System")?;
            finish(msg, CODEX_ENTRY_SCHEMA, state, test_mode)
        }
        "ApproachSettlement" => {
            let mut msg = whitelist_event(
                event,
                &[
                    "timestamp",
                    "event",
                    "StarSystem",
                    "StarPos",
                    "SystemAddress",
                    "Name",
                    "BodyID",
                    "BodyName",
                    "Latitude",
                    "Longitude",
                    "MarketID",
                    "StationGovernment",
                    "StationAllegiance",
                    "StationEconomy",
                    "StationEconomies",
                    "StationFaction",
                    "StationServices",
                ],
            )?;
            // The schema requires the body identity and surface coordinates; settlements
            // approached without them (rare) are dropped rather than rejected by EDDN.
            for required in ["Name", "BodyID", "BodyName", "Latitude", "Longitude"] {
                if !msg.contains_key(required) {
                    return None;
                }
            }
            augment_system_keyed(&mut msg, state, "StarSystem")?;
            finish(msg, APPROACH_SETTLEMENT_SCHEMA, state, test_mode)
        }
        "FCMaterials" => {
            // Fleet-carrier on-foot materials market — carrier-bound, no system augmentation.
            let mut msg = whitelist_event(
                event,
                &[
                    "timestamp",
                    "event",
                    "MarketID",
                    "CarrierName",
                    "CarrierID",
                    "Items",
                ],
            )?;
            whitelist_array_items(
                &mut msg,
                "Items",
                &["id", "Name", "Price", "Stock", "Demand"],
            );
            finish(msg, FC_MATERIALS_SCHEMA, state, test_mode)
        }
        "DockingGranted" => {
            let msg = whitelist_event(
                event,
                &[
                    "timestamp",
                    "event",
                    "MarketID",
                    "StationName",
                    "StationType",
                    "LandingPad",
                ],
            )?;
            finish(msg, DOCKING_GRANTED_SCHEMA, state, test_mode)
        }
        "DockingDenied" => {
            let msg = whitelist_event(
                event,
                &[
                    "timestamp",
                    "event",
                    "MarketID",
                    "StationName",
                    "StationType",
                    "Reason",
                ],
            )?;
            finish(msg, DOCKING_DENIED_SCHEMA, state, test_mode)
        }
        _ => None,
    }
}

/// Builds an EDDN `fsssignaldiscovered/1` envelope from a single `FSSSignalDiscovered`
/// journal event. The schema groups signals into a `signals` array; we emit one message per
/// event (a single-element array), which is valid (`minItems: 1`) and avoids stateful
/// per-system batching. Mission-target USSs are personal and skipped.
fn build_fsssignaldiscovered_message(
    event: &Value,
    state: &EddnState,
    test_mode: bool,
) -> Option<Value> {
    if event.get("USSType").and_then(|v| v.as_str()) == Some("$USS_Type_MissionTarget;") {
        return None;
    }
    // The per-signal object is whitelisted separately from the message envelope. `TimeRemaining`
    // is volatile/personal and forbidden; it is excluded by omission from the allow-list.
    let signal = whitelist_event(
        event,
        &[
            "timestamp",
            "SignalName",
            "SignalType",
            "IsStation",
            "USSType",
            "SpawningState",
            "SpawningFaction",
            "SpawningPower",
            "OpposingPower",
            "ThreatLevel",
        ],
    )?;
    if !signal.contains_key("SignalName") {
        return None;
    }

    let mut msg = Map::new();
    msg.insert(
        "timestamp".to_string(),
        event.get("timestamp")?.clone(),
    );
    msg.insert("event".to_string(), json!("FSSSignalDiscovered"));
    if let Some(addr) = event.get("SystemAddress").and_then(|v| v.as_i64()) {
        msg.insert("SystemAddress".to_string(), json!(addr));
    }
    msg.insert("signals".to_string(), json!([Value::Object(signal)]));
    augment_system_keyed(&mut msg, state, "StarSystem")?;
    finish(msg, FSS_SIGNAL_DISCOVERED_SCHEMA, state, test_mode)
}

/// Builds an EDDN `navroute/1` envelope from a `NavRoute.json` payload. Not station-bound,
/// so no docked-station gating; each route waypoint is self-contained (no augmentation).
pub fn build_navroute_message(route: &Value, state: &EddnState, test_mode: bool) -> Option<Value> {
    let mut msg = whitelist_event(route, &["timestamp", "event", "Route"])?;
    whitelist_array_items(
        &mut msg,
        "Route",
        &["StarSystem", "SystemAddress", "StarPos", "StarClass"],
    );
    // A bare NavRoute.json (route cleared on arrival) has an empty Route — nothing to share.
    let has_route = msg
        .get("Route")
        .and_then(|r| r.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false);
    if !has_route {
        return None;
    }
    finish(msg, NAV_ROUTE_SCHEMA, state, test_mode)
}

/// Wraps a message body in the EDDN envelope (`$schemaRef` + `header` + `message`).
fn wrap(schema: &str, message: Value, state: &EddnState, test_mode: bool) -> Value {
    let schema_ref = if test_mode {
        format!("{schema}/test")
    } else {
        schema.to_string()
    };
    json!({
        "$schemaRef": schema_ref,
        "header": {
            "uploaderID": state.uploader_id,
            "softwareName": SOFTWARE_NAME,
            "softwareVersion": env!("CARGO_PKG_VERSION"),
            "gameversion": state.gameversion,
            "gamebuild": state.gamebuild,
        },
        "message": message,
    })
}

/// POSTs an envelope to EDDN, honoring the EDDN retry rules:
/// never retry 400/426, wait ≥60s before retrying transient failures.
fn post(client: &reqwest::blocking::Client, url: &str, envelope: &Value) {
    const MAX_RETRIES: u32 = 2;
    for attempt in 0..=MAX_RETRIES {
        match client.post(url).json(envelope).send() {
            Ok(resp) => {
                let status = resp.status().as_u16();
                match status {
                    200 => {
                        let schema = envelope
                            .get("$schemaRef")
                            .and_then(|v| v.as_str())
                            .unwrap_or("?");
                        debug!("EDDN: accepted {schema}");
                        return;
                    }
                    400 | 426 => {
                        // Bad/outdated schema — our fault, retrying won't help.
                        error!(
                            "EDDN rejected message (HTTP {status}); not retrying. Body: {}",
                            resp.text().unwrap_or_default()
                        );
                        return;
                    }
                    other => {
                        warn!("EDDN upload failed (HTTP {other}), attempt {}", attempt + 1);
                    }
                }
            }
            Err(e) => {
                warn!("EDDN upload error: {e}, attempt {}", attempt + 1);
            }
        }
        if attempt < MAX_RETRIES {
            // EDDN asks tools to wait at least a minute before retrying.
            thread::sleep(Duration::from_secs(60));
        } else {
            error!("EDDN upload giving up after {} attempts", MAX_RETRIES + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> EddnState {
        EddnState {
            uploader_id: "TestCmdr".into(),
            gameversion: "4.0.0.1477".into(),
            gamebuild: "r289563/r0".into(),
            horizons: Some(true),
            odyssey: Some(true),
            star_system: Some("Sol".into()),
            system_address: Some(10477373803),
            star_pos: Some([0.0, 0.0, 0.0]),
            docked_market_id: Some(3228032),
            owned_carriers: HashSet::new(),
        }
    }

    #[test]
    fn fsdjump_strips_localised_and_private_fields() {
        let event = json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "event": "FSDJump",
            "StarSystem": "Sol",
            "SystemAddress": 10477373803i64,
            "StarPos": [0.0, 0.0, 0.0],
            "Wanted": true,
            "FuelLevel": 16.0,
            "JumpDist": 8.5,
            "Economy_Localised": "Industrial",
            "Factions": [{
                "Name": "Some Faction",
                "MyReputation": 12.5,
                "HomeSystem": 1234,
                "FactionState_Localised": "Boom"
            }]
        });
        let env = build_journal_message(&event, &state(), false).unwrap();
        let msg = &env["message"];
        assert!(msg.get("Wanted").is_none());
        assert!(msg.get("FuelLevel").is_none());
        assert!(msg.get("JumpDist").is_none());
        assert!(msg.get("Economy_Localised").is_none());
        let faction = &msg["Factions"][0];
        assert!(faction.get("MyReputation").is_none());
        assert!(faction.get("HomeSystem").is_none());
        assert!(faction.get("FactionState_Localised").is_none());
        assert_eq!(msg["horizons"], json!(true));
        assert_eq!(msg["odyssey"], json!(true));
        assert_eq!(env["$schemaRef"], json!(JOURNAL_SCHEMA));
        assert_eq!(env["header"]["uploaderID"], json!("TestCmdr"));
        assert_eq!(env["header"]["softwareName"], json!("EDCAS"));
    }

    #[test]
    fn scan_is_augmented_with_system_position() {
        let event = json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "event": "Scan",
            "SystemAddress": 10477373803i64,
            "BodyName": "Sol 1",
        });
        let env = build_journal_message(&event, &state(), false).unwrap();
        let msg = &env["message"];
        assert_eq!(msg["StarSystem"], json!("Sol"));
        assert_eq!(msg["StarPos"], json!([0.0, 0.0, 0.0]));
    }

    #[test]
    fn scan_in_wrong_system_is_dropped() {
        let event = json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "event": "Scan",
            "SystemAddress": 999i64,
            "BodyName": "Elsewhere 1",
        });
        assert!(build_journal_message(&event, &state(), false).is_none());
    }

    #[test]
    fn non_whitelisted_event_is_dropped() {
        let event = json!({"timestamp": "t", "event": "Music", "MusicTrack": "MainMenu"});
        assert!(build_journal_message(&event, &state(), false).is_none());
    }

    #[test]
    fn test_mode_appends_test_suffix() {
        let event = json!({
            "timestamp": "t", "event": "Location",
            "StarSystem": "Sol", "SystemAddress": 10477373803i64, "StarPos": [0.0,0.0,0.0]
        });
        let env = build_journal_message(&event, &state(), true).unwrap();
        assert_eq!(env["$schemaRef"], json!(format!("{JOURNAL_SCHEMA}/test")));
    }

    #[test]
    fn horizons_omitted_when_unknown() {
        let mut st = state();
        st.horizons = None;
        st.odyssey = None;
        let event = json!({
            "timestamp": "t", "event": "Location",
            "StarSystem": "Sol", "SystemAddress": 10477373803i64, "StarPos": [0.0,0.0,0.0]
        });
        let env = build_journal_message(&event, &st, false).unwrap();
        assert!(env["message"].get("horizons").is_none());
        assert!(env["message"].get("odyssey").is_none());
    }

    #[test]
    fn commodity_message_maps_market_items() {
        let market = json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "event": "Market",
            "MarketID": 3228032i64,
            "StationName": "Galileo",
            "StarSystem": "Sol",
            "Items": [{
                "id": 128049204,
                "Name": "$gold_name;",
                "Name_Localised": "Gold",
                "Category": "$MARKET_category_metals;",
                "BuyPrice": 0, "SellPrice": 9401, "MeanPrice": 9009,
                "StockBracket": 0, "Stock": 0, "DemandBracket": 2, "Demand": 56
            }]
        });
        let env = build_commodity_message(&market, &state(), false).unwrap();
        assert_eq!(env["$schemaRef"], json!(COMMODITY_SCHEMA));
        let c = &env["message"]["commodities"][0];
        assert_eq!(c["name"], json!("gold"));
        assert_eq!(c["sellPrice"], json!(9401));
        assert_eq!(c["demand"], json!(56));
        assert!(c.get("Category").is_none());
    }

    #[test]
    fn shipyard_not_uploaded_when_not_docked_at_that_market() {
        // A stale Shipyard.json from a different/previous station (or the player's own
        // stored-ship list) must not be uploaded.
        let mut st = state();
        st.docked_market_id = Some(99999); // docked elsewhere
        let shipyard = json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "event": "Shipyard",
            "MarketID": 3228032i64,
            "StationName": "Galileo",
            "StarSystem": "Sol",
            "PriceList": [{"id": 1, "ShipType": "corsair", "ShipPrice": 1}]
        });
        assert!(build_shipyard_message(&shipyard, &st, false).is_none());

        // And not uploaded at all when we don't know we're docked.
        st.docked_market_id = None;
        assert!(build_shipyard_message(&shipyard, &st, false).is_none());
    }

    #[test]
    fn shipyard_at_own_carrier_is_suppressed_but_market_outfitting_upload() {
        // Docked at a carrier we own: Shipyard.json is our own parked fleet.
        let mut st = state();
        let carrier_id = 3704402432i64;
        st.docked_market_id = Some(carrier_id);
        st.owned_carriers.insert(carrier_id);

        let shipyard = json!({
            "timestamp": "2026-01-01T00:00:00Z", "event": "Shipyard",
            "MarketID": carrier_id, "StationName": "Q2K-BHB", "StarSystem": "Sol",
            "PriceList": [{"id": 0, "ShipType": "cutter", "ShipPrice": 815215912i64}]
        });
        assert!(build_shipyard_message(&shipyard, &st, false).is_none());

        // The carrier's commodity market and outfitting are genuine public data — still sent.
        let market = json!({
            "timestamp": "2026-01-01T00:00:00Z", "event": "Market",
            "MarketID": carrier_id, "StationName": "Q2K-BHB", "StarSystem": "Sol",
            "Items": [{"Name": "$gold_name;", "BuyPrice": 0, "SellPrice": 9401,
                       "MeanPrice": 9009, "StockBracket": 0, "Stock": 0,
                       "DemandBracket": 2, "Demand": 56}]
        });
        assert!(build_commodity_message(&market, &st, false).is_some());

        let outfitting = json!({
            "timestamp": "2026-01-01T00:00:00Z", "event": "Outfitting",
            "MarketID": carrier_id, "StationName": "Q2K-BHB", "StarSystem": "Sol",
            "Items": [{"id": 1, "Name": "int_engine_size3_class5"}]
        });
        assert!(build_outfitting_message(&outfitting, &st, false).is_some());
    }

    #[test]
    fn carrierstats_marks_carrier_as_owned() {
        let mut st = EddnState::default();
        update_state(&mut st, &json!({"event": "CarrierStats", "CarrierID": 3704402432i64}));
        assert!(st.owned_carriers.contains(&3704402432));
    }

    #[test]
    fn shipyard_ships_are_deduplicated() {
        let shipyard = json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "event": "Shipyard",
            "MarketID": 3228032i64,
            "StationName": "Galileo",
            "StarSystem": "Sol",
            "PriceList": [
                {"id": 1, "ShipType": "Cutter", "ShipPrice": 1},
                {"id": 2, "ShipType": "Type8", "ShipPrice": 2},
                {"id": 3, "ShipType": "cutter", "ShipPrice": 3},
                {"id": 4, "ShipType": "Type8", "ShipPrice": 4}
            ]
        });
        let env = build_shipyard_message(&shipyard, &state(), false).unwrap();
        let ships = env["message"]["ships"].as_array().unwrap();
        assert_eq!(ships, &[json!("cutter"), json!("type8")]);
    }

    #[test]
    fn update_state_tracks_fileheader_and_loadgame() {
        let mut st = EddnState::default();
        update_state(&mut st, &json!({
            "event": "Fileheader", "gameversion": "4.0.0.1477", "build": "r289563/r0 "
        }));
        update_state(&mut st, &json!({
            "event": "LoadGame", "Commander": "Jameson", "Horizons": true, "Odyssey": false
        }));
        assert_eq!(st.gameversion, "4.0.0.1477");
        assert_eq!(st.gamebuild, "r289563/r0");
        assert_eq!(st.uploader_id, "Jameson");
        assert_eq!(st.horizons, Some(true));
        assert_eq!(st.odyssey, Some(false));
    }

    #[test]
    fn fssdiscoveryscan_drops_progress_and_augments_starpos() {
        let event = json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "event": "FSSDiscoveryScan",
            "Progress": 0.5,
            "BodyCount": 21,
            "NonBodyCount": 33,
            "SystemName": "Sol",
            "SystemAddress": 10477373803i64
        });
        let env = build_line_message(&event, &state(), false).unwrap();
        assert_eq!(env["$schemaRef"], json!(FSS_DISCOVERY_SCAN_SCHEMA));
        let msg = &env["message"];
        assert!(msg.get("Progress").is_none(), "Progress is personal and forbidden");
        assert_eq!(msg["BodyCount"], json!(21));
        assert_eq!(msg["SystemName"], json!("Sol"));
        assert_eq!(msg["StarPos"], json!([0.0, 0.0, 0.0]));
    }

    #[test]
    fn fssbodysignals_whitelists_signal_items_and_augments() {
        let event = json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "event": "FSSBodySignals",
            "BodyName": "Sol 3",
            "BodyID": 9,
            "SystemAddress": 10477373803i64,
            "Signals": [{
                "Type": "$SAA_SignalType_Biological;",
                "Type_Localised": "Biological",
                "Count": 3
            }]
        });
        let env = build_line_message(&event, &state(), false).unwrap();
        assert_eq!(env["$schemaRef"], json!(FSS_BODY_SIGNALS_SCHEMA));
        let msg = &env["message"];
        assert_eq!(msg["StarSystem"], json!("Sol"));
        assert_eq!(msg["StarPos"], json!([0.0, 0.0, 0.0]));
        let sig = &msg["Signals"][0];
        assert_eq!(sig["Type"], json!("$SAA_SignalType_Biological;"));
        assert_eq!(sig["Count"], json!(3));
        assert!(sig.get("Type_Localised").is_none());
    }

    #[test]
    fn fsssignaldiscovered_wraps_single_signal_and_skips_mission_uss() {
        let station = json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "event": "FSSSignalDiscovered",
            "SystemAddress": 10477373803i64,
            "SignalName": "$MULTIPLAYER_SCENARIO42_TITLE;",
            "SignalName_Localised": "Nav Beacon",
            "IsStation": true
        });
        let env = build_line_message(&station, &state(), false).unwrap();
        assert_eq!(env["$schemaRef"], json!(FSS_SIGNAL_DISCOVERED_SCHEMA));
        let msg = &env["message"];
        assert_eq!(msg["StarSystem"], json!("Sol"));
        let signals = msg["signals"].as_array().unwrap();
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0]["SignalName"], json!("$MULTIPLAYER_SCENARIO42_TITLE;"));
        assert!(signals[0].get("SignalName_Localised").is_none());
        assert!(signals[0].get("event").is_none());

        // A personal mission-target USS must not be shared.
        let mission = json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "event": "FSSSignalDiscovered",
            "SystemAddress": 10477373803i64,
            "SignalName": "$USS;",
            "USSType": "$USS_Type_MissionTarget;"
        });
        assert!(build_line_message(&mission, &state(), false).is_none());
    }

    #[test]
    fn codexentry_uses_system_key_and_drops_personal_fields() {
        let event = json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "event": "CodexEntry",
            "EntryID": 2100401,
            "Name": "$Codex_Ent_Stratum_06_Name;",
            "Name_Localised": "Stratum Tectonicas",
            "System": "Sol",
            "SystemAddress": 10477373803i64,
            "IsNewEntry": true,
            "NewTraitsDiscovered": ["foo"]
        });
        let env = build_line_message(&event, &state(), false).unwrap();
        assert_eq!(env["$schemaRef"], json!(CODEX_ENTRY_SCHEMA));
        let msg = &env["message"];
        assert_eq!(msg["System"], json!("Sol"));
        assert_eq!(msg["StarPos"], json!([0.0, 0.0, 0.0]));
        assert_eq!(msg["EntryID"], json!(2100401));
        assert!(msg.get("IsNewEntry").is_none());
        assert!(msg.get("NewTraitsDiscovered").is_none());
        assert!(msg.get("Name_Localised").is_none());
    }

    #[test]
    fn navroute_keeps_route_waypoints_and_drops_empty() {
        let route = json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "event": "NavRoute",
            "Route": [{
                "StarSystem": "Sol",
                "SystemAddress": 10477373803i64,
                "StarPos": [0.0, 0.0, 0.0],
                "StarClass": "G",
                "Extra": "should be dropped"
            }]
        });
        let env = build_navroute_message(&route, &state(), false).unwrap();
        assert_eq!(env["$schemaRef"], json!(NAV_ROUTE_SCHEMA));
        let wp = &env["message"]["Route"][0];
        assert_eq!(wp["StarClass"], json!("G"));
        assert!(wp.get("Extra").is_none());

        let empty = json!({"timestamp": "t", "event": "NavRoute", "Route": []});
        assert!(build_navroute_message(&empty, &state(), false).is_none());
    }

    #[test]
    fn dockingdenied_passes_reason_through() {
        let event = json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "event": "DockingDenied",
            "Reason": "Distance",
            "MarketID": 3228032i64,
            "StationName": "Galileo",
            "StationType": "Orbis"
        });
        let env = build_line_message(&event, &state(), false).unwrap();
        assert_eq!(env["$schemaRef"], json!(DOCKING_DENIED_SCHEMA));
        let msg = &env["message"];
        assert_eq!(msg["Reason"], json!("Distance"));
        assert_eq!(msg["StationName"], json!("Galileo"));
    }

    #[test]
    fn fcmaterials_strips_item_localised() {
        let event = json!({
            "timestamp": "2026-01-01T00:00:00Z",
            "event": "FCMaterials",
            "MarketID": 3700005i64,
            "CarrierName": "TEST CARRIER",
            "CarrierID": "X9Z-99Z",
            "Items": [{
                "id": 128961528,
                "Name": "$insight_name;",
                "Name_Localised": "Insight",
                "Price": 100,
                "Stock": 5,
                "Demand": 0
            }]
        });
        let env = build_line_message(&event, &state(), false).unwrap();
        assert_eq!(env["$schemaRef"], json!(FC_MATERIALS_SCHEMA));
        let item = &env["message"]["Items"][0];
        assert_eq!(item["Name"], json!("$insight_name;"));
        assert!(item.get("Name_Localised").is_none());
    }
}
