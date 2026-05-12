use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AppearanceSettings {
    #[serde(default = "default_color")]
    pub color: String,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self { color: default_color() }
    }
}

fn default_color() -> String {
    "orange".to_string()
}
