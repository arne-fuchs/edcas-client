use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use eframe::{App, egui, Frame};
use eframe::egui::Context;
use eframe::egui::WidgetType::CollapsingHeader;
use log::{debug, error};
use num_format::{Locale, ToFormattedString};
use serde_json::Value;
use crate::app::cargo_reader::{Cargo, CargoReader};

pub struct Mining {
    pub prospectors: VecDeque<Prospector>,
    pub cargo: Arc<Mutex<CargoReader>>
}

pub struct MiningMaterial {
    pub name: String,
    pub name_localised: String,
    pub proportion: f64,
    pub buy_price: f64
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
                                            ui.label(format!("{}%",prospector.remaining));
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
                                                label.push_str(format!("{}%",material.proportion as u64).as_str());
                                                label.push_str("\n");
                                                label.push_str(format!("{} Credits/Unit",material.buy_price as u64).as_str());
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


        egui::CentralPanel::default().show(ctx, |ui| {
            egui::CollapsingHeader::new("Cargo")
                .default_open(true)
                .show(ui, |ui| {
                    for cargo in &self.cargo.lock().unwrap().inventory{
                        let mut name = &cargo.name;
                        if cargo.name_localised != "null"{
                            name = &cargo.name_localised;
                        }
                        egui::CollapsingHeader::new(name)
                            .default_open(true)
                            .show(ui, |ui| {
                                egui::Grid::new("page_grid")
                                    .num_columns(2)
                                    .min_col_width(100.0)
                                    .max_col_width(300.0)
                                    .show(ui,|ui|{
                                        ui.label("Count: ");
                                        ui.label((cargo.count as u64).to_string());
                                        ui.end_row();
                                        if cargo.buy_price as u64 > 0 {
                                            ui.label("Avg. buy price:");
                                            ui.label(format!("{} Credits", (cargo.buy_price as u64).to_formatted_string(&Locale::en)));
                                            ui.end_row();
                                        }
                                    });
                            });
                        ui.end_row();
                    }
                });

        });
    }
}