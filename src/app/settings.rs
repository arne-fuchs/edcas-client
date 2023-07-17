mod presets;

use std::default::Default;
use std::{env, fs};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;
use eframe::{App, egui, Frame};
use eframe::egui::{Color32, Context, TextureHandle, vec2, Window};
use eframe::egui::scroll_area::ScrollBarVisibility::AlwaysVisible;
use iota_wallet::iota_client::Client;
use serde_json::json;

use crate::app::settings::ActionAtShutdownSignal::{Exit, Continue, Nothing};

use crate::egui::global_dark_light_mode_switch;
use crate::ICON_SYMBOL;

#[derive(Clone)]
pub struct AppearanceSettings {
    pub font_size: f32,
    pub font_style: String,
    pub font_id: egui::FontId,
    pub applied: bool,
}

#[derive(Clone)]
pub struct JournalReaderSettings{
    pub journal_directory: String,
    pub action_at_shutdown_signal: ActionAtShutdownSignal
}
#[derive(Clone)]
pub enum ActionAtShutdownSignal {
    Exit,Nothing,Continue
}

impl FromStr for ActionAtShutdownSignal{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Exit" => Ok(Exit),
            "nothing" => Ok(Nothing),
            "continue" => Ok(Continue),
            _ => Err("Failed to parse ActionShutdownSignal".to_string())
        }
    }
}

impl ToString for ActionAtShutdownSignal {
    fn to_string(&self) -> String {
        match self {
            Exit => "Exit".to_string(),
            Nothing => "nothing".to_string(),
            Continue => "continue".to_string()
        }
    }
}

impl PartialEq for ActionAtShutdownSignal {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

#[derive(Clone)]
pub struct ExplorerSettings{
    pub include_system_name: bool,
    pub show_gravity: bool,
    pub show_bio: bool,
    pub show_geo: bool,
    pub show_human: bool,
    pub show_xeno: bool,
    pub show_unknown: bool,
    pub show_ls: bool,
    pub show_sphere: bool,
    pub show_landable: bool,
    pub show_discovered: bool,
    pub show_mapped: bool,
}
#[derive(Clone)]
pub struct IotaSettings {
    pub node: Option<Client>,
    pub base_url: String,
    pub port: u64,
    pub n_timeout: u64,
    pub n_attempts: u64,
    pub faucet_url: String,
    pub local_pow: bool,
    pub password: String,
    pub allow_share_data: bool,
}
#[derive(Clone)]
pub struct GraphicEditorSettings{
    pub graphics_directory: String,
    pub graphic_override_content: String,
    pub show_editor: bool,
}

#[derive(Clone)]
pub struct Settings {
    pub appearance_settings: AppearanceSettings,
    pub journal_reader_settings: JournalReaderSettings,
    pub explorer_settings: ExplorerSettings,
    pub iota_settings: IotaSettings,
    pub graphic_editor_settings: GraphicEditorSettings,
    pub log_level: String,
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

        let mut journal_directory = json["journal-reader"]["journal-directory"].to_string();
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

