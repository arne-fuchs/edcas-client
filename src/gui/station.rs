use eframe::egui::collapsing_header::CollapsingState;
use eframe::egui::Context;
use eframe::{egui, App, Frame};
use log::error;

impl App for crate::edcas::station::StationState {
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
                                                    if let Err(err) = self.evm_request_writer.send(crate::edcas::backend::evm_updater::EvmRequest::StationMetaData(station.market_id)){
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
