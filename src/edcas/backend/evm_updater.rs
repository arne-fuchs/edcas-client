use crate::edcas::backend::edcas_contract::{
    BodyProperties, Faction, Floating, PlanetProperties, StarProperties, StationIdentity,
};
use crate::edcas::carrier::Carrier;
use crate::edcas::evm_interpreter::Edcas;
use crate::edcas::evm_updater::EvmUpdate::{CarrierList, StationList};
use crate::edcas::explorer::body::{BodyType, Parent};
use crate::edcas::explorer::planet::Planet;
use crate::edcas::explorer::star::Star;
use crate::edcas::settings::Settings;
use crate::edcas::station::{CommodityListening, StationMetaData};
use bus::Bus;
use chrono::DateTime;
use ethers::contract::ContractCall;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::{Http, LocalWallet, Provider, U256};
use log::error;
use std::str::FromStr;
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
                                EvmRequest::StationCommodityListener(market_id) => {
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
                                EvmRequest::SystemMetaData(system_address) => {
                                    let function_call: ContractCall<
                                        SignerMiddleware<Provider<Http>, LocalWallet>,
                                        (U256, String, String, String, String, String, String, u64),
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
                                    let mut planet_list = vec![];

                                    let mut timestamp: U256 = U256::max_value();
                                    let mut index = 0;
                                    while timestamp != U256::from(0) {
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
                                                    body_id: result.1 as i64,
                                                    parents: vec![], //TODO Implement Parents
                                                    star_system: "".to_string(), //TODO Insert Star System
                                                    system_address: system_address as i64, //TODO Convert to u64
                                                    distance_from_arrival_ls: convert_floating(
                                                        result.6.distance_from_arrival_ls.decimal,
                                                        result
                                                            .6
                                                            .distance_from_arrival_ls
                                                            .floating_point,
                                                    ),
                                                    star_type: result.5.type_,
                                                    subclass: result.5.subclass as i64,
                                                    stellar_mass: result.5.age_my as f64,
                                                    radius: convert_floating(
                                                        result.6.radius.decimal,
                                                        result.6.radius.floating_point,
                                                    ),
                                                    absolute_magnitude: convert_floating(
                                                        result.5.absolute_magnitude.decimal,
                                                        result.5.absolute_magnitude.floating_point,
                                                    ),
                                                    age_my: result.5.age_my as i64,
                                                    surface_temperature: convert_floating(
                                                        result.6.surface_temperature.decimal,
                                                        result.6.surface_temperature.floating_point,
                                                    ),
                                                    luminosity: result.5.luminosity,
                                                    semi_major_axis: None, //TODO
                                                    eccentricity: None,    //TODO
                                                    orbital_inclination: None, //TODO
                                                    periapsis: None,       //TODO
                                                    orbital_period: None,  //TODO
                                                    ascending_node: None,  //TODO
                                                    mean_anomaly: None,    //TODO
                                                    rotation_period: convert_floating(
                                                        result.6.rotation_period.decimal,
                                                        result.6.rotation_period.floating_point,
                                                    ),
                                                    axial_tilt: convert_floating(
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
                                                timestamp = U256::from(0);
                                                error!("Error getting planet data: {err}");
                                            }
                                        }
                                        index += 1;
                                    }

                                    let mut timestamp: U256 = U256::max_value();
                                    let mut index = 0;
                                    while timestamp != U256::from(0) {
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
                                                    body_id: result.1 as i64,
                                                    parents: vec![Parent {
                                                        name: "Unknown".into(),
                                                        id: result.5.parent_id.into(),
                                                    }], //TODO Implement parents
                                                    star_system: "".to_string(), //TODO Insert Star System
                                                    system_address: system_address as i64, //TODO Convert to u64
                                                    distance_from_arrival_ls: convert_floating(
                                                        result.6.distance_from_arrival_ls.decimal,
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
                                                    mass_em: convert_floating(
                                                        result.5.mass_em.decimal,
                                                        result.5.mass_em.floating_point,
                                                    ),
                                                    radius: convert_floating(
                                                        result.6.radius.decimal,
                                                        result.6.radius.floating_point,
                                                    ),
                                                    surface_gravity: convert_floating(
                                                        result.5.surface_gravity.decimal,
                                                        result.5.surface_gravity.floating_point,
                                                    ),
                                                    surface_temperature: convert_floating(
                                                        result.6.surface_temperature.decimal,
                                                        result.6.surface_temperature.floating_point,
                                                    ),
                                                    surface_pressure: convert_floating(
                                                        result.5.surface_pressure.decimal,
                                                        result.5.surface_pressure.floating_point,
                                                    ),
                                                    landable: result.5.landable,
                                                    materials: vec![], //TODO Implement Materials
                                                    composition: vec![], //TODO Implement Composition
                                                    semi_major_axis: convert_floating(
                                                        result.5.semi_major_axis.decimal,
                                                        result.5.semi_major_axis.floating_point,
                                                    ),
                                                    eccentricity: convert_floating(
                                                        result.5.eccentricity.decimal,
                                                        result.5.eccentricity.floating_point,
                                                    ),
                                                    orbital_inclination: convert_floating(
                                                        result.5.orbital_inclination.decimal,
                                                        result.5.orbital_inclination.floating_point,
                                                    ),
                                                    periapsis: convert_floating(
                                                        result.5.periapsis.decimal,
                                                        result.5.periapsis.floating_point,
                                                    ),
                                                    orbital_period: convert_floating(
                                                        result.5.orbital_period.decimal,
                                                        result.5.orbital_period.floating_point,
                                                    ),
                                                    ascending_node: convert_floating(
                                                        result.5.ascending_node.decimal,
                                                        result.5.ascending_node.floating_point,
                                                    ),
                                                    mean_anomaly: convert_floating(
                                                        result.5.mean_anomaly.decimal,
                                                        result.5.mean_anomaly.floating_point,
                                                    ),
                                                    rotation_period: convert_floating(
                                                        result.6.rotation_period.decimal,
                                                        result.6.rotation_period.floating_point,
                                                    ),
                                                    axial_tilt: convert_floating(
                                                        result.6.axial_tilt.decimal,
                                                        result.6.axial_tilt.floating_point,
                                                    ),
                                                    was_discovered: result.3,
                                                    was_mapped: result.4,
                                                    reserve_level: "".to_string(), //What?
                                                    asteroid_rings: vec![], //TODO Implement Asteroid Rings
                                                    planet_signals: vec![], //TODO Implement Planet Signals
                                                    settings: self.settings.clone(),
                                                }))
                                            }
                                            Err(err) => {
                                                timestamp = U256::from(0);
                                                error!("Error getting planet data: {err}");
                                            }
                                        }
                                        index += 1;
                                    }
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
/**
Takes the decimal point and the floating point of a Floating Struct of the edcas_contract and converts it as f64
This function is not efficient, because it works with strings.
*/
pub fn convert_floating(decimal: i128, floating_point: u8) -> f64 {
    let mut eccentricity = "".to_string();
    if decimal < 0 {
        eccentricity.push('-');
    }
    for _ in 0..floating_point {
        eccentricity.push('0');
    }
    eccentricity.push_str(decimal.abs().to_string().as_str());
    eccentricity.insert(eccentricity.len() - floating_point as usize, '.');
    f64::from_str(eccentricity.as_str()).unwrap()
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