        let mut graphics_directory = json["journal-reader"]["graphics-directory"].to_string();
        let graphics_path = Path::new(&graphics_directory);
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
                graphics_directory = String::from("");
            }
            println!("Graphics path: {}", &graphics_directory);
        }
        let mut some_client = None;
        if json["iota"]["allow-share-data"].as_bool().unwrap_or(false) {
            let mut node_url = json["iota"]["base-url"].as_str().unwrap_or("https://tangle.paesserver.de").to_string();
            node_url.push_str(":");
            node_url.push_str(json["iota"]["port"].as_str().unwrap_or("443"));
            some_client = Some(Client::builder()
                .with_node(node_url.as_str()).unwrap()
                .with_local_pow(json["local-pow"].as_bool().unwrap())
                .finish().unwrap());

        }

        let font_size = json["appearance"]["font-size"].as_f32().unwrap_or(24.0);
        let mut font_id = egui::FontId::default();

        match json["appearance"]["font-family"].as_str().unwrap_or("Proportional") {
            "Monospace" => {
                println!("Mono");
                font_id = egui::FontId::monospace(font_size);
            }
            "Proportional" | _ => {
                println!("Prop");
                font_id = egui::FontId::proportional(font_size);
            }
        }

        Self {
            appearance_settings: AppearanceSettings{
                font_size: json["appearance"]["font-size"].as_f32().unwrap_or(24.0),
                font_style: json["appearance"]["font_style"].as_str().unwrap_or("Proportional").to_string(),
                font_id,
                applied: false
            },
            journal_reader_settings: JournalReaderSettings{
                journal_directory,
                action_at_shutdown_signal: ActionAtShutdownSignal::from_str(json["journal-reader"]["action-at-shutdown-signal"].as_str().unwrap_or("Nothing")).unwrap_or(Nothing),
            },
            explorer_settings: ExplorerSettings {
                include_system_name: json["explorer"]["include_system_name"].as_bool().unwrap_or(true),
                show_gravity: json["explorer"]["show_gravity"].as_bool().unwrap_or(false),
                show_bio: json["explorer"]["show_bio"].as_bool().unwrap_or(true),
                show_geo: json["explorer"]["show_geo"].as_bool().unwrap_or(true),
                show_human: json["explorer"]["show_human"].as_bool().unwrap_or(true),
                show_xeno: json["explorer"]["show_xeno"].as_bool().unwrap_or(true),
                show_unknown: json["explorer"]["show_unknown"].as_bool().unwrap_or(true),
                show_ls: json["explorer"]["show_ls"].as_bool().unwrap_or(true),
                show_sphere: json["explorer"]["show_sphere"].as_bool().unwrap_or(true),
                show_landable: json["explorer"]["show_landable"].as_bool().unwrap_or(false),
                show_discovered: json["explorer"]["show_discovered"].as_bool().unwrap_or(true),
                show_mapped: json["explorer"]["show_mapped"].as_bool().unwrap_or(true),
            },
            iota_settings: IotaSettings {
                node: some_client,
                base_url: json["iota"]["base-url"].as_str().unwrap_or("https://tangle.paesserver.de").to_string(),
                port: json["iota"]["port"].as_u64().unwrap_or(443),
                n_timeout: json["iota"]["timeout"].as_u64().unwrap_or(5),
                n_attempts: json["iota"]["attempts"].as_u64().unwrap_or(4),
                faucet_url: json["iota"]["faucet-url"].as_str().unwrap_or("https://faucet.paesserver.de").to_string(),
                local_pow: json["iota"]["local-pow"].as_bool().unwrap_or(false),
                password: json["iota"]["password"].as_str().unwrap_or("CoUBZ9W6eRVpTKEYrgj3").to_string(),
                allow_share_data: json["allow-share-data"].as_bool().unwrap_or(false),
            },
            graphic_editor_settings: GraphicEditorSettings{
                graphics_directory: graphics_directory.clone(),
                graphic_override_content: fs::read_to_string(graphics_override_file).unwrap_or_default(),
                show_editor: false,
            },
            log_level: json["log-level"].to_string(),
        }
    }
}

