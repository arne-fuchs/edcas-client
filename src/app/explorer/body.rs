use eframe::egui;
use eframe::egui::ColorImage;
use log::debug;
use num_format::{Locale, ToFormattedString};

use crate::app::explorer::{Body, Explorer, icons};
use crate::app::explorer::body::BodyClass::{AmmoniaWorld, BeltCluster, ClassIGasGiant, ClassIIGasGiant, ClassIIIGasGiant, ClassIVGasGiant, ClassVGasGiant, EarthlikeWorld, GasGiantwithAmmoniabasedLife, GasGiantwithWaterbasedLife, HeliumRichGasGiant, HighMetalContentPlanet, HighMetalContentTerraformablePlanet, IcyBody, MetalRichBody, Ring, RockyBody, RockyBodyTerraformable, RockyIceBody, Star, Unknown, WaterGiant, WaterWorld, WaterWorldTerraformable};

struct Composition {
    pub ice: f64,
    pub rock: f64,
    pub metal: f64,
}

struct Materials {
    pub name: String,
    pub name_localised: Option<String>,
    pub percent: f64,
}

struct Rings {
    pub name: String,
    pub ring_class: String,
    pub mass_mt: f64,
    pub inner_rad: f64,
    pub outer_rad: f64,
}

struct Parent {
    pub star: Option<i64>,
    pub null: Option<i64>,
}

struct Planet {
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
    pub volcanism: String,
    pub mass_em: f64,
    pub radius: f64,
    pub surface_gravity: f64,
    pub surface_temperature: f64,
    pub surface_pressure: f64,
    pub landable: bool,
    pub materials: Vec<Materials>,
    pub composition: Composition,
    pub semi_major_axis: f64,
    pub eccentricity: f64,
    pub orbital_inclination: f64,
    pub periapsis: f64,
    pub orbital_period: f64,
    pub ascending_node: f64,
    pub mean_anomaly: f64,
    pub rotation_period: f64,
    pub axial_tilt: f64,
    pub rings: Vec<Rings>,
    pub was_discovered: bool,
    pub was_mapped: bool,
}

pub fn draw_body_info(data: &mut Explorer, ui: &mut egui::Ui) {

    let discovered: bool;
    if &data.body.was_discovered == "true" {
        discovered = true;
    }else {
        discovered = false;
    }

    let profit = get_profit_from_body(get_body_class_from_body(&data.body), discovered);
    ui.heading(&data.body.name);
    ui.end_row();
    ui.label("Class");
    ui.label(&data.body.planet_class);
    ui.end_row();
    ui.label("Terraform State");
    ui.label(&data.body.terraform_state);
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
    ui.label(&data.body.was_discovered);
    ui.end_row();
    ui.label("Mapped");
    ui.label(&data.body.was_mapped);
    ui.end_row();
    ui.label("Distance in LS");
    ui.label(&data.body.distance_from_arrival_ls);
    ui.end_row();
    ui.label("Landable");
    ui.label(&data.body.landable);
    ui.end_row();
    ui.label("");
    ui.label("");
    ui.end_row();

    //TODO Planet earnings

    ui.label("Gravity");
    ui.label(&data.body.surface_gravity);
    ui.end_row();
    ui.label("Temperature K");
    ui.label(&data.body.surface_temperature);
    ui.end_row();
    ui.label("Atmosphere");
    ui.label(&data.body.atmosphere);
    ui.end_row();
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
    BeltCluster,
    Ring,
    Star,
    Unknown,
}

