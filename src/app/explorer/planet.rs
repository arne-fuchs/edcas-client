use std::sync::Arc;
use eframe::egui::Ui;
use log::warn;
use num_format::{Locale, ToFormattedString};

use crate::app::explorer::planet::BodyClass::{AmmoniaWorld, ClassIGasGiant, ClassIIGasGiant, ClassIIIGasGiant, ClassIVGasGiant, ClassVGasGiant, EarthlikeWorld, GasGiantwithAmmoniabasedLife, GasGiantwithWaterbasedLife, HeliumRichGasGiant, HighMetalContentPlanet, HighMetalContentTerraformablePlanet, IcyBody, MetalRichBody, Ring, RockyBody, RockyBodyTerraformable, RockyIceBody, Star, Unknown, WaterGiant, WaterWorld, WaterWorldTerraformable};
use crate::app::explorer::structs::{BodyImplementation, Parent, Signal};
use crate::app::settings::Settings;

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

pub struct AsteroidRing {
    pub name: String,
    pub ring_class: String,
    pub mass_mt: f64,
    pub inner_rad: f64,
    pub outer_rad: f64,
}

pub struct Composition {
    pub name: String,
    pub percentage: f64,
}

pub struct AtmosphereComposition {
    pub name: String,
    pub percent: f64,
}

pub struct Planet {
    pub timestamp: String,
    pub event: String,
    pub scan_type: String,
    pub body_name: String,
    pub body_id: i64,
    pub parents: Vec<Parent>,
    pub star_system: String,
    pub system_address: i64,
    pub distance_from_arrival_ls: f64,
    pub tidal_lock: bool,
    pub terraform_state: String,
    pub planet_class: String,
    pub atmosphere: String,
    pub atmosphere_type: String,
    pub atmosphere_composition: Vec<AtmosphereComposition>,
    pub volcanism: String,
    pub mass_em: f64,
    pub radius: f64,
    pub surface_gravity: f64,
    pub surface_temperature: f64,
    pub surface_pressure: f64,
    pub landable: bool,
    pub materials: Vec<Composition>,
    pub composition: Vec<Composition>,
    pub semi_major_axis: f64,
    pub eccentricity: f64,
    pub orbital_inclination: f64,
    pub periapsis: f64,
    pub orbital_period: f64,
    pub ascending_node: f64,
    pub mean_anomaly: f64,
    pub rotation_period: f64,
    pub axial_tilt: f64,
    pub was_discovered: bool,
    pub was_mapped: bool,
    pub reserve_level: String,
    pub asteroid_rings: Vec<AsteroidRing>,
    pub planet_signals: Vec<Signal>,
    pub settings: Arc<Settings>,
}

