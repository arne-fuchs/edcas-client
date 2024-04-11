use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::app::evm_interpreter::edcas_contract;
use crate::app::evm_updater::EvmRequest;
use chrono::{DateTime, Utc};
use eframe::egui::collapsing_header::CollapsingState;
use eframe::egui::Context;
use eframe::{egui, App, Frame};
use log::error;

use crate::app::settings::Settings;

pub struct StationState {
    pub stations: Vec<Station>,
    pub search: String,
    pub evm_request_writer: Sender<EvmRequest>,
    pub settings: Arc<Settings>,
}

#[derive(Clone)]
pub struct Station {
    pub market_id: u64,
    pub name: String,
    pub _type: String,
    pub requested_meta_data: bool,
    pub meta_data: Option<StationMetaData>,
    pub requested_market: bool,
    pub market: Option<Vec<CommodityListening>>,
}
#[derive(Clone)]
pub struct StationMetaData {
    pub timestamp: DateTime<Utc>,
    pub services: String,
    pub system_name: String,
    pub faction: edcas_contract::Faction,
    pub government: String,
    pub economy: String,
    pub distance: edcas_contract::Floating,
    pub landingpads: String,
}
#[derive(Clone)]
pub struct CommodityListening {
    pub name: String,
    pub buy_price: u32,
    pub mean_price: u32,
    pub demand: u32,
    pub demand_bracket: u32,
    pub stock: u32,
    pub stock_bracket: u32,
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
                for station in &mut self.stations {
                    if station.name.to_ascii_lowercase().contains(search.as_str()) {
                        let id = ui.make_persistent_id(station.market_id);
                        CollapsingState::load_with_default_open(ui.ctx(), id, false)
                            .show_header(ui, |ui| {
                                ui.label(&station.name);
                            })
                            .body(|ui| {
                                egui::Grid::new(&station.name)
                                    .num_columns(1)
                                    .striped(true)
                                    .show(ui, |ui| {
                                        match &station.meta_data {
                                            None => {
                                                if !station.requested_meta_data {
                                                    if let Err(err) = self.evm_request_writer.send(EvmRequest::StationMetaDataRequest(station.market_id)){
                                                        error!("Error sending StationMetaDataRequest: {err}");
                                                    }
                                                    station.requested_meta_data = true;
                                                }else {
                                                    ui.label("Fetching...");
                                                }
                                            }
                                            Some(meta_data) => {
                                                let mut distance = meta_data.distance.decimal.to_string();
                                                distance.insert(meta_data.distance.floating_point as usize,'.');
                                                ui.label(&meta_data.system_name);
                                                ui.end_row();
                                                ui.label(distance);
                                                ui.label("LS");
                                                ui.end_row();
                                                ui.label(&meta_data.economy);
                                                ui.end_row();
                                                ui.label(&meta_data.government);
                                                ui.end_row();
                                                ui.label(&meta_data.services);
                                                ui.end_row();
                                                ui.label(&meta_data.landingpads);
                                            }
                                        }
                                    });
                            });
                    }
                }
            });
        });
    }
}
