use eframe::egui;

pub struct About {
    github_url: String,
}

impl Default for About {
    fn default() -> Self {
        Self {
            github_url: "https://github.com/arne-fuchs".to_owned(),
        }
    }
}

impl About {
    pub fn update(&self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("About");
            ui.label("Arne Fuchs");
            ui.horizontal(|ui| {
                ui.label("Github:");
                ui.hyperlink(&self.github_url);
            });
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
    }
}