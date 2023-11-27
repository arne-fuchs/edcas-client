use std::{env, fs};
use std::default::Default;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;

use eframe::{App, egui, Frame};
use eframe::egui::{Color32, Context, RichText, vec2, Window};
use eframe::egui::scroll_area::ScrollBarVisibility::AlwaysVisible;
use eframe::epaint::ahash::HashMap;
#[cfg(feature = "iota")]
use iota_sdk::client::Client;
use log::{error, info, warn};
use serde_json::json;

use crate::app::settings::ActionAtShutdownSignal::{Continue, Exit, Nothing};
use crate::egui::global_dark_light_mode_switch;

mod presets;

#[derive(Clone)]
pub struct Icon {
    pub name: String,
    pub char: String,
    pub color: Color32,
    pub enabled: bool,
}

impl Icon {
    pub fn get_richtext(&self) -> RichText {
        RichText::new(&self.char).color(self.color)
    }
}

#[derive(Clone)]
pub struct AppearanceSettings {
    pub font_size: f32,
    pub font_style: String,
    pub font_id: egui::FontId,
    pub applied: bool,
}

#[derive(Clone)]
pub struct JournalReaderSettings {
    pub journal_directory: String,
    pub action_at_shutdown_signal: ActionAtShutdownSignal,
}

#[derive(Clone)]
pub enum ActionAtShutdownSignal {
    Exit,
    Nothing,
    Continue,
}

impl FromStr for ActionAtShutdownSignal {
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
pub struct ExplorerSettings {
    pub include_system_name: bool,
}

#[derive(Clone)]
pub struct IotaSettings {
    #[cfg(feature = "iota")]
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
pub struct GraphicEditorSettings {
    pub graphics_directory: String,
    pub graphic_override_content: String,
    pub show_editor: bool,
}

//TODO Add odyssey? option and make api calls depend on it
#[derive(Clone)]
pub struct Settings {
    pub appearance_settings: AppearanceSettings,
    pub journal_reader_settings: JournalReaderSettings,
    pub explorer_settings: ExplorerSettings,
    pub iota_settings: IotaSettings,
    pub graphic_editor_settings: GraphicEditorSettings,
    pub icons: HashMap<String, Icon>,
    pub stars: HashMap<String, Icon>,
    pub planets: HashMap<String, Icon>,
    pub log_level: String,
    pub settings_path: String,
}

impl Default for Settings {
    fn default() -> Self {
        let mut settings_path = "settings.json".to_string();
        let mut settings_file = match env::var("HOME") {
            Ok(home) => {
                match File::open(format!("{}/.config/edcas-client/settings.json",home)) {
                    Ok(file) => {
                        settings_path = format!("{}/.config/edcas-client/settings.json",home);
                        file
                    }
                    Err(err) => {
                        warn!("{}",err);
                        info!("Couldn't find config file in {} -> trying to create file structure and copy to desired config folder",format!("{}/.config/edcas-client/settings.json",home));
                        match fs::create_dir_all(format!("{}/.config/edcas-client",home)) {
                            Ok(_) => {
                                info!("Created $HOME/.config/edcas-client/");
                                info!("Copying from /etc/edcas-client/settings-example.json to $HOME/.config/edcas-client/settings.json");
                                match fs::copy("/etc/edcas-client/settings-example.json", format!("{}/.config/edcas-client/settings.json",home)) {
                                    Ok(_) => {
                                        info!("Copied /etc/edcas-client/settings-example.json to {}",format!("{}/.config/edcas-client/settings.json",home));
                                    }
                                    Err(err) => {
                                        info!("Failed copying from /etc/edcas-client/settings-example.json\n Trying to copy from settings-example.json to $HOME/.config/edcas-client/settings.json: {}",err);
                                        fs::copy("settings-example.json", format!("{}/.config/edcas-client/settings.json",home)).expect("Couldn't copy settings file to $HOME/.config/edcas-client/");
                                    }
                                }

                                info!("Accessing settings file at $HOME/.config/edcas-client/settings.json");
                                settings_path = format!("{}/.config/edcas-client/settings.json",home);
                                File::open(format!("{}/.config/edcas-client/settings.json",home)).expect("Couldn't open settings file in $HOME/.config/edcas-client/")
                            }
                            Err(err) => {
                                warn!("{}",err);
                                info!("Couldn't create directories in $HOME/.config/edcas-client/");
                                match File::open("settings.json"){
                                    Ok(file) => {
                                        info!("Accessing settings file at settings.json");
                                        file
                                    }
                                    Err(err) => {
                                        warn!("{}",err);
                                        info!("Copying from settings-example.json to settings.json");
                                        fs::copy("settings-example.json", "settings.json").expect("Couldn't copy settings-exmaple.json");
                                        info!("Accessing settings file at settings.json");
                                        File::open("settings.json").unwrap()
                                    }
                                }
                            }
                        }

                    }
                }
            }
            Err(err) => {
                warn!("{}",err);
                match File::open("settings.json"){
                    Ok(file) => {
                        info!("Accessing settings file at settings.json");
                        file
                    }
                    Err(err) => {
                        warn!("{}",err);
                        info!("Copying from /etc/edcas-client/settings-example.json to settings.json");
                        fs::copy("/etc/edcas-client/settings-example.json", "settings.json").unwrap_or(fs::copy("settings-example.json", "settings.json").expect("Couldn't copy settings file to $HOME/.config/edcas-client/"));
                        info!("Accessing settings file at settings.json");
                        File::open("settings.json").unwrap()
                    }
                }
            }
        };

        let mut json_string: String = String::from("");
        settings_file.read_to_string(&mut json_string).unwrap();
        let json = json::parse(&json_string).unwrap();

        //---------------------------
        // Journal reader
        //---------------------------

        let mut journal_directory = json["journal-reader"]["directory"].to_string();
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
                journal_directory = String::from("");
            }
            println!("Journal logs: {}", &journal_directory);
        }

