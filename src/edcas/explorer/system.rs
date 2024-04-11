use std::ops::Add;
use std::sync::Arc;

use eframe::egui;
use eframe::emath::Numeric;

use crate::edcas::explorer::body::{BodyType, Signal};
use crate::edcas::settings::Settings;

#[derive(Clone)]
pub struct System {
    pub name: String,
    pub address: String,
    pub allegiance: String,
    pub economy_localised: String,
    pub second_economy_localised: String,
    pub government_localised: String,
    pub security_localised: String,
    pub population: String,
    pub body_count: String,
    pub non_body_count: String,
    pub signal_list: Vec<SystemSignal>,
    pub body_list: Vec<BodyType>,
    pub planet_signals: Vec<PlanetSignals>,
    pub index: usize,
    pub settings: Arc<Settings>,
}

#[derive(Clone)]
pub struct PlanetSignals {
    pub body_name: String,
    pub body_id: i64,
    pub signals: Vec<Signal>,
}

impl PartialEq for PlanetSignals {
    fn eq(&self, other: &Self) -> bool {
        self.body_id == other.body_id
    }
}

impl System {
    pub fn draw_body_signal_list(&self, ui: &mut egui::Ui) {
        egui::Grid::new("body_signal_grid")
            .num_columns(3)
            .striped(true)
            .min_col_width(130.0)
            .show(ui, |ui| {
                ui.label("Body");
                ui.label("Type");
                ui.label("Count");
                ui.end_row();

                for body_signal in &self.planet_signals {
                    for signal in &body_signal.signals {
                        ui.label(body_signal.body_name.trim_start_matches(&self.name));
                        if &signal.type_localised == "null" {
                            ui.label(&signal.r#type);
                        } else {
                            ui.label(&signal.type_localised);
                        }

                        let id = body_signal
                            .body_name
                            .clone()
                            .add(&signal.r#type.to_string().clone());

                        egui::Grid::new(id).num_columns(2).striped(true).show(
                            ui,
                            |ui| match signal.r#type.as_str() {
                                "$SAA_SignalType_Biological;" => {
                                    ui.label(&signal.count.to_string());
                                    ui.label(
                                        self.settings
                                            .icons
                                            .get("bio_signal")
                                            .unwrap()
                                            .get_richtext(),
                                    );
                                }
                                "$SAA_SignalType_Geological;" => {
                                    ui.label(&signal.count.to_string());
                                    ui.label(
                                        self.settings
                                            .icons
                                            .get("geo_signal")
                                            .unwrap()
                                            .get_richtext(),
                                    );
                                }
                                "$SAA_SignalType_Xenological;" => {
                                    ui.label(&signal.count.to_string());
                                    ui.label(
                                        self.settings
                                            .icons
                                            .get("xeno_signal")
                                            .unwrap()
                                            .get_richtext(),
                                    );
                                }
                                "$SAA_SignalType_Human;" => {
                                    ui.label(&signal.count.to_string());
                                    ui.label(
                                        self.settings
                                            .icons
                                            .get("human_signal")
                                            .unwrap()
                                            .get_richtext(),
                                    );
                                }
                                _ => {
                                    ui.label(&signal.count.to_string());
                                    ui.label(
                                        self.settings
                                            .icons
                                            .get("unknown_signal")
                                            .unwrap()
                                            .get_richtext(),
                                    );
                                }
                            },
                        );
                        ui.end_row();
                    }
                }
            });
    }

    fn draw_system_details(&self, ui: &mut egui::Ui) {
        ui.label("Allegiance");
        ui.label(&self.allegiance);
        ui.end_row();

        ui.label("Economy");
        ui.label(&self.economy_localised);
        ui.end_row();

        ui.label("sec. Economy");
        ui.label(&self.second_economy_localised);
        ui.end_row();

        ui.label("Government");
        ui.label(&self.government_localised);
        ui.end_row();

        ui.label("Security");
        ui.label(&self.security_localised);
        ui.end_row();

        ui.label("Population");
        ui.label(&self.population);
        ui.end_row();
    }

    pub fn draw_system_info(&self, ui: &mut egui::Ui) {
        egui::Grid::new("system_data_grid")
            .num_columns(2)
            .striped(true)
            .min_col_width(200.0)
            .show(ui, |ui| {
                self.draw_system_details(ui);
            });

        ui.separator();
        egui::Grid::new("body_count_grid")
            .num_columns(2)
            .striped(true)
            .min_col_width(200.0)
            .show(ui, |ui| {
                ui.label("Bodies");
                ui.label(&self.body_count);
                ui.end_row();
                ui.label("Non-bodies");
                ui.label(&self.non_body_count);
                ui.end_row();
            });

        if !self.body_count.eq("N/A") {
            ui.add(
                egui::ProgressBar::new(
                    (&self.body_list.len().to_f64()
                        / (&self.body_count.parse::<f64>().unwrap()
                            + &self.non_body_count.parse::<f64>().unwrap()))
                        as f32,
                )
                .text(
                    self.body_list.len().to_string().add("/").add(
                        (&self.body_count.parse::<f64>().unwrap()
                            + &self.non_body_count.parse::<f64>().unwrap())
                            .to_string()
                            .as_str(),
                    ),
                ),
            );
        }
        ui.end_row();
        ui.separator();
        ui.heading("System Signals");
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                self.draw_system_signal_list(ui);
            });
    }

    fn draw_system_signal_list(&self, ui: &mut egui::Ui) {
        egui::Grid::new("system_signal_grid")
            .num_columns(2)
            .striped(true)
            .min_col_width(130.0)
            .show(ui, |ui| {
                ui.label("Name");
                ui.label("Thread");
                ui.end_row();
                for system_signal in &self.signal_list {
                    ui.label(&system_signal.name);
                    ui.label(&system_signal.threat);

                    ui.end_row();
                }
            });
    }

    pub fn insert_body(&mut self, body: BodyType) -> usize {
        let id = body.get_id();
        self.body_list.push(body);

        self.body_list
            .sort_by(|body_a, body_b| body_a.get_id().cmp(&body_b.get_id()));

        for i in 0..self.body_list.len() {
            if id == self.body_list.get(i).unwrap().get_id() {
                return i;
            }
        }
        0
    }
}

#[derive(Clone)]
pub struct SystemSignal {
    pub timestamp: String,
    pub event: String,
    pub name: String,
    pub threat: String,
}
