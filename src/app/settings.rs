mod presets;

use std::default::Default;
use std::{env, fs};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use eframe::{App, egui, Frame};
use eframe::egui::{Color32, Context, TextBuffer, vec2, Window};
use eframe::egui::scroll_area::ScrollBarVisibility::AlwaysVisible;
use eframe::egui::TextStyle::Monospace;
use iota_wallet::iota_client::Client;
use serde_json::json;

use crate::egui::global_dark_light_mode_switch;

#[derive(Clone)]
pub struct Settings {
    pub node: Client,
    pub journal_directory: String,
    pub graphics_directory: String,
    pub base_url: String,
    pub port: u64,
    pub n_timeout: u64,
    pub n_attempts: u64,
    pub faucet_url: String,
    pub log_level: String,
    pub local_pow: bool,
    pub password: String,
    pub allow_share_data: bool,
    pub graphic_override_content: String,
    pub show_editor: bool,
}

impl Default for Settings {
    fn default() -> Self {
        let settings_file_result = File::open("settings.json");
        let mut settings_file = match settings_file_result {
            Ok(settings_file) => { settings_file }
            Err(err) => {
                println!("Couldn't find settings file: {}\n Trying to copy example file...", err);
                fs::copy("settings-example.json", "settings.json").unwrap();
                File::open("settings.json").unwrap()
            }
        };
        let mut json_string: String = String::from("");
        settings_file.read_to_string(&mut json_string).unwrap();
        let json = json::parse(&json_string).unwrap();

        let mut journal_directory = json["reader"]["journal-directory"].to_string();
        let journal_path = Path::new(&journal_directory);
        if !journal_path.exists() {
            if cfg!(target_os = "windows") {
                let mut userprofile = env::var("USERPROFILE").unwrap_or("".to_string());
                userprofile.push_str("\\Saved Games\\Frontier Developments\\Elite Dangerous");
                journal_directory = userprofile;
            } else if cfg!(target_os = "linux") {
                let mut home = env::var("HOME").unwrap_or("~".to_string());
                home.push_str("/.steam/steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous");
                journal_directory = home;
            }
            if !Path::new(&journal_directory).exists() {
                journal_directory = String::from(".");
            }
            println!("Journal logs: {}", &journal_directory);
        }

        let mut graphics_directory = json["reader"]["graphics-directory"].to_string();
        let mut graphics_path = Path::new(&graphics_directory);
        let mut graphics_override_file = graphics_directory.clone();
        if !graphics_path.exists() {
            if cfg!(target_os = "windows") {
                let mut userprofile = env::var("USERPROFILE").unwrap_or("".to_string());
                userprofile.push_str("\\AppData\\Local\\Frontier Developments\\Elite Dangerous\\Options\\Graphics");
                graphics_directory = userprofile;
                graphics_override_file = format!("{}\\GraphicsConfigurationOverride.xml", graphics_directory);
            } else if cfg!(target_os = "linux") {
                let mut home = env::var("HOME").unwrap_or("~".to_string());
                home.push_str("/.steam/root/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/AppData/Local/Frontier Developments/Elite Dangerous/Options/Graphics");
                graphics_directory = home;
                graphics_override_file = format!("{}/GraphicsConfigurationOverride.xml", graphics_directory);
            }
            if !Path::new(&graphics_directory).exists() {
                graphics_directory = String::from(".");
            }
            println!("Graphics path: {}", &graphics_directory);
        }

        let mut node_url = json["node"]["base-url"].to_string();
        node_url.push_str(":");
        node_url.push_str(json["node"]["port"].to_string().as_str());
        Self {
            node: Client::builder()
                .with_node(node_url.as_str()).unwrap()
                .with_local_pow(json["local-pow"].as_bool().unwrap())
                .finish().unwrap(),
            journal_directory: journal_directory.clone(),
            graphics_directory: graphics_directory.clone(),
            base_url: json["node"]["base-url"].to_string(),
            port: json["node"]["port"].as_u64().unwrap(),
            n_timeout: json["nft-adapter"]["timeout"].as_u64().unwrap(),
            n_attempts: json["nft-adapter"]["attempts"].as_u64().unwrap(),
            faucet_url: json["faucet-url"].to_string(),
            log_level: json["log-level"].to_string(),
            local_pow: json["local-pow"].as_bool().unwrap(),
            password: json["password"].to_string(),
            allow_share_data: json["allow-share-data"].as_bool().unwrap(),
            graphic_override_content: fs::read_to_string(graphics_override_file).unwrap_or_default(),
            show_editor: false,
        }
    }
}