        //---------------------------
        // Graphics directory
        //---------------------------
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

        //---------------------------
        // IOTA
        //---------------------------

        #[cfg(feature = "iota")]
        let some_client = {
            if json["iota"]["allow-share-data"].as_bool().unwrap_or(false) {
                let mut node_url = json["iota"]["base-url"].as_str().unwrap_or("https://tangle.paesserver.de").to_string();
                node_url.push(':');
                node_url.push_str(json["iota"]["port"].as_str().unwrap_or("443"));
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async {
                        Some(Client::builder()
                            .with_node(node_url.as_str()).unwrap()
                            .with_local_pow(json["iota"]["local-pow"].as_bool().unwrap_or(false))
                            .finish().await.unwrap())
                    })
            }else {
                None
            }
        };


        //---------------------------
        // Appearance
        //---------------------------

        let font_size = json["appearance"]["font-size"].as_f32().unwrap_or(24.0);
        let mut _font_id = egui::FontId::default();

        match json["appearance"]["font-family"].as_str().unwrap_or("Proportional") {
            "Monospace" => {
                _font_id = egui::FontId::monospace(font_size);
            }
            "Proportional" | _ => {
                _font_id = egui::FontId::proportional(font_size);
            }
        }

        //---------------------------
        // Icons
        //---------------------------
        let mut icons: HashMap<String, Icon> = HashMap::default();
        for i in 0..json["icons"].len() {
            let icon_json = &json["icons"][i];
            icons.insert(
                icon_json["name"].to_string(),
                Icon {
                    name: icon_json["name"].to_string(),
                    char: icon_json["char"].as_str().unwrap_or("⁉").to_string(),
                    color: Color32::from_rgb(icon_json["r"].as_u8().unwrap_or(255), icon_json["g"].as_u8().unwrap_or(255), icon_json["b"].as_u8().unwrap_or(255)),
                    enabled: icon_json["enabled"].as_bool().unwrap_or(true),
                },
            );
        }

        //---------------------------
        // Stars
        //---------------------------
        let mut stars: HashMap<String, Icon> = HashMap::default();
        for i in 0..json["stars"].len() {
            let stars_json = &json["stars"][i];
            stars.insert(
                stars_json["class"].to_string(),
                Icon {
                    name: stars_json["class"].to_string(),
                    char: stars_json["char"].as_str().unwrap_or("⁉").to_string(),
                    color: Color32::from_rgb(stars_json["r"].as_u8().unwrap_or(255), stars_json["g"].as_u8().unwrap_or(255), stars_json["b"].as_u8().unwrap_or(255)),
                    enabled: stars_json["enabled"].as_bool().unwrap_or(true),
                },
            );
        }

        //---------------------------
        // Planets
        //---------------------------
        let mut planets: HashMap<String, Icon> = HashMap::default();
        for i in 0..json["planets"].len() {
            let stars_json = &json["planets"][i];
            planets.insert(
                stars_json["class"].to_string(),
                Icon {
                    name: stars_json["class"].to_string(),
                    char: stars_json["char"].as_str().unwrap_or("⁉").to_string(),
                    color: Color32::from_rgb(stars_json["r"].as_u8().unwrap_or(255), stars_json["g"].as_u8().unwrap_or(255), stars_json["b"].as_u8().unwrap_or(255)),
                    enabled: stars_json["enabled"].as_bool().unwrap_or(true),
                },
            );
        }


