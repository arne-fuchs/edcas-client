use bus::BusReader;
use std::fmt::Display;
use std::str::FromStr;
use dioxus::{logger::tracing::{debug, error}, prelude::*};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize,Clone)]
pub struct JournalReaderSettings {
    #[serde(default = "default_journal_directory")]
    pub journal_directory: String,
    #[serde(default = "ActionAtShutdownSignal::default")]
    pub action_at_shutdown_signal: ActionAtShutdownSignal,
}

fn default_journal_directory() -> String {
    if cfg!(target_os = "windows") {
        let mut userprofile = std::env::var("USERPROFILE").unwrap_or("".to_string());
        userprofile.push_str("\\Saved Games\\Frontier Developments\\Elite Dangerous");
        userprofile
    } else if cfg!(target_os = "linux") {
        let mut home = std::env::var("HOME").unwrap_or("~".to_string());
        home.push_str("/.steam/steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous");
        if std::path::Path::new(&home).exists(){
            debug!("Found journal path: {}",&home);
            home
        } else {
            home = std::env::var("HOME").unwrap_or("~".to_string());
            home.push_str("/.var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous");
            if std::path::Path::new(&home).exists(){
                debug!("Found journal path: {}",&home);
                home
            }else{
                debug!("Did not found journal path");
                String::default()
            }
        }
    } else {
        error!("Unknown OS!");
        String::default()
    }
}

pub struct JournalReadStatus {
    pub current_log: u32,
    pub total_logs: u32,
    pub log_index_updates: BusReader<i64>,
}

#[derive(Clone, Serialize, Deserialize)]
#[derive(Default)]
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

#[component]
pub fn settings_view(settings: Signal< crate::desktop::settings::Settings>) -> Element {
    rsx!{
        div { class: "ed-value-box mt-10",
            div { class: "flex justify-center",
                p {
                    class: "text-base m-5 \
                    sm:text-xl  \
                    md:text-2xl \
                    lg:text-3xl \
                    xl:text-4xl",
                    "Journal Reader"
                }
            },
            div { class: "flex  \
                 row-border \
                ",
                div{
                    class:"text-center content-center h-11",
                    p {
                        class: "ml-10 mr-10 ",
                        "Journal Directory"
                    }
                }
                div{
                    class:"text-center content-center h-11",
                    input {
                        class: "mr-10 w-2xs \
                        ed-input-text
                        ",
                        type: "text",
                        oninput: move |evt| {
                            let mut copy = settings.cloned();
                            copy.journal_reader.journal_directory = evt.value();
                            settings.replace(copy);
                        },
                        value: "{settings.read().journal_reader.journal_directory}"
                    }
                }
            }
            div { class: "flex -mt-[4px] \
                 row-border \
                ",
                div{
                    class:"text-center content-center h-11",
                    p {
                        class: "ml-10 mr-10 ",
                        "Action at shutdown"
                    }
                }
                div{
                    class:"text-center content-center h-11",
                    select {
                        class: "w-2xs ed-input-select bg-black/50 border border-black/50 text-black block ",
                        value: "{settings.read().journal_reader.action_at_shutdown_signal.to_string()}",
                        onchange: move |evt| {
                            let mut copy = settings.cloned();
                            copy.journal_reader.action_at_shutdown_signal = ActionAtShutdownSignal::from_str(evt.value().as_str()).unwrap();
                            debug!("{}",copy.journal_reader.action_at_shutdown_signal.to_string());
                            settings.replace(copy);
                        },
                        option{
                            value: "Exit",
                            "Exit"
                        },
                        option{
                            value: "Nothing",
                            "Nothing"
                        },
                        option{
                            value: "Continue",
                            "Continue"
                        }
                    }
                }
            }
        }
    }
}