impl App for Settings {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        let Self {
            ..
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([60.0, 5.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.heading("Settings");
                    ui.end_row();

                    ui.label("Journal File Settings");
                    ui.end_row();

                    if Path::new(&self.journal_directory).exists() {
                        ui.label("Journal Directory:");
                    } else {
                        ui.label("Journal Directory: ⚠ Path invalid");
                    }
                    ui.text_edit_singleline(&mut self.journal_directory);
                    if ui.button("Open").clicked(){
                        opener::open(&self.journal_directory).unwrap();
                    }
                    ui.end_row();
                    ui.end_row();
                    ui.label("Graphics Override");
                    if ui.button("Open Editor").clicked(){
                        self.show_editor = !self.show_editor;
                    }
                    ui.end_row();
                    if Path::new(&self.graphics_directory).exists() {
                        ui.label("Graphics Directory:");
                    } else {
                        ui.label("Graphics Directory: ⚠ Path invalid");
                    }
                    ui.text_edit_singleline(&mut self.graphics_directory);
                    if ui.button("Open").clicked(){
                        opener::open(&self.graphics_directory).unwrap();
                    }
                    ui.end_row();
                    ui.end_row();
                    if self.show_editor {
                        Window::new("Editor")
                            .fixed_size(vec2(800f32,600f32))
                            .show(&ctx, |ui| {
                                egui::Grid::new("preset_buttons")
                                    .show(ui, |ui| {
                                        ui.hyperlink_to(
                                            "Fandom Article",
                                            "https://elite-dangerous.fandom.com/wiki/Graphics_Mods"
                                        );
                                        egui::ComboBox::from_id_source("Presets_Combo_Box")
                                            .selected_text("Presets")
                                            .show_ui(ui, |ui|{
                                                ui.style_mut().wrap = Some(false);
                                                ui.set_min_width(60.0);
                                                ui.selectable_value(&mut self.graphic_override_content,presets::get_increase_texture_resolution_preset(),"Increased Textures");
                                                ui.selectable_value(&mut self.graphic_override_content,presets::get_increased_star_count_preset(),"Increased Star Count");
                                                ui.selectable_value(&mut self.graphic_override_content,presets::get_better_skybox_preset(),"Better Skybox");
                                                ui.selectable_value(&mut self.graphic_override_content,presets::get_8gb_plus_preset(),"8Gb+ VRAM");
                                            });
                                        if ui.button("Load custom preset").clicked(){
                                            self.graphic_override_content = fs::read_to_string("custom_graphics_override.xml").unwrap();
                                        }
                                        if ui.button("Save as custom preset").clicked(){
                                            fs::write("custom_graphics_override.xml",self.graphic_override_content.clone()).unwrap();
                                        }
                                    });
                                ui.end_row();
                                egui::ScrollArea::vertical()
                                    .scroll_bar_visibility(AlwaysVisible)
                                    .show(ui, |ui| {
                                        ui.add(
                                            egui::TextEdit::multiline(&mut self.graphic_override_content)
                                                .font(egui::TextStyle::Monospace) // for cursor height
                                                .code_editor()
                                                .desired_rows(10)
                                                .text_color(Color32::from_rgb(255,165,0))
                                                .font(egui::FontId::monospace(10.0))
                                                .lock_focus(true)
                                                .desired_width(f32::INFINITY)
                                        );
                                    });
                                egui::Grid::new("editor_buttons")
                                    .show(ui, |ui| {
                                        if ui.button("Save").clicked() {
                                            fs::write(format!("{}/GraphicsConfigurationOverride.xml", self.graphics_directory.clone()),self.graphic_override_content.clone()).unwrap();
                                        }
                                        if ui.button("Close").clicked() {
                                            self.show_editor = false;
                                        }
                                        if ui.button("Reset").clicked() {
                                            self.graphic_override_content = fs::read_to_string(format!("{}/GraphicsConfigurationOverride.xml", self.graphics_directory.clone())).unwrap();
                                        }
                                        if ui.button("Defaults").clicked() {
                                            self.graphic_override_content = "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<GraphicsConfig />\n".into();
                                        }
                                    });
                            });
                    }

                    ui.end_row();

                    ui.label("Connection Settings for the EDCAS Network");
                    ui.end_row();
                    ui.label("Allow to share journal log data:");
                    ui.checkbox(&mut self.allow_share_data, "");
                    ui.end_row();
                    ui.label("Node Url:");
                    ui.text_edit_singleline(&mut self.base_url);
                    ui.end_row();

                    ui.label("Port:");
                    ui.text_edit_singleline(&mut self.port.to_string());
                    ui.end_row();

                    ui.label("Nft Adapter Timeout:");
                    ui.add(egui::Slider::new(&mut self.n_timeout, 0..=20).suffix(" Seconds"));
                    ui.end_row();

                    ui.label("Nft Adapter Attempts:");
                    ui.add(egui::Slider::new(&mut self.n_attempts, 0..=20).suffix(" Attempts"));
                    ui.end_row();
                    ui.end_row();
                    ui.label("Requires restart to apply settings")
                });


            //Apply Button
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                if ui.button("Save").clicked() {
                    let json = json!(
                        {
                            "reader": {
                                "journal-directory": self.journal_directory,
                                "graphics-directory": self.graphics_directory,
                            },
                            "node": {
                                "base-url": self.base_url,
                                "port": self.port
                            },
                            "nft-adapter": {
                                "timeout": self.n_timeout,
                                "attempts": self.n_attempts
                            },
                            "local-pow": false,
                            "log-level": "Debug",
                            "password": self.password,
                            "allow-share-data": self.allow_share_data
                        }
                    );
                    let mut settings_file: File = File::create("settings.json").unwrap();
                    settings_file.write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes()).unwrap();
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                global_dark_light_mode_switch(ui);
            });
        });
    }
}