        Self {
            appearance_settings: AppearanceSettings {
                font_size: json["appearance"]["font-size"].as_f32().unwrap_or(24.0),
                font_style: json["appearance"]["font_style"].as_str().unwrap_or("Proportional").to_string(),
                font_id: _font_id,
                applied: false,
            },
            journal_reader_settings: JournalReaderSettings {
                journal_directory,
                action_at_shutdown_signal: ActionAtShutdownSignal::from_str(json["journal-reader"]["action-at-shutdown-signal"].as_str().unwrap_or("Nothing")).unwrap_or(Nothing),
            },
            explorer_settings: ExplorerSettings {
                include_system_name: json["explorer"]["include_system_name"].as_bool().unwrap_or(true),
            },
            iota_settings: IotaSettings {
                #[cfg(feature = "iota")]
                node: some_client,
                base_url: json["iota"]["base-url"].as_str().unwrap_or("https://tangle.paesserver.de").to_string(),
                port: json["iota"]["port"].as_u64().unwrap_or(443),
                n_timeout: json["iota"]["timeout"].as_u64().unwrap_or(5),
                n_attempts: json["iota"]["attempts"].as_u64().unwrap_or(4),
                faucet_url: json["iota"]["faucet-url"].as_str().unwrap_or("https://faucet.paesserver.de").to_string(),
                local_pow: json["iota"]["local-pow"].as_bool().unwrap_or(false),
                password: json["iota"]["password"].as_str().unwrap_or("CoUBZ9W6eRVpTKEYrgj3").to_string(),
                allow_share_data: json["iota"]["allow-share-data"].as_bool().unwrap_or(false),
            },
            graphic_editor_settings: GraphicEditorSettings {
                graphics_directory: graphics_directory.clone(),
                graphic_override_content: fs::read_to_string(graphics_override_file).unwrap_or_default(),
                show_editor: false,
            },
            icons,
            stars,
            planets,
            log_level: json["log-level"].to_string(),
            settings_path,
        }
    }
}

