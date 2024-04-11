use crate::app::carrier::Carrier;
use crate::app::evm_interpreter::edcas_contract::{Faction, Floating, StationIdentity};
use crate::app::evm_interpreter::Edcas;
use crate::app::evm_updater::EvmUpdate::{CarrierListUpdate, StationListUpdate};
use crate::app::explorer::system::System;
use crate::app::settings::Settings;
use crate::app::station::{CommodityListening, StationMetaData};
use bus::Bus;
use chrono::DateTime;
use ethers::contract::ContractCall;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::{Http, LocalWallet, Provider, U256};
use log::error;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

#[derive(Clone)]
pub enum EvmUpdate {
    CarrierListUpdate(Vec<Carrier>),
    StationListUpdate(Vec<StationIdentity>),
    StationMetaDataUpdate(u64, StationMetaData),
    StationCommodityListeningUpdate(u64, Vec<CommodityListening>),
}

#[derive(Clone)]
pub enum EvmRequest {
    StationMetaDataRequest(u64),
    StationCommodityListenerRequest(u64),
    SystemMetaDataRequest(u64),
}

pub struct EvmUpdater {
    writer: Bus<EvmUpdate>,
    receiver: Receiver<EvmRequest>,
    settings: Arc<Settings>,
}

pub fn initialize(
    writer: Bus<EvmUpdate>,
    receiver: Receiver<EvmRequest>,
    settings: Arc<Settings>,
) -> EvmUpdater {
    EvmUpdater {
        writer,
        receiver,
        settings,
    }
}

impl EvmUpdater {
    pub fn run_update(&mut self) {
        if let Some(contract) = &self.settings.evm_settings.contract {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    //Receive Requests
                    {
                        if let Ok(request) = self.receiver.try_recv() {
                            match request {
                                EvmRequest::StationMetaDataRequest(market_id) => {
                                    let function_call: ContractCall<
                                        SignerMiddleware<Provider<Http>, LocalWallet>,
                                        (
                                            bool,
                                            U256,
                                            u64,
                                            String,
                                            Faction,
                                            String,
                                            String,
                                            String,
                                            Floating,
                                            String,
                                        ),
                                    > = contract.station_map(market_id);
                                    match function_call.legacy().call().await {
                                        Ok(result) => {
                                            self.writer.broadcast(EvmUpdate::StationMetaDataUpdate(
                                                market_id,
                                                StationMetaData {
                                                    timestamp: DateTime::from_timestamp(
                                                        result.1.as_u64() as i64,
                                                        0,
                                                    )
                                                    .unwrap(),
                                                    services: result.7,
                                                    system_name: result.3,
                                                    faction: result.4,
                                                    government: result.5,
                                                    economy: result.6,
                                                    distance: result.8,
                                                    landingpads: result.9,
                                                },
                                            ))
                                        }
                                        Err(err) => {
                                            error!("Error getting station metadata: {err}");
                                        }
                                    }
                                }
                                EvmRequest::StationCommodityListenerRequest(market_id) => {
                                    todo!("Implement");
                                    let function_call: ContractCall<
                                        SignerMiddleware<Provider<Http>, LocalWallet>,
                                        (u32, u32, u32, u32, u32, u32, u32),
                                    > = contract.commodity_listening_map(market_id, "".to_string());
                                    match function_call.legacy().call().await {
                                        Ok(result) => {}
                                        Err(err) => {
                                            error!("Error getting station metadata: {err}");
                                        }
                                    }
                                }
                                EvmRequest::SystemMetaDataRequest(system_address) => {
                                    todo!("Implement");
                                }
                            }
                        }
                    }

                    //Update carrier list
                    {
                        let function_call: ContractCall<
                            SignerMiddleware<Provider<Http>, LocalWallet>,
                            Vec<u64>,
                        > = contract.get_carrier_ids();
                        let mut carriers = Vec::new();
                        match function_call.legacy().call().await {
                            Ok(results) => {
                                for carrier_id in results {
                                    if let Ok(carrier) = get_carrier(contract, carrier_id).await {
                                        carriers.push(carrier);
                                    }
                                }
                            }
                            Err(err) => {
                                error!("Error getting carriers ids: {err}");
                            }
                        }
                        if !carriers.is_empty() {
                            self.writer.broadcast(CarrierListUpdate(carriers));
                        }
                    }

                    //Update station list
                    {
                        let function_call: ContractCall<
                            SignerMiddleware<Provider<Http>, LocalWallet>,
                            Vec<StationIdentity>,
                        > = contract.get_stations();
                        match function_call.legacy().call().await {
                            Ok(mut results) => {
                                if !results.is_empty() {
                                    results.sort_by_key(|a| a.name.clone());
                                    self.writer.broadcast(StationListUpdate(results));
                                }
                            }
                            Err(err) => {
                                error!("Error getting station ids: {err}");
                            }
                        }
                    }
                });
        }
    }
}
pub async fn get_carrier(contract: &Edcas, carrier_id: u64) -> Result<Carrier, ()> {
    let function_call = contract.carrier_map(carrier_id);
    match function_call.call().await {
        Ok(result) => {
            let carrier = Carrier {
                timestamp: DateTime::from_timestamp(result.1.as_u64() as i64, 0).unwrap(),
                name: result.2,
                callsign: result.3,
                services: result.4,
                docking_access: result.5,
                allow_notorious: result.6,
                current_system: result.7,
                current_body: result.8,
                next_system: result.9,
                next_body: result.10,
                departure: DateTime::from_timestamp(result.11.as_u64() as i64, 0).unwrap(),
            };
            Ok(carrier)
        }
        Err(err) => {
            error!("Error getting carriers: {err}");
            Err(())
        }
    }
}
