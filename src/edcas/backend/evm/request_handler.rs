use crate::edcas::backend;
use crate::edcas::backend::evm::edcas_contract::*;
use crate::edcas::backend::evm::journal_interpreter::Edcas;
use crate::edcas::carrier::Carrier;
use crate::edcas::explorer::body::{BodyType, Parent, Signal};
use crate::edcas::explorer::planet::Planet;
use crate::edcas::explorer::star::Star;
use crate::edcas::request_handler::EvmUpdate::{CarrierList, StationList};
use crate::edcas::settings::Settings;
use crate::edcas::station::{CommodityListening, StationMetaData};
use bus::Bus;
use chrono::DateTime;
use ethers::contract::{ContractCall, ContractError};
use ethers::middleware::SignerMiddleware;
use ethers::prelude::{Http, LocalWallet, Provider, U256};
use log::{debug, error};
use std::sync::mpsc::Receiver;
use std::sync::Arc;

#[derive(Clone)]
pub enum EvmUpdate {
    CarrierList(Vec<Carrier>),
    StationList(Vec<StationIdentity>),
    StationMetaData(u64, StationMetaData),
    StationCommodityListening(u64, Vec<CommodityListening>),
    SystemMetaData(u64, SystemMetaData),
    PlanetList(u64, Vec<BodyType>),
    //System address, body id, signal
}
/**
+ StationMetaData(u64): MarketId from station
* StationCommodityListener(u64): MarketId from station
* SystemMetaData(u64): SystemAddress from system
*/
#[derive(Clone)]
pub enum EvmRequest {
    StationMetaData(u64),
    StationCommodityListener(u64),
    SystemMetaData(u64),
    SystemPlanetData(u64),
}

pub struct EvmUpdater {
    writer: Bus<EvmUpdate>,
    receiver: Receiver<EvmRequest>,
    settings: Arc<Settings>,
}

