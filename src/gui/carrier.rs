use eframe::egui::collapsing_header::CollapsingState;
use eframe::egui::Context;
use eframe::{egui, App, Frame};

impl App for crate::edcas::carrier::CarrierState {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.label("Search: ");
                ui.text_edit_singleline(&mut self.search);
            });
            ui.end_row();

            let search = self.search.to_ascii_lowercase();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for carrier in &self.carriers {
                    if carrier.name.contains(search.as_str())
                        || carrier
                            .callsign
                            .to_ascii_lowercase()
                            .contains(search.as_str())
                        || carrier
                            .next_system
                            .to_ascii_lowercase()
                            .contains(search.as_str())
                        || carrier
                            .next_body
                            .to_ascii_lowercase()
                            .contains(search.as_str())
                        || carrier
                            .current_system
                            .to_ascii_lowercase()
                            .contains(search.as_str())
                        || carrier
                            .current_body
                            .to_ascii_lowercase()
                            .contains(search.as_str())
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
