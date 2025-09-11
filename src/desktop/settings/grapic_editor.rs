use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize,Clone)]
pub struct GraphicEditorSettings {
    #[serde(default = "default_graphics_directory")]
    pub graphics_directory: String,
    #[serde(default = "graphic_override_content")]
    pub graphic_override_content: String,
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
        home
    } else {
        panic!("Unknown OS");
    }
}

fn graphic_override_content() -> String {
    "GraphicsConfigurationOverride.xml".to_string()
}
