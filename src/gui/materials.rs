use crate::edcas::materials::{Material, MaterialState};
use eframe::egui::{vec2, Color32, Context, Ui, Widget, Window};
use eframe::{egui, App, Frame};
use std::collections::HashMap;

impl App for MaterialState {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let Self {
            raw,
            manufactured,
            encoded,
            showing: _,
            search: _,
        } = self;

        print_material_info_window_if_available(&mut self.showing, ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.label("Search: ");
                ui.text_edit_singleline(&mut self.search);
            });
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("inventory_grid")
                    .num_columns(3)
                    .min_col_width(ui.available_width() / 3_f32)
                    .max_col_width(ui.available_width() / 3_f32)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Encoded");
                        ui.label("Manufactured");
                        ui.label("Raw");
                        ui.end_row();
                        draw_materials(&mut self.showing, ui, encoded, &self.search);
                        draw_materials(&mut self.showing, ui, manufactured, &self.search);
                        draw_materials(&mut self.showing, ui, raw, &self.search);
                        ui.end_row();
                    });
            });
        });
    }
}
fn draw_materials(
    showing: &mut Option<Material>,
    ui: &mut Ui,
    materials: &HashMap<String, Material>,
    search: &String,
) {
    ui.vertical(|ui| {
        let mut en_iter = materials.iter();
        let mut option_material = en_iter.next();
        while option_material.is_some() {
            let material = option_material.unwrap().1;
            if material
                .name_localised
                .to_lowercase()
                .contains(&search.to_lowercase())
                || material
                    .name
                    .to_lowercase()
                    .contains(&search.to_lowercase())
            {
                ui.vertical(|ui| {
                    ui.vertical_centered(|ui| {
                        if ui.button(material.get_name()).clicked() {
                            let _ = showing.replace(material.clone());
                        }
                        let mut percentage = 0f32;
                        if material.maximum != 0 {
                            percentage = material.count as f32 / material.maximum as f32;
                        }
                        let color = convert_color(percentage);
                        let _ = egui::ProgressBar::new(percentage)
                            .text(format!("{}/{}", material.count, material.maximum))
                            .fill(Color32::from_rgb(color.0, color.1, color.2))
                            .desired_width(ui.available_width() / 1.2)
                            .ui(ui);
                    });
                });
                ui.separator();
            }
            option_material = en_iter.next();
        }
    });
}

fn convert_color(value: f32) -> (u8, u8, u8) {
    // Scale the value from 0.0 to 1.0 to the range 0 to 255
    let scaled_value = (value * 255.0).round() as u8;

    // Calculate the green and red components based on the scaled value
    let mut red = 255 - scaled_value;
    let mut green = scaled_value;

    red = (red as f32 * 0.6).round() as u8;
    green = (green as f32 * 0.6).round() as u8;

    // Return the resulting color as a tuple (R, G, B)
    (red, green, 0) // Assuming a fixed blue value of 0
}

pub fn print_material_info_window_if_available(showing: &mut Option<Material>, ctx: &Context) {
    match showing.clone() {
        None => {}
        Some(material) => {
            Window::new(material.get_name())
                .collapsible(false)
                .resizable(true)
                .default_size(vec2(ctx.available_rect().width() / 1.1, 600f32))
                .show(ctx, |ui| {
                    egui::Grid::new("materials_description")
                        .num_columns(2)
                        .max_col_width(ui.available_width() / 1.3)
                        .show(ui, |ui| {
                            ui.label(&material.description);
                            ui.vertical(|ui| {
                                ui.label(format!("Grade: {}", &material.grade));
                                ui.label(format!("Category: {}", &material.category));
                                let mut percentage = 0f32;
                                if material.maximum != 0 {
                                    percentage = material.count as f32 / material.maximum as f32;
                                }
                                let color = convert_color(percentage);
                                let _ = egui::ProgressBar::new(percentage)
                                    .text(format!("{}/{}", material.count, material.maximum))
                                    .fill(Color32::from_rgb(color.0, color.1, color.2))
                                    .desired_width(ui.available_width() / 1.2)
                                    .ui(ui);
                            });
                        });

                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::Grid::new("material_info_grid")
                            .num_columns(4)
                            .min_col_width(ui.available_width() / 4.0)
                            .max_col_width(ui.available_width() / 4.0)
                            .show(ui, |ui| {
                                egui::Grid::new("material_location")
                                    .striped(true)
                                    .num_columns(1)
                                    .show(ui, |ui| {
                                        ui.heading("Locations");
                                        ui.end_row();
                                        for text in &material.locations {
                                            ui.label(text);
                                            ui.end_row();
                                        }
                                    });
                                egui::Grid::new("material_sources")
                                    .striped(true)
                                    .num_columns(1)
                                    .show(ui, |ui| {
                                        ui.heading("Sources");
                                        ui.end_row();
                                        for text in &material.sources {
                                            ui.label(text);
                                            ui.end_row();
                                        }
                                    });
                                egui::Grid::new("material_engineering")
                                    .striped(true)
                                    .num_columns(1)
                                    .show(ui, |ui| {
                                        ui.label("Engineering");
                                        ui.end_row();
                                        for text in &material.engineering {
                                            ui.label(text);
                                            ui.end_row();
                                        }
                                    });
                                egui::Grid::new("material_synthesis")
                                    .striped(true)
                                    .num_columns(1)
                                    .show(ui, |ui| {
                                        ui.heading("Synthesis");
                                        ui.end_row();
                                        for text in &material.synthesis {
                                            ui.label(text);
                                            ui.end_row();
                                        }
                                    });
                            });
                    });
                    ui.separator();
                    ui.vertical_centered(|ui| {
                        if ui.button("Close").clicked() {
                            showing.take();
                        }
                    });
                });
        }
    }
}
