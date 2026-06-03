//! EDDN (Elite Dangerous Data Network) uploader.
//!
//! Mirrors the edcas-API upload thread in [`crate::journal_reader`], but instead of
//! forwarding raw journal lines to the edcas server it converts them into the public
//! EDDN schemas and POSTs them to the EDDN gateway, the same way EDMC / EDDiscovery do.
//!
//! Only a curated, sanitized subset of data is sent. See the EDDN developer docs:
//! <https://github.com/EDCD/EDDN/blob/live/docs/Developers.md>
#![cfg(not(target_arch = "wasm32"))]

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

        let mut state = EddnState::default();
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
    let timestamp = outfitting.get("timestamp").and_then(|v| v.as_str())?;
    let items = outfitting.get("Items").and_then(|v| v.as_array())?;

    let modules: Vec<String> = items
        .iter()
        .filter_map(|it| it.get("Name").and_then(|v| v.as_str()))
        .map(|s| s.to_lowercase())
        .collect();
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
    let timestamp = shipyard.get("timestamp").and_then(|v| v.as_str())?;
    let price_list = shipyard.get("PriceList").and_then(|v| v.as_array())?;

    let ships: Vec<String> = price_list
        .iter()
        .filter_map(|s| s.get("ShipType").and_then(|v| v.as_str()))
        .map(|s| s.to_lowercase())
        .collect();
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
