use dioxus::logger::tracing::{debug, error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct GraphicEditorSettings {
    #[serde(default = "default_graphics_directory")]
    pub graphics_directory: String,
    #[serde(default = "graphic_override_content")]
    pub graphic_override_content: String,
}

impl Default for GraphicEditorSettings {
    fn default() -> Self {
        Self {
            graphics_directory: default_graphics_directory(),
            graphic_override_content: graphic_override_content(),
        }
    }
}

fn default_graphics_directory() -> String {
    if cfg!(target_os = "windows") {
        let mut userprofile = std::env::var("USERPROFILE").unwrap_or("".to_string());
        userprofile.push_str(
            "\\AppData\\Local\\Frontier Developments\\Elite Dangerous\\Options\\Graphics",
        );
        userprofile
    } else if cfg!(target_os = "linux") {
        let mut home = std::env::var("HOME").unwrap_or("~".to_string());
        home.push_str("/.steam/root/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/AppData/Local/Frontier Developments/Elite Dangerous/Options/Graphics");
        if std::path::Path::new(&home).exists() {
            debug!("Found graphics path: {}", &home);
            home
        } else {
            home = std::env::var("HOME").unwrap_or("~".to_string());
            home.push_str("/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/AppData/Local/Frontier Developments/Elite Dangerous/Options/Graphics");
            if std::path::Path::new(&home).exists() {
                debug!("Found graphics path: {}", &home);
                home
            } else {
                debug!("Did not found graphics path");
                String::default()
            }
        }
    } else {
        error!("Unknown OS!");
        String::default()
    }
}

fn graphic_override_content() -> String {
    "GraphicsConfigurationOverride.xml".to_string()
}