impl App for Settings {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let Self {
            ..
        } = self;

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Appearance");
                egui::Grid::new("appearance_grid")
                    .num_columns(2)
                    .spacing([60.0, 5.0])
                    .min_col_width(300.0)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Font-style:");
                        egui::introspection::font_id_ui(ui, &mut self.appearance_settings.font_id);
                        ui.end_row();
                        if ui.button("Apply").clicked() {
                            self.appearance_settings.applied = false;
                        }
                    });
                ui.separator();

                ui.heading("Journal File Settings");
                egui::Grid::new("journal_grid")
                    .num_columns(2)
                    .spacing([60.0, 5.0])
                    .min_col_width(300.0)
                    .striped(true)
                    .show(ui, |ui| {
                        if Path::new(&self.journal_reader_settings.journal_directory).exists() {
                            ui.label("Journal Directory:");
                        } else {
                            ui.label("Journal Directory: ⚠ Path invalid");
                        }
                        ui.text_edit_singleline(&mut self.journal_reader_settings.journal_directory);
                        if ui.button("Open").clicked() {
                            opener::open(&self.journal_reader_settings.journal_directory).unwrap();
                        }
                        ui.end_row();
                        ui.label("Action after reaching shutdown:");
                        egui::ComboBox::from_label("")
                            .selected_text(self.journal_reader_settings.action_at_shutdown_signal.to_string())
                            .show_ui(ui, |ui| {
                                ui.style_mut().wrap = Some(false);
                                ui.set_min_width(60.0);
                                ui.selectable_value(&mut self.journal_reader_settings.action_at_shutdown_signal, Exit, "exit");
                                ui.selectable_value(&mut self.journal_reader_settings.action_at_shutdown_signal, Continue, "continue");
                                ui.selectable_value(&mut self.journal_reader_settings.action_at_shutdown_signal, Nothing, "nothing");
                            });
                    });
                ui.separator();

                egui::CollapsingHeader::new("Explorer").show(ui, |ui| {
                    ui.checkbox(&mut self.explorer_settings.include_system_name, "Include system in body name");
                    egui::CollapsingHeader::new("Icons").show(ui, |ui| {
                        egui::Grid::new("explorer_icon_grid")
                            .num_columns(2)
                            .spacing([60.0, 5.0])
                            .min_col_width(300.0)
                            .striped(true)
                            .show(ui, |ui| {

                                for icon in &mut self.icons {
                                    ui.horizontal(|ui| {
                                        ui.text_edit_singleline(&mut icon.1.char);
                                        ui.label(icon.0);
                                    });
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut icon.1.enabled, "Enabled");
                                        ui.color_edit_button_srgba(&mut icon.1.color);
                                        ui.label("Color");
                                    });
                                    ui.end_row();
                                }
                            });
                    });
                    ui.end_row();
                    egui::CollapsingHeader::new("Stars").show(ui, |ui| {
                        egui::Grid::new("explorer_star_icon_grid")
                            .num_columns(2)
                            .spacing([60.0, 5.0])
                            .min_col_width(300.0)
                            .striped(true)
                            .show(ui, |ui| {
                                for star in &mut self.stars {
                                    ui.horizontal(|ui| {
                                        ui.text_edit_singleline(&mut star.1.char);
                                        ui.label(star.0);
                                    });
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut star.1.enabled, "Enabled");
                                        ui.color_edit_button_srgba(&mut star.1.color);
                                        ui.label("Color");
                                    });
                                    ui.end_row();
                                }
                            });
                    });
                    ui.end_row();
                    egui::CollapsingHeader::new("Planets").show(ui, |ui| {
                        egui::Grid::new("explorer_planet_icon_grid")
                            .num_columns(2)
                            .spacing([60.0, 5.0])
                            .min_col_width(300.0)
                            .striped(true)
                            .show(ui, |ui| {
                                for planet in &mut self.planets {
                                    ui.horizontal(|ui| {
                                        ui.text_edit_singleline(&mut planet.1.char);
                                        ui.label(planet.0);
                                    });
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut planet.1.enabled, "Enabled");
                                        ui.color_edit_button_srgba(&mut planet.1.color);
                                        ui.label("Color");
                                    });
                                    ui.end_row();
                                }
                            });
                    });
                    ui.end_row();

                });
                ui.separator();
                ui.heading("Graphics Override");
                egui::Grid::new("graphics_grid")
                    .num_columns(2)
                    .spacing([60.0, 5.0])
                    .min_col_width(300.0)
                    .striped(true)
                    .show(ui, |ui| {
                        if ui.button("Open Editor").clicked() {
                            self.graphic_editor_settings.show_editor = !self.graphic_editor_settings.show_editor;
                        }
                        ui.end_row();
                        if Path::new(&self.graphic_editor_settings.graphics_directory).exists() {
                            ui.label("Graphics Directory:");
                        } else {
                            ui.label("Graphics Directory: ⚠ Path invalid");
                        }
                        ui.text_edit_singleline(&mut self.graphic_editor_settings.graphics_directory);
                        if ui.button("Open").clicked() {
                            opener::open(&self.graphic_editor_settings.graphics_directory).unwrap();
                        }
                        ui.end_row();
                        if self.graphic_editor_settings.show_editor {
                            Window::new("Editor")
                                .fixed_size(vec2(800f32, 600f32))
                                .show(ctx, |ui| {
                                    egui::Grid::new("preset_buttons")
                                        .show(ui, |ui| {
                                            ui.hyperlink_to(
                                                "Fandom Article",
                                                "https://elite-dangerous.fandom.com/wiki/Graphics_Mods",
                                            );
                                            egui::ComboBox::from_id_source("Presets_Combo_Box")
                                                .selected_text("Presets")
                                                .show_ui(ui, |ui| {
                                                    ui.style_mut().wrap = Some(false);
                                                    ui.set_min_width(60.0);
                                                    ui.selectable_value(&mut self.graphic_editor_settings.graphic_override_content, presets::get_increase_texture_resolution_preset(), "Increased Textures");
                                                    ui.selectable_value(&mut self.graphic_editor_settings.graphic_override_content, presets::get_increased_star_count_preset(), "Increased Star Count");
                                                    ui.selectable_value(&mut self.graphic_editor_settings.graphic_override_content, presets::get_better_skybox_preset(), "Better Skybox");
                                                    ui.selectable_value(&mut self.graphic_editor_settings.graphic_override_content, presets::get_8gb_plus_preset(), "8Gb+ VRAM");
                                                });
                                            if ui.button("Load custom preset").clicked() {
                                                self.graphic_editor_settings.graphic_override_content = match env::var("HOME") {
                                                    Ok(home) => {
                                                        match  fs::read_to_string(format!("{}/.local/share/edcas-client/custom_graphics_override.xml",home)) {
                                                            Ok(file) => {
                                                                file
                                                            }
                                                            Err(_) => {
                                                                fs::read_to_string("custom_graphics_override.xml").unwrap()
                                                            }
                                                        }
                                                    }
                                                    Err(_) => {
                                                        fs::read_to_string("custom_graphics_override.xml").unwrap()
                                                    }
                                                }
                                            }
                                            if ui.button("Save as custom preset").clicked() {
                                                match env::var("HOME") {
                                                    Ok(home) => {
                                                        match  fs::write(format!("{}/.local/share/edcas-client/custom_graphics_override.xml",home),self.graphic_editor_settings.graphic_override_content.clone()) {
                                                            Ok(_) => {}
                                                            Err(_) => {
                                                                fs::write("custom_graphics_override.xml", self.graphic_editor_settings.graphic_override_content.clone()).unwrap();
                                                            }
                                                        }
                                                    }
                                                    Err(_) => {
                                                        fs::write("custom_graphics_override.xml", self.graphic_editor_settings.graphic_override_content.clone()).unwrap();
                                                    }
                                                };
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
                                                    .text_color(Color32::from_rgb(255, 165, 0))
                                                    .font(egui::FontId::monospace(10.0))
                                                    .lock_focus(true)
                                                    .desired_width(f32::INFINITY)
                                            );
                                        });
                                    egui::Grid::new("editor_buttons")
                                        .show(ui, |ui| {
                                            if ui.button("Save").clicked() {
                                                match fs::write(format!("{}/GraphicsConfigurationOverride.xml", self.graphic_editor_settings.graphics_directory.clone()), self.graphic_editor_settings.graphic_override_content.clone()) {
                                                    Ok(_) => {}
                                                    Err(err) => {
                                                        error!("Failed to save settings: {}",err);
                                                        panic!("Failed to save settings: {}",err);
                                                    }
                                                }
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
                    });

                ui.separator();
                let info_ui = |ui: &mut egui::Ui| {
                    ui.heading("What data is being shared?");
                    ui.label("Because of the early development, many things can change between releases.\n\
                    So currently you have to assume that potentially everything will be shared over the EDCAS network and therefore is being available in the internet \n\
                    what is being written in the journal log.\n\
                    If you do not want that, please leave this function disabled.\n\
                    Keep in mind, that your experience might decrease if you leave this disabled.");
                };
                #[cfg(feature = "iota")]{
                    ui.heading("EDCAS Network").on_hover_ui(info_ui);
                    egui::Grid::new("network_grid")
                        .num_columns(2)
                        .spacing([60.0, 5.0])
                        .min_col_width(300.0)
                        .striped(true)
                        .show(ui, |ui| {
                            ui.label("Allow to share journal log data:").on_hover_ui(info_ui);
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
                        });
                }
            });
            ui.separator();
            ui.end_row();
            ui.label("Requires restart to apply settings");
            ui.end_row();
            ui.separator();

            //Apply Button
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                if ui.button("Save 💾").clicked() {
                    self.save_settings_to_file();
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                global_dark_light_mode_switch(ui);
            });
        });
    }
}