impl BodyImplementation for Planet {
    fn print_side_panel_information(&self, ui: &mut Ui) {
        let profit = get_profit_from_body(get_body_class_from_body(self), self.was_discovered);
        ui.heading(&self.body_name);
        ui.end_row();
        ui.label("Class");
        ui.label(&self.planet_class);
        ui.end_row();
        ui.label("Terraform State");
        ui.label(&self.terraform_state);
        ui.end_row();
        ui.label("Profit");
        ui.end_row();
        ui.label("Discovery");
        ui.label(profit.0.to_formatted_string(&Locale::en));
        ui.end_row();
        ui.label("Mapping");
        ui.label(profit.1.to_formatted_string(&Locale::en));
        ui.end_row();
        ui.label("");
        ui.label("");
        ui.end_row();

        ui.label("Discovered");
        ui.label(&self.was_discovered.to_string());
        ui.end_row();
        ui.label("Mapped");
        ui.label(&self.was_mapped.to_string());
        ui.end_row();
        ui.label("Distance in LS");
        ui.label(&self.distance_from_arrival_ls.to_string());
        ui.end_row();
        ui.label("Landable");
        ui.label(&self.landable.to_string());
        ui.end_row();
        ui.label("");
        ui.label("");
        ui.end_row();
        ui.label("Gravity");
        ui.label(&self.surface_gravity.to_string());
        ui.end_row();
        ui.label("Radius");
        ui.label(&self.radius.to_string());
        ui.end_row();
        ui.label("Temperature K");
        ui.label(&self.surface_temperature.to_string());
        ui.end_row();
        ui.label("Atmosphere");
        ui.label(&self.atmosphere);
        ui.end_row();
        for atmosphere_composition in &self.atmosphere_composition{
            ui.label(&atmosphere_composition.name);
            ui.label(format!("{}%",&atmosphere_composition.percent));
            ui.end_row();
        }
        ui.label("Reserve level");
        ui.label(&self.reserve_level);
        ui.end_row();
        ui.heading("Material");
        ui.end_row();
        for material in &self.materials{
            ui.label(&material.name);
            ui.label(format!("{}%",&material.percentage.to_string()));
            ui.end_row();
        }
        ui.heading("Composition");
        ui.end_row();
        for composition in &self.composition{
            ui.label(&composition.name);
            ui.label(format!("{}%",&composition.percentage.to_string()));
            ui.end_row();
        }
        ui.heading("Rings");
        ui.end_row();
        for ring in &self.asteroid_rings {
            ui.label(&ring.ring_class);
            ui.vertical(|ui|{
                ui.label(&ring.outer_rad.to_string());
                ui.label(&ring.inner_rad.to_string());
            });
            ui.end_row();
        }
    }

    fn print_header_content(&self, ui: &mut Ui, system_index: &mut usize, body_index: usize) {
        if self.settings.planets.get(self.planet_class.as_str()).unwrap().enabled{
            ui.label(self.settings.planets.get(self.planet_class.as_str()).unwrap().get_richtext().size(self.radius.log(1.5) as f32));
        }

        let mut body_name = self.body_name.to_string();
        if !self.settings.explorer_settings.include_system_name{
            body_name.replace_range(0..self.star_system.len(),"");
        }
        if ui.selectable_label(false, &body_name).clicked() {
            *system_index = body_index;
        };

        if !self.asteroid_rings.is_empty() && self.settings.icons.get("ring").unwrap().enabled{
            ui.label("|");
            ui.label(self.settings.icons.get("ring").unwrap().get_richtext());
        }

        if self.landable && self.settings.icons.get("landable").unwrap().enabled{
            ui.label("|");
            ui.label(self.settings.icons.get("landable").unwrap().get_richtext());
        }

        if !self.planet_signals.is_empty(){
            for signal in &self.planet_signals{

                //⛓ -> xeno
                //🌱 🌿 🌴-> bio
                //🌋 🗻 -> xeno
                //✋ -> human
                //？ -> unknown
                //⛅☁ -> Atmosphere
                match signal.r#type.as_str() {
                    "$SAA_SignalType_Biological;" => {
                        if self.settings.icons.get("bio_signal").unwrap().enabled {
                            ui.label("|");
                            ui.label(&signal.count.to_string());
                            ui.label(self.settings.icons.get("bio_signal").unwrap().get_richtext());
                        }
                    }
                    "$SAA_SignalType_Geological;" => {
                        if self.settings.icons.get("geo_signal").unwrap().enabled {
                            ui.label("|");
                            ui.label(&signal.count.to_string());
                            ui.label(self.settings.icons.get("geo_signal").unwrap().get_richtext());
                        }
                    }
                    "$SAA_SignalType_Xenological;" => {
                        if self.settings.icons.get("xeno_signal").unwrap().enabled {
                            ui.label("|");
                            ui.label(&signal.count.to_string());
                            ui.label(self.settings.icons.get("xeno_signal").unwrap().get_richtext());
                        }
                    }
                    "$SAA_SignalType_Human;" => {
                        if self.settings.icons.get("human_signal").unwrap().enabled {
                            ui.label("|");
                            ui.label(&signal.count.to_string());
                            ui.label(self.settings.icons.get("human_signal").unwrap().get_richtext());
                        }
                    }
                    _ => {
                        if self.settings.icons.get("unknown_signal").unwrap().enabled {
                            ui.label("|");
                            ui.label(&signal.count.to_string());
                            ui.label(self.settings.icons.get("unknown_signal").unwrap().get_richtext());
                        }
                    }
                }
            }
        }
        if self.settings.icons.get("gravity").unwrap().enabled {
            ui.label("|");
            ui.label(self.surface_gravity.to_string());
            ui.label("G");
            ui.label(self.settings.icons.get("gravity").unwrap().get_richtext());
        }
        if self.settings.icons.get("distance").unwrap().enabled{
            ui.label("|");
            ui.label((self.distance_from_arrival_ls as u64).to_formatted_string(&Locale::en));
            ui.label("LS");
            ui.label(self.settings.icons.get("distance").unwrap().get_richtext());
        }
        if self.was_discovered && self.settings.icons.get("discovered").unwrap().enabled{
            ui.label("|");
            ui.label(self.settings.icons.get("discovered").unwrap().get_richtext());
        }
        if self.was_discovered && self.settings.icons.get("mapped").unwrap().enabled{
            ui.label("|");
            ui.label(self.settings.icons.get("mapped").unwrap().get_richtext());
        }


    }

    fn get_name(&self) -> &str {
        self.body_name.as_str()
    }

    fn get_id(&self) -> i64 {
        self.body_id
    }

    fn get_signals(&self) -> Vec<Signal> {
        self.planet_signals.clone()
    }

    fn set_signals(&mut self, signals: Vec<Signal>) {
        self.planet_signals = signals;
    }

    fn get_parents(&self) -> Vec<Parent> {
        self.parents.clone()
    }
}

