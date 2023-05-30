use eframe::egui::ColorImage;
use log::debug;

use crate::app::explorer::{Explorer};

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

pub struct Icons {
    pub biological: ColorImage,
    pub geological: ColorImage,
    pub human: ColorImage,
    pub xeno: ColorImage,
    pub unknown: ColorImage,
}

impl Default for Icons {
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

pub fn get_icon_from_string(icon_string: String, icons: &Icons) -> &ColorImage {
    let str : &str = icon_string.as_str();
    match str {
        "$SAA_SignalType_Biological;" => { &icons.biological }
        "$SAA_SignalType_Geological;" => { &icons.geological }
        "$SAA_SignalType_Xenological;" => { &icons.xeno }
        "$SAA_SignalType_Human;" => { &icons.human }
        _ => {
            debug!("{}", &icon_string);
            &icons.unknown
        }
    }
}