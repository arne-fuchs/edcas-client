use std::default::Default;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use eframe::{App, egui, Frame};
use eframe::egui::Context;
use iota_wallet::iota_client::Client;
use serde_json::json;

use crate::egui::global_dark_light_mode_switch;

#[derive(Clone)]
pub struct Settings {
    pub node: Client,
    pub journal_directory: String,
    pub base_url: String,
    pub port: u64,
    pub n_timeout: u64,
    pub n_attempts: u64,
    pub faucet_url: String,
    pub log_level: String,
    pub local_pow: bool,
    pub password: String,
}

impl Default for Settings {
    fn default() -> Self {
        let mut settings_file: File = File::open("settings.json").unwrap();
        let mut json_string: String = String::from("");
        settings_file.read_to_string(&mut json_string).unwrap();
        let json = json::parse(&json_string).unwrap();

        let mut journal_path = json["reader"]["journal-directory"].to_string();
        let path = Path::new(&journal_path);
        if !path.exists() {
            if cfg!(windows) {
                journal_path = String::from("%USERPROFILE%\\Saved Games\\Frontier Developments\\Elite Dangerous");
            } else if cfg!(linux) {
                journal_path = String::from("~/.steam/steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous");
            }
        }
        let mut node_url = json["node"]["base-url"].to_string();
        node_url.push_str(":");
        node_url.push_str(json["node"]["port"].to_string().as_str());
        Self {
            node: Client::builder()
                .with_node(node_url.as_str()).unwrap()
                .with_local_pow(json["local-pow"].as_bool().unwrap())
                .finish().unwrap(),
            journal_directory: journal_path.to_owned(),
            base_url: json["node"]["base-url"].to_string(),
            port: json["node"]["port"].as_u64().unwrap(),
            n_timeout: json["nft-adapter"]["timeout"].as_u64().unwrap(),
            n_attempts: json["nft-adapter"]["attempts"].as_u64().unwrap(),
            faucet_url: json["faucet-url"].to_string(),
            log_level: json["log-level"].to_string(),
            local_pow: json["local-pow"].as_bool().unwrap(),
            password: json["password"].to_string()
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
                    ui.end_row();
                    ui.end_row();

                    ui.label("Connection Settings for the EDCAS Network");
                    ui.end_row();
                    ui.label("Node Url:");
                    ui.text_edit_singleline(&mut self.base_url);
                    ui.end_row();

                    ui.label("Port:");
                    ui.text_edit_singleline(&mut self.port.to_string());
                    ui.end_row();

                    ui.label("Faucet URL:");
                    ui.text_edit_singleline(&mut self.faucet_url);
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
                            },
                            "node": {
                                "base-url": self.base_url,
                                "port": self.port
                            },
                            "nft-adapter": {
                                "timeout": self.n_timeout,
                                "attempts": self.n_attempts
                            },
                            "faucet-url": self.faucet_url,
                            "local-pow": false,
                            "log-level": "Debug"
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