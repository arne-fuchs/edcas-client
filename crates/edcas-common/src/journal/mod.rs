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
pub use station::{MarketJournal, OutfittingJournal, ShipyardJournal};
pub use station::CarrierStats;
pub use travel::{CarrierJump, FsdJump, Location, SupercruiseExit};

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
    SupercruiseExit(SupercruiseExit),
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
            // Companion-file journal events: the journal log also writes a brief trigger line
            // (e.g. {"event":"Market","MarketID":...}) with NO Items/PriceList.  Only the
            // full companion-file payload (uploaded separately via the watch loop) contains
            // actual data.  The non-empty guard prevents the empty trigger lines from being
            // dispatched to the DB and wiping existing data.
            "Market" => serde_json::from_value::<MarketJournal>(value)
                .ok()
                .filter(|m| !m.items.is_empty())
                .map(|m| Self::Commodities(Commodities::from(m))),
            "Outfitting" => serde_json::from_value::<OutfittingJournal>(value)
                .ok()
                .filter(|m| !m.items.is_empty())
                .map(|m| Self::Outfitting(Outfitting::from(m))),
            "Shipyard" => serde_json::from_value::<ShipyardJournal>(value)
                .ok()
                .filter(|s| !s.price_list.is_empty())
                .map(|s| Self::Shipyard(Shipyard::from(s))),
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
            "SupercruiseExit" => serde_json::from_value::<SupercruiseExit>(value)
                .ok()
                .map(Self::SupercruiseExit),
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
            Self::SupercruiseExit(_) => "SupercruiseExit",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real journal lines captured from actual game logs.  If any of these stop
    // parsing after a struct change the test will immediately tell you which
    // event type broke and show the deserialisation error via `unwrap()`.

    #[test]
    fn parse_fsd_jump() {
        let line = r#"{ "timestamp":"2022-06-28T15:16:44Z", "event":"FSDJump", "Taxi":false, "Multicrew":false, "StarSystem":"Baminyi", "SystemAddress":7269097612681, "StarPos":[91.90625,-110.34375,-75.12500], "SystemAllegiance":"Empire", "SystemEconomy":"$economy_Terraforming;", "SystemEconomy_Localised":"Terraformierung", "SystemSecondEconomy":"$economy_Military;", "SystemSecondEconomy_Localised":"Militaer", "SystemGovernment":"$government_Patronage;", "SystemGovernment_Localised":"Patronat", "SystemSecurity":"$SYSTEM_SECURITY_medium;", "SystemSecurity_Localised":"Mittlere Sicherheit", "Population":213202, "Body":"Baminyi", "BodyID":0, "BodyType":"Star", "Powers":[ "A. Lavigny-Duval" ], "PowerplayState":"Exploited", "JumpDist":20.974, "FuelUsed":10.195133, "FuelLevel":35.244869, "Factions":[ { "Name":"Baminyi Defence Force", "FactionState":"None", "Government":"Dictatorship", "Influence":0.109533, "Allegiance":"Empire", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Gluecklich", "MyReputation":0.0 } ], "SystemFaction":{ "Name":"United Imperial Loyalists", "FactionState":"Expansion" } }"#;
        let event = JournalEvent::from_line(line).expect("FSDJump must parse");
        assert!(matches!(event, JournalEvent::FsdJump(ref e) if e.star_system == "Baminyi"));
    }

    #[test]
    fn parse_location() {
        let line = r#"{ "timestamp":"2022-06-28T01:06:37Z", "event":"Location", "DistFromStarLS":750.392172, "Docked":false, "Taxi":false, "Multicrew":false, "StarSystem":"Hyades Sector OX-K b8-0", "SystemAddress":672296281473, "StarPos":[108.3125,-113.46875,-87.8125], "SystemAllegiance":"", "SystemEconomy":"$economy_None;", "SystemSecondEconomy":"$economy_None;", "SystemGovernment":"$government_None;", "SystemSecurity":"$GAlAXY_MAP_INFO_state_anarchy;", "Population":0, "Body":"Hyades Sector OX-K b8-0 5 A Ring", "BodyID":18, "BodyType":"PlanetaryRing" }"#;
        let event = JournalEvent::from_line(line).expect("Location must parse");
        assert!(matches!(event, JournalEvent::Location(_)));
    }

    #[test]
    fn parse_scan_star() {
        let line = r#"{ "timestamp":"2022-06-28T15:17:40Z", "event":"Scan", "ScanType":"AutoScan", "BodyName":"Hyades Sector TO-H b10-3 A", "BodyID":1, "Parents":[ {"Null":0} ], "StarSystem":"Hyades Sector TO-H b10-3", "SystemAddress":7269097678225, "DistanceFromArrivalLS":0.0, "StarType":"M", "Subclass":2, "StellarMass":0.339844, "Radius":323511040.0, "AbsoluteMagnitude":9.052567, "Age_MY":3588, "SurfaceTemperature":3205.0, "Luminosity":"Va", "SemiMajorAxis":1248822689056.0, "Eccentricity":0.019439, "OrbitalInclination":-87.853424, "Periapsis":289.407812, "OrbitalPeriod":9045423030.853271, "AscendingNode":-60.523416, "MeanAnomaly":318.242694, "RotationPeriod":138253.627944, "AxialTilt":0.0, "WasDiscovered":true, "WasMapped":false }"#;
        let event = JournalEvent::from_line(line).expect("Scan (star) must parse");
        assert!(matches!(event, JournalEvent::Scan(ref e) if e.star_type.as_deref() == Some("M")));
    }

    #[test]
    fn parse_saa_signals_found() {
        let line = r#"{ "timestamp":"2022-06-28T01:06:37Z", "event":"SAASignalsFound", "BodyName":"Hyades Sector OX-K b8-0 5 A Ring", "SystemAddress":672296281473, "BodyID":18, "Signals":[ { "Type":"Alexandrite", "Type_Localised":"Alexandrit", "Count":6 }, { "Type":"tritium", "Count":5 } ] }"#;
        let event = JournalEvent::from_line(line).expect("SAASignalsFound must parse");
        assert!(matches!(event, JournalEvent::SaaSignalsFound(_)));
    }

    #[test]
    fn parse_fss_body_signals() {
        let line = r#"{ "timestamp":"2022-07-13T23:18:47Z", "event":"FSSBodySignals", "BodyName":"Synuefe XO-P c22-17 C 7", "BodyID":19, "SystemAddress":4757716439746, "Signals":[ { "Type":"$SAA_SignalType_Biological;", "Type_Localised":"Biologisch", "Count":6 } ] }"#;
        let event = JournalEvent::from_line(line).expect("FSSBodySignals must parse");
        assert!(matches!(event, JournalEvent::FssBodySignals(_)));
    }

    #[test]
    fn parse_fss_signal_discovered() {
        let line = r#"{ "timestamp":"2022-06-28T00:22:10Z", "event":"FSSSignalDiscovered", "SystemAddress":672296281473, "SignalName":"PERERA*PANCETA QZV-WQB", "IsStation":true }"#;
        let event = JournalEvent::from_line(line).expect("FSSSignalDiscovered must parse");
        assert!(matches!(event, JournalEvent::FssSignalDiscovered(_)));
    }

    #[test]
    fn parse_fss_discovery_scan() {
        let line = r#"{ "timestamp":"2022-06-28T15:17:46Z", "event":"FSSDiscoveryScan", "Progress":0.217941, "BodyCount":18, "NonBodyCount":4, "SystemName":"Hyades Sector TO-H b10-3", "SystemAddress":7269097678225 }"#;
        let event = JournalEvent::from_line(line).expect("FSSDiscoveryScan must parse");
        assert!(matches!(event, JournalEvent::FssDiscoveryScan(_)));
    }

    #[test]
    fn parse_fss_all_bodies_found() {
        let line = r#"{ "timestamp":"2022-06-27T16:44:53Z", "event":"FSSAllBodiesFound", "SystemName":"Col 285 Sector RY-R d4-128", "SystemAddress":4408540662123, "Count":3 }"#;
        let event = JournalEvent::from_line(line).expect("FSSAllBodiesFound must parse");
        assert!(matches!(event, JournalEvent::FssAllBodiesFound(_)));
    }

    #[test]
    fn parse_docked() {
        let line = r#"{ "timestamp":"2022-06-27T15:41:32Z", "event":"Docked", "StationName":"Kumiho Sky", "StationType":"MegaShip", "Taxi":false, "Multicrew":false, "StarSystem":"Di Jian", "SystemAddress":2862335674731, "MarketID":129009496, "StationFaction":{ "Name":"Sirius Corporation" }, "StationGovernment":"$government_Corporate;", "StationGovernment_Localised":"Konzernpolitik", "StationServices":[ "dock", "autodock" ], "StationEconomy":"$economy_HighTech;", "StationEconomy_Localised":"Hightech", "StationEconomies":[ { "Name":"$economy_HighTech;", "Name_Localised":"Hightech", "Proportion":1.0 } ], "DistFromStarLS":4166.909544, "LandingPads":{ "Small":4, "Medium":2, "Large":1 } }"#;
        let event = JournalEvent::from_line(line).expect("Docked must parse");
        assert!(matches!(event, JournalEvent::Docked(ref e) if e.market_id == 129009496));
    }

    #[test]
    fn parse_carrier_jump() {
        let line = r#"{ "timestamp":"2024-04-01T20:21:08Z", "event":"CarrierJump", "Docked":true, "StationName":"Q2K-BHB", "StationType":"FleetCarrier", "MarketID":3704402432, "StationFaction":{ "Name":"FleetCarrier" }, "StationGovernment":"$government_Carrier;", "StationGovernment_Localised":"Private Ownership", "StationServices":[ "dock" ], "StationEconomy":"$economy_Carrier;", "StationEconomy_Localised":"Private Enterprise", "StationEconomies":[ { "Name":"$economy_Carrier;", "Name_Localised":"Private Enterprise", "Proportion":1.0 } ], "Taxi":false, "Multicrew":false, "StarSystem":"Dulos", "SystemAddress":13865362204089, "StarPos":[29.0,-71.34375,45.5], "SystemAllegiance":"Independent", "SystemEconomy":"$economy_Agri;", "SystemSecondEconomy":"$economy_Refinery;", "SystemGovernment":"$government_Dictatorship;", "SystemSecurity":"$SYSTEM_SECURITY_medium;", "Population":564373351, "Body":"Dulos", "BodyID":0, "BodyType":"Star" }"#;
        let event = JournalEvent::from_line(line).expect("CarrierJump must parse");
        assert!(matches!(event, JournalEvent::CarrierJump(_)));
    }

    #[test]
    fn parse_carrier_stats() {
        let line = r#"{ "timestamp":"2022-08-13T12:55:49Z", "event":"CarrierStats", "CarrierID":3704402432, "Callsign":"Q2K-BHB", "Name":"FUXBAU", "DockingAccess":"all", "AllowNotorious":false, "FuelLevel":944, "JumpRangeCurr":500.0, "JumpRangeMax":500.0, "PendingDecommission":false, "SpaceUsage":{ "TotalCapacity":25000, "Crew":0, "Cargo":19763, "CargoSpaceReserved":3669, "ShipPacks":0, "ModulePacks":0, "FreeSpace":1568 }, "Finance":{ "CarrierBalance":71771324, "ReserveBalance":0, "AvailableBalance":59237876, "ReservePercent":0 }, "Crew":[], "ShipPacks":[], "ModulePacks":[] }"#;
        let event = JournalEvent::from_line(line).expect("CarrierStats must parse");
        assert!(matches!(event, JournalEvent::CarrierStats(ref e) if e.callsign == "Q2K-BHB"));
    }

    #[test]
    fn parse_colonisation_construction_depot() {
        let line = r#"{ "timestamp":"2025-04-16T05:30:48Z", "event":"ColonisationConstructionDepot", "MarketID":3955401474, "ConstructionProgress":0.538431, "ConstructionComplete":false, "ConstructionFailed":false, "ResourcesRequired":[ { "Name":"$aluminium_name;", "Name_Localised":"Aluminium", "RequiredAmount":1958, "ProvidedAmount":1958, "Payment":3239 } ] }"#;
        let event = JournalEvent::from_line(line).expect("ColonisationConstructionDepot must parse");
        assert!(matches!(event, JournalEvent::ColonisationConstructionDepot(_)));
    }

    #[test]
    fn parse_nav_beacon_scan() {
        let line = r#"{ "timestamp":"2022-07-03T10:54:43Z", "event":"NavBeaconScan", "SystemAddress":1591025322331, "NumBodies":57 }"#;
        let event = JournalEvent::from_line(line).expect("NavBeaconScan must parse");
        assert!(matches!(event, JournalEvent::NavBeaconScan(_)));
    }

    #[test]
    fn parse_scan_organic() {
        let line = r#"{ "timestamp":"2022-07-07T22:11:38Z", "event":"ScanOrganic", "ScanType":"Log", "Genus":"$Codex_Ent_Brancae_Name;", "Genus_Localised":"Hirnbaeume", "Species":"$Codex_Ent_Seed_Name;", "Species_Localised":"Roseum-Hirnbaum", "SystemAddress":2879909340529, "Body":37 }"#;
        let event = JournalEvent::from_line(line).expect("ScanOrganic must parse");
        assert!(matches!(event, JournalEvent::ScanOrganic(_)));
    }

    #[test]
    fn parse_supercruise_exit() {
        let line = r#"{ "timestamp":"2022-06-27T15:39:34Z", "event":"SupercruiseExit", "Taxi":false, "Multicrew":false, "StarSystem":"Di Jian", "SystemAddress":2862335674731, "Body":"Kumiho Sky", "BodyID":23, "BodyType":"Station" }"#;
        let event = JournalEvent::from_line(line).expect("SupercruiseExit must parse");
        assert!(matches!(event, JournalEvent::SupercruiseExit(_)));
    }

    #[test]
    fn parse_scan_bary_centre() {
        let line = r#"{ "timestamp":"2022-07-03T22:12:04Z", "event":"ScanBaryCentre", "StarSystem":"Hyades Sector AL-W b2-0", "SystemAddress":670685996369, "BodyID":2, "SemiMajorAxis":7546637058258.0, "Eccentricity":0.461492, "OrbitalInclination":-10.04883, "Periapsis":238.057624, "OrbitalPeriod":44170473217.96417, "AscendingNode":-89.395452, "MeanAnomaly":232.273394 }"#;
        let event = JournalEvent::from_line(line).expect("ScanBaryCentre must parse");
        assert!(matches!(event, JournalEvent::ScanBaryCentre(_)));
    }

    #[test]
    fn unknown_event_returns_none() {
        let line = r#"{ "timestamp":"2022-06-27T15:39:34Z", "event":"Music", "MusicTrack":"MainMenu" }"#;
        assert!(JournalEvent::from_line(line).is_none());
    }

    #[test]
    fn invalid_json_returns_none() {
        assert!(JournalEvent::from_line("not json at all").is_none());
        assert!(JournalEvent::from_line("").is_none());
    }
}
