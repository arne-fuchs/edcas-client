use std::env;
use std::sync::Arc;
use std::time::Duration;

use bus::BusReader;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use ethers::utils::hex::hex;
use json::JsonValue;
use log::{error, info};

use edcas_contract::{BodyProperties, PlanetProperties, StarProperties};

use crate::app::evm_interpreter::SendError::{NonRepeatableError, RepeatableError};
use crate::app::settings::Settings;

mod edcas_contract;

pub struct EvmInterpreter {
    bus: BusReader<JsonValue>,
    settings: Arc<Settings>,
    contract: edcas_contract::EDCAS<SignerMiddleware<Provider<Http>, LocalWallet>>,
}

impl EvmInterpreter {
    pub fn run(&mut self) {
        let Self {
            bus,
            settings: _,
            contract: _,
        } = self;

        match bus.recv() {
            Err(_) => {}
            Ok(json) => {
                //let json = json.clone();
                let event = json["event"].as_str().unwrap();

                info!("EVM event received: {}", event);

                match event {
                    "FSDJump" | "CarrierJump" => {
                        //TODO Learn tokio
                        tokio::runtime::Builder::new_multi_thread()
                            .enable_all()
                            .build()
                            .unwrap()
                            .block_on(async move {
                                let contract = self.contract.clone();
                                let function_call: FunctionCall<
                                    Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
                                    SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                                    (),
                                > = contract.register_system(
                                    json["SystemAddress"].as_u64().unwrap(),
                                    json["StarSystem"].to_string(),
                                    json["SystemAllegiance"].to_string(),
                                    json["SystemEconomy"].to_string(),
                                    json["SystemSecondEconomy"].to_string(),
                                    json["SystemSecurity"].to_string(),
                                    json["Population"].as_u64().unwrap_or(0),
                                );
                                //execute_send(function_call).await;
                                execute_send_repeatable(function_call).await;
                            });
                    }
                    "Scan" => {
                        //TODO Learn tokio

                        tokio::runtime::Builder::new_multi_thread()
                            .enable_all()
                            .build()
                            .unwrap()
                            .block_on(async move {
                                let contract = self.contract.clone();
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
                                        let function_call: FunctionCall<
                                            Arc<
                                                SignerMiddleware<
                                                    Provider<Http>,
                                                    Wallet<SigningKey>,
                                                >,
                                            >,
                                            SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                                            (),
                                        > = contract.register_planet(
                                            json["SystemAddress"].as_u64().unwrap(),
                                            json["BodyID"].as_u8().unwrap(),
                                            json["BodyName"].to_string(),
                                            json["WasDiscovered"].as_bool().unwrap(),
                                            json["WasMapped"].as_bool().unwrap(),
                                            extract_planet_properties(&json),
                                            extract_body_properties(&json),
                                        );
                                        execute_send_repeatable(function_call).await;
                                    } else {
                                        //Star
                                        //{"AbsoluteMagnitude":8.518448,"Age_MY":446,"AxialTilt":0,"BodyID":0,"BodyName":"Hyades Sector BB-N b7-5",
                                        // "DistanceFromArrivalLS":0,"Luminosity":"Va","Radius":374854272.0,"RotationPeriod":192595.293946,"ScanType":"AutoScan",
                                        // "StarPos":[12.1875,-74.90625,-120.5],"StarSystem":"Hyades Sector BB-N b7-5","StarType":"M","StellarMass":0.394531,"Subclass":1,
                                        // "SurfaceTemperature":3367.0,"SystemAddress":11666070513017,"WasDiscovered":true,"WasMapped":false,"event":"Scan","horizons":true,
                                        // "odyssey":true,"timestamp":"2024-03-26T21:27:53Z"}
                                        let function_call: FunctionCall<
                                            Arc<
                                                SignerMiddleware<
                                                    Provider<Http>,
                                                    Wallet<SigningKey>,
                                                >,
                                            >,
                                            SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                                            (),
                                        > = contract.register_star(
                                            json["SystemAddress"].as_u64().unwrap(),
                                            json["BodyID"].as_u8().unwrap(),
                                            json["BodyName"].to_string(),
                                            json["WasDiscovered"].as_bool().unwrap(),
                                            json["WasMapped"].as_bool().unwrap(),
                                            extract_star_properties(&json),
                                            extract_body_properties(&json),
                                        );
                                        execute_send_repeatable(function_call).await;
                                    }
                                } else {
                                    //TODO Interpret Belt Cluster and Ring
                                }
                            });
                    }
                    _ => {}
                }
            }
        }
    }
}
pub fn initialize(bus_reader: BusReader<JsonValue>, settings: Arc<Settings>) -> EvmInterpreter {
    info!("Loading wallet");

    let node_url = settings.evm_settings.url.as_str();
    let pirvate_key = settings.evm_settings.private_key.as_str();
    let retry = settings.evm_settings.n_attempts;
    let timeout = settings.evm_settings.n_timeout;
    let sc_address = settings.evm_settings.smart_contract_address.as_str();

    info!("Using URL:{}", &node_url);

    let contract = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let provider = Provider::connect(node_url).await;

            let wallet: LocalWallet = pirvate_key.parse::<LocalWallet>().unwrap();
            info!("EVM Address: {:?}", wallet.address());

            let mut result =
                SignerMiddleware::new_with_provider_chain(provider.clone(), wallet.clone()).await;
            let mut retries = 0;
            while result.is_err() && retries < retry {
                retries += 1;
                tokio::time::sleep(Duration::from_secs(timeout)).await;
                result =
                    SignerMiddleware::new_with_provider_chain(provider.clone(), wallet.clone())
                        .await;
            }

            let client: SignerMiddleware<Provider<Http>, LocalWallet> = result.unwrap();

            let edcas_address = sc_address.parse::<Address>().unwrap();

            edcas_contract::EDCAS::new(edcas_address, Arc::new(client.clone()))
        });

    EvmInterpreter {
        bus: bus_reader,
        settings,
        contract,
    }
}