impl Settings {
    pub fn save_settings_to_file(&mut self) {
        let icon_array: serde_json::Value = self.icons
            .iter()
            .map(|(_, icon)| json!(
                            {
                                "name": icon.name,
                                "char": icon.char,
                                "r": icon.color.r(),
                                "g": icon.color.g(),
                                "b": icon.color.b(),
                                "enabled": icon.enabled
                            }
                        )).collect();
        let star_array: serde_json::Value = self.stars
            .iter()
            .map(|(_, star)| json!(
                            {
                                "class": star.name,
                                "char": star.char,
                                "r": star.color.r(),
                                "g": star.color.g(),
                                "b": star.color.b(),
                                "enabled": star.enabled
                            }
                        )).collect();
        let planet_array: serde_json::Value = self.planets
            .iter()
            .map(|(_, planet)| json!(
                            {
                                "class": planet.name,
                                "char": planet.char,
                                "r": planet.color.r(),
                                "g": planet.color.g(),
                                "b": planet.color.b(),
                                "enabled": planet.enabled
                            }
                        )).collect();

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
                                "include_system_name": self.explorer_settings.include_system_name
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
                            "icons": icon_array,
                            "stars": star_array,
                            "planets": planet_array,
                            "graphics-editor": {
                                "graphics-directory": self.graphic_editor_settings.graphics_directory
                            }
                        }
                    );
        match File::create(&self.settings_path) {
            Ok(mut settings_file) => {
                match settings_file.write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes()) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("Failed to save settings: {}",err);
                        panic!("Failed to save settings: {}",err);
                    }
                };
            }
            Err(err) => {
                error!("Failed to create settings: {}",err);
                panic!("Failed to create settings: {}",err);
            }
        };

    }
}