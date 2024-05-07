use eframe::egui::TextureHandle;
use eframe::{egui, App};

impl App for crate::edcas::about::About {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let texture: TextureHandle =
                    ui.ctx()
                        .load_texture("logo", self.logo.clone(), egui::TextureOptions::LINEAR);
                let image = egui::Image::new(&texture).max_width(512.0).rounding(10.0);
                ui.add(image);
                ui.label("Discord:");
                ui.hyperlink("https://discord.gg/fsstTkAw");
                ui.end_row();
                ui.label("Github:");
                ui.hyperlink(&self.github_url);
                ui.end_row();

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
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
