use std::sync::Arc;

use chrono::{DateTime, Utc};
use eframe::egui::collapsing_header::{CollapsingState, HeaderResponse};
use eframe::egui::Context;
use eframe::{egui, App, Frame};

use crate::app::settings::Settings;

pub struct CarrierState {
    pub carriers: Vec<Carrier>,
    pub search: String,
    pub settings: Arc<Settings>,
}

#[derive(Clone)]
pub struct Carrier {
    pub timestamp: DateTime<Utc>,
    pub name: String,
    pub callsign: String,
    pub services: String,
    pub docking_access: String,
    pub allow_notorious: bool,
    pub current_system: String,
    pub current_body: String,
    pub next_system: String,
    pub next_body: String,
    pub departure: DateTime<Utc>,
}

impl App for CarrierState {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.label("Search: ");
                ui.text_edit_singleline(&mut self.search);
            });
            ui.end_row();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for carrier in &self.carriers {
                    if carrier.name.contains(self.search.as_str())
                        || carrier.callsign.contains(self.search.as_str())
                        || carrier.next_system.contains(self.search.as_str())
                        || carrier.next_body.contains(self.search.as_str())
                        || carrier.current_system.contains(self.search.as_str())
                        || carrier.current_body.contains(self.search.as_str())
                    {
                        let id = ui.make_persistent_id(carrier.callsign.clone());
                        CollapsingState::load_with_default_open(ui.ctx(), id, false)
                            .show_header(ui, |ui| {
                                ui.label(format!("{} - {}", carrier.name, carrier.callsign));
                            })
                            .body(|ui| {
                                egui::Grid::new("carrier_grid")
                                    .num_columns(1)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.label(format!("Last update: {}", carrier.timestamp));
                                        ui.end_row();
                                        ui.label(format!(
                                            "Location: {} - {}",
                                            carrier.current_system, carrier.current_body
                                        ));
                                        ui.end_row();
                                        ui.label(format!(
                                            "Next Jump: {} - {} at {}",
                                            carrier.next_system,
                                            carrier.next_body,
                                            carrier.departure
                                        ));
                                        ui.end_row();
                                        ui.label(carrier.services.clone());
                                    });
                            });
                    }
                }
            });
        });
    }
}
