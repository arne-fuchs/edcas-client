use std::env;
use std::sync::Arc;
use std::time::Duration;

use bus::BusReader;
use chrono::DateTime;
use ethers::core::k256::ecdsa::SigningKey;
use ethers::prelude::*;
use json::JsonValue;
use log::{debug, error, info};

use crate::edcas::backend::evm::edcas_contract;
use crate::edcas::backend::evm::edcas_contract::{
    BodyProperties, PlanetProperties, StarProperties,
};
use crate::edcas::backend::evm::journal_interpreter::SendError::{
    NonRepeatableError, RepeatableError,
};
use crate::edcas::backend::floating;

use crate::edcas::settings::EvmSettings;

pub type Edcas = edcas_contract::EDCAS<SignerMiddleware<Provider<Http>, LocalWallet>>;
pub type ContractCall = FunctionCall<
    Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
    (),
>;

pub struct EvmInterpreter {
    bus: BusReader<JsonValue>,
    contract: Edcas,
}

impl EvmInterpreter {
    pub fn run_loop(&mut self) {
        let Self { bus, contract: _ } = self;
        let contract = self.contract.clone();
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                loop {
                    let contract = contract.clone();
                    match bus.recv() {
                        Err(_) => {}
                        Ok(json) => {
                            //let json = json.clone();
                            let event = json["event"].as_str().unwrap_or("");

                            debug!("EVM event received: {}", event);

                            match event {
                                "FSDJump" => {
                                    tokio::spawn(async move {
                                        let _ = process_jump(json, contract).await;
                                    });
                                }
                                "FSSDiscoveryScan" => {
                                    tokio::spawn(async move {
                                        let system_address = json["SystemAddress"].as_u64().unwrap();
                                        let body_count = json["BodyCount"].as_u8().unwrap();
                                        debug!("Call set_body_count: {system_address}-{body_count}");
                                        let function_call: FunctionCall<
                                            Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
                                            SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                                            (),
                                        > = contract.set_body_count(
                                            system_address,
                                            body_count,
                                            DateTime::parse_from_rfc3339(json["timestamp"].as_str().unwrap())
                                                .unwrap()
                                                .timestamp()
                                                .into(),
                                        );
                                        //execute_send(function_call).await;
                                        let _ = execute_send_repeatable(function_call).await;
                                    });
                                }
                                "Scan" => {
                                    tokio::spawn(async move {
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
                                                debug!(
                                        "Call register_planet: {system_address}-{body_id}-{}",
                                        json["BodyName"]
                                    );
                                                let function_call: FunctionCall<
                                                    Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
                                                    SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                                                    (),
                                                > = contract.register_planet(
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
                                                match execute_send_repeatable(function_call).await {
                                                    Ok(receipt) => {
                                                        debug!("Call register_planet successful {}-{}: {:?} - BlockNr.{:?}",system_address,body_id,receipt.transaction_hash,receipt.block_number);
                                                    }
                                                    Err(error) => {
                                                        debug!(
                                                "Call register_planet failed {}-{}: {}",
                                                system_address, body_id, error
                                            );
                                                    }
                                                }
                                            } else {
                                                //Star
                                                //{"AbsoluteMagnitude":8.518448,"Age_MY":446,"AxialTilt":0,"BodyID":0,"BodyName":"Hyades Sector BB-N b7-5",
                                                // "DistanceFromArrivalLS":0,"Luminosity":"Va","Radius":374854272.0,"RotationPeriod":192595.293946,"ScanType":"AutoScan",
                                                // "StarPos":[12.1875,-74.90625,-120.5],"StarSystem":"Hyades Sector BB-N b7-5","StarType":"M","StellarMass":0.394531,"Subclass":1,
                                                // "SurfaceTemperature":3367.0,"SystemAddress":11666070513017,"WasDiscovered":true,"WasMapped":false,"event":"Scan","horizons":true,
                                                // "odyssey":true,"timestamp":"2024-03-26T21:27:53Z"}
                                                let body_id = json["BodyID"].as_u8().unwrap();
                                                let system_address = json["SystemAddress"].as_u64().unwrap();
                                                debug!(
                                        "Call register_star: {system_address}-{body_id}-{}",
                                        json["BodyName"]
                                    );
                                                let function_call: FunctionCall<
                                                    Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
                                                    SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                                                    (),
                                                > = contract.register_star(
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
                                                match execute_send_repeatable(function_call).await {
                                                    Ok(receipt) => {
                                                        debug!("Call register_star successful {}-{}: {:?} - BlockNr.{:?}",system_address,body_id,receipt.transaction_hash.to_string(),receipt.block_number);
                                                    }
                                                    Err(error) => {
                                                        debug!(
                                                "Call register_star failed {}-{}: {}",
                                                system_address, body_id, error
                                            );
                                                    }
                                                }
                                            }
                                        } else {
                                            debug!("Belt Cluster -> unimplemented")
                                            //TODO Interpret Belt Cluster and Ring
                                        }
                                    });
                                }
                                //{ "timestamp":"2022-07-07T20:58:06Z", "event":"SAASignalsFound", "BodyName":"IC 2391 Sector YE-A d103 B 1", "SystemAddress":3549631072611, "BodyID":15, "Signals":[ { "Type":"$SAA_SignalType_Guardian;", "Type_Localised":"Guardian", "Count":1 }, { "Type":"$SAA_SignalType_Human;", "Type_Localised":"Menschlich", "Count":9 } ] }
                                //{ "timestamp":"2022-09-07T17:50:41Z", "event":"FSSBodySignals", "BodyName":"Synuefe EN-H d11-106 6 a", "BodyID":31, "SystemAddress":3652777380195, "Signals":[ { "Type":"$SAA_SignalType_Biological;", "Type_Localised":"Biologisch", "Count":1 }, { "Type":"$SAA_SignalType_Geological;", "Type_Localised":"Geologisch", "Count":3 } ] }
                                "FSSBodySignals" | "SAASignalsFound" => {
                                    tokio::spawn(async move {
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
                                            debug!("Call register_planet_signal: {system_address}-{body_id}-{type_}-{}",json["BodyName"]);
                                            let function_call: FunctionCall<
                                                Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
                                                SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                                                (),
                                            > = contract.register_planet_signal(
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
                                            let _ = execute_send_repeatable(function_call).await;
                                        }
                                    });
                                }
                                //Carrier
                                "CarrierJumpRequest" => {
                                    //{
                                    //     "timestamp": "2020-04-20T09:30:58Z",
                                    //     "event": "CarrierJumpRequest",
                                    //     "CarrierID": 3700005632,
                                    //     "SystemName": "Paesui Xena",
                                    //     "Body": "Paesui Xena A",
                                    //     "SystemAddress": 7269634680241,
                                    //     "BodyID": 1,
                                    //     "DepartureTime":"2020-04-20T09:45:00Z"
                                    // }

                                    tokio::spawn(async move {
                                        let contract = contract;
                                        debug!("Call emit_carrier_jump");
                                        let function_call: FunctionCall<
                                            Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
                                            SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                                            (),
                                        > = contract.emit_carrier_jump(
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
                                        //execute_send(function_call).await;
                                        let _ = execute_send_repeatable(function_call).await;
                                    });
                                }
                                "CarrierJumpCancelled" => {
                                    tokio::spawn(async move {
                                        debug!("Call cancel_carrier_jump");
                                        let function_call: FunctionCall<
                                            Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
                                            SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                                            (),
                                        > = contract.cancel_carrier_jump(json["CarrierID"].as_u64().unwrap());
                                        //execute_send(function_call).await;
                                        let _ = execute_send_repeatable(function_call).await;
                                    });
                                }
                                "CarrierJump" => {
                                    //{ "timestamp":"2023-09-09T23:59:09Z", "event":"CarrierJump", "Docked":true, "StationName":"Q2K-BHB", "StationType":"FleetCarrier",
                                    // "MarketID":3704402432, "StationFaction":{ "Name":"FleetCarrier" }, "StationGovernment":"$government_Carrier;",
                                    // "StationGovernment_Localised":"Privateigentum", "StationServices":[ "dock", "autodock", "commodities", "contacts", "exploration", "outfitting", "crewlounge", "rearm", "refuel", "repair", "shipyard", "engineer", "flightcontroller", "stationoperations", "stationMenu", "carriermanagement", "carrierfuel", "livery", "voucherredemption", "socialspace", "bartender", "vistagenomics" ],
                                    // "StationEconomy":"$economy_Carrier;", "StationEconomy_Localised":"Privatunternehmen",
                                    // "StationEconomies":[ { "Name":"$economy_Carrier;", "Name_Localised":"Privatunternehmen", "Proportion":1.000000 } ], "Taxi":false, "Multicrew":false,
                                    // "StarSystem":"Plio Broae ML-D c2", "SystemAddress":637165713922, "StarPos":[2112.75000,719.12500,50162.93750], "SystemAllegiance":"",
                                    // "SystemEconomy":"$economy_None;", "SystemEconomy_Localised":"n/v", "SystemSecondEconomy":"$economy_None;", "SystemSecondEconomy_Localised":"n/v",
                                    // "SystemGovernment":"$government_None;", "SystemGovernment_Localised":"n/v", "SystemSecurity":"$GAlAXY_MAP_INFO_state_anarchy;",
                                    // "SystemSecurity_Localised":"Anarchie", "Population":0, "Body":"Plio Broae ML-D c2", "BodyID":0, "BodyType":"Star" }
                                    tokio::spawn(async move {
                                        let _ = process_jump(json.clone(), contract).await;
                                    });
                                }
                                "CarrierBuy" => {
                                    //{
                                    //     "timestamp": "2020-03-11T15:31:46Z",
                                    //     "event": "CarrierBuy",
                                    //     "CarrierID": 3700029440,
                                    //     "BoughtAtMarket": 3221301504,
                                    //     "Location": "Kakmbutan",
                                    //     "SystemAddress": 3549513615723,
                                    //     "Price": 4875000000,
                                    //     "Variant": "CarrierDockB",
                                    //     "Callsign": "P07-V3L"
                                    // }
                                    tokio::spawn(async move {
                                        let mut services = String::new();
                                        for entry in 0..json["Crew"].len() {
                                            if json["Crew"][entry]["Activated"].as_bool().unwrap_or(false) {
                                                if !services.is_empty() {
                                                    services.push(',');
                                                }
                                                services.push_str(
                                                    json["Crew"][entry]["CrewRole"].to_string().as_str(),
                                                );
                                            }
                                        }

                                        debug!("Call register_carrier");
                                        let function_call: FunctionCall<
                                            Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
                                            SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                                            (),
                                        > = contract.register_carrier(
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
                                        //execute_send(function_call).await;
                                        let _ = execute_send_repeatable(function_call).await;
                                    });
                                }
                                "CarrierDecommission" => {
                                    //{
                                    //     "timestamp": "2020-03-11T15:12:26Z",
                                    //     "event": "CarrierDecommission",
                                    //     "CarrierID": 3700005632,
                                    //     "ScrapRefund": 1746872629,
                                    //     "ScrapTime": 1584601200
                                    // }
                                }
                                "CarrierStats" => {
                                    //{ "timestamp":"2024-03-31T18:14:39Z", "event":"CarrierStats", "CarrierID":3704402432, "Callsign":"Q2K-BHB", "Name":"FUXBAU",
                                    // "DockingAccess":"all", "AllowNotorious":true, "FuelLevel":885, "JumpRangeCurr":500.000000, "JumpRangeMax":500.000000,
                                    // "PendingDecommission":false, "SpaceUsage":{ "TotalCapacity":25000, "Crew":6170, "Cargo":2853, "CargoSpaceReserved":2169, "ShipPacks":0,
                                    // "ModulePacks":4453, "FreeSpace":9355 }, "Finance":{ "CarrierBalance":28458349715, "ReserveBalance":86981722, "AvailableBalance":28148606640,
                                    // "ReservePercent":0, "TaxRate_shipyard":0, "TaxRate_rearm":100, "TaxRate_outfitting":100, "TaxRate_refuel":100, "TaxRate_repair":100 },
                                    // "Crew":[ { "CrewRole":"BlackMarket", "Activated":false }, { "CrewRole":"Captain", "Activated":true, "Enabled":true, "CrewName":"Vada Cannon" }, { "CrewRole":"Refuel", "Activated":true, "Enabled":true, "CrewName":"Donna Moon" }, { "CrewRole":"Repair", "Activated":true, "Enabled":true, "CrewName":"Darnell Grant" }, { "CrewRole":"Rearm", "Activated":true, "Enabled":true, "CrewName":"Eiza York" }, { "CrewRole":"Commodities", "Activated":true, "Enabled":true, "CrewName":"Jewel King" }, { "CrewRole":"VoucherRedemption", "Activated":true, "Enabled":true, "CrewName":"Ezra Ramirez" }, { "CrewRole":"Exploration", "Activated":true, "Enabled":true, "CrewName":"Kasey Callahan" }, { "CrewRole":"Shipyard", "Activated":true, "Enabled":true, "CrewName":"Abby Cooke" }, { "CrewRole":"Outfitting", "Activated":true, "Enabled":true, "CrewName":"Jayne Callahan" }, { "CrewRole":"CarrierFuel", "Activated":true, "Enabled":true, "CrewName":"Abraham Strickland" }, { "CrewRole":"VistaGenomics", "Activated":true, "Enabled":true, "CrewName":"Melinda Reilly" }, { "CrewRole":"PioneerSupplies", "Activated":false }, { "CrewRole":"Bartender", "Activated":true, "Enabled":true, "CrewName":"Dean Barlow" } ],
                                    // "ShipPacks":[  ], "ModulePacks":[ { "PackTheme":"VehicleSupport", "PackTier":1 }, { "PackTheme":"Storage", "PackTier":2 }, { "PackTheme":"Limpets", "PackTier":1 }, { "PackTheme":"Sensors", "PackTier":3 }, { "PackTheme":"Mining Tools", "PackTier":3 }, { "PackTheme":"Mining Utilities", "PackTier":2 }, { "PackTheme":"ShipUtilities", "PackTier":2 }, { "PackTheme":"TravelEnhancements", "PackTier":3 } ] }
                                    tokio::spawn(async move {
                                        let mut services = String::new();
                                        for entry in 0..json["Crew"].len() {
                                            if json["Crew"][entry]["Activated"].as_bool().unwrap_or(false) {
                                                if !services.is_empty() {
                                                    services.push_str(",");
                                                }
                                                services.push_str(
                                                    json["Crew"][entry]["CrewRole"].to_string().as_str(),
                                                );
                                            }
                                        }

                                        debug!("Call register_carrier");
                                        let function_call: FunctionCall<
                                            Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
                                            SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                                            (),
                                        > = contract.register_carrier(
                                            json["CarrierID"].as_u64().unwrap(),
                                            json["Name"].as_str().unwrap().to_string(),
                                            json["Callsign"].as_str().unwrap().to_string(),
                                            services,
                                            json["DockingAccess"].as_str().unwrap().to_string(),
                                            json["AllowNotorious"].as_bool().unwrap(),
                                            DateTime::parse_from_rfc3339(json["timestamp"].as_str().unwrap())
                                                .unwrap()
                                                .timestamp()
                                                .into(),
                                        );
                                        //execute_send(function_call).await;
                                        let _ = execute_send_repeatable(function_call).await;
                                    });
                                }
                                "CarrierTradeOrder" => {}
                                "CarrierFinance" => {}
                                "CarrierDepositFuel" => {}
                                "CarrierDockingPermission" => {}
                                "CarrierCrewServices" => {}
                                "CarrierModulePack" => {}
                                "CarrierBankTransfer" => {}
                                "Docked" => {
                                    //{ "timestamp":"2024-04-02T21:22:42Z", "event":"Docked", "StationName":"Q2K-BHB", "StationType":"FleetCarrier", "Taxi":false, "Multicrew":false,
                                    // "StarSystem":"Dulos", "SystemAddress":13865362204089, "MarketID":3704402432,
                                    // "StationFaction":{ "Name":"FleetCarrier" }, "StationGovernment":"$government_Carrier;", "StationGovernment_Localised":"Private Ownership",
                                    // "StationServices":[ "dock", "autodock", "commodities", "contacts", "exploration", "outfitting", "crewlounge", "rearm", "refuel", "repair", "shipyard", "engineer", "flightcontroller", "stationoperations", "stationMenu", "carriermanagement", "carrierfuel", "livery", "voucherredemption", "socialspace", "bartender", "vistagenomics" ],
                                    // "StationEconomy":"$economy_Carrier;", "StationEconomy_Localised":"Private Enterprise",
                                    // "StationEconomies":[ { "Name":"$economy_Carrier;", "Name_Localised":"Private Enterprise", "Proportion":1.000000 } ],
                                    // "DistFromStarLS":0.000000, "LandingPads":{ "Small":4, "Medium":4, "Large":8 } }

                                    //{ "timestamp":"2024-04-02T19:42:24Z", "event":"Docked", "StationName":"Milnor Station", "StationType":"Ocellus", "Taxi":false, "Multicrew":false,
                                    // "StarSystem":"Dulos", "SystemAddress":13865362204089, "MarketID":3223819264,
                                    // "StationFaction":{ "Name":"The Sovereign Justice Collective", "FactionState":"Bust" },
                                    // "StationGovernment":"$government_Dictatorship;", "StationGovernment_Localised":"Dictatorship",
                                    // "StationServices":[ "dock", "autodock", "commodities", "contacts", "exploration", "missions", "outfitting", "crewlounge", "rearm", "refuel", "repair", "shipyard", "tuning", "engineer", "missionsgenerated", "flightcontroller", "stationoperations", "powerplay", "searchrescue", "stationMenu", "shop", "livery", "socialspace", "bartender", "vistagenomics", "pioneersupplies", "apexinterstellar", "frontlinesolutions" ],
                                    // "StationEconomy":"$economy_Refinery;", "StationEconomy_Localised":"Refinery",
                                    // "StationEconomies":[ { "Name":"$economy_Refinery;", "Name_Localised":"Refinery", "Proportion":1.000000 } ],
                                    // "DistFromStarLS":20.275191, "LandingPads":{ "Small":11, "Medium":13, "Large":6 } }
                                    tokio::spawn(async move {
                                        let mut services = String::new();
                                        for entry in 0..json["StationServices"].len() {
                                            if !services.is_empty() {
                                                services.push(',');
                                            }
                                            services.push_str(json["StationServices"][entry].as_str().unwrap());
                                        }
                                        if json["StationType"].as_str().unwrap() == "FleetCarrier" {
                                            debug!("Call register_carrier");
                                            let function_call: ContractCall = contract.register_carrier(
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
                                            //execute_send(function_call).await;
                                            let _ = execute_send_repeatable(function_call).await;
                                            debug!("Call report_carrier_location");
                                            let function_call: ContractCall = contract.report_carrier_location(
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
                                            //execute_send(function_call).await;
                                            let _ = execute_send_repeatable(function_call).await;
                                        } else {
                                            debug!("Call register_station");
                                            let function_call: ContractCall = contract.register_station(
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
                                            let _ = execute_send_repeatable(function_call).await;
                                        }
                                    });
                                }
                                "" => {
                                    tokio::spawn(async move {
                                        if !json["commodities"].is_empty() {
                                            let market_id = json["marketId"].as_u64().unwrap();
                                            let size = json["commodities"].len();
                                            for i in 0..size {
                                                let commoditiy = &json["commodities"][i];
                                                let function_call: FunctionCall<
                                                    Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
                                                    SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
                                                    (),
                                                > = contract.register_commodity_listening(
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
                                                let _ = execute_send_repeatable(function_call).await;
                                            }
                                        }
                                    });
                                }
                                _ => {}
                            }
                        }
                    }
                }
            });
    }
}
pub fn initialize(bus_reader: BusReader<JsonValue>, evm_settings: &EvmSettings) -> EvmInterpreter {
    let contract = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move { get_contract(evm_settings).await });

    EvmInterpreter {
        bus: bus_reader,
        contract,
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
}
async fn execute_send_repeatable(
    function_call: FunctionCall<
        Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
        SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
        (),
    >,
) -> Result<TransactionReceipt, String> {
    while let Err(err) = match execute_send(function_call.clone()).await {
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
            NonRepeatableError(err) => return Err(err),
        }
    }
    Err("Unknown".into())
}
async fn execute_send(
    function_call: FunctionCall<
        Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
        SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
        (),
    >,
) -> Result<TransactionReceipt, SendError> {
    match function_call.legacy().send().await {
        Ok(pending) => match pending.await {
            Ok(receipt) => {
                if let Some(receipt) = receipt {
                    if let Some(hash) = receipt.block_hash {
                        info!("Success calling function: {:?}", hash);
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
async fn process_jump(
    json: JsonValue,
    contract: edcas_contract::EDCAS<SignerMiddleware<Provider<Http>, LocalWallet>>,
) -> Result<TransactionReceipt, String> {
    debug!("Call register_system");
    let function_call: FunctionCall<
        Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
        SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
        (),
    > = contract.register_system(
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
    execute_send_repeatable(function_call).await
}

fn get_revert_message(bytes: Bytes) -> String {
    if bytes.len() < 134 {
        let n = bytes.split_at(134 / 2).1;
        let n: &[u8] = n.split(|b| *b == 0u8).next().unwrap();
        return String::from_utf8(n.to_vec()).unwrap();
    }
    bytes.to_string()
}