pub enum BodyClass {
    AmmoniaWorld,
    EarthlikeWorld,
    WaterWorld,
    WaterWorldTerraformable,
    HighMetalContentPlanet,
    HighMetalContentTerraformablePlanet,
    IcyBody,
    MetalRichBody,
    RockyBody,
    RockyBodyTerraformable,
    RockyIceBody,
    ClassIGasGiant,
    ClassIIGasGiant,
    ClassIIIGasGiant,
    ClassIVGasGiant,
    ClassVGasGiant,
    GasGiantwithAmmoniabasedLife,
    GasGiantwithWaterbasedLife,
    HeliumRichGasGiant,
    WaterGiant,
    Ring,
    Star,
    Unknown,
}

pub fn get_body_class_from_body(planet: &Planet) -> BodyClass {
    if planet.body_name.contains(" Ring"){
        return Ring
    }

    match planet.planet_class.as_str() {
        "Ammonia world" => AmmoniaWorld,
        "Earthlike body" => EarthlikeWorld,
        "Water world" => {
            if planet.terraform_state == "Terraformable"{
                return WaterWorldTerraformable;
            }
            WaterWorld
        },
        "High metal content body" => {
            if planet.terraform_state == "Terraformable"{
                return HighMetalContentTerraformablePlanet;
            }
            HighMetalContentPlanet
        },
        "Icy body" => IcyBody,
        "Metal rich body" => MetalRichBody,
        "Rocky body" => {
            if planet.terraform_state == "Terraformable"{
                return RockyBodyTerraformable;
            }
            RockyBody
        },
        "Rocky ice body" => RockyIceBody,
        "Sudarsky class I gas giant" => ClassIGasGiant,
        "Sudarsky class II gas giant" => ClassIIGasGiant,
        "Sudarsky class III gas giant" => ClassIIIGasGiant,
        "Sudarsky class IV gas giant" => ClassIVGasGiant,
        "Sudarsky class V gas giant" => ClassVGasGiant,
        "Gas giant with ammonia based life" => GasGiantwithAmmoniabasedLife,
        "Gas giant with water based life" => GasGiantwithWaterbasedLife,
        "Helium rich gas giant" => HeliumRichGasGiant,
        "Water giant" => WaterGiant,
        "Star" => Star,
        _ => {
            if planet.planet_class.is_empty() || planet.planet_class.eq("N/A") || planet.planet_class.eq("null"){
                return Star;
            }
            //FIXME If stars come as child, their "Planet Class" cannot be determined
            //[src/app/journal_reader.rs:75] &line = "{ \"timestamp\":\"2022-10-31T00:20:41Z\", \"event\":\"Scan\", \"ScanType\":\"AutoScan\", \"BodyName\":\"Kyloall UO-A e147 A 5\", \"BodyID\":28, \"Parents\":[ {\"Star\":1}, {\"Null\":0} ], \"StarSystem\":\"Kyloall UO-A e147\", \"SystemAddress\":632435992772, \"DistanceFromArrivalLS\":3039.256581, \"StarType\":\"Y\", \"Subclass\":0, \"StellarMass\":0.031250, \"Radius\":85388072.000000, \"AbsoluteMagnitude\":18.674301, \"Age_MY\":308, \"SurfaceTemperature\":646.000000, \"Luminosity\":\"V\", \"SemiMajorAxis\":912356770038.604736, \"Eccentricity\":0.001826, \"OrbitalInclination\":0.081750, \"Periapsis\":334.272320, \"OrbitalPeriod\":236886096.000671, \"AscendingNode\":-58.389303, \"MeanAnomaly\":43.307253, \"RotationPeriod\":322169.147143, \"AxialTilt\":-1.353121, \"WasDiscovered\":false, \"WasMapped\":false }\r\n"
            warn!("Couldn't find planet class: {}", &planet.planet_class.as_str());
            Unknown
        }
    }
}

