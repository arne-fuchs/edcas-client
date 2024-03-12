use std::sync::Arc;

use json::{JsonValue, Null};
use log::{debug, error};

use crate::app::settings::Settings;

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
    pub mean_price: f64,
    pub highest_sell_price: u64,
    pub highest_sell_station: String,
    pub highest_sell_system: String,
    pub lowest_buy_price: u64,
    pub lowest_buy_station: String,
    pub lowest_buy_system: String,
    pub price_history: Vec<PricePoint>
}

pub struct PricePoint {
    pub buy_price: f64,
    pub sell_price: f64,
    pub mean_price: f64,
    pub timestamp: u64,
}

pub fn initialize(settings: Arc<Settings>) -> CargoReader {
    let mut directory_path= settings.journal_reader_settings.journal_directory.clone();
    if cfg!(target_os = "windows") {
        directory_path.push_str("\\Cargo.json");
    } else if cfg!(target_os = "linux") {
        directory_path.push_str("/Cargo.json");
    }


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
                                        let mut mean_price = -1f64;
                                        let mut highest_sell_price = 0u64;
                                        let mut highest_sell_station = String::from("N/A");
                                        let mut highest_sell_system = String::from("N/A");
                                        let mut lowest_buy_price = 0u64;
                                        let mut lowest_buy_station = String::from("N/A");
                                        let mut lowest_buy_system = String::from("N/A");
                                        let mut price_history:Vec<PricePoint> = vec![];

                                        for old_cargo in &self.inventory{
                                            if old_cargo.name == name {
                                                buy_price = old_cargo.buy_price;
                                                sell_price = old_cargo.sell_price;
                                                mean_price = old_cargo.mean_price;
                                                highest_sell_price = old_cargo.highest_sell_price;
                                                highest_sell_station = old_cargo.highest_sell_station.clone();
                                                highest_sell_system = old_cargo.highest_sell_system.clone();
                                                lowest_buy_price = old_cargo.lowest_buy_price;
                                                lowest_buy_station = old_cargo.lowest_buy_station.clone();
                                                lowest_buy_system = old_cargo.lowest_buy_system.clone();
                                            }
                                        }

                                        //No old api data found -> requesting it from edcas api
                                        if buy_price == -1f64 {
                                            let answer: Option<JsonValue> = tokio::runtime::Builder::new_current_thread()
                                                .enable_all()
                                                .build()
                                                .unwrap()
                                                .block_on(async {
                                                    let url = format!("https://api.edcas.de/data/odyssey/commodity/{}",name.clone());
                                                    debug!("Api call to edcas: {}", url.clone());
                                                    let result = reqwest::get(url.clone()).await;
                                                    return match result {
                                                        Ok(response) => {
                                                            let text = response.text().await.unwrap();
                                                            let result = json::parse(text.as_str());
                                                            return match result {
                                                                Ok(json) => {
                                                                    Some(json)
                                                                }
                                                                Err(err) => {
                                                                    error!("Couldn't parse answer to json: {}",err);
                                                                    error!("Value: {}", text);
                                                                    None
                                                                }
                                                            }
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
                                                    mean_price = json["avg_price"].as_f64().unwrap_or(0f64);
                                                    highest_sell_price = json["highest_sell_price"]["sell_price"].as_u64().unwrap_or(0);
                                                    highest_sell_station = json["highest_sell_price"]["station"].to_string();
                                                    highest_sell_system = json["highest_sell_price"]["system"].to_string();
                                                    lowest_buy_price = json["lowest_buy_price"]["buy_price"].as_u64().unwrap_or(0);
                                                    lowest_buy_station = json["lowest_buy_price"]["station"].to_string();
                                                    lowest_buy_system = json["lowest_buy_price"]["system"].to_string();
                                                }
                                            }

                                        }

                                        let history_answer: Option<JsonValue> = tokio::runtime::Builder::new_current_thread()
                                            .enable_all()
                                            .build()
                                            .unwrap()
                                            .block_on(async {
                                                let url = format!("https://api.edcas.de/data/odyssey/commodity_history/{}",name.clone());
                                                debug!("Api call to edcas: {}", url.clone());
                                                let result = reqwest::get(url.clone()).await;
                                                return match result {
                                                    Ok(response) => {
                                                        let text = response.text().await.unwrap();
                                                        let result = json::parse(text.as_str());
                                                        return match result {
                                                            Ok(json) => {
                                                                Some(json)
                                                            }
                                                            Err(err) => {
                                                                error!("Couldn't parse answer to json: {}",err);
                                                                error!("Value: {}", text);
                                                                None
                                                            }
                                                        }
                                                    }
                                                    Err(err) => {
                                                        error!("Couldn't reach edcas api under {} Reason: {}", url.clone(),err);
                                                        None
                                                    }
                                                }
                                            });

                                        match history_answer {
                                            None => {}
                                            Some(history) => {
                                                for i in 0..history["prices"].len(){
                                                    let price = &history["prices"][i];
                                                    price_history.push(PricePoint{
                                                        buy_price: price["buy_price"].as_f64().unwrap(),
                                                        sell_price: price["sell_price"].as_f64().unwrap(),
                                                        mean_price: price["mean_price"].as_f64().unwrap(),
                                                        timestamp: price["timestamp"].as_u64().unwrap(),
                                                    })
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
                                            mean_price,
                                            highest_sell_price,
                                            highest_sell_station,
                                            highest_sell_system,
                                            lowest_buy_price,
                                            lowest_buy_station,
                                            lowest_buy_system,
                                            price_history,
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