use eframe::egui::TextureHandle;
use eframe::{egui, App};

impl App for crate::edcas::news::News {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let texture: TextureHandle =
                    ui.ctx()
                        .load_texture("logo", self.logo.clone(), egui::TextureOptions::LINEAR);
                let image = egui::Image::new(&texture).max_width(512.0).rounding(10.0);
                ui.vertical_centered(|ui| {
                    ui.add(image);
                    ui.heading("Galnet News");
                });

                ui.separator();
                ui.end_row();
                for news in &self.articles {
                    ui.vertical_centered(|ui| {
                        ui.end_row();
                        ui.separator();
                        ui.heading(&news.title);
                        ui.separator();
                        ui.heading(&news.date);
                    });
                    ui.separator();
                    egui::Grid::new(&news.title)
                        .num_columns(3)
                        .min_col_width(200.0)
                        .max_col_width(1000.0)
                        .show(ui, |ui| {
                            ui.label("");
                            ui.label(&news.text.replace('\n', "\n\n"));
                            ui.label("");
                            ui.end_row();
                        });
                    ui.separator();
                }
            });
        });
    }
}