fn extract_planet_properties(json: &JsonValue) -> PlanetProperties {
    PlanetProperties {
        atmosphere: json["Atmosphere"].to_string(),
        class: json["PlanetClass"].to_string(),
        landable: json["Landable"].as_bool().unwrap_or(false),
        terraform_state: json["TerraformState"].to_string(),
        volcanism: json["Volcanism"].to_string(),
        tidal_lock: json["TidalLock"]
            .as_bool()
            .unwrap_or_else(|| panic!("Tidal Lock not parseable {}", json)),
        mass_em: edcas_contract::Floating {
            decimal: json["MassEM"].to_string().replace('.', "").parse().unwrap(),
            floating_point: json["MassEM"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
        surface_gravity: edcas_contract::Floating {
            decimal: json["SurfaceGravity"]
                .to_string()
                .replace('.', "")
                .parse()
                .unwrap(),
            floating_point: json["SurfaceGravity"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
        surface_pressure: edcas_contract::Floating {
            decimal: json["SurfacePressure"]
                .to_string()
                .replace('.', "")
                .parse()
                .unwrap(),
            floating_point: json["SurfacePressure"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
        ascending_node: edcas_contract::Floating {
            decimal: json["AscendingNode"]
                .to_string()
                .replace('.', "")
                .parse()
                .unwrap_or(0),
            floating_point: json["AscendingNode"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
        eccentricity: edcas_contract::Floating {
            decimal: json["Eccentricity"]
                .to_string()
                .replace('.', "")
                .parse()
                .unwrap_or_else(|_| panic!("Eccentricity invalid parse: {}", json)),
            floating_point: json["Eccentricity"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
        mean_anomaly: edcas_contract::Floating {
            decimal: json["MeanAnomaly"]
                .to_string()
                .replace('.', "")
                .parse()
                .unwrap_or(0),
            floating_point: json["MeanAnomaly"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
        orbital_inclination: edcas_contract::Floating {
            decimal: json["OrbitalInclination"]
                .to_string()
                .replace('.', "")
                .parse()
                .unwrap(),
            floating_point: json["OrbitalInclination"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
        orbital_period: edcas_contract::Floating {
            decimal: json["OrbitalPeriod"]
                .to_string()
                .replace('.', "")
                .parse()
                .unwrap(),
            floating_point: json["OrbitalPeriod"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
        periapsis: edcas_contract::Floating {
            decimal: json["Periapsis"]
                .to_string()
                .replace('.', "")
                .parse()
                .unwrap(),
            floating_point: json["Periapsis"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
        semi_major_axis: edcas_contract::Floating {
            decimal: json["SemiMajorAxis"]
                .to_string()
                .replace('.', "")
                .parse()
                .unwrap(),
            floating_point: json["SemiMajorAxis"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
    }
}
fn extract_star_properties(json: &JsonValue) -> StarProperties {
    StarProperties {
        subclass: json["Subclass"].as_u8().unwrap(),
        age_my: json["Age_MY"].as_u16().unwrap(),
        type_: json["StarType"].to_string(),
        luminosity: json["Luminosity"].to_string(),
        stellar_mass: edcas_contract::Floating {
            decimal: json["StellarMass"]
                .to_string()
                .replace('.', "")
                .parse()
                .unwrap(),
            floating_point: json["StellarMass"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
        absolute_magnitude: edcas_contract::Floating {
            decimal: json["AbsoluteMagnitude"]
                .to_string()
                .replace('.', "")
                .parse()
                .unwrap_or_else(|_| panic!("AbsoluteMagnitude parse error: {}", json)),
            floating_point: json["AbsoluteMagnitude"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
    }
}
fn extract_body_properties(json: &JsonValue) -> BodyProperties {
    BodyProperties {
        radius: edcas_contract::Floating {
            decimal: json["Radius"].to_string().replace('.', "").parse().unwrap(),
            floating_point: json["Radius"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
        distance_from_arrival_ls: edcas_contract::Floating {
            decimal: json["DistanceFromArrivalLS"]
                .to_string()
                .replace('.', "")
                .parse()
                .unwrap(),
            floating_point: json["DistanceFromArrivalLS"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
        axial_tilt: edcas_contract::Floating {
            decimal: json["AxialTilt"]
                .to_string()
                .replace('.', "")
                .parse()
                .unwrap(),
            floating_point: json["AxialTilt"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
        rotation_period: edcas_contract::Floating {
            decimal: json["RotationPeriod"]
                .to_string()
                .replace('.', "")
                .parse()
                .unwrap(),
            floating_point: json["RotationPeriod"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
        surface_temperature: edcas_contract::Floating {
            decimal: json["SurfaceTemperature"]
                .to_string()
                .replace('.', "")
                .parse()
                .unwrap(),
            floating_point: json["SurfaceTemperature"]
                .to_string()
                .split('.')
                .nth(1)
                .unwrap_or("")
                .len() as u8,
        },
    }
}
enum SendError {
    RepeatableError(String),
    NonRepeatableError(String),
}
async fn execute_send_repeatable(
    function_call: FunctionCall<
        Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
        SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
        (),
    >,
) {
    while let Err(err) = execute_send(function_call.clone()).await {
        match err {
            RepeatableError(_) => {
                tokio::time::sleep(Duration::from_secs(
                    env::var("DURATION_TIMEOUT")
                        .unwrap_or("5".into())
                        .parse()
                        .unwrap_or(5),
                ))
                .await;
            }
            NonRepeatableError(_) => break,
        }
    }
}
async fn execute_send(
    function_call: FunctionCall<
        Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
        SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
        (),
    >,
) -> Result<H256, SendError> {
    match function_call.legacy().send().await {
        Ok(pending) => match pending.await {
            Ok(receipt) => {
                if let Some(receipt) = receipt {
                    if let Some(hash) = receipt.block_hash {
                        info!("{:?}", hash);
                        Ok(hash)
                    } else {
                        Err(NonRepeatableError("Receipt without hash".into()))
                    }
                } else {
                    Err(NonRepeatableError("No Receipt".into()))
                }
            }
            Err(err) => match err {
                ProviderError::JsonRpcClientError(err) => {
                    error!("JsonRpcClientError: {}", err);
                    Err(RepeatableError(err.to_string()))
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
                error!("Revert: {}", message);
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
                error!("MiddlewareError: {}", e.to_string());
                Err(RepeatableError(e.to_string()))
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

fn get_revert_message(bytes: Bytes) -> String {
    match hex::encode(&bytes).as_str() {
        "08c379a00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000001953797374656d206e6f7420726567697374657265642079657400000000000000" => {
            String::from("System not registered yet")
        }
        &_ => {
            hex::encode(&bytes).replace("000000",".")
        }
    }
}
