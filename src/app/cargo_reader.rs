use std::fmt::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::ptr::null;
use std::string::FromUtf8Error;
use image::io::Reader;
use json::{Error as JsonError, JsonValue, Null};
use log::{debug, error};
use serde_json::Value;

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
    pub buy_price: f64,
    pub sell_price: f64,
    pub avg_price: f64
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
                                    let mut new_inventory: Vec<Cargo> = vec![];
                                    while cargo_json != Null {
                                        let name =  cargo_json["Name"].to_string();
                                        let mut buy_price = -1f64;
                                        let mut sell_price = -1f64;
                                        let mut avg_price = -1f64;
                                        for old_cargo in &self.inventory{
                                            if old_cargo.name == name {
                                                buy_price = old_cargo.buy_price;
                                                sell_price = old_cargo.sell_price;
                                                avg_price = old_cargo.avg_price;
                                            }
                                        }

                                        //No old api data found -> requesting it from edcas api
                                        if buy_price == -1f64 {
                                            let answer: Option<Value> = tokio::runtime::Builder::new_current_thread()
                                                .enable_all()
                                                .build()
                                                .unwrap()
                                                .block_on(async {
                                                    let url = format!("https://api.edcas.de/data/commodity/{}",name.clone());
                                                    debug!("Api call to edcas: {}", url.clone());
                                                    let result = reqwest::get(url.clone()).await;
                                                    return match result {
                                                        Ok(response) => {
                                                            let json_data: Value = response.json().await.unwrap();
                                                            Some(json_data)
                                                        }
                                                        Err(err) => {
                                                            error!("Couldn't reach edcas api under {} Reason: {}", url.clone(),err);
                                                            None
                                                        }
                                                    }
                                                });

                                            match answer {
                                                None => {}
                                                Some(json) => {
                                                    buy_price = json["buy_price"].as_f64().unwrap_or(0f64);
                                                    sell_price = json["sell_price"].as_f64().unwrap_or(0f64);
                                                    avg_price = json["avg_price"].as_f64().unwrap_or(0f64);
                                                }
                                            }

                                        }

                                        new_inventory.push(Cargo {
                                            name: cargo_json["Name"].to_string(),
                                            name_localised: cargo_json["Name_Localised"].to_string(),
                                            count: cargo_json["Count"].as_i64().unwrap_or(-1),
                                            stolen: cargo_json["Stolen"].as_i64().unwrap_or(-1),
                                            buy_price,
                                            sell_price,
                                            avg_price
                                        });
                                        cargo_json = json["Inventory"].pop();
                                    }
                                    self.inventory.clear();
                                    self.inventory = new_inventory;
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