/**
Returns tupel of profit
0 -> est. Earn for Discorvery
1 -> est. Earn for Discovery + Mapping
!!!Earnings are estimated. Formular for profit is not known in the moment!!!
**/
//TODO Put this in a configurable file
pub fn get_profit_from_body(class: BodyClass, discovered: bool) -> (i32, i32){
    match class {
        AmmoniaWorld => { if discovered { (143463, 1724965) } else { (373004, 597762) } }
        EarthlikeWorld => { if discovered { (270290, 1126206) } else { (702753, 3249900) } }
        WaterWorld => { if discovered { (99747, 415613) } else { (259343, 1199337) } }
        WaterWorldTerraformable => { if discovered { (268616, 1119231) } else { (698400, 3229773) } }
        HighMetalContentPlanet => { if discovered { (14070, 58624) } else { (36581, 169171) } }
        HighMetalContentTerraformablePlanet => { if discovered { (163948, 683116) } else { (426264, 1971272) } }
        IcyBody => { if discovered { (500, 1569) } else { (1300, 4527) } }
        MetalRichBody => { if discovered { (31632, 131802) } else { (82244, 380341) } }
        RockyBody => { if discovered { (500, 1476) } else { (1300, 4260) } }
        RockyBodyTerraformable => { if discovered { (129504, 539601) } else { (336711, 1557130) } }
        RockyIceBody => { if discovered { (500, 1752) } else { (1300, 5057) } }
        ClassIGasGiant => { if discovered { (3845, 16021) } else { (9997, 46233) } }
        ClassIIGasGiant => { if discovered { (28405, 118354) } else { (73853, 341536) } }
        ClassIIIGasGiant => { if discovered { (995, 4145) } else { (2587, 11963) } }
        ClassIVGasGiant => { if discovered { (1119, 4663) } else { (2910, 13457) } }
        ClassVGasGiant => { if discovered { (966, 4023) } else { (2510, 11609) } }
        GasGiantwithAmmoniabasedLife => { if discovered { (774, 3227) } else { (2014, 9312) } }
        GasGiantwithWaterbasedLife => { if discovered { (883, 3679) } else { (2295, 10616) } }
        HeliumRichGasGiant => { if discovered { (900, 3749) } else { (2339, 10818) } }
        WaterGiant => { if discovered { (667, 2779) } else { (1734, 8019) } }
        Ring => { if discovered { (0, 0) } else { (0, 0) } }
        Star => { if discovered { (0, 0) } else { (0, 0) } }
        Unknown => { if discovered { (0, 0) } else { (0, 0) } }
    }
}
