pub mod colonisation;
pub mod scan;
pub mod station;
pub mod travel;
pub mod types;

pub use colonisation::ColonisationConstructionDepot;
pub use scan::{
    FssAllBodiesFound, FssBodySignals, FssDiscoveryScan, FssSignalDiscovered, NavBeaconScan,
    SaaSignalsFound, Scan, ScanBaryCentre, ScanOrganic,
};
pub use station::{Commodities, Docked, Outfitting, Shipyard};
pub use station::CarrierStats;
pub use travel::{CarrierJump, FsdJump, Location};

/// Discriminated union of all journal events that edcas processes.
///
/// Use `JournalEvent::from_line` or `JournalEvent::from_json` to parse.
/// Unknown / unhandled event types simply return `None`.
#[derive(Debug, Clone)]
pub enum JournalEvent {
    FsdJump(FsdJump),
    Location(Location),
    CarrierJump(CarrierJump),
    Scan(Scan),
    ScanBaryCentre(ScanBaryCentre),
    SaaSignalsFound(SaaSignalsFound),
    FssBodySignals(FssBodySignals),
    FssSignalDiscovered(FssSignalDiscovered),
    Docked(Docked),
    Commodities(Commodities),
    Outfitting(Outfitting),
    Shipyard(Shipyard),
    ColonisationConstructionDepot(ColonisationConstructionDepot),
    CarrierStats(CarrierStats),
    FssDiscoveryScan(FssDiscoveryScan),
    FssAllBodiesFound(FssAllBodiesFound),
    NavBeaconScan(NavBeaconScan),
    ScanOrganic(ScanOrganic),
}

impl JournalEvent {
    /// Parse a raw JSON line from a journal file or EDDN message field.
    pub fn from_line(line: &str) -> Option<Self> {
        let value: serde_json::Value = serde_json::from_str(line).ok()?;
        Self::from_json(value)
    }

    /// Parse a `serde_json::Value` already extracted from the journal or EDDN message.
    pub fn from_json(value: serde_json::Value) -> Option<Self> {
        let event_type = value.get("event")?.as_str()?.to_owned();
        Self::from_tagged(&event_type, value)
    }

    /// Parse an EDDN message that may use non-journal schemas (commodities, outfitting, shipyard).
    pub fn from_eddn_message(value: serde_json::Value) -> Option<Self> {
        if let Some(event) = value.get("event").and_then(|v| v.as_str()).map(str::to_owned) {
            return Self::from_tagged(&event, value);
        }
        // Non-journal EDDN schemas
        if value.get("commodities").is_some() {
            return serde_json::from_value::<Commodities>(value)
                .ok()
                .map(Self::Commodities);
        }
        if value.get("modules").is_some() {
            return serde_json::from_value::<Outfitting>(value)
                .ok()
                .map(Self::Outfitting);
        }
        if value.get("ships").is_some() {
            return serde_json::from_value::<Shipyard>(value)
                .ok()
                .map(Self::Shipyard);
        }
        None
    }

    fn from_tagged(event_type: &str, value: serde_json::Value) -> Option<Self> {
        match event_type {
            "FSDJump" => serde_json::from_value::<FsdJump>(value)
                .ok()
                .map(Self::FsdJump),
            "Location" => serde_json::from_value::<Location>(value)
                .ok()
                .map(Self::Location),
            "CarrierJump" => serde_json::from_value::<CarrierJump>(value)
                .ok()
                .map(Self::CarrierJump),
            "Scan" => serde_json::from_value::<Scan>(value)
                .ok()
                .map(Self::Scan),
            "ScanBaryCentre" => serde_json::from_value::<ScanBaryCentre>(value)
                .ok()
                .map(Self::ScanBaryCentre),
            "SAASignalsFound" => serde_json::from_value::<SaaSignalsFound>(value)
                .ok()
                .map(Self::SaaSignalsFound),
            "FSSBodySignals" => serde_json::from_value::<FssBodySignals>(value)
                .ok()
                .map(Self::FssBodySignals),
            "FSSSignalDiscovered" => serde_json::from_value::<FssSignalDiscovered>(value)
                .ok()
                .map(Self::FssSignalDiscovered),
            "Docked" => serde_json::from_value::<Docked>(value)
                .ok()
                .map(Self::Docked),
            "ColonisationConstructionDepot" => {
                serde_json::from_value::<ColonisationConstructionDepot>(value)
                    .ok()
                    .map(Self::ColonisationConstructionDepot)
            }
            "CarrierStats" => serde_json::from_value::<CarrierStats>(value)
                .ok()
                .map(Self::CarrierStats),
            "FSSDiscoveryScan" => serde_json::from_value::<FssDiscoveryScan>(value)
                .ok()
                .map(Self::FssDiscoveryScan),
            "FSSAllBodiesFound" => serde_json::from_value::<FssAllBodiesFound>(value)
                .ok()
                .map(Self::FssAllBodiesFound),
            "NavBeaconScan" => serde_json::from_value::<NavBeaconScan>(value)
                .ok()
                .map(Self::NavBeaconScan),
            "ScanOrganic" => serde_json::from_value::<ScanOrganic>(value)
                .ok()
                .map(Self::ScanOrganic),
            _ => None,
        }
    }

    pub fn event_type(&self) -> &'static str {
        match self {
            Self::FsdJump(_) => "FSDJump",
            Self::Location(_) => "Location",
            Self::CarrierJump(_) => "CarrierJump",
            Self::Scan(_) => "Scan",
            Self::ScanBaryCentre(_) => "ScanBaryCentre",
            Self::SaaSignalsFound(_) => "SAASignalsFound",
            Self::FssBodySignals(_) => "FSSBodySignals",
            Self::FssSignalDiscovered(_) => "FSSSignalDiscovered",
            Self::Docked(_) => "Docked",
            Self::Commodities(_) => "commodities",
            Self::Outfitting(_) => "outfitting",
            Self::Shipyard(_) => "shipyard",
            Self::ColonisationConstructionDepot(_) => "ColonisationConstructionDepot",
            Self::CarrierStats(_) => "CarrierStats",
            Self::FssDiscoveryScan(_) => "FSSDiscoveryScan",
            Self::FssAllBodiesFound(_) => "FSSAllBodiesFound",
            Self::NavBeaconScan(_) => "NavBeaconScan",
            Self::ScanOrganic(_) => "ScanOrganic",
        }
    }
}
