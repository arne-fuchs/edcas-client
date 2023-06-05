use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use eframe::{App, egui, Frame};
use eframe::egui::Context;
use crate::app::cargo_reader::{Cargo, CargoReader};

pub struct Mining {
    pub prospectors: VecDeque<Prospector>,
    pub cargo: Arc<Mutex<CargoReader>>
}

pub struct MiningMaterial {
    pub name: String,
    pub name_localised: String,
    pub proportion: f64,
}

pub struct Prospector {
    pub timestamp: String,
    pub event: String,
    pub materials: Vec<MiningMaterial>,
    pub content: String,
    pub content_localised: String,
    pub remaining: f64,
}

impl App for Mining {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let Self {
            prospectors,cargo
        } = self;

        egui::SidePanel::left("prospect_data")
            .show(ctx, |ui| {
                ui.heading("Prospectors");
                ui.separator();
                egui::ScrollArea::vertical()
                    .stick_to_right(true)
                    .show(ui, |ui| {
                        for i in 0..=5 {
                            let some_prospector = prospectors.get(i);
                            match some_prospector {
                                None => {}
                                Some(prospector) => {
                                    egui::Grid::new(i)
                                        .num_columns(2)
                                        .striped(true)
                                        .spacing([10.0, 5.0])
                                        .min_col_width(200.0)
                                        .show(ui, |ui| {
                                            if prospector.content_localised == "null" {
                                                ui.label(&prospector.content);
                                            } else {
                                                ui.label(&prospector.content_localised);
                                            }
                                            ui.end_row();
                                            ui.label("Remaining: ");
                                            ui.label(prospector.remaining.to_string());
                                            ui.end_row();
                                        });

                                    let len = prospector.materials.len().clone();
                                    egui::Grid::new(&prospector.timestamp)
                                        .num_columns(len)
                                        .striped(true)
                                        .spacing([10.0, 5.0])
                                        .min_col_width(200.0)
                                        .show(ui, |ui| {
                                            for material in &prospector.materials {
                                                let mut label : String = String::new();
                                                if material.name_localised == "null" {
                                                    label.push_str(&material.name);
                                                } else {
                                                    label.push_str(&material.name_localised);
                                                }
                                                label.push_str("\n");
                                                label.push_str(material.proportion.to_string().as_str());
                                                ui.label(label);
                                            }
                                        });
                                    ui.end_row();
                                    ui.separator();
                                    ui.end_row();
                                }
                            }
                        }
                    });
            });

        egui::SidePanel::left("cargo_data").show(ctx, |ui| {
            egui::Grid::new("page_grid")
                .num_columns(2)
                .striped(true)
                .min_col_width(100.0)
                .max_col_width(300.0)
                .show(ui,|ui|{
                    for cargo in &self.cargo.lock().unwrap().inventory{
                        if cargo.name_localised != "null"{
                            ui.label(&cargo.name_localised);
                        }else {
                            ui.label(&cargo.name);
                        }
                        ui.label(cargo.count.to_string());
                        ui.end_row();

                    }
                });
        });
    }
}