#[derive(Clone)]
pub struct SystemMetaData {
    pub name: String,
    pub address: u64,
    pub allegiance: String,
    pub economy: String,
    pub second_economy: String,
    pub government: String,
    pub security: String,
    pub population: String,
    pub body_count: u8,
    pub x: f64,
    pub y: f64,
    pub z: f64,
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
                                EvmRequest::StationMetaData(market_id) => {
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
                                            self.writer.broadcast(EvmUpdate::StationMetaData(
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
                                EvmRequest::StationCommodityListener(_market_id) => {
                                    todo!("Implement");
                                    let function_call: ContractCall<
                                        SignerMiddleware<Provider<Http>, LocalWallet>,
                                        (u32, u32, u32, u32, u32, u32, u32),
                                    > = contract
                                        .commodity_listening_map(_market_id, "".to_string());
                                    match function_call.legacy().call().await {
                                        Ok(_result) => {}
                                        Err(err) => {
                                            error!("Error getting station metadata: {err}");
                                        }
                                    }
                                }
                                EvmRequest::SystemMetaData(system_address) => {
                                    let function_call: ContractCall<
                                        SignerMiddleware<Provider<Http>, LocalWallet>,
                                        (
                                            U256,
                                            String,
                                            String,
                                            String,
                                            String,
                                            String,
                                            String,
                                            u64,
                                            u8,
                                            Floating,
                                            Floating,
                                            Floating,
                                        ),
                                    > = contract.system_map(system_address);
                                    match function_call.legacy().call().await {
                                        Ok(result) => {
                                            self.writer.broadcast(EvmUpdate::SystemMetaData(
                                                system_address,
                                                SystemMetaData {
                                                    name: result.1,
                                                    address: system_address,
                                                    allegiance: result.2,
                                                    economy: result.4,
                                                    second_economy: result.5,
                                                    government: result.3,
                                                    security: result.6,
                                                    population: result.7.to_string(),
                                                    body_count: result.8,
                                                    x: backend::floating::floating_to_f64(
                                                        result.9.decimal,
                                                        result.9.floating_point,
                                                    ),
                                                    y: backend::floating::floating_to_f64(
                                                        result.10.decimal,
                                                        result.10.floating_point,
                                                    ),
                                                    z: backend::floating::floating_to_f64(
                                                        result.11.decimal,
                                                        result.11.floating_point,
                                                    ),
                                                },
                                            ))
                                        }
                                        Err(err) => {
                                            error!("Error getting system metadata: {err}");
                                        }
                                    }
                                }
                                EvmRequest::SystemPlanetData(system_address) => {
                                    //TODO Stars and Belt Cluster
                                    debug!("Call get_highest_body_id_from_system: {system_address}");
                                    let function_call: ContractCall<
                                        SignerMiddleware<Provider<Http>, LocalWallet>,
                                        u8,
                                    > = contract.get_highest_body_id_from_system(system_address);
                                    let highest_id = match function_call.legacy().call().await {
                                        Ok(result) => {
                                            debug!("highest_id received: {}", result);
                                            result
                                        }
                                        Err(_) => 0,
                                    };

                                    let mut planet_list = vec![];

                                    let mut timestamp: U256 = U256::max_value();
                                    let mut index = 1;
                                    while index <= highest_id
                                        || (highest_id == 0 && timestamp != U256::from(0)) {
                                        debug!("Call star_map: {system_address}-{index}");
                                        let function_call: ContractCall<
                                            SignerMiddleware<Provider<Http>, LocalWallet>,
                                            (
                                                U256,
                                                u8,
                                                String,
                                                bool,
                                                bool,
                                                StarProperties,
                                                BodyProperties,
                                            ),
                                        > = contract.star_map(system_address, index.into());
                                        match function_call.legacy().call().await {
                                            Ok(result) => {
                                                planet_list.push(BodyType::Star(Star {
                                                    timestamp: result.0.to_string(),
                                                    event: "EVM".to_string(),
                                                    scan_type: "N/A".to_string(),
                                                    body_name: result.2,
                                                    body_id: result.1 as u64,
                                                    parents: vec![], //TODO Implement Parents
                                                    star_system: "".to_string(), //TODO Insert Star System
                                                    system_address: system_address as i64, //TODO Convert to u64
                                                    distance_from_arrival_ls:
                                                    backend::floating::floating_to_f64(
                                                        result
                                                            .6
                                                            .distance_from_arrival_ls
                                                            .decimal,
                                                        result
                                                            .6
                                                            .distance_from_arrival_ls
                                                            .floating_point,
                                                    ),
                                                    star_type: result.5.type_,
                                                    subclass: result.5.subclass as i64,
                                                    stellar_mass: result.5.age_my as f64,
                                                    radius: backend::floating::floating_to_f64(
                                                        result.6.radius.decimal,
                                                        result.6.radius.floating_point,
                                                    ),
                                                    absolute_magnitude:
                                                    backend::floating::floating_to_f64(
                                                        result.5.absolute_magnitude.decimal,
                                                        result
                                                            .5
                                                            .absolute_magnitude
                                                            .floating_point,
                                                    ),
                                                    age_my: result.5.age_my as i64,
                                                    surface_temperature:
                                                    backend::floating::floating_to_f64(
                                                        result.6.surface_temperature.decimal,
                                                        result
                                                            .6
                                                            .surface_temperature
                                                            .floating_point,
                                                    ),
                                                    luminosity: result.5.luminosity,
                                                    semi_major_axis: None, //TODO
                                                    eccentricity: None,    //TODO
                                                    orbital_inclination: None, //TODO
                                                    periapsis: None,       //TODO
                                                    orbital_period: None,  //TODO
                                                    ascending_node: None,  //TODO
                                                    mean_anomaly: None,    //TODO
                                                    rotation_period:
                                                    backend::floating::floating_to_f64(
                                                        result.6.rotation_period.decimal,
                                                        result.6.rotation_period.floating_point,
                                                    ),
                                                    axial_tilt: backend::floating::floating_to_f64(
                                                        result.6.axial_tilt.decimal,
                                                        result.6.axial_tilt.floating_point,
                                                    ),
                                                    was_discovered: result.3,
                                                    was_mapped: result.4,
                                                    asteroid_rings: vec![], //TODO
                                                    settings: self.settings.clone(),
                                                }))
                                            }
                                            Err(err) => {
                                                error!("Error getting star data: {err}");

                                                debug!("Call get_planet_signals: {system_address}-{index}");
                                                let function_call: ContractCall<
                                                    SignerMiddleware<Provider<Http>, LocalWallet>,
                                                    Vec<PlanetSignal>,
                                                > = contract.get_planet_signals(system_address, index);
                                                let planet_signals: Vec<Signal> = function_call
                                                    .legacy()
                                                    .call()
                                                    .await
                                                    .unwrap_or_else(|err| {
                                                        error!("Error getting planet signal data: {err}");
                                                        vec![]
                                                    })
                                                    .iter()
                                                    .map(|planet_signal| {
                                                        let r#type: String = match planet_signal.type_ {
                                                            4 => "$SAA_SignalType_Human;".into(),
                                                            3 => "$SAA_SignalType_Biological;".into(),
                                                            2 => "$SAA_SignalType_Xenological;".into(),
                                                            1 => "$SAA_SignalType_Geological;".into(),
                                                            _ => "Unknown".into(),
                                                        };
                                                        let type_localised = r#type
                                                            .split('_')
                                                            .last()
                                                            .unwrap_or("Unknown;")
                                                            .replace(";", "")
                                                            .to_string();
                                                        Signal {
                                                            r#type,
                                                            type_localised,
                                                            count: planet_signal.count as u64,
                                                        }
                                                    })
                                                    .collect();
                                                debug!("Call planet_map: {system_address}-{index}");
                                                let function_call: ContractCall<
                                                    SignerMiddleware<Provider<Http>, LocalWallet>,
                                                    (
                                                        U256,
                                                        u8,
                                                        String,
                                                        bool,
                                                        bool,
                                                        PlanetProperties,
                                                        BodyProperties,
                                                    ),
                                                > = contract.planet_map(system_address, index.into());
                                                match function_call.legacy().call().await {
                                                    Ok(result) => {
                                                        planet_list.push(BodyType::Planet(Planet {
                                                            timestamp: result.0.to_string(),
                                                            event: "EVM".to_string(),
                                                            scan_type: "N/A".to_string(),
                                                            body_name: result.2,
                                                            body_id: result.1 as u64,
                                                            parents: vec![Parent {
                                                                name: "Unknown".into(),
                                                                id: result.5.parent_id.into(),
                                                            }], //TODO Implement parents
                                                            star_system: "".to_string(), //TODO Insert Star System
                                                            system_address: system_address as i64, //TODO Convert to u64
                                                            distance_from_arrival_ls:
                                                            backend::floating::floating_to_f64(
                                                                result
                                                                    .6
                                                                    .distance_from_arrival_ls
                                                                    .decimal,
                                                                result
                                                                    .6
                                                                    .distance_from_arrival_ls
                                                                    .floating_point,
                                                            ),
                                                            tidal_lock: result.5.tidal_lock,
                                                            terraform_state: result.5.terraform_state,
                                                            planet_class: result.5.class,
                                                            atmosphere: result.5.atmosphere,
                                                            atmosphere_type: "".to_string(), //TODO Implement Atmosphere Type
                                                            atmosphere_composition: vec![], //TODO Implement Atmosphere Composition
                                                            volcanism: result.5.volcanism,
                                                            mass_em: backend::floating::floating_to_f64(
                                                                result.5.mass_em.decimal,
                                                                result.5.mass_em.floating_point,
                                                            ),
                                                            radius: backend::floating::floating_to_f64(
                                                                result.6.radius.decimal,
                                                                result.6.radius.floating_point,
                                                            ),
                                                            surface_gravity:
                                                            backend::floating::floating_to_f64(
                                                                result.5.surface_gravity.decimal,
                                                                result.5.surface_gravity.floating_point,
                                                            ),
                                                            surface_temperature:
                                                            backend::floating::floating_to_f64(
                                                                result.6.surface_temperature.decimal,
                                                                result
                                                                    .6
                                                                    .surface_temperature
                                                                    .floating_point,
                                                            ),
                                                            surface_pressure:
                                                            backend::floating::floating_to_f64(
                                                                result.5.surface_pressure.decimal,
                                                                result
                                                                    .5
                                                                    .surface_pressure
                                                                    .floating_point,
                                                            ),
                                                            landable: result.5.landable,
                                                            materials: vec![], //TODO Implement Materials
                                                            composition: vec![], //TODO Implement Composition
                                                            semi_major_axis:
                                                            backend::floating::floating_to_f64(
                                                                result.5.semi_major_axis.decimal,
                                                                result.5.semi_major_axis.floating_point,
                                                            ),
                                                            eccentricity:
                                                            backend::floating::floating_to_f64(
                                                                result.5.eccentricity.decimal,
                                                                result.5.eccentricity.floating_point,
                                                            ),
                                                            orbital_inclination:
                                                            backend::floating::floating_to_f64(
                                                                result.5.orbital_inclination.decimal,
                                                                result
                                                                    .5
                                                                    .orbital_inclination
                                                                    .floating_point,
                                                            ),
                                                            periapsis: backend::floating::floating_to_f64(
                                                                result.5.periapsis.decimal,
                                                                result.5.periapsis.floating_point,
                                                            ),
                                                            orbital_period:
                                                            backend::floating::floating_to_f64(
                                                                result.5.orbital_period.decimal,
                                                                result.5.orbital_period.floating_point,
                                                            ),
                                                            ascending_node:
                                                            backend::floating::floating_to_f64(
                                                                result.5.ascending_node.decimal,
                                                                result.5.ascending_node.floating_point,
                                                            ),
                                                            mean_anomaly:
                                                            backend::floating::floating_to_f64(
                                                                result.5.mean_anomaly.decimal,
                                                                result.5.mean_anomaly.floating_point,
                                                            ),
                                                            rotation_period:
                                                            backend::floating::floating_to_f64(
                                                                result.6.rotation_period.decimal,
                                                                result.6.rotation_period.floating_point,
                                                            ),
                                                            axial_tilt: backend::floating::floating_to_f64(
                                                                result.6.axial_tilt.decimal,
                                                                result.6.axial_tilt.floating_point,
                                                            ),
                                                            was_discovered: result.3,
                                                            was_mapped: result.4,
                                                            reserve_level: "".to_string(), //What?
                                                            asteroid_rings: vec![], //TODO Implement Asteroid Rings
                                                            planet_signals,
                                                            settings: self.settings.clone(),
                                                        }))
                                                    }
                                                    Err(err) => {
                                                        timestamp = U256::from(0);
                                                        error!("Error getting planet data: {err}");
                                                    }
                                                }
                                            }
                                        }
                                        index += 1;
                                    }
                                    planet_list.sort_by_key(|planet_a| planet_a.get_id());
                                    self.writer.broadcast(EvmUpdate::PlanetList(
                                        system_address,
                                        planet_list,
                                    ))
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
                            self.writer.broadcast(CarrierList(carriers));
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
                                    self.writer.broadcast(StationList(results));
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
