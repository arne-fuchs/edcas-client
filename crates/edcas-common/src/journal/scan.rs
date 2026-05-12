use serde::{Deserialize, Serialize};

use crate::journal::types::{AtmosphereComposition, Composition, Material, Parent, Ring};

/// Covers both star scans (StarType present) and body/planet scans.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Scan {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "ScanType")]
    pub scan_type: Option<String>,
    #[serde(rename = "BodyName")]
    pub body_name: String,
    #[serde(rename = "BodyID")]
    pub body_id: i32,
    #[serde(rename = "SystemAddress")]
    pub system_address: i64,
    #[serde(rename = "StarPos")]
    pub star_pos: Option<Vec<f32>>,
    #[serde(rename = "StarSystem")]
    pub star_system: Option<String>,
    // Star-specific (absent for bodies)
    #[serde(rename = "StarType")]
    pub star_type: Option<String>,
    #[serde(rename = "Subclass")]
    pub subclass: Option<i32>,
    #[serde(rename = "StellarMass")]
    pub stellar_mass: Option<f32>,
    #[serde(rename = "AbsoluteMagnitude")]
    pub absolute_magnitude: Option<f32>,
    #[serde(rename = "Age_MY")]
    pub age_my: Option<i32>,
    #[serde(rename = "Luminosity")]
    pub luminosity: Option<String>,
    // Body-specific (absent for stars)
    #[serde(rename = "PlanetClass")]
    pub planet_class: Option<String>,
    #[serde(rename = "MassEM")]
    pub mass_em: Option<f32>,
    #[serde(rename = "SurfacePressure")]
    pub surface_pressure: Option<f32>,
    #[serde(rename = "Landable", default)]
    pub landable: bool,
    #[serde(rename = "TidalLock", default)]
    pub tidal_lock: bool,
    #[serde(rename = "TerraformState")]
    pub terraform_state: Option<String>,
    #[serde(rename = "Atmosphere")]
    pub atmosphere: Option<String>,
    #[serde(rename = "AtmosphereType")]
    pub atmosphere_type: Option<String>,
    #[serde(rename = "AtmosphereComposition")]
    pub atmosphere_composition: Option<Vec<AtmosphereComposition>>,
    #[serde(rename = "Volcanism")]
    pub volcanism: Option<String>,
    #[serde(rename = "Composition")]
    pub composition: Option<Composition>,
    #[serde(rename = "Materials")]
    pub materials: Option<Vec<Material>>,
    // Orbital mechanics (shared, absent for main stars)
    #[serde(rename = "Radius")]
    pub radius: Option<f32>,
    #[serde(rename = "DistanceFromArrivalLS")]
    pub distance_from_arrival_ls: Option<f32>,
    #[serde(rename = "RotationPeriod")]
    pub rotation_period: Option<f32>,
    #[serde(rename = "OrbitalPeriod")]
    pub orbital_period: Option<f32>,
    #[serde(rename = "Eccentricity")]
    pub eccentricity: Option<f32>,
    #[serde(rename = "OrbitalInclination")]
    pub orbital_inclination: Option<f32>,
    #[serde(rename = "AxialTilt")]
    pub axial_tilt: Option<f32>,
    #[serde(rename = "AscendingNode")]
    pub ascending_node: Option<f32>,
    #[serde(rename = "MeanAnomaly")]
    pub mean_anomaly: Option<f32>,
    #[serde(rename = "Periapsis")]
    pub periapsis: Option<f32>,
    #[serde(rename = "SemiMajorAxis")]
    pub semi_major_axis: Option<f32>,
    #[serde(rename = "SurfaceTemperature")]
    pub surface_temperature: Option<f32>,
    #[serde(rename = "SurfaceGravity")]
    pub surface_gravity: Option<f32>,
    #[serde(rename = "Rings")]
    pub rings: Option<Vec<Ring>>,
    #[serde(rename = "ReserveLevel")]
    pub reserve_level: Option<String>,
    #[serde(rename = "Parents")]
    pub parents: Option<Vec<Parent>>,
    #[serde(rename = "WasDiscovered", default)]
    pub was_discovered: bool,
    #[serde(rename = "WasMapped", default)]
    pub was_mapped: bool,
    #[serde(rename = "EstimatedValue")]
    pub estimated_value: Option<i64>,
    #[serde(rename = "horizons", default)]
    pub horizons: bool,
    #[serde(rename = "odyssey", default)]
    pub odyssey: bool,
}

