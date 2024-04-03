use std::sync::Arc;

use crate::app::evm_interpreter::edcas_contract;
use chrono::{DateTime, Utc};
use eframe::egui::collapsing_header::{CollapsingState, HeaderResponse};
use eframe::egui::Context;
use eframe::{egui, App, Frame};

use crate::app::settings::Settings;

pub struct StationState {
    pub stations: Vec<Station>,
    pub search: String,
    pub settings: Arc<Settings>,
}

#[derive(Clone)]
pub struct Station {
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub _type: String,
    pub services: String,
    pub system_name: String,
    pub faction: edcas_contract::Faction,
    pub government: String,
    pub economy: String,
    pub distance: edcas_contract::Floating,
    pub landingpads: String,
}

impl App for StationState {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.label("Search: ");
                ui.text_edit_singleline(&mut self.search);
            });
            ui.end_row();

            let search = self.search.to_ascii_lowercase();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for station in &self.stations {
                    if station.name.to_ascii_lowercase().contains(search.as_str())
                        || station
                            .system_name
                            .to_ascii_lowercase()
                            .contains(search.as_str())
                    {
                        let id = ui.make_persistent_id(station.name.clone());
                        CollapsingState::load_with_default_open(ui.ctx(), id, false)
                            .show_header(ui, |ui| {
                                ui.label(format!("{} - {}", station.system_name, station.name));
                            })
                            .body(|ui| {
                                egui::Grid::new("carrier_grid")
                                    .num_columns(1)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        let mut distance = station.distance.decimal.to_string();
                                        distance
                                            .insert(station.distance.floating_point as usize, '.');
                                        ui.label(station.services.clone());
                                        ui.end_row();
                                        ui.label(format!("Last update: {}", station.timestamp));
                                        ui.end_row();
                                        ui.label(format!("Distance from Main Star: {}", distance));
                                        ui.end_row();
                                        ui.label(format!("Type: {}", station._type));
                                        ui.end_row();
                                        ui.label(format!("Economy: {}", station.economy));
                                        ui.end_row();
                                        ui.label(format!("Government: {}", station.government));
                                        ui.end_row();
                                        ui.label(format!("Landing pads: {}", station.landingpads));
                                    });
                            });
                    }
                }
            });
        });
    }
}
