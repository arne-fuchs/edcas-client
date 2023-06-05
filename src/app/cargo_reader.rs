use std::fs::File;
use std::io::{BufReader, Read};
use std::ptr::null;
use std::string::FromUtf8Error;
use image::io::Reader;
use json::Null;
use log::error;

pub struct CargoReader {
    directory_path: String,
    pub inventory: Vec<Cargo>,
    hash: String,
}

pub struct Cargo {
    pub name: String,
    pub name_localised: String,
    pub count: i64,
    pub stolen: i64,
}

pub fn initialize(mut directory_path: String) -> CargoReader {
    directory_path.push_str("/Cargo.json");

    CargoReader {
        directory_path,
        inventory: Vec::new(),
        hash: String::new(),
    }
}

impl CargoReader {
    pub fn run(&mut self) {
        let read_result = std::fs::read(&self.directory_path);
        match read_result {
            Err(err) => {
                error!("Couldn't read cargo file: {}.\n Path: {}", err, &self.directory_path);
            }
            Ok(bytes) => {
                let string_result = String::from_utf8(bytes);
                match string_result {
                    Ok(cargo_string) => {
                        let hash = sha256::digest(cargo_string.as_str());
                        if hash != self.hash {
                            self.hash = hash;
                            let json_result = json::parse(cargo_string.as_str());
                            match json_result {
                                Ok(json) => {
                                    let mut json = json.clone();
                                    let mut cargo_json = json["Inventory"].pop();
                                    self.inventory.clear();
                                    while cargo_json != Null {
                                        self.inventory.push(Cargo {
                                            name: cargo_json["Name"].to_string(),
                                            name_localised: cargo_json["Name_Localised"].to_string(),
                                            count: cargo_json["Count"].as_i64().unwrap_or(-1),
                                            stolen: cargo_json["Stolen"].as_i64().unwrap_or(-1),
                                        });
                                        cargo_json = json["Inventory"].pop();
                                    }
                                }
                                Err(err) => {
                                    error!("Couldn't parse cargo string to json: {}", err);
                                }
                            }
                        }
                    }
                    Err(err) => {
                        error!("Couldn't parse bytes to string for cargo: {}", err);
                    }
                }
            }
        }
    }
}