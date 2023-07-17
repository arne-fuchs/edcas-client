use std::sync::Arc;
use eframe::egui;
use eframe::egui::{TextureHandle, Ui};
use num_format::{Locale, ToFormattedString};
use crate::app::explorer::structs::{BodyImplementation, Parent};
use crate::app::explorer::system::System;
use crate::app::settings::Settings;
use crate::ICON_BODY;

pub struct Star {
    pub timestamp: String,
    pub event: String,
    pub scan_type: String,
    pub body_name: String,
    pub body_id: i64,
    pub parents: Vec<Parent>,
    pub star_system: String,
    pub system_address: i64,
    pub distance_from_arrival_ls: f64,
    pub star_type: String,
    pub subclass: i64,
    pub stellar_mass: f64,
    pub radius: f64,
    pub absolute_magnitude: f64,
    pub age_my: i64,
    pub surface_temperature: f64,
    pub luminosity: String,
    pub semi_major_axis: Option<f64>,
    pub eccentricity: Option<f64>,
    pub orbital_inclination: Option<f64>,
    pub periapsis: Option<f64>,
    pub orbital_period: Option<f64>,
    pub ascending_node: Option<f64>,
    pub mean_anomaly: Option<f64>,
    pub rotation_period: f64,
    pub axial_tilt: f64,
    pub was_discovered: bool,
    pub was_mapped: bool,
    pub settings: Arc<Settings>,
}

impl BodyImplementation for Star {
    fn print_side_panel_information(&self, ui: &mut Ui) {
        ui.heading(&self.body_name);
        ui.end_row();
        ui.label("Type");
        ui.label(&self.star_type);
        ui.end_row();
        ui.label("Subclass");
        ui.label(&self.subclass.to_string());
        ui.end_row();
        ui.label("Age in My");
        ui.label(&self.age_my.to_formatted_string(&Locale::en));
        ui.end_row();
        ui.label("Stellar Mass");
        ui.label(&self.stellar_mass.to_string());
        ui.end_row();
        ui.label("Radius");
        ui.label(&self.radius.to_string());
        ui.end_row();
        ui.label("");
        ui.label("");
        ui.end_row();
        ui.label("Discovered");
        ui.label(&self.was_discovered.to_string());
        ui.end_row();
        ui.label("Distance in LS");
        ui.label(&self.distance_from_arrival_ls.to_string());
        ui.end_row();
        ui.label("Temperature K");
        ui.label(&self.surface_temperature.to_string());
        ui.end_row();
    }

    fn print_header_content(&self, ui: &mut Ui, system_index: &mut usize, body_index: usize){
        let texture: TextureHandle = ui.ctx().load_texture(
            "parentless-body-icon",
            ICON_BODY.lock().unwrap().star.clone(),
            egui::TextureOptions::LINEAR,
        );

        let img_size = 32.0 * texture.size_vec2() / texture.size_vec2().y;
        ui.image(&texture, img_size);
        let mut body_name = self.body_name.to_string();
        if !self.settings.explorer_settings.include_system_name{
            let system_name = self.star_system.clone();
            body_name.replace_range(0..system_name.len(),"");
        }

        if ui.selectable_label(false, &body_name).clicked() {
            *system_index = body_index;
        };
        if self.was_discovered && self.settings.explorer_settings.show_discovered{
            ui.label("|🚩");
        }
        if self.was_mapped && self.settings.explorer_settings.show_mapped{
            ui.label("|🗺");
        }
    }

    fn get_name(&self) -> &str {
        self.body_name.as_str()
    }

    fn get_id(&self) -> i64 {
        self.body_id
    }

    fn get_parents(&self) -> Vec<Parent> {
        self.parents.clone()
    }
}

//TODO Get new icons from star type