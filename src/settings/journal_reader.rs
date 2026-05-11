use tracing::{debug, error};
use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct JournalReaderSettings {
    #[serde(default = "default_journal_directory")]
    pub journal_directory: String,
    #[serde(default = "ActionAtShutdownSignal::default")]
    pub action_at_shutdown_signal: ActionAtShutdownSignal,
}

impl Default for JournalReaderSettings {
    fn default() -> Self {
        Self {
            journal_directory: default_journal_directory(),
            action_at_shutdown_signal: Default::default(),
        }
    }
}

fn default_journal_directory() -> String {
    if cfg!(target_os = "windows") {
        let mut userprofile = std::env::var("USERPROFILE").unwrap_or("".to_string());
        userprofile.push_str("\\Saved Games\\Frontier Developments\\Elite Dangerous");
        userprofile
    } else if cfg!(target_os = "linux") {
        let mut home = std::env::var("HOME").unwrap_or("~".to_string());
        home.push_str("/.steam/steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous");
        if std::path::Path::new(&home).exists() {
            debug!("Found journal path: {}", &home);
            home
        } else {
            home = std::env::var("HOME").unwrap_or("~".to_string());
            home.push_str("/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous");
            if std::path::Path::new(&home).exists() {
                debug!("Found journal path: {}", &home);
                home
            } else {
                debug!("Did not found journal path");
                String::default()
            }
        }
    } else {
        error!("Unknown OS!");
        String::default()
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub enum ActionAtShutdownSignal {
    Exit,
    Nothing,
    #[default]
    Continue,
}

impl PartialEq for ActionAtShutdownSignal {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Display for ActionAtShutdownSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ActionAtShutdownSignal::Exit => "Exit".to_string(),
            ActionAtShutdownSignal::Nothing => "Nothing".to_string(),
            ActionAtShutdownSignal::Continue => "Continue".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl FromStr for ActionAtShutdownSignal {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "exit" => Ok(ActionAtShutdownSignal::Exit),
            "nothing" => Ok(ActionAtShutdownSignal::Nothing),
            "continue" => Ok(ActionAtShutdownSignal::Continue),
            _ => Err("Failed to parse ActionShutdownSignal".to_string()),
        }
    }
}
