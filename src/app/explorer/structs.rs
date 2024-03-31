use std::sync::Arc;

use eframe::egui;
use json::JsonValue;
use log::debug;

use crate::app::explorer::belt_cluster::BeltCluster;
use crate::app::explorer::planet::{AsteroidRing, Planet};
use crate::app::explorer::ring::Ring;
use crate::app::explorer::star::Star;
use crate::app::settings::Settings;

pub fn generate_from_json(json: JsonValue, settings: Arc<Settings>) -> Box<dyn BodyImplementation> {
    //TODO Parents implementation
    //TODO Atmosphere Composition implementation
    //TODO Materials?
    //TODO Rings
    debug!("Generate from json: {}", json);

    //"Parents":[{"Planet":5},{"Star":0}]
    let mut parents: Vec<Parent> = vec![];
    for i in 0..json["Parents"].len() {
        let parent = &json["Parents"][i];
        for entry in parent.entries() {
            parents.push(Parent {
                name: entry.0.to_string(),
                id: entry.1.as_i64().unwrap(),
            })
        }
    }

    let mut asteroid_rings = vec![];
    if !json["Rings"].is_null() {
        for i in 0..json["Rings"].len() {
            let asteroid_ring = &json["Rings"][i];
            asteroid_rings.push(AsteroidRing {
                name: asteroid_ring["Name"].to_string(),
                ring_class: asteroid_ring["RingClass"].to_string(),
                mass_mt: asteroid_ring["MassMT"].as_f64().unwrap(),
                inner_rad: asteroid_ring["InnerRad"].as_f64().unwrap(),
                outer_rad: asteroid_ring["OuterRad"].as_f64().unwrap(),
            });
        }
    }

    if json["StarType"].is_null() {
        if json["BodyName"].to_string().contains("Belt Cluster") {
            Box::new(BeltCluster {
                timestamp: json["Timestamp"].to_string(),
                event: json["event"].to_string(),
                scan_type: json["ScanType"].to_string(),
                body_name: json["BodyName"].to_string(),
                body_id: json["BodyID"].as_i64().unwrap(),
                parents,
                star_system: json["StarSystem"].to_string(),
                system_address: json["SystemAddress"].as_i64().unwrap(),
                distance_from_arrival_ls: json["DistanceFromArrivalLS"].as_f64().unwrap(),
                was_discovered: json["WasDiscovered"].to_string().parse().unwrap(),
                was_mapped: json["WasMapped"].to_string().parse().unwrap(),
                settings: settings.clone(),
            })
        } else if json["BodyName"].to_string().contains("Ring") {
            //{"timestamp":"2023-07-17T18:18:37Z","event":"Scan","ScanType":"AutoScan","BodyName":"Scheau Blao BB-W b57-45 1 B Ring","BodyID":7,"Parents":[{"Planet":5},{"Star":0}],"StarSystem":"Scheau Blao BB-W b57-45","SystemAddress":99684979268081,"DistanceFromArrivalLS":535.932803,"SemiMajorAxis":208998668.193817,"Eccentricity":0,"OrbitalInclination":0,"Periapsis":0,"OrbitalPeriod":44426.782131,"AscendingNode":0,"MeanAnomaly":190.333503,"WasDiscovered":false,"WasMapped":false}
            Box::new(Ring {
                timestamp: json["Timestamp"].to_string(),
                event: json["event"].to_string(),
                scan_type: json["ScanType"].to_string(),
                body_name: json["BodyName"].to_string(),
                body_id: json["BodyID"].as_i64().unwrap(),
                parents,
                star_system: json["StarSystem"].to_string(),
                system_address: json["SystemAddress"].as_i64().unwrap(),
                distance_from_arrival_ls: json["DistanceFromArrivalLS"].as_f64().unwrap(),
                semi_major_axis: json["SemiMajorAxis"].as_f64().unwrap(),
                eccentricity: json["Eccentricity"].as_f64().unwrap(),
                orbital_inclination: json["OrbitalInclination"].as_f64().unwrap(),
                periapsis: json["Periapsis"].as_f64().unwrap(),
                orbital_period: json["OrbitalPeriod"].as_f64().unwrap(),
                ascending_node: json["AscendingNode"].as_f64().unwrap(),
                mean_anomaly: json["MeanAnomaly"].as_f64().unwrap(),
                was_discovered: json["WasDiscovered"].to_string().parse().unwrap(),
                was_mapped: json["WasMapped"].to_string().parse().unwrap(),
                ring_signals: vec![],
                settings: settings.clone(),
            })
        } else {
            //{ "timestamp":"2022-10-16T23:51:17Z", "event":"Scan", "ScanType":"Detailed", "BodyName":"Ogmar A 6", "BodyID":40, "Parents":[ {"Star":1}, {"Null":0} ], "StarSystem":"Ogmar", "SystemAddress":84180519395914, "DistanceFromArrivalLS":3376.246435, "TidalLock":false, "TerraformState":"", "PlanetClass":"Sudarsky class I gas giant", "Atmosphere":"", "AtmosphereComposition":[ { "Name":"Hydrogen", "Percent":73.044167 }, { "Name":"Helium", "Percent":26.955832 } ], "Volcanism":"", "MassEM":24.477320, "Radius":22773508.000000, "SurfaceGravity":18.811067, "SurfaceTemperature":62.810730, "SurfacePressure":0.000000, "Landable":false, "SemiMajorAxis":1304152250289.916992, "Eccentricity":0.252734, "OrbitalInclination":156.334694, "Periapsis":269.403039, "OrbitalPeriod":990257555.246353, "AscendingNode":-1.479320, "MeanAnomaly":339.074691, "RotationPeriod":37417.276422, "AxialTilt":0.018931, "WasDiscovered":true, "WasMapped":true }

            //{ "timestamp":"2023-07-19T17:19:51Z", "event":"Scan", "ScanType":"Detailed", "BodyName":"Phaa Chroa YL-B b5-4 A 9",
            // "BodyID":36, "Parents":[ {"Star":1}, {"Null":0} ], "StarSystem":"Phaa Chroa YL-B b5-4", "SystemAddress":9544091982377,
            // "DistanceFromArrivalLS":1916.882666, "TidalLock":false, "TerraformState":"", "PlanetClass":"Icy body",
            // "Atmosphere":"helium atmosphere", "AtmosphereType":"Helium",
            // "AtmosphereComposition":[ { "Name":"Helium", "Percent":89.334976 }, { "Name":"Hydrogen", "Percent":8.427828 }, { "Name":"Neon", "Percent":2.237205 } ],
            // "Volcanism":"major water geysers volcanism", "MassEM":4.940075, "Radius":13127075.000000, "SurfaceGravity":11.426337,
            // "SurfaceTemperature":53.316639, "SurfacePressure":68322.453125, "Landable":false,
            // "Composition":{ "Ice":0.657375, "Rock":0.202757, "Metal":0.099446 }, "SemiMajorAxis":575420260429.382324,
            // "Eccentricity":0.006722, "OrbitalInclination":0.013744, "Periapsis":170.971110, "OrbitalPeriod":368017596.006393,
            // "AscendingNode":-4.262981, "MeanAnomaly":78.392818, "RotationPeriod":101857.106605, "AxialTilt":-0.011263,
            // "Rings":[ { "Name":"Phaa Chroa YL-B b5-4 A 9 A Ring", "RingClass":"eRingClass_Rocky", "MassMT":2.501e+09, "InnerRad":2.166e+07, "OuterRad":2.3866e+07 }, { "Name":"Phaa Chroa YL-B b5-4 A 9 B Ring", "RingClass":"eRingClass_Icy", "MassMT":9.0988e+10, "InnerRad":2.3966e+07, "OuterRad":6.2742e+07 } ],
            // "ReserveLevel":"PristineResources", "WasDiscovered":false, "WasMapped":false }
            Box::new(Planet {
                timestamp: json["Timestamp"].to_string(),
                event: json["event"].to_string(),
                scan_type: json["ScanType"].to_string(),
                body_name: json["BodyName"].to_string(),
                body_id: json["BodyID"].as_i64().unwrap(),
                parents,
                star_system: json["StarSystem"].to_string(),
                system_address: json["SystemAddress"].as_i64().unwrap(),
                distance_from_arrival_ls: json["DistanceFromArrivalLS"].as_f64().unwrap(),
                tidal_lock: json["TidalLock"].to_string().parse().unwrap(),
                terraform_state: json["TerraformState"].to_string(),
                planet_class: json["PlanetClass"].to_string(),
                atmosphere: json["Atmosphere"].to_string(),
                atmosphere_type: "".to_string(),
                atmosphere_composition: vec![],
                volcanism: json["Volcanism"].to_string(),
                mass_em: json["MassEM"].as_f64().unwrap(),
                radius: json["Radius"].as_f64().unwrap(),
                surface_gravity: json["SurfaceGravity"].as_f64().unwrap(),
                surface_temperature: json["SurfaceTemperature"].as_f64().unwrap(),
                surface_pressure: json["SurfacePressure"].as_f64().unwrap(),
                landable: json["Landable"].to_string().parse().unwrap(),
                materials: vec![],
                composition: vec![],
                semi_major_axis: json["SemiMajorAxis"].as_f64().unwrap(),
                eccentricity: json["Eccentricity"].as_f64().unwrap(),
                orbital_inclination: json["OrbitalInclination"].as_f64().unwrap(),
                periapsis: json["Periapsis"].as_f64().unwrap(),
                orbital_period: json["OrbitalPeriod"].as_f64().unwrap(),
                ascending_node: json["AscendingNode"].as_f64().unwrap(),
                mean_anomaly: json["MeanAnomaly"].as_f64().unwrap(),
                rotation_period: json["RotationPeriod"].as_f64().unwrap(),
                axial_tilt: json["AxialTilt"].as_f64().unwrap(),
                was_discovered: json["WasDiscovered"].to_string().parse().unwrap(),
                was_mapped: json["WasMapped"].to_string().parse().unwrap(),
                reserve_level: json["PristineResources"].to_string(),
                asteroid_rings,
                planet_signals: vec![],
                settings: settings.clone(),
            })
        }
    } else {
        //{ "timestamp":"2023-07-12T21:52:23Z", "event":"Scan", "ScanType":"AutoScan", "BodyName":"Lasao DX-Z b43-37 A", "BodyID":1, "Parents":[ {"Null":0} ], "StarSystem":"Lasao DX-Z b43-37", "SystemAddress":82108367853945, "DistanceFromArrivalLS":0.000000, "StarType":"M", "Subclass":7, "StellarMass":0.285156, "Radius":307783360.000000, "AbsoluteMagnitude":10.356186, "Age_MY":3076, "SurfaceTemperature":2434.000000, "Luminosity":"Va", "SemiMajorAxis":514860939979.553223, "Eccentricity":0.153621, "OrbitalInclination":2.176175, "Periapsis":6.939240, "OrbitalPeriod":4139431655.406952, "AscendingNode":-133.798577, "MeanAnomaly":169.548183, "RotationPeriod":118438.397553, "AxialTilt":0.000000, "Rings":[ { "Name":"Lasao DX-Z b43-37 A A Belt", "RingClass":"eRingClass_Rocky", "MassMT":7.2313e+13, "InnerRad":5.0784e+08, "OuterRad":1.6453e+09 } ], "WasDiscovered":false, "WasMapped":false }
        Box::new(Star {
            timestamp: json["Timestamp"].to_string(),
            event: json["event"].to_string(),
            scan_type: json["ScanType"].to_string(),
            body_name: json["BodyName"].to_string(),
            body_id: json["BodyID"].as_i64().unwrap(),
            parents,
            star_system: json["StarSystem"].to_string(),
            system_address: json["SystemAddress"].as_i64().unwrap(),
            distance_from_arrival_ls: json["DistanceFromArrivalLS"].as_f64().unwrap(),
            star_type: json["StarType"].to_string(),
            subclass: json["Subclass"].as_i64().unwrap(),
            stellar_mass: json["StellarMass"].as_f64().unwrap(),
            radius: json["Radius"].as_f64().unwrap(),
            absolute_magnitude: json["AbsoluteMagnitude"].as_f64().unwrap(),
            age_my: json["Age_MY"].as_i64().unwrap(),
            surface_temperature: json["SurfaceTemperature"].as_f64().unwrap(),
            luminosity: json["Luminosity"].to_string(),
            semi_major_axis: json["SemiMajorAxis"].as_f64(),
            eccentricity: json["Eccentricity"].as_f64(),
            orbital_inclination: json["OrbitalInclination"].as_f64(),
            periapsis: json["Periapsis"].as_f64(),
            orbital_period: json["OrbitalPeriod"].as_f64(),
            ascending_node: json["AscendingNode"].as_f64(),
            mean_anomaly: json["MeanAnomaly"].as_f64(),
            rotation_period: json["RotationPeriod"].as_f64().unwrap(),
            axial_tilt: json["AxialTilt"].as_f64().unwrap(),
            was_discovered: json["WasDiscovered"].as_bool().unwrap(),
            was_mapped: json["WasMapped"].as_bool().unwrap(),
            asteroid_rings,
            settings: settings.clone(),
        })
    }
}

pub trait BodyImplementation {
    fn print_side_panel_information(&self, ui: &mut egui::Ui);
    fn print_header_content(&self, ui: &mut egui::Ui, system_index: &mut usize, body_index: usize);
    fn get_name(&self) -> &str;
    fn get_id(&self) -> i64;
    fn get_signals(&self) -> Vec<Signal> {
        vec![]
    }
    fn set_signals(&mut self, _signals: Vec<Signal>) {}
    fn get_parents(&self) -> Vec<Parent>;
    fn get_body(&self) -> BodyType;
}
pub(crate) enum BodyType<'a> {
    Star(&'a Star),
    Planet(&'a Planet),
    Ring(&'a Ring),
    BeltCluster(&'a BeltCluster),
}

impl PartialEq for dyn BodyImplementation {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

#[derive(Clone)]
pub struct Parent {
    pub name: String,
    pub id: i64,
}

#[derive(Clone)]
pub struct Signal {
    pub r#type: String,
    pub type_localised: String,
    pub count: i64,
}

impl Default for Signal {
    fn default() -> Self {
        Self {
            r#type: "N/A".to_string(),
            type_localised: "N/A".to_string(),
            count: 0,
        }
    }
}
