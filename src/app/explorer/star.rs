use std::sync::Arc;

use eframe::egui::Ui;
use num_format::{Locale, ToFormattedString};

use crate::app::explorer::planet::AsteroidRing;
use crate::app::explorer::body::{BodyImplementation, Parent};
use crate::app::settings::Settings;
#[derive(Clone)]
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
    pub asteroid_rings: Vec<AsteroidRing>,
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
        ui.heading("Rings");
        ui.end_row();
        for ring in &self.asteroid_rings {
            ui.label(&ring.ring_class);
            ui.vertical(|ui| {
                ui.label(format!("{}km", &ring.outer_rad));
                ui.label(format!("{}km", &ring.inner_rad));
            });
            ui.end_row();
        }
    }
    fn print_header_content(&self, ui: &mut Ui, system_index: &mut usize, body_index: usize) {
        if self
            .settings
            .stars
            .get(self.star_type.as_str())
            .unwrap()
            .enabled
        {
            ui.label(
                self.settings
                    .stars
                    .get(self.star_type.as_str())
                    .unwrap()
                    .get_richtext()
                    .size(self.radius.log(1.7) as f32),
            );
        }
        let mut body_name = self.body_name.to_string();
        if !self.settings.explorer_settings.include_system_name {
            body_name.replace_range(0..self.star_system.len(), "");
        }
        if ui.selectable_label(false, &body_name).clicked() {
            *system_index = body_index;
        };

        for ring in &self.asteroid_rings {
            if self
                .settings
                .icons
                .get(ring.ring_class.as_str())
                .unwrap()
                .enabled
            {
                ui.label("|");
                ui.label(
                    self.settings
                        .icons
                        .get(ring.ring_class.as_str())
                        .unwrap()
                        .get_richtext(),
                );
            }
        }

        if self.was_discovered && self.settings.icons.get("discovered").unwrap().enabled {
            ui.label("|");
            ui.label(
                self.settings
                    .icons
                    .get("discovered")
                    .unwrap()
                    .get_richtext(),
            );
        }
        if self.was_mapped && self.settings.icons.get("mapped").unwrap().enabled {
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

    fn get_parents(&self) -> Vec<Parent> {
        self.parents.clone()
    }

    fn get_body(&self) -> crate::app::explorer::body::BodyType {
        //FIXME Inefficient
        crate::app::explorer::body::BodyType::Star(self.clone())
    }
}