impl App for Settings {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let Self {
            ..
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui,|ui|{
                egui::Grid::new("my_grid")
                    .num_columns(2)
                    .spacing([60.0, 5.0])
                    .min_col_width(300.0)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.heading("Settings");
                        ui.end_row();
                        ui.heading("Appearance");
                        ui.end_row();
                        ui.end_row();
                        ui.label("Font-style:");
                        egui::introspection::font_id_ui(ui, &mut self.appearance_settings.font_id);
                        ui.end_row();
                        if ui.button("Apply").clicked() {
                            self.appearance_settings.applied = false;
                        }
                        ui.end_row();
                        ui.separator();
                        ui.end_row();

                        ui.label("Journal File Settings");
                        ui.end_row();

                        if Path::new(&self.journal_reader_settings.journal_directory).exists() {
                            ui.label("Journal Directory:");
                        } else {
                            ui.label("Journal Directory: ⚠ Path invalid");
                        }
                        ui.text_edit_singleline(&mut self.journal_reader_settings.journal_directory);
                        if ui.button("Open").clicked(){
                            opener::open(&self.journal_reader_settings.journal_directory).unwrap();
                        }
                        ui.end_row();
                        ui.label("Action after reaching shutdown:");
                        egui::ComboBox::from_label("")
                            .selected_text(self.journal_reader_settings.action_at_shutdown_signal.to_string())
                            .show_ui(ui,|ui|{
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(60.0);
                                ui.selectable_value(&mut self.journal_reader_settings.action_at_shutdown_signal,Exit,"exit");
                                ui.selectable_value(&mut self.journal_reader_settings.action_at_shutdown_signal,Continue,"continue");
                                ui.selectable_value(&mut self.journal_reader_settings.action_at_shutdown_signal,Nothing,"nothing");
                            });
                        ui.end_row();
                        ui.separator();
                        ui.end_row();
                        ui.heading("Explorer");
                        ui.end_row();
                        ui.label("Include system in body name:");
                        ui.checkbox(&mut self.explorer_settings.include_system_name,"");
                        let multiplikator = 24.0;
                        ui.end_row();
                        ui.horizontal(|ui|{
                            ui.label("⬇");
                            ui.checkbox(&mut self.explorer_settings.show_gravity,"Gravity");
                        });
                        ui.horizontal(|ui|{
                            ui.label("🌱");
                            ui.checkbox(&mut self.explorer_settings.show_bio,"Bio");
                        });
                        ui.end_row();
                        ui.horizontal(|ui|{
                            ui.label("🌋");
                            ui.checkbox(&mut self.explorer_settings.show_geo,"Geo");
                        });
                        ui.horizontal(|ui|{
                            ui.label("✋");
                            ui.checkbox(&mut self.explorer_settings.show_human,"Human");
                        });
                        ui.end_row();
                        ui.horizontal(|ui|{
                            ui.label("👽");
                            ui.checkbox(&mut self.explorer_settings.show_xeno,"Xeno");
                        });
                        ui.horizontal(|ui|{
                            ui.label("？");
                            ui.checkbox(&mut self.explorer_settings.show_unknown,"Unknown");
                        });
                        ui.end_row();
                        ui.horizontal(|ui|{
                            ui.label("➡");
                            ui.checkbox(&mut self.explorer_settings.show_ls,"Distance from Main Star");
                        });

                        ui.end_row();
                        ui.horizontal(|ui|{
                            let texture: TextureHandle = ui.ctx().load_texture(
                                "tree-view-landable-sphere_icon",
                                ICON_SYMBOL.lock().unwrap().landable_sphere.clone(),
                                egui::TextureOptions::LINEAR,
                            );
                            let img_size = multiplikator * texture.size_vec2() / texture.size_vec2().y;
                            ui.image(&texture,img_size);
                            ui.checkbox(&mut self.explorer_settings.show_sphere,"Sphere (Landable)");
                        });
                        ui.horizontal(|ui|{
                            let texture: TextureHandle = ui.ctx().load_texture(
                                "tree-view-landable-landable_icon",
                                ICON_SYMBOL.lock().unwrap().landable.clone(),
                                egui::TextureOptions::LINEAR,
                            );
                            let img_size = multiplikator * texture.size_vec2() / texture.size_vec2().y;
                            ui.image(&texture,img_size);
                            let texture: TextureHandle = ui.ctx().load_texture(
                                "tree-view-landable-not-landable_icon",
                                ICON_SYMBOL.lock().unwrap().not_landable.clone(),
                                egui::TextureOptions::LINEAR,
                            );
                            ui.image(&texture,img_size);
                            ui.checkbox(&mut self.explorer_settings.show_landable,"Icon (Landable)");
                        });


                        ui.end_row();
                        ui.horizontal(|ui|{
                            ui.label("🚩 ");
                            ui.checkbox(&mut self.explorer_settings.show_discovered,"Discovered");
                        });
                        ui.horizontal(|ui|{
                            ui.label("🗺");
                            ui.checkbox(&mut self.explorer_settings.show_mapped,"Mapped");
                        });

                        ui.end_row();
                        ui.separator();
                        ui.end_row();
                        ui.heading("Graphics Override");
                        if ui.button("Open Editor").clicked(){
                            self.graphic_editor_settings.show_editor = !self.graphic_editor_settings.show_editor;
                        }
                        ui.end_row();
                        if Path::new(&self.graphic_editor_settings.graphics_directory).exists() {
                            ui.label("Graphics Directory:");
                        } else {
                            ui.label("Graphics Directory: ⚠ Path invalid");
                        }
                        ui.text_edit_singleline(&mut self.graphic_editor_settings.graphics_directory);
                        if ui.button("Open").clicked(){
                            opener::open(&self.graphic_editor_settings.graphics_directory).unwrap();
                        }
                        ui.end_row();
                        ui.separator();
                        ui.end_row();
                        if self.graphic_editor_settings.show_editor {
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
                                                    ui.selectable_value(&mut self.graphic_editor_settings.graphic_override_content,presets::get_increase_texture_resolution_preset(),"Increased Textures");
                                                    ui.selectable_value(&mut self.graphic_editor_settings.graphic_override_content,presets::get_increased_star_count_preset(),"Increased Star Count");
                                                    ui.selectable_value(&mut self.graphic_editor_settings.graphic_override_content,presets::get_better_skybox_preset(),"Better Skybox");
                                                    ui.selectable_value(&mut self.graphic_editor_settings.graphic_override_content,presets::get_8gb_plus_preset(),"8Gb+ VRAM");
                                                });
                                            if ui.button("Load custom preset").clicked(){
                                                self.graphic_editor_settings.graphic_override_content = fs::read_to_string("custom_graphics_override.xml").unwrap();
                                            }
                                            if ui.button("Save as custom preset").clicked(){
                                                fs::write("custom_graphics_override.xml",self.graphic_editor_settings.graphic_override_content.clone()).unwrap();
                                            }
                                        });
                                    ui.end_row();
                                    egui::ScrollArea::vertical()
                                        .scroll_bar_visibility(AlwaysVisible)
                                        .show(ui, |ui| {
                                            ui.add(
                                                egui::TextEdit::multiline(&mut self.graphic_editor_settings.graphic_override_content)
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
                                                fs::write(format!("{}/GraphicsConfigurationOverride.xml", self.graphic_editor_settings.graphics_directory.clone()),self.graphic_editor_settings.graphic_override_content.clone()).unwrap();
                                            }
                                            if ui.button("Exit").clicked() {
                                                self.graphic_editor_settings.show_editor = false;
                                            }
                                            if ui.button("Reset").clicked() {
                                                self.graphic_editor_settings.graphic_override_content = fs::read_to_string(format!("{}/GraphicsConfigurationOverride.xml", self.graphic_editor_settings.graphics_directory.clone())).unwrap();
                                            }
                                            if ui.button("Defaults").clicked() {
                                                self.graphic_editor_settings.graphic_override_content = "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n<GraphicsConfig />\n".into();
                                            }
                                        });
                                });
                        }

                        ui.end_row();

                        ui.heading("EDCAS Network");
                        ui.end_row();
                        ui.label("Allow to share journal log data:");
                        ui.checkbox(&mut self.iota_settings.allow_share_data, "");
                        ui.end_row();
                        ui.label("Node Url:");
                        ui.text_edit_singleline(&mut self.iota_settings.base_url);
                        ui.end_row();
                        ui.label("Port:");
                        ui.text_edit_singleline(&mut self.iota_settings.port.to_string());
                        ui.end_row();
                        ui.label("Faucet Url:");
                        ui.text_edit_singleline(&mut self.iota_settings.faucet_url);
                        ui.end_row();
                        ui.label("Nft Adapter Timeout:");
                        ui.add(egui::Slider::new(&mut self.iota_settings.n_timeout, 0..=20).suffix(" Seconds"));
                        ui.end_row();
                        ui.label("Nft Adapter Attempts:");
                        ui.add(egui::Slider::new(&mut self.iota_settings.n_attempts, 0..=20).suffix(" Attempts"));
                        ui.end_row();
                        ui.separator();
                        ui.end_row();
                        ui.label("Requires restart to apply settings");
                        ui.end_row();
                        ui.separator();
                        ui.end_row();
                    });
            });

            //Apply Button
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                if ui.button("Save 💾").clicked() {
                    let json = json!(
                        {
                            "log-level": "Debug",
                            "appearance": {
                                "font-size": self.appearance_settings.font_size,
                                "font-family": self.appearance_settings.font_id.family.to_string(),
                            },
                            "journal-reader": {
                                "directory": self.journal_reader_settings.journal_directory,
                                "action-at-shutdown-signal": self.journal_reader_settings.action_at_shutdown_signal.to_string()
                            },
                            "explorer": {
                                "include_system_name": self.explorer_settings.include_system_name,
                                "show_gravity": self.explorer_settings.show_gravity,
                                "show_bio": self.explorer_settings.show_bio,
                                "show_geo": self.explorer_settings.show_geo,
                                "show_human": self.explorer_settings.show_human,
                                "show_xeno": self.explorer_settings.show_xeno,
                                "show_unknown": self.explorer_settings.show_unknown,
                                "show_landable": self.explorer_settings.show_landable,
                                "show_ls": self.explorer_settings.show_ls,
                                "show_sphere": self.explorer_settings.show_sphere,
                                "show_discovered": self.explorer_settings.show_discovered,
                                "show_mapped": self.explorer_settings.show_mapped
                            },
                            "iota": {
                                "base-url": self.iota_settings.base_url,
                                "port": self.iota_settings.port,
                                "timeout": self.iota_settings.n_timeout,
                                "attempts": self.iota_settings.n_attempts,
                                "local-pow": self.iota_settings.local_pow,
                                "password": self.iota_settings.password,
                                "allow-share-data": self.iota_settings.allow_share_data
                            },
                            "graphics-editor": {
                                "graphics-directory": self.graphic_editor_settings.graphics_directory
                            }
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