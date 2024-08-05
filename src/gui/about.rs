use eframe::egui::include_image;
use eframe::{egui, App};

impl App for crate::edcas::about::About {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add(
                    egui::Image::new(include_image!("../../graphics/logo/edcas.png"))
                        .max_width(512.0)
                        .rounding(10.0),
                );
                ui.label("🐟 Discord:");
                ui.hyperlink(&self.discord_url);
                ui.end_row();
                ui.label(" Github:");
                ui.hyperlink(&self.github_url);
                ui.end_row();
                ui.label("📄 Version:");
                ui.hyperlink_to(
                    option_env!("CARGO_PKG_VERSION").unwrap_or("Unknown"),
                    "https://github.com/arne-fuchs/edcas-client/releases",
                );
                ui.end_row();

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("powered by ");
                        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                        ui.label(" and ");
                        ui.hyperlink_to(
                            "eframe",
                            "https://github.com/emilk/egui/tree/master/crates/eframe",
                        );
                        ui.label(".");
                    });
                });
            });
        });
    }
}
