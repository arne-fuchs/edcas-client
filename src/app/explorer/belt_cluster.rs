use std::sync::Arc;
use eframe::egui;
use eframe::egui::{TextureHandle, Ui};
use crate::app::explorer::structs::{BodyImplementation, Parent};
use crate::app::settings::Settings;
use crate::ICON_BODY;

pub struct BeltCluster {
    pub timestamp: String,
    pub event: String,
    pub scan_type: String,
    pub body_name: String,
    pub body_id: i64,
    pub parents: Vec<Parent>,
    pub star_system: String,
    pub system_address: i64,
    pub distance_from_arrival_ls: f64,
    pub was_discovered: bool,
    pub was_mapped: bool,
    pub settings: Arc<Settings>,
}

impl BodyImplementation for BeltCluster {
    fn print_side_panel_information(&self, ui: &mut Ui) {
        ui.heading(&self.body_name);
        ui.end_row();
        ui.label("Discovered");
        ui.label(&self.was_discovered.to_string());
        ui.end_row();
        ui.label("Distance in LS");
        ui.label(&self.distance_from_arrival_ls.to_string());
    }

    fn print_header_content(&self, ui: &mut Ui, system_index: &mut usize, body_index: usize){
        let texture: TextureHandle = ui.ctx().load_texture(
            "parentless-body-icon",
            ICON_BODY.lock().unwrap().belt_cluster.clone(),
            egui::TextureOptions::LINEAR,
        );

        let img_size = 32.0 * texture.size_vec2() / texture.size_vec2().y;
        ui.image(&texture, img_size);
        let mut body_name = self.body_name.to_string();
        if !self.settings.explorer_settings.include_system_name{
            let system_name = self.star_system.clone();
            body_name.replace_range(0..system_name.len(),"");
        }

        if self.was_discovered && self.settings.explorer_settings.show_discovered{
            body_name.push_str("|🚩");
        }
        if self.was_mapped && self.settings.explorer_settings.show_mapped{
            body_name.push_str("|🗺");
        }

        if ui.selectable_label(false, &body_name).clicked() {
            *system_index = body_index;
        };

    }

    fn get_name(&self) -> &str {
        self.body_name.as_str()
    }

    fn get_id(&self) -> i64 {
        self.body_id
    }

    fn get_parents(&self) -> Vec<Parent> {
        self.parents.clone()
    }
}