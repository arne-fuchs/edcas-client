use eframe::egui::ColorImage;
use log::{debug, warn};

pub fn get_colorimage_from_path(path: &str) -> ColorImage {
    debug!("{}", &path);
    let image = image::io::Reader::open(path).unwrap().decode().unwrap();
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    )
}

pub struct PlanetSignalIcons {
    pub biological: ColorImage,
    pub geological: ColorImage,
    pub human: ColorImage,
    pub xeno: ColorImage,
    pub unknown: ColorImage,
}

impl Default for PlanetSignalIcons {
    fn default() -> Self {
        Self {
            biological: get_colorimage_from_path("graphics/de.paesserver/signals/biological.png"),
            geological: get_colorimage_from_path("graphics/de.paesserver/signals/geological.png"),
            human: get_colorimage_from_path("graphics/de.paesserver/signals/human.png"),
            xeno: get_colorimage_from_path("graphics/de.paesserver/signals/xeno.png"),
            unknown: get_colorimage_from_path("graphics/de.paesserver/signals/unknown.png"),
        }
    }
}

impl PlanetSignalIcons {
    pub fn get_planet_signal_icon_from_string(&self, icon_string: String) -> &ColorImage {
        let str : &str = icon_string.as_str();
        match str {
            "$SAA_SignalType_Biological;" => { &self.biological }
            "$SAA_SignalType_Geological;" => { &self.geological }
            "$SAA_SignalType_Xenological;" => { &self.xeno }
            "$SAA_SignalType_Human;" => { &self.human }
            _ => {
                warn!("Icon for string not found: {}", &icon_string);
                &self.unknown
            }
        }
    }
}


pub struct BodyIcons {
    pub ammonia_world: ColorImage,
    pub earthlike_world: ColorImage,
    pub water_world: ColorImage,
    pub water_word_terraformable: ColorImage,
    pub high_metal_content_planet: ColorImage,
    pub icy_body: ColorImage,
    pub metal_rich_body: ColorImage,
    pub rocky_body: ColorImage,
    pub rocky_terraformable_body: ColorImage,
    pub rocky_ice_body: ColorImage,
    pub class_i_gas_giant: ColorImage,
    pub class_ii_gas_giant: ColorImage,
    pub class_iii_gas_giant: ColorImage,
    pub class_iv_gas_giant: ColorImage,
    pub class_v_gas_giant: ColorImage,
    pub gas_giant_with_ammoniabased_life: ColorImage,
    pub gas_giant_with_waterbased_life: ColorImage,
    pub helium_rich_gas_giant: ColorImage,
    pub water_giant: ColorImage,
    pub belt_cluster: ColorImage,
    pub ring: ColorImage,
    pub star: ColorImage,
    pub unknown: ColorImage,
}

impl Default for BodyIcons {
    fn default() -> Self {
        Self {
            ammonia_world: get_colorimage_from_path("graphics/de.paesserver/planets/planet-ammonia-world.png"),
            earthlike_world: get_colorimage_from_path("graphics/de.paesserver/planets/planet-earth-like-world.png"),
            water_world: get_colorimage_from_path("graphics/de.paesserver/planets/planet-water-world.png"),
            water_word_terraformable: get_colorimage_from_path("graphics/de.paesserver/planets/planet-water-world-terraformable.png"),
            high_metal_content_planet: get_colorimage_from_path("graphics/de.paesserver/planets/planet-rocky.png"),
            icy_body: get_colorimage_from_path("graphics/de.paesserver/planets/planet-icy.png"),
            metal_rich_body: get_colorimage_from_path("graphics/de.paesserver/planets/planet-rocky.png"),
            rocky_body: get_colorimage_from_path("graphics/de.paesserver/planets/planet-rocky.png"),
            rocky_terraformable_body: get_colorimage_from_path("graphics/de.paesserver/planets/planet-rocky-terraformable.png"),
            rocky_ice_body: get_colorimage_from_path("graphics/de.paesserver/planets/planet-rocky-ice.png"),
            class_i_gas_giant: get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-class-I.png"),
            class_ii_gas_giant: get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-class-II.png"),
            class_iii_gas_giant: get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-class-III.png"),
            class_iv_gas_giant: get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-class-IV.png"),
            class_v_gas_giant: get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-class-V.png"),
            gas_giant_with_ammoniabased_life: get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-ammoniabased-life.png"),
            gas_giant_with_waterbased_life: get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-waterbased-life.png"),
            helium_rich_gas_giant: get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-helium.png"),
            water_giant: get_colorimage_from_path("graphics/de.paesserver/planets/planet-gas-giant-water.png"),
            belt_cluster: get_colorimage_from_path("graphics/de.paesserver/galaxy-map/asteroids.png"),
            ring: get_colorimage_from_path("graphics/de.paesserver/galaxy-map/asteroids.png"),
            star: get_colorimage_from_path("graphics/de.paesserver/galaxy-map/star.png"),
            unknown: get_colorimage_from_path("graphics/de.paesserver/signals/unknown.png"),
        }
    }
}

pub struct Symbols{
    pub arrow_down: ColorImage,
    pub landable: ColorImage,
    pub not_landable: ColorImage,
    pub landable_sphere: ColorImage,
    pub binoculars: ColorImage,
    pub map: ColorImage,
    pub shoe: ColorImage,
}

impl Default for Symbols {
    fn default() -> Self {
        Symbols{
            arrow_down: get_colorimage_from_path("graphics/de.paesserver/symbols/arrow_down.png"),
            landable: get_colorimage_from_path("graphics/de.paesserver/symbols/landable.png"),
            not_landable: get_colorimage_from_path("graphics/de.paesserver/symbols/not_landable.png"),
            landable_sphere: get_colorimage_from_path("graphics/de.paesserver/symbols/landable_sphere.png"),
            binoculars: get_colorimage_from_path("graphics/de.paesserver/symbols/binoculars.png"),
            map: get_colorimage_from_path("graphics/de.paesserver/symbols/map.png"),
            shoe: get_colorimage_from_path("graphics/de.paesserver/symbols/shoe.png"),
        }
    }
}