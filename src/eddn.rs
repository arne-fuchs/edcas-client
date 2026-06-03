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

/// Journal events that the EDDN `journal/1` schema accepts. Everything else is dropped.
const ALLOWED_JOURNAL_EVENTS: &[&str] =
    &["Docked", "FSDJump", "Scan", "Location", "SAASignalsFound", "CarrierJump"];

/// Faction sub-object keys that must be stripped from FSDJump / Location events.
const FACTION_PRIVATE_KEYS: &[&str] =
    &["HappiestSystem", "HomeSystem", "MyReputation", "SquadronFaction"];

#[derive(Clone, Copy)]
pub enum CompanionKind {
    Market,
    Outfitting,
    Shipyard,
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
                        if let Some(envelope) = build_journal_message(&v, &state, config.test_mode) {
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
                        };
                        if let Some(envelope) = envelope {
                            post(&client, &config.url, &envelope);
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
    // Cross-check: if the event names a system that differs from our tracked one, the
    // augmented StarPos/StarSystem would be wrong — drop the message.
    if let (Some(ev_addr), Some(state_addr)) = (
        msg.get("SystemAddress").and_then(|v| v.as_i64()),
        state.system_address,
    ) {
        if ev_addr != state_addr {
            return None;
        }
    }

    if !msg.contains_key("StarSystem") {
        msg.insert("StarSystem".to_string(), json!(state.star_system.clone()?));
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
                    200 => return,
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
}