pub fn get_body_class_from_body(body: &Body) -> BodyClass {
    if body.name.contains(" Ring"){
        return Ring
    }

    match body.planet_class.as_str() {
        "Ammonia world" => AmmoniaWorld,
        "Earthlike body" => EarthlikeWorld,
        //TODO Create water world terraformable icon
        "Water world" => {
            if body.terraform_state == "Terraformable"{
                return WaterWorldTerraformable;
            }
            WaterWorld
        },
        "High metal content body" => {
            if body.terraform_state == "Terraformable"{
                return HighMetalContentTerraformablePlanet;
            }
            HighMetalContentPlanet
        },
        "Icy body" => IcyBody,
        "Metal rich body" => MetalRichBody,
        "Rocky body" => {
            if body.terraform_state == "Terraformable"{
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
            if body.planet_class.is_empty() || body.planet_class.eq("N/A") || body.planet_class.eq("null"){
                return Star;
            }
            //FIXME If stars come as child, their "Planet Class" cannot be determined
            //[src/app/journal_reader.rs:75] &line = "{ \"timestamp\":\"2022-10-31T00:20:41Z\", \"event\":\"Scan\", \"ScanType\":\"AutoScan\", \"BodyName\":\"Kyloall UO-A e147 A 5\", \"BodyID\":28, \"Parents\":[ {\"Star\":1}, {\"Null\":0} ], \"StarSystem\":\"Kyloall UO-A e147\", \"SystemAddress\":632435992772, \"DistanceFromArrivalLS\":3039.256581, \"StarType\":\"Y\", \"Subclass\":0, \"StellarMass\":0.031250, \"Radius\":85388072.000000, \"AbsoluteMagnitude\":18.674301, \"Age_MY\":308, \"SurfaceTemperature\":646.000000, \"Luminosity\":\"V\", \"SemiMajorAxis\":912356770038.604736, \"Eccentricity\":0.001826, \"OrbitalInclination\":0.081750, \"Periapsis\":334.272320, \"OrbitalPeriod\":236886096.000671, \"AscendingNode\":-58.389303, \"MeanAnomaly\":43.307253, \"RotationPeriod\":322169.147143, \"AxialTilt\":-1.353121, \"WasDiscovered\":false, \"WasMapped\":false }\r\n"
            debug!("{}", &body.planet_class.as_str());
            Unknown
        }
    }
}

/**
Returns tubel of profit
0 -> est. Earn for Discorvery
1 -> est. Earn for Discovery + Mapping
!!!Earnings are estimated. Formular for profit is not known in the moment!!!
**/
pub fn get_profit_from_body(class: BodyClass, discovered: bool) -> (i32, i32){
    match class {
        AmmoniaWorld => { return if discovered { (143463, 1724965) } else { (373004, 597762) } }
        EarthlikeWorld => { return if discovered { (270290, 1126206) } else { (702753, 3249900) } }
        WaterWorld => { return if discovered { (99747, 415613) } else { (259343, 1199337) } }
        WaterWorldTerraformable => { return if discovered { (268616, 1119231) } else { (698400, 3229773) } }
        HighMetalContentPlanet => { return if discovered { (14070, 58624) } else { (36581, 169171) } }
        HighMetalContentTerraformablePlanet => { return if discovered { (163948, 683116) } else { (426264, 1971272) } }
        IcyBody => { return if discovered { (500, 1569) } else { (1300, 4527) } }
        MetalRichBody => { return if discovered { (31632, 131802) } else { (82244, 380341) } }
        RockyBody => { return if discovered { (500, 1476) } else { (1300, 4260) } }
        RockyBodyTerraformable => { return if discovered { (129504, 539601) } else { (336711, 1557130) } }
        RockyIceBody => { return if discovered { (500, 1752) } else { (1300, 5057) } }
        ClassIGasGiant => { return if discovered { (3845, 16021) } else { (9997, 46233) } }
        ClassIIGasGiant => { return if discovered { (28405, 118354) } else { (73853, 341536) } }
        ClassIIIGasGiant => { return if discovered { (995, 4145) } else { (2587, 11963) } }
        ClassIVGasGiant => { return if discovered { (1119, 4663) } else { (2910, 13457) } }
        ClassVGasGiant => { return if discovered { (966, 4023) } else { (2510, 11609) } }
        GasGiantwithAmmoniabasedLife => { return if discovered { (774, 3227) } else { (2014, 9312) } }
        GasGiantwithWaterbasedLife => { return if discovered { (883, 3679) } else { (2295, 10616) } }
        HeliumRichGasGiant => { return if discovered { (900, 3749) } else { (2339, 10818) } }
        WaterGiant => { return if discovered { (667, 2779) } else { (1734, 8019) } }
        BeltCluster => { return if discovered { (0, 0) } else { (0, 0) } }
        Ring => { return if discovered { (0, 0) } else { (0, 0) } }
        Star => { return if discovered { (0, 0) } else { (0, 0) } }
        Unknown => { return if discovered { (0, 0) } else { (0, 0) } }
    }
}

pub fn get_color_image_from_planet_class(planet_class: BodyClass, icons: &Icons) -> &ColorImage {
    match planet_class {
        AmmoniaWorld => { &icons.ammonia_world }
        EarthlikeWorld => { &icons.earthlike_world }
        WaterWorld => { &icons.water_world }
        WaterWorldTerraformable => { &icons.water_word_terraformable}
        HighMetalContentPlanet => { &icons.high_metal_content_planet }
        HighMetalContentTerraformablePlanet => {&icons.rocky_terraformable_body}
        IcyBody => { &icons.icy_body }
        MetalRichBody => { &icons.metal_rich_body }
        RockyBody => { &icons.rocky_body }
        RockyBodyTerraformable => { &icons.rocky_terraformable_body }
        RockyIceBody => { &icons.rocky_ice_body }
        ClassIGasGiant => { &icons.class_i_gas_giant }
        ClassIIGasGiant => { &icons.class_ii_gas_giant }
        ClassIIIGasGiant => { &icons.class_iii_gas_giant }
        ClassIVGasGiant => { &icons.class_iv_gas_giant }
        ClassVGasGiant => { &icons.class_v_gas_giant }
        GasGiantwithAmmoniabasedLife => { &icons.gas_giant_with_ammoniabased_life }
        GasGiantwithWaterbasedLife => { &icons.gas_giant_with_waterbased_life }
        HeliumRichGasGiant => { &icons.helium_rich_gas_giant }
        WaterGiant => { &icons.water_giant }
        BeltCluster => { &icons.belt_cluster }
        Ring => { &icons.ring }
        Star => { &icons.star}
        Unknown => { &icons.unknown }
    }
}

pub struct Icons {
    ammonia_world: ColorImage,
    earthlike_world: ColorImage,
    water_world: ColorImage,
    water_word_terraformable: ColorImage,
    high_metal_content_planet: ColorImage,
    icy_body: ColorImage,
    metal_rich_body: ColorImage,
    rocky_body: ColorImage,
    rocky_terraformable_body: ColorImage,
    rocky_ice_body: ColorImage,
    class_i_gas_giant: ColorImage,
    class_ii_gas_giant: ColorImage,
    class_iii_gas_giant: ColorImage,
    class_iv_gas_giant: ColorImage,
    class_v_gas_giant: ColorImage,
    gas_giant_with_ammoniabased_life: ColorImage,
    gas_giant_with_waterbased_life: ColorImage,
    helium_rich_gas_giant: ColorImage,
    water_giant: ColorImage,
    belt_cluster: ColorImage,
    ring: ColorImage,
    star: ColorImage,
    unknown: ColorImage,
}

impl Default for Icons {
    fn default() -> Self {
        Self {
            ammonia_world: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-ammonia-world.png"),
            earthlike_world: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-earth-like-world.png"),
            water_world: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-water-world.png"),
            //TODO Create Terraformable Water World Icon
            water_word_terraformable: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-water-world.png"),
            high_metal_content_planet: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-rocky.png"),
            icy_body: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-icy.png"),
            metal_rich_body: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-rocky.png"),
            rocky_body: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-rocky.png"),
            rocky_terraformable_body: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-rocky-terraformable.png"),
            rocky_ice_body: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-rocky-ice.png"),
            class_i_gas_giant: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-class-I.png"),
            class_ii_gas_giant: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-class-II.png"),
            class_iii_gas_giant: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-class-III.png"),
            class_iv_gas_giant: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-class-IV.png"),
            class_v_gas_giant: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-class-V.png"),
            gas_giant_with_ammoniabased_life: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-ammoniabased-life.png"),
            gas_giant_with_waterbased_life: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-waterbased-life.png"),
            helium_rich_gas_giant: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-helium.png"),
            water_giant: icons::get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-water.png"),
            belt_cluster: icons::get_colorimage_from_path("graphics/de.paesserver/galaxy-map/asteroids.png"),
            ring: icons::get_colorimage_from_path("graphics/de.paesserver/galaxy-map/asteroids.png"),
            star: icons::get_colorimage_from_path("graphics/de.paesserver/galaxy-map/star.png"),
            unknown: icons::get_colorimage_from_path("graphics/de.paesserver/signals/unknown.png"),
        }
    }
}
