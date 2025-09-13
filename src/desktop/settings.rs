use dioxus::logger::tracing::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::{
    collections::HashMap,
    io::Read,
    path::Path,
};

pub mod appearance;
pub mod explorer;
pub mod grapic_editor;
pub mod journal_reader;
pub mod icons;
use icons::Icon;

#[derive(Serialize, Deserialize,Clone)]
pub struct Settings {
    pub appearance: appearance::AppearanceSettings,
    pub journal_reader: journal_reader::JournalReaderSettings,
    pub explorer: explorer::ExplorerSettings,
    pub graphics_editor: grapic_editor::GraphicEditorSettings,
    #[serde(default = "HashMap::default")]
    pub icons: HashMap<String, Icon>,
    #[serde(default = "HashMap::default")]
    pub stars: HashMap<String, Icon>,
    #[serde(default = "HashMap::default")]
    pub planets: HashMap<String, Icon>,
}

impl Default for Settings {
    fn default() -> Self {
        let mut settings_path = "settings.json".to_string();
        let mut settings_file = match std::env::var("HOME") {
            Ok(home) => {
                settings_path = format!("{}/.config/edcas-client/settings.json", home);
                if std::path::Path::new(&settings_path).exists() {
                    //File exists -> we can open it
                    File::open(&settings_path).expect("Unable to open file")
                } else {
                    info!("Couldn't find config file in {} -> trying to create file structure and copy to desired config folder",settings_path);
                    create_setting_json(home)
                }
            }
            Err(err) => {
                warn!("{}", err);
                match std::fs::File::open("settings.json") {
                    Ok(file) => {
                        info!("Accessing settings file at settings.json");
                        file
                    }
                    Err(err) => {
                        warn!("{}", err);
                        if std::path::Path::new("/etc/edcas-client/settings-example.json").exists()
                        {
                            info!(
                                "Copying from /etc/edcas-client/settings-example.json to settings.json"
                            );
                            std::fs::copy("/etc/edcas-client/settings-example.json", "settings.json").expect("Couldn't copy settings file from /etc/edcas-client/settings-example.json to local settings.json");
                        } else {
                            std::fs::copy("settings-example.json", "settings.json").expect(
                                "Couldn't copy settings file to $HOME/.config/edcas-client/",
                            );
                        }
                        info!("Accessing settings file at settings.json");
                        std::fs::File::open("settings.json").unwrap()
                    }
                }
            }
        };

        let mut json_string: String = String::default();
        settings_file.read_to_string(&mut json_string).unwrap();
        let mut settings: Settings =
            serde_json::from_str(&json_string).expect("Invalid json settings file");
        //---------------------------
        // Journal reader
        //---------------------------

        if !Path::new(&settings.journal_reader.journal_directory).exists() {
            warn!(
                "journal path {} does not exists",
                settings.journal_reader.journal_directory
            );
        }
        debug!(
            "Journal logs: {}",
            &settings.journal_reader.journal_directory
        );

        //---------------------------
        // Graphics directory
        //---------------------------
        if !Path::new(&settings.graphics_editor.graphics_directory).exists() {
            warn!(
                "graphics path {} does not exists",
                &settings.graphics_editor.graphics_directory
            );
        } else if cfg!(target_os = "windows") {
            settings.graphics_editor.graphic_override_content = format!(
                "{}\\GraphicsConfigurationOverride.xml",
                &settings.graphics_editor.graphic_override_content
            );
        } else if cfg!(target_os = "linux") {
            settings.graphics_editor.graphic_override_content = format!(
                "{}/GraphicsConfigurationOverride.xml",
                &settings.graphics_editor.graphic_override_content
            );
        }

        debug!(
            "Graphics path: {}",
            &settings.graphics_editor.graphics_directory
        );

        //---------------------------
        // Appearance
        //---------------------------

        //---------------------------
        // Icons
        //---------------------------

        //---------------------------
        // Stars
        //---------------------------

        //---------------------------
        // Planets
        //---------------------------

        settings
    }
}

fn create_setting_json(home: String) -> File {
    match std::fs::create_dir_all(format!("{}/.config/edcas-client", home)) {
        Ok(_) => {
            info!("Created $HOME/.config/edcas-client/");
            info!("Copying from /etc/edcas-client/settings-example.json to $HOME/.config/edcas-client/settings.json");
            match std::fs::copy(
                "/etc/edcas-client/settings-example.json",
                format!("{}/.config/edcas-client/settings.json", home),
            ) {
                Ok(_) => {
                    info!(
                        "Copied /etc/edcas-client/settings-example.json to {}",
                        format!("{}/.config/edcas-client/settings.json", home)
                    );
                }
                Err(err) => {
                    info!("Failed copying from /etc/edcas-client/settings-example.json\n Trying to copy from settings-example.json to $HOME/.config/edcas-client/settings.json: {}",err);
                    std::fs::copy(
                        "settings-example.json",
                        format!("{}/.config/edcas-client/settings.json", home),
                    )
                    .expect("Couldn't copy settings file to $HOME/.config/edcas-client/");
                }
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                info!(
                    "Setting permissions: {:?}",
                    std::fs::set_permissions(
                        format!("{}/.config/edcas-client/settings.json", home),
                        std::fs::Permissions::from_mode(0o644)
                    )
                );
            }

            info!("Accessing settings file at $HOME/.config/edcas-client/settings.json");
            std::fs::File::open(format!("{}/.config/edcas-client/settings.json", home))
                .expect("Couldn't open settings file in $HOME/.config/edcas-client/")
        }
        Err(err) => {
            warn!("{}", err);
            info!("Couldn't create directories in $HOME/.config/edcas-client/");
            match std::fs::File::open("settings.json") {
                Ok(file) => {
                    info!("Accessing settings file at settings.json");
                    file
                }
                Err(err) => {
                    warn!("{}", err);
                    info!("Copying from settings-example.json to settings.json");
                    match std::fs::copy("settings-example.json", "settings.json") {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Error copying settings file: {}", err);
                            panic!("Error copying settings file: {}", err);
                        }
                    }
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;

                        info!(
                            "Setting permissions: {:?}",
                            std::fs::set_permissions(
                                "settings.json",
                                std::fs::Permissions::from_mode(0o644)
                            )
                        );
                    }
                    #[cfg(windows)]
                    {
                        fs::metadata("settings.json")
                            .unwrap()
                            .permissions()
                            .set_readonly(false);
                    }
                    info!("Accessing settings file at settings.json");
                    std::fs::File::open("settings.json").unwrap()
                }
            }
        }
    }
}
