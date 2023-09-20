use std::env;
use eframe::egui;
use eframe::egui::{ColorImage, TextureHandle};

pub struct About {
    github_url: String,
    logo: ColorImage
}

impl Default for About {
    fn default() -> Self {
        let mut logo_path = image::io::Reader::open("graphics\\logo\\edcas.png");
        match env::var("HOME") {
            Ok(home) => {
                match image::io::Reader::open(format!("{}/.local/share/edcas-client/graphics/logo/edcas.png", home)) {
                    Ok(_) => {
                        logo_path = image::io::Reader::open(format!("{}/.local/share/edcas-client/graphics/logo/edcas.png", home));
                    }
                    Err(_) => {
                        logo_path = image::io::Reader::open("graphics/logo/edcas.png");
                    }
                }
            }
            Err(_) => {}
        }
        let image = logo_path.unwrap().decode().unwrap();
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        let color_image = ColorImage::from_rgba_unmultiplied(
            size,
            pixels.as_slice(),
        );
        Self {
            github_url: "https://github.com/arne-fuchs".to_owned(),
            logo: color_image,
        }
    }
}

impl About {
    pub fn update(&self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("About");
            ui.label("Arne Fuchs");
            let texture: TextureHandle = ui.ctx().load_texture(
                "logo",
                self.logo.clone(),
                egui::TextureOptions::LINEAR,
            );
            let img_size = 256.0 * texture.size_vec2() / texture.size_vec2().y;
            ui.image(&texture, img_size);
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