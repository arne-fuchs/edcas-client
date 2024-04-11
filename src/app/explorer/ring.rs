use std::sync::Arc;

use eframe::egui::Ui;
use num_format::{Locale, ToFormattedString};

use crate::app::explorer::body::{BodyImplementation, Parent, Signal};
use crate::app::settings::Settings;
#[derive(Clone)]
pub struct Ring {
    pub timestamp: String,
    pub event: String,
    pub scan_type: String,
    pub body_name: String,
    pub body_id: i64,
    pub parents: Vec<Parent>,
    pub star_system: String,
    pub system_address: i64,
    pub distance_from_arrival_ls: f64,
    pub semi_major_axis: f64,
    pub eccentricity: f64,
    pub orbital_inclination: f64,
    pub periapsis: f64,
    pub orbital_period: f64,
    pub ascending_node: f64,
    pub mean_anomaly: f64,
    pub was_discovered: bool,
    pub was_mapped: bool,
    pub ring_signals: Vec<Signal>,
    pub settings: Arc<Settings>,
}

impl BodyImplementation for Ring {
    fn print_side_panel_information(&self, ui: &mut Ui) {
        ui.heading(&self.body_name);
        ui.end_row();
        for signal in &self.ring_signals {
            let mut signal_name = signal.type_localised.clone();
            if &signal.type_localised.to_lowercase() == "null" {
                signal_name = signal.r#type.clone();
            }
            ui.label(signal_name);
            ui.label(signal.count.to_string());
            ui.end_row();
        }
    }
    fn print_header_content(&self, ui: &mut Ui, system_index: &mut usize, body_index: usize) {
        let mut body_name = self.body_name.to_string();
        if !self.settings.explorer_settings.include_system_name {
            let system_name = self.star_system.clone();
            body_name.replace_range(0..system_name.len(), "");
        }
        if ui.selectable_label(false, &body_name).clicked() {
            *system_index = body_index;
        };

        if !self.ring_signals.is_empty() {
            for signal in &self.ring_signals {
                body_name.push('|');
                body_name.push_str(&signal.count.to_string());
                body_name.push(' ');
                let mut signal_name = signal.type_localised.clone();
                if &signal.type_localised.to_lowercase() == "null" {
                    signal_name = signal.r#type.clone();
                }
                signal_name.replace_range(signal_name.len() / 2..signal_name.len(), "");
                signal_name.push('.');
                body_name.push_str(signal_name.as_str());
            }
        }
        if self.settings.icons.get("distance").unwrap().enabled {
            ui.label("|");
            ui.label((self.distance_from_arrival_ls as u64).to_formatted_string(&Locale::en));
            ui.label(" LS");
            ui.label(self.settings.icons.get("distance").unwrap().get_richtext());
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

    fn get_signals(&self) -> Vec<Signal> {
        self.ring_signals.clone()
    }

    fn set_signals(&mut self, signals: Vec<Signal>) {
        self.ring_signals = signals;
    }

    fn get_parents(&self) -> Vec<Parent> {
        self.parents.clone()
    }

    fn get_body(&self) -> crate::app::explorer::body::BodyType {
        //FIXME Inefficient
        crate::app::explorer::body::BodyType::Ring(self.clone())
    }
}