impl Scan {
    pub fn is_star(&self) -> bool {
        self.star_type.is_some()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScanBaryCentre {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "StarSystem")]
    pub star_system: String,
    #[serde(rename = "SystemAddress")]
    pub system_address: i64,
    #[serde(rename = "BodyID")]
    pub body_id: i32,
    #[serde(rename = "DistanceFromArrivalLS", default)]
    pub distance_from_arrival_ls: f32,
    #[serde(rename = "Parents")]
    pub parents: Option<Vec<Parent>>,
    #[serde(rename = "SemiMajorAxis")]
    pub semi_major_axis: Option<f32>,
    #[serde(rename = "Eccentricity")]
    pub eccentricity: Option<f32>,
    #[serde(rename = "OrbitalInclination")]
    pub orbital_inclination: Option<f32>,
    #[serde(rename = "Periapsis")]
    pub periapsis: Option<f32>,
    #[serde(rename = "OrbitalPeriod")]
    pub orbital_period: Option<f32>,
    #[serde(rename = "AscendingNode")]
    pub ascending_node: Option<f32>,
    #[serde(rename = "MeanAnomaly")]
    pub mean_anomaly: Option<f32>,
    #[serde(rename = "horizons", default)]
    pub horizons: bool,
    #[serde(rename = "odyssey", default)]
    pub odyssey: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SaaSignalsFound {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "SystemAddress")]
    pub system_address: i64,
    #[serde(rename = "BodyName")]
    pub body_name: String,
    #[serde(rename = "BodyID")]
    pub body_id: i32,
    #[serde(rename = "Signals")]
    pub signals: Vec<SaaSignal>,
    #[serde(rename = "Genuses")]
    pub genuses: Option<Vec<SaaGenus>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SaaSignal {
    #[serde(rename = "Type")]
    pub signal_type: String,
    #[serde(rename = "Type_Localised")]
    pub signal_type_localised: Option<String>,
    #[serde(rename = "Count")]
    pub count: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SaaGenus {
    #[serde(rename = "Genus")]
    pub genus: String,
    #[serde(rename = "Genus_Localised")]
    pub genus_localised: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FssSignalDiscovered {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "SystemAddress")]
    pub system_address: i64,
    #[serde(rename = "SignalName")]
    pub signal_name: String,
    #[serde(rename = "SignalName_Localised")]
    pub signal_name_localised: Option<String>,
    #[serde(rename = "USSType")]
    pub uss_type: Option<String>,
    #[serde(rename = "USSType_Localised")]
    pub uss_type_localised: Option<String>,
    #[serde(rename = "SpawningState")]
    pub spawning_state: Option<String>,
    #[serde(rename = "SpawningState_Localised")]
    pub spawning_state_localised: Option<String>,
    #[serde(rename = "SpawningFaction")]
    pub spawning_faction: Option<String>,
    #[serde(rename = "ThreatLevel")]
    pub threat_level: Option<i32>,
    #[serde(rename = "TimeRemaining")]
    pub time_remaining: Option<f32>,
    #[serde(rename = "IsStation", default)]
    pub is_station: bool,
    #[serde(rename = "BodyID")]
    pub body_id: Option<i32>,
    #[serde(rename = "BodyName")]
    pub body_name: Option<String>,
    #[serde(rename = "horizons", default)]
    pub horizons: bool,
    #[serde(rename = "odyssey", default)]
    pub odyssey: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FssBodySignals {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "SystemAddress")]
    pub system_address: i64,
    #[serde(rename = "BodyName")]
    pub body_name: String,
    #[serde(rename = "BodyID")]
    pub body_id: i32,
    #[serde(rename = "Signals")]
    pub signals: Vec<SaaSignal>,
    #[serde(rename = "horizons", default)]
    pub horizons: bool,
    #[serde(rename = "odyssey", default)]
    pub odyssey: bool,
}
