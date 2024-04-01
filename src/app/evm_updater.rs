use crate::app::carrier::Carrier;
use crate::app::evm_updater::EvmUpdate::CarrierListUpdate;
use crate::app::settings::Settings;
use bus::Bus;
use chrono::DateTime;
use ethers::contract::ContractCall;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::{Http, LocalWallet, Provider};
use std::sync::Arc;

#[derive(Clone)]
pub enum EvmUpdate {
    CarrierListUpdate(Vec<Carrier>),
}
pub struct EvmUpdater {
    bus: Bus<EvmUpdate>,
    settings: Arc<Settings>,
}

pub fn initialize(bus: Bus<EvmUpdate>, settings: Arc<Settings>) -> EvmUpdater {
    EvmUpdater { bus, settings }
}

impl EvmUpdater {
    pub fn run_update(&mut self) {
        if let Some(contract) = &self.settings.evm_settings.contract {
            let carriers = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let function_call: ContractCall<
                        SignerMiddleware<Provider<Http>, LocalWallet>,
                        Vec<u64>,
                    > = contract.get_carrier_ids();
                    let mut carriers = Vec::new();
                    if let Ok(results) = function_call.legacy().call().await {
                        for carrier_id in results {
                            let function_call = contract.carrier_map(carrier_id);
                            if let Ok(result) = function_call.call().await {
                                let carrier = Carrier {
                                    name: result.1,
                                    callsign: result.2,
                                    services: result.3,
                                    docking_access: result.4,
                                    allow_notorious: result.5,
                                    current_system: result.6,
                                    current_body: result.7,
                                    next_system: result.8,
                                    next_body: result.9,
                                    departure: DateTime::from_timestamp(
                                        result.10.as_u64() as i64,
                                        0,
                                    )
                                    .unwrap(),
                                };
                                carriers.push(carrier);
                            } else {
                                return vec![];
                            }
                        }
                    }
                    carriers
                });
            if !carriers.is_empty() {
                self.bus.broadcast(CarrierListUpdate(carriers));
            }
        }
    }
}
