use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use std::{env, thread};
use std::str::FromStr;
use bus::BusReader;
use chrono::DateTime;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::middleware::signer::SignerMiddlewareError;
use ethers::prelude::*;
use ethers::utils::hex;
use json::JsonValue;
use log::{debug, error, info};
use tokio::runtime::Runtime;
use regex::Regex;
use crate::edcas::backend::evm::edcas_contract;
use crate::edcas::backend::evm::edcas_contract::{
    BodyProperties, PlanetProperties, StarProperties,
};
use crate::edcas::backend::evm::journal_interpreter::SendError::{NonceRecalculationRequired, NonRepeatableError, RepeatableError};
use crate::edcas::backend::floating;

use crate::edcas::settings::EvmSettings;

pub type Edcas = edcas_contract::EDCAS<SignerMiddleware<Provider<Http>, LocalWallet>>;
pub type ContractCall = FunctionCall<
    Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
    (),
>;

const NUMBER_OF_WORKERS: usize = 1000;

pub struct EvmInterpreter {
    bus: BusReader<JsonValue>,
    contract: Edcas,
    nonce: U256,
    /// Queue for function calls and its nonce. The nonce needs to be changed if a previous transaction failed
    queue: VecDeque<ContractCall>,
    pool: Vec<thread::JoinHandle<Result<TransactionReceipt, SendError>>>,
}

