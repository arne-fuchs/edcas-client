use eframe::{App, egui, Frame};
use eframe::egui::{Context, Ui};

pub struct InventoryState {
    pub raw: Vec<Material>,
    pub manufactured: Vec<Material>,
    pub encoded: Vec<Material>,
    pub cargo: Vec<Cargo>,
    pub refinery: Vec<Refinery>,
}

pub struct Cargo {}

pub struct Refinery {}

pub struct Material {
    pub name: String,
    pub name_localised: String,
    pub count: u64,
}

impl App for InventoryState {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let Self {
            raw, manufactured, encoded, cargo, refinery
        } = self;

        egui::CentralPanel::default()
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui,|ui| {
                    egui::Grid::new("inventory_grid")
                        .num_columns(6)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Raw");
                            ui.label("Count");
                            ui.label("Encoded");
                            ui.label("Count");
                            ui.label("Manufactured");
                            ui.label("Count");
                            ui.end_row();
                            let lenghts = vec![encoded.len(), manufactured.len(), raw.len()];
                            let largest_index = lenghts.iter().max().unwrap().to_owned();
                            let mut index: usize = 0;
                            while index < largest_index {
                                draw_materials(ui,raw,index);
                                draw_materials(ui,encoded,index);
                                draw_materials(ui,manufactured,index);
                                ui.end_row();
                                index = index+1;
                            }
                        });
                });
            });
    }
}

fn draw_materials(ui: &mut Ui, materials: &mut Vec<Material>, index: usize){
    if index < materials.len() {
        let name = materials[index].name.to_string();
        let name_localized = materials[index].name_localised.to_string();
        let count = materials[index].count.to_string();
        if name_localized != "null" {
            ui.label(name_localized);
        }else {
            ui.label(name);
        }
        ui.label(count);
    }else {
        //Print 2 labels so following data wont get in the wrong table
        ui.label("");
        ui.label("");
    }
}