impl EvmInterpreter {
    pub fn run_loop(&mut self) {
        //This loop should never block so thread communication is never blocked
        loop {
            //Emptying the thread communication bus
            while let Ok(json) = self.bus.try_recv() {
                match json["event"].as_str().unwrap_or("") {
                    "FSDJump" => {
                        debug!("Add register_system to queue");
                        let function_call: ContractCall = self.contract.register_system(
                            json["SystemAddress"].as_u64().unwrap(),
                            json["StarSystem"].to_string(),
                            json["SystemGovernment"].to_string(),
                            json["SystemAllegiance"].to_string(),
                            json["SystemEconomy"].to_string(),
                            json["SystemSecondEconomy"].to_string(),
                            json["SystemSecurity"].to_string(),
                            json["Population"].as_u64().unwrap_or(0),
                            floating::generate_floating_from_string(json["StarPos"][0].to_string()),
                            floating::generate_floating_from_string(json["StarPos"][1].to_string()),
                            floating::generate_floating_from_string(json["StarPos"][2].to_string()),
                            DateTime::parse_from_rfc3339(json["timestamp"].as_str().unwrap())
                                .unwrap()
                                .timestamp()
                                .into(),
                        );
                        self.queue.push_back(function_call);
                    },
                    "FSSDiscoveryScan" => {
                        debug!("Add set_body_count to queue");
                        let system_address = json["SystemAddress"].as_u64().unwrap();
                        let body_count = json["BodyCount"].as_u8().unwrap();
                        let function_call: ContractCall = self.contract.set_body_count(
                            system_address,
                            body_count,
                            DateTime::parse_from_rfc3339(json["timestamp"].as_str().unwrap())
                                .unwrap()
                                .timestamp()
                                .into(),
                        );
                        self.queue.push_back(function_call);
                    },
                    "Scan" => {
                        if !json["BodyName"].to_string().contains("Belt Cluster")
                            && !json["BodyName"].to_string().contains("Ring")
                        {
                            if !json.has_key("StarType") {
                                //Planet (Body)
                                //Body
                                //{ "timestamp":"2022-10-16T23:51:17Z", "event":"Scan", "ScanType":"Detailed", "BodyName":"Ogmar A 6", "BodyID":40,
                                // "Parents":[ {"Star":1}, {"Null":0} ],
                                // "StarSystem":"Ogmar", "SystemAddress":84180519395914, "DistanceFromArrivalLS":3376.246435,
                                // "TidalLock":false, "TerraformState":"", "PlanetClass":"Sudarsky class I gas giant",
                                // "Atmosphere":"", "AtmosphereComposition":[ { "Name":"Hydrogen", "Percent":73.044167 }, { "Name":"Helium", "Percent":26.955832 } ],
                                // "Volcanism":"", "MassEM":24.477320, "Radius":22773508.000000, "SurfaceGravity":18.811067, "SurfaceTemperature":62.810730,
                                // "SurfacePressure":0.000000, "Landable":false, "SemiMajorAxis":1304152250289.916992, "Eccentricity":0.252734,
                                // "OrbitalInclination":156.334694, "Periapsis":269.403039, "OrbitalPeriod":990257555.246353, "AscendingNode":-1.479320,
                                // "MeanAnomaly":339.074691, "RotationPeriod":37417.276422, "AxialTilt":0.018931, "WasDiscovered":true, "WasMapped":true }
                                let body_id = json["BodyID"].as_u8().unwrap();
                                let system_address = json["SystemAddress"].as_u64().unwrap();
                                let function_call: ContractCall = self.contract.register_planet(
                                    system_address,
                                    body_id,
                                    json["BodyName"].to_string(),
                                    json["WasDiscovered"].as_bool().unwrap(),
                                    json["WasMapped"].as_bool().unwrap(),
                                    extract_planet_properties(&json),
                                    extract_body_properties(&json),
                                    DateTime::parse_from_rfc3339(
                                        json["timestamp"].as_str().unwrap(),
                                    )
                                        .unwrap()
                                        .timestamp()
                                        .into(),
                                );
                                debug!("Add register_planet to queue");
                                self.queue.push_back(function_call);
                            } else {
                                //Star
                                //{"AbsoluteMagnitude":8.518448,"Age_MY":446,"AxialTilt":0,"BodyID":0,"BodyName":"Hyades Sector BB-N b7-5",
                                // "DistanceFromArrivalLS":0,"Luminosity":"Va","Radius":374854272.0,"RotationPeriod":192595.293946,"ScanType":"AutoScan",
                                // "StarPos":[12.1875,-74.90625,-120.5],"StarSystem":"Hyades Sector BB-N b7-5","StarType":"M","StellarMass":0.394531,"Subclass":1,
                                // "SurfaceTemperature":3367.0,"SystemAddress":11666070513017,"WasDiscovered":true,"WasMapped":false,"event":"Scan","horizons":true,
                                // "odyssey":true,"timestamp":"2024-03-26T21:27:53Z"}
                                let body_id = json["BodyID"].as_u8().unwrap();
                                let system_address = json["SystemAddress"].as_u64().unwrap();
                                let function_call: ContractCall = self.contract.register_star(
                                    system_address,
                                    body_id,
                                    json["BodyName"].to_string(),
                                    json["WasDiscovered"].as_bool().unwrap(),
                                    json["WasMapped"].as_bool().unwrap(),
                                    extract_star_properties(&json),
                                    extract_body_properties(&json),
                                    DateTime::parse_from_rfc3339(
                                        json["timestamp"].as_str().unwrap(),
                                    )
                                        .unwrap()
                                        .timestamp()
                                        .into(),
                                );
                                debug!("Add register_star to queue");
                                self.queue.push_back(function_call);
                            }
                        }
                    },
                    "FSSBodySignals" | "SAASignalsFound" => {
                        let system_address = json["SystemAddress"].as_u64().unwrap();
                        let body_id = json["BodyID"].as_u8().unwrap();
                        for i in 0..json["Signals"].len() {
                            let type_ = {
                                //enum PlanetSignalType {
                                //     unknown,geo,xeno,bio,human
                                // }
                                match json["Signals"][i]["Type"].as_str().unwrap() {
                                    "$SAA_SignalType_Human;" => 4,
                                    "$SAA_SignalType_Biological;" => 3,
                                    "$SAA_SignalType_Xenological;" => 2,
                                    "$SAA_SignalType_Geological;" => 1,
                                    &_ => 0,
                                }
                            };
                            let function_call: ContractCall = self.contract.register_planet_signal(
                                system_address,
                                body_id,
                                type_,
                                json["Signals"][i]["Count"].as_u8().unwrap(),
                                DateTime::parse_from_rfc3339(
                                    json["timestamp"].as_str().unwrap(),
                                )
                                    .unwrap()
                                    .timestamp()
                                    .into(),
                            );
                            debug!("Add register_planet_signal to queue");
                            self.queue.push_back(function_call);
                        }
                    },
                    "CarrierJumpRequest" => {
                        let function_call: ContractCall = self.contract.emit_carrier_jump(
                            json["CarrierID"].as_u64().unwrap(),
                            json["SystemName"].as_str().unwrap().to_string(),
                            json["Body"].as_str().unwrap_or("Unknown").to_string(),
                            DateTime::parse_from_rfc3339(
                                json["DepartureTime"].as_str().unwrap_or(""),
                            )
                                .unwrap_or(DateTime::default())
                                .timestamp()
                                .into(),
                        );
                        debug!("Add emit_carrier_jump to queue");
                        self.queue.push_back(function_call);
                    },
                    "CarrierJumpCancelled" => {
                        let function_call: ContractCall = self.contract.cancel_carrier_jump(json["CarrierID"].as_u64().unwrap());
                        debug!("Add cancel_carrier_jump to queue");
                        self.queue.push_back(function_call);
                    },
                    "CarrierBuy" => {
                        let function_call: ContractCall = self.contract.register_carrier(
                            json["CarrierID"].as_u64().unwrap(),
                            "Carrier".to_string(),
                            json["Callsign"].as_str().unwrap().to_string(),
                            "".to_string(),
                            "".to_string(),
                            false,
                            DateTime::parse_from_rfc3339(json["timestamp"].as_str().unwrap())
                                .unwrap()
                                .timestamp()
                                .into(),
                        );
                        debug!("Add register_carrier to queue");
                        self.queue.push_back(function_call);
                    },
                    "CarrierStats" => {
                        let mut services = String::new();
                        for entry in 0..json["StationServices"].len() {
                            if !services.is_empty() {
                                services.push(',');
                            }
                            services.push_str(json["StationServices"][entry].as_str().unwrap());
                        }
                        if json["StationType"].as_str().unwrap() == "FleetCarrier" {
                            debug!("Call register_carrier");
                            let function_call: ContractCall = self.contract.register_carrier(
                                json["MarketID"].as_u64().unwrap(),
                                "Fleet Carrier".to_string(),
                                json["StationName"].as_str().unwrap().to_string(),
                                services,
                                "".to_string(),
                                false,
                                DateTime::parse_from_rfc3339(
                                    json["timestamp"].as_str().unwrap(),
                                )
                                    .unwrap()
                                    .timestamp()
                                    .into(),
                            );
                            debug!("Add register_carrier to queue");
                            self.queue.push_back(function_call);
                            let function_call: ContractCall = self.contract.report_carrier_location(
                                json["MarketID"].as_u64().unwrap(),
                                json["StarSystem"].as_str().unwrap().to_string(),
                                "Unkown".to_string(),
                                DateTime::parse_from_rfc3339(
                                    json["timestamp"].as_str().unwrap(),
                                )
                                    .unwrap()
                                    .timestamp()
                                    .into(),
                            );
                            debug!("Call report_carrier_location to queue");
                            self.queue.push_back(function_call);
                        } else {
                            debug!("Call register_station  to queue");
                            let function_call: ContractCall = self.contract.register_station(
                                json["MarketID"].as_u64().unwrap(),
                                json["StationName"].as_str().unwrap().to_string(),
                                json["StationType"].as_str().unwrap().to_string(),
                                json["SystemAddress"].as_u64().unwrap(),
                                json["StarSystem"].as_str().unwrap().to_string(),
                                edcas_contract::Faction {
                                    name: json["StationFaction"]["Name"]
                                        .as_str()
                                        .unwrap()
                                        .to_string(),
                                    state: json["StationFaction"]["FactionState"]
                                        .as_str()
                                        .unwrap_or("")
                                        .to_string(),
                                },
                                json["StationGovernment"].as_str().unwrap().to_string(),
                                json["StationEconomy"].as_str().unwrap().to_string(),
                                services,
                                floating::generate_floating_from_string(
                                    json["DistFromStarLS"].to_string(),
                                ),
                                json["LandingPads"].to_string(),
                                DateTime::parse_from_rfc3339(
                                    json["timestamp"].as_str().unwrap(),
                                )
                                    .unwrap()
                                    .timestamp()
                                    .into(),
                            );
                            //execute_send(function_call).await;
                            self.queue.push_back(function_call);
                        }
                    }
                    "" => {
                        if !json["commodities"].is_empty() {
                            let market_id = json["marketId"].as_u64().unwrap();
                            let size = json["commodities"].len();
                            for i in 0..size {
                                let commoditiy = &json["commodities"][i];
                                let function_call: ContractCall = self.contract.register_commodity_listening(
                                    market_id,
                                    commoditiy["name"].as_str().unwrap().to_ascii_lowercase(),
                                    edcas_contract::CommodityListening {
                                        buy_price: commoditiy["buyPrice"].as_u32().unwrap_or(0),
                                        sell_price: commoditiy["sellPrice"]
                                            .as_u32()
                                            .unwrap_or(0),
                                        mean_price: commoditiy["meanPrice"]
                                            .as_u32()
                                            .unwrap_or(0),
                                        stock: commoditiy["stock"].as_u32().unwrap_or(0),
                                        demand: commoditiy["demand"].as_u32().unwrap_or(0),
                                        stock_bracket: commoditiy["stockBracket"]
                                            .as_u32()
                                            .unwrap_or(0),
                                        demand_bracket: commoditiy["demandBracket"]
                                            .as_u32()
                                            .unwrap_or(0),
                                    },
                                );
                                //execute_send(function_call).await;
                                debug!("Call register_commodity_listening to queue");
                                self.queue.push_back(function_call);
                            }
                        }
                    }
                    &_ => {}
                }
            }
            //Queue has a list of transactions which has to be executed sequentially

            //There is a maximum of parallel working workers (NUMBER_OF_WORKERS)
            //If there are fewer workers than the maximum number of workers are allowed, a new job can be applied
            if !self.queue.is_empty() && self.pool.len() < NUMBER_OF_WORKERS {
                if let Some(function_call) = self.queue.pop_front() {
                    let nonce = self.nonce;
                    self.nonce = nonce + 1;
                    let thread: thread::JoinHandle<Result<TransactionReceipt, SendError>> =
                        thread::Builder::new()
                            .name(String::from("Evm-Int-Worker"))
                            .spawn(move || {
                                Runtime::new().unwrap().block_on(async {
                                    execute_send_repeatable(function_call,nonce).await
                                })
                            })
                            .unwrap();
                    self.pool.push(thread);
                    debug!("Queue size: {}",self.queue.len());
                }
            } else {
                thread::sleep(Duration::from_secs(1));
                //Here we have max number of workers reached.
                let mut num_of_failes = 0;

                let handles = std::mem::take(&mut self.pool);
                for handle in handles {
                    if handle.is_finished() {
                        match handle.join().unwrap() {
                            Ok(transaction) => {
                                info!("Evm call successfully: 0x{:?}",hex::encode(transaction.transaction_hash.0));
                            }
                            Err(err) => {
                                //If something failed we have to recalculate the nonces
                                match err {
                                    RepeatableError(error) |
                                    NonRepeatableError(error) => {
                                        info!("Evm call unsuccessfully: {:?}", error);
                                    }
                                    NonceRecalculationRequired(function_call, error) => {
                                        self.queue.push_front(function_call);
                                        info!("Evm call unsuccessfully: {:?}", error);
                                        if error.contains("invalid transaction nonce") {
                                            let re = Regex::new(r"want (\d+)").unwrap();
                                            if let Some(captures) = re.captures(error.as_str()) {
                                                if let Some(want_number) = captures.get(1) {
                                                    self.nonce = U256::from_str(want_number.as_str()).unwrap();
                                                }
                                            }
                                        }
                                    }
                                }
                                num_of_failes += 1;
                            }
                        }
                    }else {
                        self.pool.push(handle);
                    }
                }
                if num_of_failes > 0 {
                    while !self.pool.is_empty() {
                        info!("Waiting pool to finish... (Remaining: {})",self.pool.len());
                        let handles = std::mem::take(&mut self.pool);
                        for handle in handles {
                            if !handle.is_finished() {
                                self.pool.push(handle);
                            }else {
                                match handle.join().unwrap() {
                                    Ok(transaction) => {
                                        info!("Evm call successfully: 0x{:?}",hex::encode(transaction.transaction_hash.0));
                                    }
                                    Err(err) => {
                                        match err {
                                            RepeatableError(error) |
                                            NonRepeatableError(error) => {
                                                info!("Evm call unsuccessfully: {:?}", error);
                                            }
                                            NonceRecalculationRequired(function_call, error) => {
                                                self.queue.push_front(function_call);
                                                info!("Evm call unsuccessfully: {:?}", error);
                                                if error.contains("invalid transaction nonce") {
                                                    let re = Regex::new(r"want (\d+)").unwrap();
                                                    if let Some(captures) = re.captures(error.as_str()) {
                                                        if let Some(want_number) = captures.get(1) {
                                                            self.nonce = U256::from_str(want_number.as_str()).unwrap();
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        num_of_failes += 1;
                                    }
                                }
                            }
                        }
                        thread::sleep(Duration::from_secs(1));
                    }

                    info!("Finished waiting -> calculating actual nonce (Current: {})",self.nonce);
                    // Set new nonces for every transaction
                    self.nonce = self.nonce - num_of_failes;
                    info!("New nonce: {}",self.nonce);
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
    }
}
pub fn initialize(bus_reader: BusReader<JsonValue>, evm_settings: &EvmSettings) -> EvmInterpreter {
    let (contract, nonce) = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            (
                get_contract(evm_settings).await,
                get_nonce(evm_settings).await,
            )
        });

    debug!("Current Nonce: {}", nonce);

    EvmInterpreter {
        bus: bus_reader,
        contract,
        nonce,
        queue: VecDeque::new(),
        pool: Vec::new(),
    }
}
pub async fn get_contract(
    evm_settings: &EvmSettings,
) -> edcas_contract::EDCAS<SignerMiddleware<Provider<Http>, LocalWallet>> {
    let sc_address = evm_settings.smart_contract_address.to_string();
    let sc_address = sc_address.as_str();

    let node_url = evm_settings.url.to_string();
    let private = evm_settings.private_key.to_string();
    let retry = evm_settings.n_attempts;
    let timeout = evm_settings.n_timeout;

    let client: SignerMiddleware<Provider<Http>, LocalWallet> =
        get_client(node_url, private, retry, timeout).await;

    let edcas_address = sc_address.parse::<Address>().unwrap();

    edcas_contract::EDCAS::new(edcas_address, Arc::new(client.clone()))
}

pub async fn get_nonce(evm_settings: &EvmSettings) -> U256 {
    let node_url = evm_settings.url.to_string();
    let private = evm_settings.private_key.to_string();
    let retry = evm_settings.n_attempts;
    let timeout = evm_settings.n_timeout;

    let client: SignerMiddleware<Provider<Http>, LocalWallet> =
        get_client(node_url, private, retry, timeout).await;

    client
        .get_transaction_count(client.address(), None)
        .await
        .unwrap()
}

pub async fn get_client(
    node_url: String,
    private_key: String,
    retry: u64,
    timeout: u64,
) -> SignerMiddleware<Provider<Http>, LocalWallet> {
    info!("Loading wallet");

    info!("Using URL:{}", &node_url);

    let provider = Provider::connect(node_url.as_str()).await;

    let wallet: LocalWallet = private_key.parse::<LocalWallet>().unwrap();
    info!("EVM Address: {:?}", wallet.address());

    let mut result =
        SignerMiddleware::new_with_provider_chain(provider.clone(), wallet.clone()).await;
    let mut retries = 0;
    while result.is_err() && retries < retry {
        retries += 1;
        tokio::time::sleep(Duration::from_secs(timeout)).await;
        result = SignerMiddleware::new_with_provider_chain(provider.clone(), wallet.clone()).await;
    }
    result.unwrap()
}

fn extract_planet_properties(json: &JsonValue) -> PlanetProperties {
    let mut parent_id = 0;
    for i in 0..json["Parents"].len() {
        let parent = &json["Parents"][i];
        for entry in parent.entries() {
            if entry.1.as_u8().unwrap() > parent_id {
                parent_id = entry.1.as_u8().unwrap();
            }
        }
    }
    PlanetProperties {
        atmosphere: json["Atmosphere"].to_string(),
        class: json["PlanetClass"].to_string(),
        landable: json["Landable"].as_bool().unwrap_or(false),
        terraform_state: json["TerraformState"].to_string(),
        volcanism: json["Volcanism"].to_string(),
        tidal_lock: json["TidalLock"].as_bool().unwrap_or({
            error!("Tidal Lock not parseable {}", json);
            false
        }),
        parent_id,
        mass_em: floating::generate_floating_from_string(json["MassEM"].to_string()),
        surface_gravity: floating::generate_floating_from_string(
            json["SurfaceGravity"].to_string(),
        ),
        surface_pressure: floating::generate_floating_from_string(
            json["SurfacePressure"].to_string(),
        ),
        ascending_node: floating::generate_floating_from_string(json["AscendingNode"].to_string()),
        eccentricity: floating::generate_floating_from_string(json["Eccentricity"].to_string()),
        mean_anomaly: floating::generate_floating_from_string(json["MeanAnomaly"].to_string()),
        orbital_inclination: floating::generate_floating_from_string(
            json["OrbitalInclination"].to_string(),
        ),
        orbital_period: floating::generate_floating_from_string(json["OrbitalPeriod"].to_string()),
        periapsis: floating::generate_floating_from_string(json["Periapsis"].to_string()),
        semi_major_axis: floating::generate_floating_from_string(json["SemiMajorAxis"].to_string()),
    }
}
fn extract_star_properties(json: &JsonValue) -> StarProperties {
    StarProperties {
        subclass: json["Subclass"].as_u8().unwrap(),
        age_my: json["Age_MY"].as_u16().unwrap(),
        type_: json["StarType"].to_string(),
        luminosity: json["Luminosity"].to_string(),
        stellar_mass: floating::generate_floating_from_string(json["StellarMass"].to_string()),
        absolute_magnitude: floating::generate_floating_from_string(
            json["AbsoluteMagnitude"].to_string(),
        ),
    }
}
fn extract_body_properties(json: &JsonValue) -> BodyProperties {
    BodyProperties {
        radius: floating::generate_floating_from_string(json["Radius"].to_string()),
        distance_from_arrival_ls: floating::generate_floating_from_string(
            json["DistanceFromArrivalLS"].to_string(),
        ),
        axial_tilt: floating::generate_floating_from_string(json["AxialTilt"].to_string()),
        rotation_period: floating::generate_floating_from_string(
            json["RotationPeriod"].to_string(),
        ),
        surface_temperature: floating::generate_floating_from_string(
            json["SurfaceTemperature"].to_string(),
        ),
    }
}
enum SendError {
    RepeatableError(String),
    NonRepeatableError(String),
    NonceRecalculationRequired(ContractCall,String),
}

async fn execute_send_repeatable(
    function_call: ContractCall,
    nonce: U256,
) -> Result<TransactionReceipt, SendError> {
    while let Err(err) = match execute_send(function_call.clone(), nonce).await {
        Ok(receipt) => return Ok(receipt),
        Err(err) => Err::<(), SendError>(err),
    } {
        match err {
            RepeatableError(_) => {
                tokio::time::sleep(Duration::from_secs(
                    env::var("DURATION_TIMEOUT")
                        .unwrap_or("1".into())
                        .parse()
                        .unwrap_or(1),
                ))
                .await;
            }
            NonRepeatableError(err) => return Err(NonRepeatableError(err)),
            NonceRecalculationRequired(function_call,err) => return Err(NonceRecalculationRequired(function_call,err)),
        }
    }
    Err(NonRepeatableError("Unknown".into()))
}
async fn execute_send(
    function_call: ContractCall,
    nonce: U256,
) -> Result<TransactionReceipt, SendError> {
    match function_call.clone().legacy().nonce(nonce).send().await {
        Ok(pending) => match pending.await {
            Ok(receipt) => {
                if let Some(receipt) = receipt {
                    if let Some(_hash) = receipt.block_hash {
                        //info!("Success calling function: {:?}", hash);
                        Ok(receipt)
                    } else {
                        Err(RepeatableError("Receipt without hash".into()))
                    }
                } else {
                    Err(RepeatableError("No Receipt".into()))
                }
            }
            Err(err) => match err {
                ProviderError::JsonRpcClientError(err) => {
                    error!("JsonRpcClientError: {}", err);
                    Err(NonceRecalculationRequired(function_call,err.to_string()))
                }
                ProviderError::EnsError(err) => {
                    error!("EnsError: {}", err);
                    Err(NonRepeatableError(err.to_string()))
                }
                ProviderError::EnsNotOwned(err) => {
                    error!("EnsNotOwned: {}", err);
                    Err(NonRepeatableError(err.to_string()))
                }
                ProviderError::SerdeJson(err) => {
                    error!("SerdeJson: {}", err);
                    Err(NonRepeatableError(err.to_string()))
                }
                ProviderError::HexError(err) => {
                    error!("HexError: {}", err);
                    Err(NonRepeatableError(err.to_string()))
                }
                ProviderError::HTTPError(err) => {
                    error!("HTTPError: {}", err);
                    Err(NonRepeatableError(err.to_string()))
                }
                ProviderError::CustomError(err) => {
                    error!("CustomError: {}", err);
                    Err(NonRepeatableError(err.to_string()))
                }
                ProviderError::UnsupportedRPC => {
                    error!("UnsupportedRPC");
                    Err(NonRepeatableError(err.to_string()))
                }
                ProviderError::UnsupportedNodeClient => {
                    error!("UnsupportedNodeClient");
                    Err(NonRepeatableError(err.to_string()))
                }
                ProviderError::SignerUnavailable => {
                    error!("SignerUnavailable");
                    Err(NonRepeatableError(err.to_string()))
                }
            },
        },
        Err(err) => match err {
            ContractError::Revert(err) => {
                let message = get_revert_message(err);
                info!("Revert: {}", message);
                Err(NonRepeatableError(message))
            }
            ContractError::DecodingError(err) => {
                error!("DecodingError: {}", err);
                Err(NonRepeatableError(err.to_string()))
            }
            ContractError::AbiError(err) => {
                error!("AbiError: {}", err);
                Err(NonRepeatableError(err.to_string()))
            }
            ContractError::DetokenizationError(err) => {
                error!("DetokenizationError: {}", err);
                Err(NonRepeatableError(err.to_string()))
            }
            ContractError::MiddlewareError { e } => {
                match e {
                    SignerMiddlewareError::MiddlewareError(e) => {
                        match e {
                            ProviderError::JsonRpcClientError(_) => {
                                error!("MiddlewareError: JsonRpcClientError: {}", e.to_string());
                                if e.to_string().contains("nonce") {
                                    return Err(NonceRecalculationRequired(function_call,format!("MiddlewareError: JsonRpcClientError: {}",e)))
                                }
                                Err(NonRepeatableError(format!("MiddlewareError: JsonRpcClientError: {}",e)))
                            },
                            ProviderError::HTTPError(_) => {
                                error!("MiddlewareError: HTTPError: {}", e.to_string());
                                Err(RepeatableError(format!("MiddlewareError: HTTPError: {}",e)))
                            },
                            ProviderError::SerdeJson(_) |
                            ProviderError::HexError(_) |
                            ProviderError::EnsError(_) |
                            ProviderError::EnsNotOwned(_) |
                            ProviderError::CustomError(_) |
                            ProviderError::UnsupportedRPC |
                            ProviderError::UnsupportedNodeClient |
                            ProviderError::SignerUnavailable => {
                                error!("MiddlewareError: {}", e.to_string());
                                Err(NonRepeatableError(format!("MiddlewareError: {}",e)))
                            }
                        }
                    },
                    SignerMiddlewareError::SignerError(_) |
                    SignerMiddlewareError::NonceMissing |
                    SignerMiddlewareError::GasPriceMissing |
                    SignerMiddlewareError::GasMissing |
                    SignerMiddlewareError::WrongSigner |
                    SignerMiddlewareError::DifferentChainID => {
                        error!("SignerMiddlewareError: {}", e.to_string());
                        Err(NonRepeatableError(format!("SignerMiddlewareError: {}",e)))
                    }
                }
            }
            ContractError::ProviderError { e } => {
                error!("ProviderError: {}", e);
                Err(NonRepeatableError(e.to_string()))
            }
            ContractError::ConstructorError => {
                error!("ConstructorError");
                Err(NonRepeatableError(err.to_string()))
            }
            ContractError::ContractNotDeployed => {
                error!("ContractNotDeployed");
                Err(NonRepeatableError(err.to_string()))
            }
        },
    }
}
async fn process_jump(
    json: JsonValue,
    contract: edcas_contract::EDCAS<SignerMiddleware<Provider<Http>, LocalWallet>>,
    nonce: U256,
) -> Result<TransactionReceipt, SendError> {
    debug!("Call register_system");
    let function_call: ContractCall = contract.register_system(
        json["SystemAddress"].as_u64().unwrap(),
        json["StarSystem"].to_string(),
        json["SystemGovernment"].to_string(),
        json["SystemAllegiance"].to_string(),
        json["SystemEconomy"].to_string(),
        json["SystemSecondEconomy"].to_string(),
        json["SystemSecurity"].to_string(),
        json["Population"].as_u64().unwrap_or(0),
        floating::generate_floating_from_string(json["StarPos"][0].to_string()),
        floating::generate_floating_from_string(json["StarPos"][1].to_string()),
        floating::generate_floating_from_string(json["StarPos"][2].to_string()),
        DateTime::parse_from_rfc3339(json["timestamp"].as_str().unwrap())
            .unwrap()
            .timestamp()
            .into(),
    );
    //execute_send(function_call).await;
    execute_send_repeatable(function_call, nonce).await
}

fn get_revert_message(bytes: Bytes) -> String {
    if bytes.len() < 134 {
        let n = bytes.split_at(134 / 2).1;
        let n: &[u8] = n.split(|b| *b == 0u8).next().unwrap();
        return String::from_utf8(n.to_vec()).unwrap();
    }
    bytes.to_string()
}
