use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;

use json::{JsonValue, Null};
use log::{debug, error, info, warn};

use crate::app::explorer::belt_cluster::BeltCluster;
use crate::app::explorer::planet::Planet;
use crate::app::explorer::star::Star;
use crate::app::explorer::structs::{Parent, Signal};
use crate::app::explorer::system::{PlanetSignals, System, SystemSignal};
use crate::app::explorer::{structs, Explorer};
use crate::app::materials::{Material, MaterialState};
use crate::app::mining::{Mining, MiningMaterial, Prospector};
use crate::app::settings::Settings;

pub fn interpret_json(
    json: JsonValue,
    explorer: &mut Explorer,
    materials: &mut MaterialState,
    mining: &mut Mining,
    settings: Arc<Settings>,
) {
    let event = json["event"].as_str().unwrap();
    info!("Interpreter event received: {}", event);
    let now = Instant::now();
    //println!("{}",&json);

    match event {
        //Navigation
        //{ "timestamp":"2022-10-16T20:54:45Z", "event":"Location", "DistFromStarLS":1007.705243, "Docked":true, "StationName":"Q2K-BHB", "StationType":"FleetCarrier", "MarketID":3704402432, "StationFaction":{ "Name":"FleetCarrier" }, "StationGovernment":"$government_Carrier;", "StationGovernment_Localised":"Privateigentum", "StationServices":[ "dock", "autodock", "commodities", "contacts", "exploration", "outfitting", "crewlounge", "rearm", "refuel", "repair", "shipyard", "engineer", "flightcontroller", "stationoperations", "stationMenu", "carriermanagement", "carrierfuel", "livery", "voucherredemption", "socialspace", "bartender", "vistagenomics" ], "StationEconomy":"$economy_Carrier;", "StationEconomy_Localised":"Privatunternehmen", "StationEconomies":[ { "Name":"$economy_Carrier;", "Name_Localised":"Privatunternehmen", "Proportion":1.000000 } ], "Taxi":false, "Multicrew":false, "StarSystem":"Colonia", "SystemAddress":3238296097059, "StarPos":[-9530.50000,-910.28125,19808.12500], "SystemAllegiance":"Independent", "SystemEconomy":"$economy_Tourism;", "SystemEconomy_Localised":"Tourismus", "SystemSecondEconomy":"$economy_HighTech;", "SystemSecondEconomy_Localised":"Hightech", "SystemGovernment":"$government_Cooperative;", "SystemGovernment_Localised":"Kooperative", "SystemSecurity":"$SYSTEM_SECURITY_low;", "SystemSecurity_Localised":"Geringe Sicherheit", "Population":583869, "Body":"Colonia 2 c", "BodyID":18, "BodyType":"Planet", "Factions":[ { "Name":"Jaques", "FactionState":"Investment", "Government":"Cooperative", "Influence":0.454092, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand1;", "Happiness_Localised":"In Hochstimmung", "MyReputation":100.000000, "RecoveringStates":[ { "State":"PublicHoliday", "Trend":0 } ], "ActiveStates":[ { "State":"Investment" }, { "State":"CivilLiberty" } ] }, { "Name":"Colonia Council", "FactionState":"Boom", "Government":"Cooperative", "Influence":0.331337, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":100.000000, "ActiveStates":[ { "State":"Boom" } ] }, { "Name":"People of Colonia", "FactionState":"None", "Government":"Cooperative", "Influence":0.090818, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":27.956400 }, { "Name":"Holloway Bioscience Institute", "FactionState":"None", "Government":"Corporate", "Influence":0.123752, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":-9.420000, "RecoveringStates":[ { "State":"PirateAttack", "Trend":0 } ] } ], "SystemFaction":{ "Name":"Jaques", "FactionState":"Investment" } }
        //{ "timestamp":"2022-10-16T23:25:31Z", "event":"FSDJump", "Taxi":false, "Multicrew":false, "StarSystem":"Ogmar", "SystemAddress":84180519395914, "StarPos":[-9534.00000,-905.28125,19802.03125], "SystemAllegiance":"Independent", "SystemEconomy":"$economy_HighTech;", "SystemEconomy_Localised":"Hightech", "SystemSecondEconomy":"$economy_Military;", "SystemSecondEconomy_Localised":"Militär", "SystemGovernment":"$government_Confederacy;", "SystemGovernment_Localised":"Konföderation", "SystemSecurity":"$SYSTEM_SECURITY_medium;", "SystemSecurity_Localised":"Mittlere Sicherheit", "Population":151752, "Body":"Ogmar A", "BodyID":1, "BodyType":"Star", "JumpDist":8.625, "FuelUsed":0.024493, "FuelLevel":31.975506, "Factions":[ { "Name":"Jaques", "FactionState":"Election", "Government":"Cooperative", "Influence":0.138384, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand1;", "Happiness_Localised":"In Hochstimmung", "MyReputation":100.000000, "PendingStates":[ { "State":"Outbreak", "Trend":0 } ], "ActiveStates":[ { "State":"Election" } ] }, { "Name":"ICU Colonial Corps", "FactionState":"War", "Government":"Communism", "Influence":0.119192, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":96.402496, "PendingStates":[ { "State":"Expansion", "Trend":0 } ], "ActiveStates":[ { "State":"War" } ] }, { "Name":"Societas Eruditorum de Civitas Dei", "FactionState":"War", "Government":"Dictatorship", "Influence":0.119192, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":46.414799, "ActiveStates":[ { "State":"War" } ] }, { "Name":"GalCop Colonial Defence Commission", "FactionState":"Boom", "Government":"Confederacy", "Influence":0.406061, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":-75.000000, "ActiveStates":[ { "State":"Boom" } ] }, { "Name":"Likedeeler of Colonia", "FactionState":"None", "Government":"Democracy", "Influence":0.068687, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":4.002500 }, { "Name":"Colonia Tech Combine", "FactionState":"Election", "Government":"Cooperative", "Influence":0.138384, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":4.850000, "ActiveStates":[ { "State":"Election" } ] }, { "Name":"Milanov's Reavers", "FactionState":"Bust", "Government":"Anarchy", "Influence":0.010101, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":0.000000, "RecoveringStates":[ { "State":"Terrorism", "Trend":0 } ], "ActiveStates":[ { "State":"Bust" } ] } ], "SystemFaction":{ "Name":"GalCop Colonial Defence Commission", "FactionState":"Boom" }, "Conflicts":[ { "WarType":"election", "Status":"active", "Faction1":{ "Name":"Jaques", "Stake":"Guerrero Military Base", "WonDays":1 }, "Faction2":{ "Name":"Colonia Tech Combine", "Stake":"", "WonDays":0 } }, { "WarType":"war", "Status":"active", "Faction1":{ "Name":"ICU Colonial Corps", "Stake":"Boulaid Command Facility", "WonDays":1 }, "Faction2":{ "Name":"Societas Eruditorum de Civitas Dei", "Stake":"Chatterjee's Respite", "WonDays":0 } } ] }
        "FSDJump" | "Location" | "CarrierJump" => {
            let mut system = System {
                name: json["StarSystem"].to_string(),
                address: json["SystemAddress"].to_string(),
                allegiance: json["SystemAllegiance"].to_string(),
                economy_localised: json["SystemEconomy_Localised"].to_string(),
                second_economy_localised: json["SystemSecondEconomy_Localised"].to_string(),
                government_localised: json["SystemGovernment_Localised"].to_string(),
                security_localised: json["SystemSecurity_Localised"].to_string(),
                population: json["Population"].to_string(),
                body_count: "N/A".to_string(),
                non_body_count: "N/A".to_string(),
                signal_list: vec![],
                body_list: vec![],
                planet_signals: vec![],
                index: 0,
                settings: settings.clone(),
            };

            let address = system.address.clone();
            let answer: Option<JsonValue> = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    let url = format!(
                        "https://api.edcas.de/data/odyssey/system/{}",
                        address.clone()
                    );
                    debug!("Api call to edcas: {}", url.clone());
                    let result = reqwest::get(url.clone()).await;
                    return match result {
                        Ok(response) => {
                            let text = response.text().await.unwrap();
                            let result = json::parse(text.as_str());
                            return match result {
                                Ok(json) => Some(json),
                                Err(err) => {
                                    error!("Couldn't parse answer to json: {}", err);
                                    error!("Value: {}", text);
                                    None
                                }
                            };
                        }
                        Err(err) => {
                            error!(
                                "Couldn't reach edcas api under {} Reason: {}",
                                url.clone(),
                                err
                            );
                            None
                        }
                    };
                });

            match answer {
                None => {}
                Some(json) => {
                    for i in 0..json["stars"].len() {
                        let star_json = &json["stars"][i];

                        let mut parents: Vec<Parent> = vec![];
                        for j in 0..star_json["parents"].len() {
                            let parent = &star_json["parents"][j];
                            for entry in parent.entries() {
                                parents.push(Parent {
                                    name: entry.0.to_string(),
                                    id: entry.1.as_i64().unwrap(),
                                });
                            }
                        }

                        system.insert_body(Box::new(Star {
                            timestamp: "".to_string(),
                            event: "API".to_string(),
                            scan_type: "API".to_string(),
                            body_name: star_json["body_name"].to_string(),
                            body_id: star_json["body_id"].as_i64().unwrap(),
                            parents,
                            star_system: system.name.clone(),
                            system_address: i64::from_str(address.as_str()).unwrap(),
                            distance_from_arrival_ls: f64::from_str(
                                star_json["distance_from_arrival_ls"].to_string().as_str(),
                            )
                            .unwrap(),
                            star_type: star_json["star_type"].to_string(),
                            subclass: i64::from_str(star_json["subclass"].to_string().as_str())
                                .unwrap(),
                            stellar_mass: f64::from_str(
                                star_json["stellar_mass"].to_string().as_str(),
                            )
                            .unwrap(),
                            radius: f64::from_str(star_json["radius"].to_string().as_str())
                                .unwrap(),
                            absolute_magnitude: f64::from_str(
                                star_json["absolute_magnitude"].to_string().as_str(),
                            )
                            .unwrap(),
                            age_my: i64::from_str(star_json["age_my"].to_string().as_str())
                                .unwrap(),
                            surface_temperature: f64::from_str(
                                star_json["surface_temperature"].to_string().as_str(),
                            )
                            .unwrap(),
                            luminosity: star_json["luminosity"].to_string(),
                            semi_major_axis: f64::from_str(
                                star_json["semi_major_axis"].to_string().as_str(),
                            )
                            .ok(),
                            eccentricity: f64::from_str(
                                star_json["eccentricity"].to_string().as_str(),
                            )
                            .ok(),
                            orbital_inclination: f64::from_str(
                                star_json["orbital_inclination"].to_string().as_str(),
                            )
                            .ok(),
                            periapsis: f64::from_str(star_json["periapsis"].to_string().as_str())
                                .ok(),
                            orbital_period: f64::from_str(
                                star_json["orbital_period"].to_string().as_str(),
                            )
                            .ok(),
                            ascending_node: f64::from_str(
                                star_json["ascending_node"].to_string().as_str(),
                            )
                            .ok(),
                            mean_anomaly: f64::from_str(
                                star_json["mean_anomaly"].to_string().as_str(),
                            )
                            .ok(),
                            rotation_period: f64::from_str(
                                star_json["rotation_period"].to_string().as_str(),
                            )
                            .unwrap(),
                            axial_tilt: f64::from_str(star_json["axial_tilt"].to_string().as_str())
                                .unwrap(),
                            was_discovered: star_json["was_discovered"].as_bool().unwrap(),
                            was_mapped: star_json["was_mapped"].as_bool().unwrap(),
                            asteroid_rings: vec![],
                            settings: settings.clone(),
                        }));
                    }
                    for i in 0..json["planets"].len() {
                        let planet_json = &json["planets"][i];

                        let mut parents: Vec<Parent> = vec![];
                        //"parents": [{"Star": 0 },{"Planet": 47 },{"Null": 51 }]
                        for j in 0..planet_json["parents"].len() {
                            let parent = &planet_json["parents"][j];
                            for entry in parent.entries() {
                                parents.push(Parent {
                                    name: entry.0.to_string(),
                                    id: entry.1.as_i64().unwrap(),
                                });
                            }
                        }

                        if planet_json["body_name"]
                            .as_str()
                            .unwrap()
                            .contains("Belt Cluster")
                        {
                            system.insert_body(Box::new(BeltCluster {
                                timestamp: "".to_string(),
                                event: "API".to_string(),
                                scan_type: "API".to_string(),
                                body_name: planet_json["body_name"].to_string(),
                                body_id: planet_json["body_id"].as_i64().unwrap(),
                                parents,
                                star_system: system.name.clone(),
                                system_address: i64::from_str(address.as_str()).unwrap(),
                                distance_from_arrival_ls: f64::from_str(
                                    planet_json["distance_from_arrival_ls"].to_string().as_str(),
                                )
                                .unwrap(),
                                was_discovered: planet_json["was_discovered"].as_bool().unwrap(),
                                was_mapped: planet_json["was_mapped"].as_bool().unwrap(),
                                settings: settings.clone(),
                            }));
                        } else {
                            let planet = Planet {
                                timestamp: "".to_string(),
                                event: "API".to_string(),
                                scan_type: "API".to_string(),
                                body_name: planet_json["body_name"].to_string(),
                                body_id: planet_json["body_id"].as_i64().unwrap(),
                                parents,
                                star_system: system.name.clone(),
                                system_address: i64::from_str(address.as_str()).unwrap(),
                                distance_from_arrival_ls: f64::from_str(
                                    planet_json["distance_from_arrival_ls"].to_string().as_str(),
                                )
                                .unwrap(),
                                tidal_lock: planet_json["was_discovered"].as_bool().unwrap(),
                                terraform_state: planet_json["terraform_state"].to_string(),
                                planet_class: planet_json["planet_class"].to_string(),
                                atmosphere: planet_json["atmosphere"].to_string(),
                                atmosphere_type: planet_json["atmosphere_type"].to_string(),
                                atmosphere_composition: vec![],
                                volcanism: planet_json["volcanism"].to_string(),
                                mass_em: f64::from_str(planet_json["mass_em"].to_string().as_str())
                                    .unwrap_or(-1.0),
                                radius: f64::from_str(planet_json["radius"].to_string().as_str())
                                    .unwrap_or(-1.0),
                                surface_gravity: f64::from_str(
                                    planet_json["surface_gravity"].to_string().as_str(),
                                )
                                .unwrap_or(-1.0),
                                surface_temperature: f64::from_str(
                                    planet_json["surface_temperature"].to_string().as_str(),
                                )
                                .unwrap_or(-1.0),
                                surface_pressure: f64::from_str(
                                    planet_json["surface_pressure"].to_string().as_str(),
                                )
                                .unwrap_or(-1.0),
                                landable: planet_json["landable"].as_bool().unwrap(),
                                materials: vec![],
                                composition: vec![],
                                semi_major_axis: f64::from_str(
                                    planet_json["semi_major_axis"].to_string().as_str(),
                                )
                                .unwrap(),
                                eccentricity: f64::from_str(
                                    planet_json["eccentricity"].to_string().as_str(),
                                )
                                .unwrap(),
                                orbital_inclination: f64::from_str(
                                    planet_json["orbital_inclination"].to_string().as_str(),
                                )
                                .unwrap(),
                                periapsis: f64::from_str(
                                    planet_json["periapsis"].to_string().as_str(),
                                )
                                .unwrap(),
                                orbital_period: f64::from_str(
                                    planet_json["orbital_period"].to_string().as_str(),
                                )
                                .unwrap(),
                                ascending_node: f64::from_str(
                                    planet_json["ascending_node"].to_string().as_str(),
                                )
                                .unwrap(),
                                mean_anomaly: f64::from_str(
                                    planet_json["mean_anomaly"].to_string().as_str(),
                                )
                                .unwrap(),
                                rotation_period: f64::from_str(
                                    planet_json["rotation_period"].to_string().as_str(),
                                )
                                .unwrap_or(-1.0),
                                axial_tilt: f64::from_str(
                                    planet_json["axial_tilt"].to_string().as_str(),
                                )
                                .unwrap_or(-1.0),
                                was_discovered: planet_json["was_discovered"].as_bool().unwrap(),
                                was_mapped: planet_json["was_mapped"].as_bool().unwrap(),
                                reserve_level: planet_json["reserve_level"].to_string(),
                                asteroid_rings: vec![],
                                planet_signals: vec![],
                                settings: settings.clone(),
                            };
                            if planet.surface_gravity != -1.0 {
                                system.insert_body(Box::new(planet));
                            }
                        }
                    }
                }
            }

            explorer.systems.push(system);

            explorer.index = explorer.systems.len() - 1;

            //if explorer.index == explorer.pages.len()-1{
            //    explorer.index = explorer.pages.len();
            //}

            info!(
                "Found system: {}",
                explorer.systems.last().unwrap().name.clone()
            );
        }
        "SupercruiseEntry" => {}
        "SupercruiseExit" => {}
        //{ "timestamp":"2022-10-16T23:25:05Z", "event":"StartJump", "JumpType":"Hyperspace", "StarSystem":"Ogmar", "SystemAddress":84180519395914, "StarClass":"K" }
        "StartJump" => {} //If jump has been initialised
        //{ "timestamp":"2022-10-16T23:24:46Z", "event":"FSDTarget", "Name":"Ogmar", "SystemAddress":84180519395914, "StarClass":"K", "RemainingJumpsInRoute":1 }
        "FSDTarget" => {}     //If system has been targeted
        "NavRoute" => {}      //If route has been set -> check json for further information
        "NavRouteClear" => {} //If navigation is complete -> no further information

        //Approaching
        "ApproachSettlement" => {}
        "ApproachBody" => {}
        "LeaveBody" => {}
        "Liftoff" => {}
        "Touchdown" => {}
        "Embark" => {}
        "Disembark" => {}

        //Scanning
        "DiscoveryScan" => {}
        "FSSAllBodiesFound" => {}
        //{ "timestamp":"2022-10-16T23:46:48Z", "event":"FSSDiscoveryScan", "Progress":0.680273, "BodyCount":21, "NonBodyCount":80, "SystemName":"Ogmar", "SystemAddress":84180519395914 }
        "FSSDiscoveryScan" => {
            if !explorer.systems.is_empty() {
                let len = explorer.systems.len() - 1;
                let system = &mut explorer.systems[len];
                system.body_count = json["BodyCount"].to_string();
                system.non_body_count = json["NonBodyCount"].to_string();
            }
        } //Honk
        //{ "timestamp":"2022-07-07T20:58:06Z", "event":"SAASignalsFound", "BodyName":"IC 2391 Sector YE-A d103 B 1", "SystemAddress":3549631072611, "BodyID":15, "Signals":[ { "Type":"$SAA_SignalType_Guardian;", "Type_Localised":"Guardian", "Count":1 }, { "Type":"$SAA_SignalType_Human;", "Type_Localised":"Menschlich", "Count":9 } ] }
        "FSSBodySignals" | "SAASignalsFound" => {
            //TODO Implement NFT
            //{ "timestamp":"2022-09-07T17:50:41Z", "event":"FSSBodySignals", "BodyName":"Synuefe EN-H d11-106 6 a", "BodyID":31, "SystemAddress":3652777380195, "Signals":[ { "Type":"$SAA_SignalType_Biological;", "Type_Localised":"Biologisch", "Count":1 }, { "Type":"$SAA_SignalType_Geological;", "Type_Localised":"Geologisch", "Count":3 } ] }
            if !explorer.systems.is_empty() {
                let mut signals: Vec<Signal> = Vec::new();

                for i in 0..json["Signals"].len() {
                    signals.push(Signal {
                        r#type: json["Signals"][i]["Type"].to_string(),
                        type_localised: json["Signals"][i]["Type_Localised"].to_string(),
                        count: json["Signals"][i]["Count"].as_i64().unwrap_or(-1),
                    })
                }

                let planet_signals = PlanetSignals {
                    body_name: json["BodyName"].to_string(),
                    body_id: json["BodyID"].as_i64().unwrap(),
                    signals: signals.clone(),
                };

                info!(
                    "Body {} number of signals: {}",
                    json["BodyName"].to_string(),
                    signals.len().clone()
                );

                let len = explorer.systems.len() - 1;

                let mut found = false;
                for i in 0..explorer.systems[len].planet_signals.len() {
                    let planet_signal = &mut explorer.systems[len].planet_signals[i];
                    if planet_signal.body_id == planet_signals.body_id {
                        planet_signal.signals = planet_signals.signals.clone();
                        found = true;
                    }
                }
                if !found {
                    explorer.systems[len].planet_signals.push(planet_signals);
                    explorer.systems[len]
                        .planet_signals
                        .sort_by(|a, b| a.body_id.cmp(&b.body_id));
                }
            }
        }
        "FSSSignalDiscovered" => {
            //{ "timestamp":"2023-05-29T22:40:26Z", "event":"FSSSignalDiscovered", "SystemAddress":672296347049, "SignalName":"$MULTIPLAYER_SCENARIO80_TITLE;", "SignalName_Localised":"Unbewachtes Navigationssignal" }
            // { "timestamp":"2023-05-29T22:40:26Z", "event":"FSSSignalDiscovered", "SystemAddress":672296347049, "SignalName":"THE GENERAL MELCHETT X5W-0XL", "IsStation":true }
            //{ "timestamp":"2023-05-29T22:40:42Z", "event":"FSSSignalDiscovered", "SystemAddress":672296347049, "SignalName":"$USS_HighGradeEmissions;", "SignalName_Localised":"Unidentifizierte Signalquelle",
            // "USSType":"$USS_Type_ValuableSalvage;", "USSType_Localised":"Verschlüsselte Emissionen", "SpawningState":"", "SpawningFaction":"Murus Major Industry", "ThreatLevel":0, "TimeRemaining":707.545837 }
            if !explorer.systems.is_empty() {
                let mut name = json["SignalName_Localised"].to_string();
                if name == *"null" {
                    name = json["SignalName"].to_string();
                    if name == *"null" {
                        name = json["USSType_Localised"].to_string();
                    }
                }

                let mut thread = json["ThreatLevel"].to_string();
                if thread == *"null" {
                    thread = "".to_string();
                }

                let system_signal = SystemSignal {
                    timestamp: json["timestamp"].to_string(),
                    event: json["event"].to_string(),
                    name,
                    threat: thread,
                };
                let len = explorer.systems.len() - 1;
                explorer.systems[len].signal_list.push(system_signal);
                explorer.systems[len].signal_list.sort_by(|a, b| {
                    if a.name == b.name {
                        a.threat.cmp(&b.threat)
                    } else {
                        a.name.cmp(&b.name)
                    }
                });
            }
        }
        "SAAScanComplete" => {}
        "Scan" => {
            //{ "timestamp":"2022-10-16T23:51:17Z", "event":"Scan", "ScanType":"Detailed", "BodyName":"Ogmar A 6", "BodyID":40, "Parents":[ {"Star":1}, {"Null":0} ], "StarSystem":"Ogmar", "SystemAddress":84180519395914, "DistanceFromArrivalLS":3376.246435, "TidalLock":false, "TerraformState":"", "PlanetClass":"Sudarsky class I gas giant", "Atmosphere":"", "AtmosphereComposition":[ { "Name":"Hydrogen", "Percent":73.044167 }, { "Name":"Helium", "Percent":26.955832 } ], "Volcanism":"", "MassEM":24.477320, "Radius":22773508.000000, "SurfaceGravity":18.811067, "SurfaceTemperature":62.810730, "SurfacePressure":0.000000, "Landable":false, "SemiMajorAxis":1304152250289.916992, "Eccentricity":0.252734, "OrbitalInclination":156.334694, "Periapsis":269.403039, "OrbitalPeriod":990257555.246353, "AscendingNode":-1.479320, "MeanAnomaly":339.074691, "RotationPeriod":37417.276422, "AxialTilt":0.018931, "WasDiscovered":true, "WasMapped":true }
            info!("Body found: {}", json["BodyName"].to_string());
            if !explorer.systems.is_empty() {
                let mut body = structs::generate_from_json(json.clone(), settings.clone());

                let len = explorer.systems.len() - 1;

                for planet_signal in &mut explorer.systems[len].planet_signals {
                    if planet_signal.body_id == body.get_id() {
                        body.set_signals(planet_signal.signals.clone());
                    }
                }

                let len = explorer.systems.len() - 1;
                if !explorer.systems[len].body_list.contains(&body) {
                    let index = explorer.systems[len].insert_body(body);
                    explorer.systems[len].index = index;
                }
            }
        }
        //Planet scan with fss
        "ScanBaryCentre" => {}

        //Maintenance
        "RefuelAll" => {}
        "Resupply" => {}
        "Repair" => {}
        "BuyDrones" => {}
        "SellDrones" => {}
        "BuyAmmo" => {}
        //{ "timestamp":"2022-10-16T23:55:55Z", "event":"ReservoirReplenished", "FuelMain":30.905506, "FuelReservoir":1.070000 }
        "ReservoirReplenished" => {} //If reservoir needs to drain more fuel from main tank
        "RepairAll" => {}
        "RebootRepair" => {}
        "RestockVehicle" => {}

        //Docking
        "DockingRequested" => {}
        "DockingGranted" => {}
        "Docked" => {
            //{ "timestamp":"2023-09-09T23:59:09Z", "event":"CarrierJump", "Docked":true, "StationName":"Q2K-BHB", "StationType":"FleetCarrier", "MarketID":3704402432, "StationFaction":{ "Name":"FleetCarrier" }, "StationGovernment":"$government_Carrier;", "StationGovernment_Localised":"Privateigentum", "StationServices":[ "dock", "autodock", "commodities", "contacts", "exploration", "outfitting", "crewlounge", "rearm", "refuel", "repair", "shipyard", "engineer", "flightcontroller", "stationoperations", "stationMenu", "carriermanagement", "carrierfuel", "livery", "voucherredemption", "socialspace", "bartender", "vistagenomics" ], "StationEconomy":"$economy_Carrier;", "StationEconomy_Localised":"Privatunternehmen", "StationEconomies":[ { "Name":"$economy_Carrier;", "Name_Localised":"Privatunternehmen", "Proportion":1.000000 } ], "Taxi":false, "Multicrew":false, "StarSystem":"Plio Broae ML-D c2", "SystemAddress":637165713922, "StarPos":[2112.75000,719.12500,50162.93750], "SystemAllegiance":"", "SystemEconomy":"$economy_None;", "SystemEconomy_Localised":"n/v", "SystemSecondEconomy":"$economy_None;", "SystemSecondEconomy_Localised":"n/v", "SystemGovernment":"$government_None;", "SystemGovernment_Localised":"n/v", "SystemSecurity":"$GAlAXY_MAP_INFO_state_anarchy;", "SystemSecurity_Localised":"Anarchie", "Population":0, "Body":"Plio Broae ML-D c2", "BodyID":0, "BodyType":"Star" }
        }
        "Undocked" => {
            //{ "timestamp":"2023-09-09T18:29:17Z", "event":"Undocked", "StationName":"Q2K-BHB", "StationType":"FleetCarrier", "MarketID":3704402432, "Taxi":false, "Multicrew":false }
        }

        //Engineer
        "EngineerProgress" => {}
        "EngineerCraft" => {
            //{ "timestamp":"2023-12-05T20:54:13Z", "event":"EngineerCraft", "Slot":"PowerDistributor",
            // "Module":"int_powerdistributor_size7_class5",
            // "Ingredients":[
            // { "Name":"hybridcapacitors", "Name_Localised":"Hybridkondensatoren", "Count":1 },
            // { "Name":"industrialfirmware", "Name_Localised":"Gecrackte Industrie-Firmware", "Count":1 },
            // { "Name":"chemicalmanipulators", "Name_Localised":"Chemische Manipulatoren", "Count":1 } ],
            // "Engineer":"The Dweller", "EngineerID":300180, "BlueprintID":128673738, "BlueprintName":"PowerDistributor_HighFrequency",
            // "Level":4, "Quality":0.267800, "ExperimentalEffect":"special_powerdistributor_fast",
            // "ExperimentalEffect_Localised":"Superleiter",
            // "Modifiers":[
            // { "Label":"WeaponsCapacity", "Value":56.217598, "OriginalValue":61.000000, "LessIsGood":0 }, { "Label":"WeaponsRecharge", "Value":8.209770, "OriginalValue":6.100000, "LessIsGood":0 }, { "Label":"EnginesCapacity", "Value":37.785599, "OriginalValue":41.000000, "LessIsGood":0 }, { "Label":"EnginesRecharge", "Value":5.383456, "OriginalValue":4.000000, "LessIsGood":0 }, { "Label":"SystemsCapacity", "Value":37.785599, "OriginalValue":41.000000, "LessIsGood":0 }, { "Label":"SystemsRecharge", "Value":5.383456, "OriginalValue":4.000000, "LessIsGood":0 } ] }
            let ingrediants = &json["Ingredients"];
            let count = ingrediants.len();
            for i in 0..count {
                let ingrediant = &ingrediants[i];
                let name = &ingrediant["Name"].to_string();
                let (material, category) = match materials.raw.get(name) {
                    None => match materials.encoded.get(name) {
                        None => match materials.manufactured.get(name) {
                            None => {
                                error!("Didn't found material: {}", &ingrediant);
                                (None, "")
                            }
                            Some(material) => (Some(material.clone()), "Manufactured"),
                        },
                        Some(material) => (Some(material.clone()), "Encoded"),
                    },
                    Some(material) => (Some(material.clone()), "Raw"),
                };
                match category {
                    "Manufactured" => {
                        if let Some(material) = material {
                            let mut cloned_material = material.clone();
                            materials.manufactured.remove(name);
                            cloned_material.count -= ingrediant["Count"].as_u64().unwrap();
                            materials.manufactured.insert(name.clone(), cloned_material);
                        } else {
                            error!(
                                "Didn't found manufactured material in material list: {}",
                                &ingrediant
                            );
                        }
                    }
                    "Encoded" => {
                        if let Some(material) = material {
                            let mut cloned_material = material.clone();
                            materials.encoded.remove(name);
                            cloned_material.count -= ingrediant["Count"].as_u64().unwrap();
                            materials.encoded.insert(name.clone(), cloned_material);
                        } else {
                            error!(
                                "Didn't found encoded material in material list: {}",
                                &ingrediant
                            );
                        }
                    }
                    "Raw" => {
                        if let Some(material) = material {
                            let mut cloned_material = material.clone();
                            materials.raw.remove(name);
                            cloned_material.count -= ingrediant["Count"].as_u64().unwrap();
                            materials.raw.insert(name.clone(), cloned_material);
                        } else {
                            error!(
                                "Didn't found raw material in material list: {}",
                                &ingrediant
                            );
                        }
                    }
                    _ => {
                        error!("Unknown material: {}", &ingrediant);
                    }
                }
            }
        }
        "EngineerContribution" => {}

        //Ship management
        "Shipyard" => {}
        "StoredShips" => {}
        "ShipyardSwap" => {}
        "ShipLocker" => {}
        "ModuleBuy" => {}
        "Outfitting" => {}
        "ModuleInfo" => {}
        "StoredModules" => {}
        "DockingCancelled" => {}
        "ShipyardBuy" => {}
        "ShipyardNew" => {}
        "ShipyardTransfer" => {}
        "ModuleStore" => {}
        "ModuleSell" => {}
        "ModuleSellRemote" => {}
        "ModuleSwap" => {}

        //On foot
        "Backpack" => {}
        "BackpackChange" => {}
        "CollectItems" => {}
        "UpgradeSuit" => {}
        "Loadout" => {}
        "LoadoutEquipModule" => {}
        "SuitLoadout" => {}
        "UseConsumable" => {}
        "ScanOrganic" => {}

        //Market
        "MarketBuy" => {}
        "Market" => {}
        "MarketSell" => {}

        //SRV
        "LaunchSRV" => {}
        "DockSRV" => {}

        //Ship fight
        "ShipTargeted" => {}
        "UnderAttack" => {}
        "ShieldState" => {}
        "HullDamage" => {}

        //Cargo, Materials & Mining & Drones
        //{ "timestamp":"2022-09-07T20:08:23Z", "event":"Materials",
        // "Raw":[ { "Name":"sulphur", "Name_Localised":"Schwefel", "Count":300 }, { "Name":"manganese", "Name_Localised":"Mangan", "Count":236 }, { "Name":"vanadium", "Count":95 }, { "Name":"nickel", "Count":300 }, { "Name":"phosphorus", "Name_Localised":"Phosphor", "Count":296 }, { "Name":"iron", "Name_Localised":"Eisen", "Count":300 }, { "Name":"germanium", "Count":239 }, { "Name":"chromium", "Name_Localised":"Chrom", "Count":213 }, { "Name":"carbon", "Name_Localised":"Kohlenstoff", "Count":257 }, { "Name":"molybdenum", "Name_Localised":"Molibdän", "Count":153 }, { "Name":"cadmium", "Name_Localised":"Kadmium", "Count":13 }, { "Name":"selenium", "Name_Localised":"Selen", "Count":14 }, { "Name":"mercury", "Name_Localised":"Quecksilber", "Count":19 }, { "Name":"yttrium", "Count":22 }, { "Name":"zinc", "Name_Localised":"Zink", "Count":250 }, { "Name":"ruthenium", "Count":24 }, { "Name":"arsenic", "Name_Localised":"Arsen", "Count":24 }, { "Name":"tungsten", "Name_Localised":"Wolfram", "Count":75 }, { "Name":"tellurium", "Name_Localised":"Tellur", "Count":12 }, { "Name":"tin", "Name_Localised":"Zinn", "Count":131 }, { "Name":"antimony", "Name_Localised":"Antimon", "Count":45 }, { "Name":"niobium", "Name_Localised":"Niob", "Count":44 }, { "Name":"zirconium", "Count":48 }, { "Name":"technetium", "Count":39 }, { "Name":"lead", "Name_Localised":"Blei", "Count":90 }, { "Name":"boron", "Name_Localised":"Bor", "Count":14 }, { "Name":"polonium", "Count":8 } ],
        // "Manufactured":[ { "Name":"hybridcapacitors", "Name_Localised":"Hybridkondensatoren", "Count":197 }, { "Name":"heatdispersionplate", "Name_Localised":"Wärmeverteilungsplatte", "Count":67 }, { "Name":"gridresistors", "Name_Localised":"Gitterwiderstände", "Count":242 }, { "Name":"mechanicalequipment", "Name_Localised":"Mechanisches Equipment", "Count":220 }, { "Name":"fedcorecomposites", "Name_Localised":"Core Dynamics Kompositwerkstoffe", "Count":100 }, { "Name":"protoheatradiators", "Name_Localised":"Proto-Wärmestrahler", "Count":6 }, { "Name":"salvagedalloys", "Name_Localised":"Geborgene Legierungen", "Count":300 }, { "Name":"highdensitycomposites", "Name_Localised":"Komposite hoher Dichte", "Count":200 }, { "Name":"mechanicalscrap", "Name_Localised":"Mechanischer Schrott", "Count":64 }, { "Name":"chemicalprocessors", "Name_Localised":"Chemische Prozessoren", "Count":250 }, { "Name":"focuscrystals", "Name_Localised":"Laserkristalle", "Count":200 }, { "Name":"imperialshielding", "Name_Localised":"Imperiale Schilde", "Count":53 }, { "Name":"precipitatedalloys", "Name_Localised":"Gehärtete Legierungen", "Count":200 }, { "Name":"galvanisingalloys", "Name_Localised":"Galvanisierende Legierungen", "Count":250 }, { "Name":"shieldingsensors", "Name_Localised":"Schildsensoren", "Count":200 }, { "Name":"chemicaldistillery", "Name_Localised":"Chemiedestillerie", "Count":200 }, { "Name":"heatconductionwiring", "Name_Localised":"Wärmeleitungsverdrahtung", "Count":128 }, { "Name":"phasealloys", "Name_Localised":"Phasenlegierungen", "Count":195 }, { "Name":"wornshieldemitters", "Name_Localised":"Gebrauchte Schildemitter", "Count":300 }, { "Name":"shieldemitters", "Name_Localised":"Schildemitter", "Count":250 }, { "Name":"mechanicalcomponents", "Name_Localised":"Mechanische Komponenten", "Count":11 }, { "Name":"compoundshielding", "Name_Localised":"Verbundschilde", "Count":150 }, { "Name":"protolightalloys", "Name_Localised":"Leichte Legierungen (Proto)", "Count":145 }, { "Name":"refinedfocuscrystals", "Name_Localised":"Raffinierte Laserkristalle", "Count":150 }, { "Name":"heatexchangers", "Name_Localised":"Wärmeaustauscher", "Count":6 }, { "Name":"conductiveceramics", "Name_Localised":"Elektrokeramiken", "Count":44 }, { "Name":"uncutfocuscrystals", "Name_Localised":"Fehlerhafte Fokuskristalle", "Count":250 }, { "Name":"temperedalloys", "Name_Localised":"Vergütete Legierungen", "Count":92 }, { "Name":"basicconductors", "Name_Localised":"Einfache Leiter", "Count":140 }, { "Name":"crystalshards", "Name_Localised":"Kristallscherben", "Count":288 }, { "Name":"unknownenergycell", "Name_Localised":"Thargoiden-Energiezelle", "Count":171 }, { "Name":"unknowntechnologycomponents", "Name_Localised":"Technologiekomponenten der Thargoiden", "Count":150 }, { "Name":"unknownenergysource", "Name_Localised":"Sensorenfragment", "Count":100 }, { "Name":"unknowncarapace", "Name_Localised":"Thargoiden-Krustenschale", "Count":220 }, { "Name":"unknownorganiccircuitry", "Name_Localised":"Organischer Schaltkreis der Thargoiden", "Count":100 }, { "Name":"chemicalmanipulators", "Name_Localised":"Chemische Manipulatoren", "Count":72 }, { "Name":"exquisitefocuscrystals", "Name_Localised":"Erlesene Laserkristalle", "Count":89 }, { "Name":"configurablecomponents", "Name_Localised":"Konfigurierbare Komponenten", "Count":36 }, { "Name":"heatvanes", "Name_Localised":"Wärmeleitbleche", "Count":1 }, { "Name":"biotechconductors", "Name_Localised":"Biotech-Leiter", "Count":57 }, { "Name":"conductivepolymers", "Name_Localised":"Leitfähige Polymere", "Count":5 }, { "Name":"thermicalloys", "Name_Localised":"Thermische Legierungen", "Count":150 }, { "Name":"conductivecomponents", "Name_Localised":"Leitfähige Komponenten", "Count":169 }, { "Name":"fedproprietarycomposites", "Name_Localised":"Kompositwerkstoffe", "Count":150 }, { "Name":"electrochemicalarrays", "Name_Localised":"Elektrochemische Detektoren", "Count":133 }, { "Name":"compactcomposites", "Name_Localised":"Kompaktkomposite", "Count":101 }, { "Name":"filamentcomposites", "Name_Localised":"Filament-Komposite", "Count":250 }, { "Name":"chemicalstorageunits", "Name_Localised":"Lagerungseinheiten für Chemiestoffe", "Count":57 }, { "Name":"protoradiolicalloys", "Name_Localised":"Radiologische Legierungen (Proto)", "Count":39 }, { "Name":"guardian_powercell", "Name_Localised":"Guardian-Energiezelle", "Count":300 }, { "Name":"guardian_powerconduit", "Name_Localised":"Guardian-Energieleiter", "Count":250 }, { "Name":"guardian_techcomponent", "Name_Localised":"Guardian-Technologiekomponenten", "Count":160 }, { "Name":"guardian_sentinel_weaponparts", "Name_Localised":"Guardian-Wache-Waffenteile", "Count":200 }, { "Name":"pharmaceuticalisolators", "Name_Localised":"Pharmazeutische Isolatoren", "Count":27 }, { "Name":"militarygradealloys", "Name_Localised":"Militärqualitätslegierungen", "Count":63 }, { "Name":"guardian_sentinel_wreckagecomponents", "Name_Localised":"Guardian-Wrackteilkomponenten", "Count":300 }, { "Name":"heatresistantceramics", "Name_Localised":"Hitzefeste Keramik", "Count":87 }, { "Name":"polymercapacitors", "Name_Localised":"Polymerkondensatoren", "Count":91 }, { "Name":"tg_biomechanicalconduits", "Name_Localised":"Biomechanische Leiter", "Count":105 }, { "Name":"tg_wreckagecomponents", "Name_Localised":"Wrackteilkomponenten", "Count":144 }, { "Name":"tg_weaponparts", "Name_Localised":"Waffenteile", "Count":135 }, { "Name":"tg_propulsionelement", "Name_Localised":"Schubantriebelemente", "Count":100 }, { "Name":"militarysupercapacitors", "Name_Localised":"Militärische Superkondensatoren", "Count":1 }, { "Name":"improvisedcomponents", "Name_Localised":"Behelfskomponenten", "Count":4 } ],
        // "Encoded":[ { "Name":"shielddensityreports", "Name_Localised":"Untypische Schildscans ", "Count":200 }, { "Name":"shieldcyclerecordings", "Name_Localised":"Gestörte Schildzyklus-Aufzeichnungen", "Count":234 }, { "Name":"encryptedfiles", "Name_Localised":"Ungewöhnliche verschlüsselte Files", "Count":92 }, { "Name":"bulkscandata", "Name_Localised":"Anormale Massen-Scan-Daten", "Count":192 }, { "Name":"decodedemissiondata", "Name_Localised":"Entschlüsselte Emissionsdaten", "Count":112 }, { "Name":"encryptioncodes", "Name_Localised":"Getaggte Verschlüsselungscodes", "Count":33 }, { "Name":"shieldsoakanalysis", "Name_Localised":"Inkonsistente Schildleistungsanalysen", "Count":250 }, { "Name":"scanarchives", "Name_Localised":"Unidentifizierte Scan-Archive", "Count":112 }, { "Name":"disruptedwakeechoes", "Name_Localised":"Atypische FSA-Stör-Aufzeichnungen", "Count":228 }, { "Name":"archivedemissiondata", "Name_Localised":"Irreguläre Emissionsdaten", "Count":65 }, { "Name":"legacyfirmware", "Name_Localised":"Spezial-Legacy-Firmware", "Count":78 }, { "Name":"scrambledemissiondata", "Name_Localised":"Außergewöhnliche verschlüsselte Emissionsdaten", "Count":84 }, { "Name":"encodedscandata", "Name_Localised":"Divergente Scandaten", "Count":30 }, { "Name":"fsdtelemetry", "Name_Localised":"Anormale FSA-Telemetrie", "Count":123 }, { "Name":"wakesolutions", "Name_Localised":"Seltsame FSA-Zielorte", "Count":93 }, { "Name":"emissiondata", "Name_Localised":"Unerwartete Emissionsdaten", "Count":142 }, { "Name":"shieldpatternanalysis", "Name_Localised":"Abweichende Schildeinsatz-Analysen", "Count":78 }, { "Name":"scandatabanks", "Name_Localised":"Scan-Datenbanken unter Verschluss", "Count":68 }, { "Name":"consumerfirmware", "Name_Localised":"Modifizierte Consumer-Firmware", "Count":48 }, { "Name":"symmetrickeys", "Name_Localised":"Offene symmetrische Schlüssel", "Count":24 }, { "Name":"shieldfrequencydata", "Name_Localised":"Verdächtige Schildfrequenz-Daten", "Count":50 }, { "Name":"compactemissionsdata", "Name_Localised":"Anormale kompakte Emissionsdaten", "Count":18 }, { "Name":"adaptiveencryptors", "Name_Localised":"Adaptive Verschlüsselungserfassung", "Count":64 }, { "Name":"encryptionarchives", "Name_Localised":"Atypische Verschlüsselungsarchive", "Count":63 }, { "Name":"dataminedwake", "Name_Localised":"FSA-Daten-Cache-Ausnahmen", "Count":19 }, { "Name":"securityfirmware", "Name_Localised":"Sicherheits-Firmware-Patch", "Count":29 }, { "Name":"embeddedfirmware", "Name_Localised":"Modifizierte integrierte Firmware", "Count":58 }, { "Name":"tg_residuedata", "Name_Localised":"Thargoiden-Rückstandsdaten", "Count":55 }, { "Name":"tg_compositiondata", "Name_Localised":"Materialzusammensetzungsdaten der Thargoiden", "Count":49 }, { "Name":"tg_structuraldata", "Name_Localised":"Thargoiden-Strukturdaten", "Count":49 }, { "Name":"unknownshipsignature", "Name_Localised":"Thargoiden-Schiffssignatur", "Count":37 }, { "Name":"unknownwakedata", "Name_Localised":"Thargoiden-Sogwolkendaten", "Count":55 }, { "Name":"ancienthistoricaldata", "Name_Localised":"Gamma-Muster-Obeliskendaten", "Count":150 }, { "Name":"ancienttechnologicaldata", "Name_Localised":"Epsilon-Muster-Obeliskendaten", "Count":150 }, { "Name":"ancientbiologicaldata", "Name_Localised":"Alpha-Muster-Obeliskendaten", "Count":150 }, { "Name":"ancientlanguagedata", "Name_Localised":"Delta-Muster-Obeliskendaten", "Count":150 }, { "Name":"ancientculturaldata", "Name_Localised":"Beta-Muster-Obeliskendaten", "Count":150 }, { "Name":"classifiedscandata", "Name_Localised":"Geheimes Scan-Fragment", "Count":18 }, { "Name":"hyperspacetrajectories", "Name_Localised":"Exzentrische Hyperraum-Routen", "Count":104 }, { "Name":"guardian_weaponblueprint", "Name_Localised":"Guardian-Waffenbauplanfragment", "Count":4 }, { "Name":"guardian_moduleblueprint", "Name_Localised":"Guardian-Modulbauplanfragment", "Count":7 }, { "Name":"guardian_vesselblueprint", "Name_Localised":"Guardian-Schiffsbauplanfragment", "Count":8 }, { "Name":"tg_shipflightdata", "Name_Localised":"Schiffsflugdaten", "Count":18 }, { "Name":"tg_shipsystemsdata", "Name_Localised":"Schiffssysteme-Daten", "Count":45 } ] }
        "Materials" => {
            let mut json = json.clone();

            {
                let mut material_json = json["Raw"].pop();
                while material_json != Null {
                    let result = materials.raw.get(&material_json["Name"].to_string());
                    match result {
                        None => {
                            warn!("Unknown material found! {}", material_json);
                            warn!("Looked for {}", &material_json["Name"].to_string());
                            let new_material = Material {
                                name: material_json["Name"].to_string(),
                                name_localised: material_json["Name_Localised"].to_string(),
                                grade: 0,
                                count: material_json["Count"].as_u64().unwrap(),
                                maximum: 0,
                                category: "".to_string(),
                                locations: vec![],
                                sources: vec![],
                                engineering: vec![],
                                synthesis: vec![],
                                description: "".to_string(),
                            };
                            materials
                                .raw
                                .insert(material_json["Name"].to_string(), new_material);
                        }
                        Some(material) => {
                            let updated_material = Material {
                                name: material.name.clone(),
                                name_localised: material_json["Name_Localised"].to_string(),
                                grade: material.grade,
                                count: material_json["Count"].as_u64().unwrap(),
                                maximum: material.maximum,
                                category: material.category.clone(),
                                locations: material.locations.clone(),
                                sources: material.sources.clone(),
                                engineering: material.engineering.clone(),
                                synthesis: material.synthesis.clone(),
                                description: material.description.clone(),
                            };
                            materials
                                .raw
                                .insert(material.name.clone(), updated_material);
                        }
                    }
                    material_json = json["Raw"].pop();
                }
            }
            {
                let mut material_json = json["Encoded"].pop();
                while material_json != Null {
                    let result = materials.encoded.get(&material_json["Name"].to_string());
                    match result {
                        None => {
                            warn!("Unknown material found! {}", material_json);
                            warn!("Looked for {}", &material_json["Name"].to_string());
                            let new_material = Material {
                                name: material_json["Name"].to_string(),
                                name_localised: material_json["Name_Localised"].to_string(),
                                grade: 0,
                                count: material_json["Count"].as_u64().unwrap(),
                                maximum: 0,
                                category: "".to_string(),
                                locations: vec![],
                                sources: vec![],
                                engineering: vec![],
                                synthesis: vec![],
                                description: "".to_string(),
                            };
                            materials
                                .encoded
                                .insert(material_json["Name"].to_string(), new_material);
                        }
                        Some(material) => {
                            let updated_material = Material {
                                name: material.name.clone(),
                                name_localised: material_json["Name_Localised"].to_string(),
                                grade: material.grade,
                                count: material_json["Count"].as_u64().unwrap(),
                                maximum: material.maximum,
                                category: material.category.clone(),
                                locations: material.locations.clone(),
                                sources: material.sources.clone(),
                                engineering: material.engineering.clone(),
                                synthesis: material.synthesis.clone(),
                                description: material.description.clone(),
                            };
                            materials
                                .encoded
                                .insert(material.name.clone(), updated_material);
                        }
                    }
                    material_json = json["Encoded"].pop();
                }
            }
            {
                let mut material_json = json["Manufactured"].pop();
                while material_json != Null {
                    let result = materials
                        .manufactured
                        .get(&material_json["Name"].to_string());
                    match result {
                        None => {
                            warn!("Unknown material found! {}", material_json);
                            warn!("Looked for {}", &material_json["Name"].to_string());
                            let new_material = Material {
                                name: material_json["Name"].to_string(),
                                name_localised: material_json["Name_Localised"].to_string(),
                                grade: 0,
                                count: material_json["Count"].as_u64().unwrap(),
                                maximum: 0,
                                category: "".to_string(),
                                locations: vec![],
                                sources: vec![],
                                engineering: vec![],
                                synthesis: vec![],
                                description: "".to_string(),
                            };
                            materials
                                .manufactured
                                .insert(material_json["Name"].to_string(), new_material);
                        }
                        Some(material) => {
                            let updated_material = Material {
                                name: material.name.clone(),
                                name_localised: material_json["Name_Localised"].to_string(),
                                grade: material.grade,
                                count: material_json["Count"].as_u64().unwrap(),
                                maximum: material.maximum,
                                category: material.category.clone(),
                                locations: material.locations.clone(),
                                sources: material.sources.clone(),
                                engineering: material.engineering.clone(),
                                synthesis: material.synthesis.clone(),
                                description: material.description.clone(),
                            };
                            materials
                                .manufactured
                                .insert(material.name.clone(), updated_material);
                        }
                    }
                    material_json = json["Manufactured"].pop();
                }
            }
        }
        "Cargo" => {}
        "MaterialCollected" => {
            //{ "timestamp":"2023-12-05T19:44:43Z", "event":"MaterialCollected", "Category":"Manufactured", "Name":"shieldemitters", "Name_Localised":"Schildemitter", "Count":3 }            let material_category = json["Category"].to_string();
            let name = &json["Name"].to_string();
            let material_category = json["Category"].to_string();
            match material_category.as_str() {
                "Manufactured" => {
                    if let Some(material) = materials.manufactured.get(name) {
                        let mut cloned_material = material.clone();
                        materials.manufactured.remove(name);
                        cloned_material.count += json["Count"].as_u64().unwrap();
                        materials.manufactured.insert(name.clone(), cloned_material);
                    } else {
                        error!(
                            "Didn't found manufactured material in material list: {}",
                            &json
                        );
                    }
                }
                "Encoded" => {
                    if let Some(material) = materials.encoded.get(name) {
                        let mut cloned_material = material.clone();
                        materials.encoded.remove(name);
                        cloned_material.count += json["Count"].as_u64().unwrap();
                        materials.encoded.insert(name.clone(), cloned_material);
                    } else {
                        error!("Didn't found encoded material in material list: {}", &json);
                    }
                }
                "Raw" => {
                    if let Some(material) = materials.raw.get(name) {
                        let mut cloned_material = material.clone();
                        materials.raw.remove(name);
                        cloned_material.count += json["Count"].as_u64().unwrap();
                        materials.raw.insert(name.clone(), cloned_material);
                    } else {
                        error!("Didn't found raw material in material list: {}", &json);
                    }
                }
                _ => {
                    error!("Unknown material: {}", &json);
                }
            }
        }
        "Synthesis" => {}
        "EjectCargo" => {}
        "DropItems" => {}
        "LaunchDrone" => {}
        "MiningRefined" => {}
        "ProspectedAsteroid" => {
            //{ "timestamp":"2023-06-05T12:05:12Z", "event":"ProspectedAsteroid", "Materials":[ { "Name":"rutile", "Name_Localised":"Rutil", "Proportion":35.986309 }, { "Name":"Bauxite", "Name_Localised":"Bauxit", "Proportion":13.713245 } ], "Content":"$AsteroidMaterialContent_Low;", "Content_Localised":"Materialgehalt: Niedrig", "Remaining":100.000000 }
            let mut json = json.clone();

            let mut materials: Vec<MiningMaterial> = Vec::new();
            let mut material_json = json["Materials"].pop();
            while material_json != Null {
                let mut buy_price = 0f64;
                let answer: Option<JsonValue> = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap()
                    .block_on(async {
                        let url = format!(
                            "https://api.edcas.de/data/odyssey/commodity/{}",
                            material_json["Name"].to_string().to_lowercase()
                        );
                        debug!("Api call to edcas: {}", url.clone());
                        let result = reqwest::get(url.clone()).await;
                        return match result {
                            Ok(response) => {
                                let text = response.text().await.unwrap();
                                let result = json::parse(text.as_str());
                                return match result {
                                    Ok(json) => Some(json),
                                    Err(err) => {
                                        error!("Couldn't parse answer to json: {}", err);
                                        error!("Value: {}", text);
                                        None
                                    }
                                };
                            }
                            Err(err) => {
                                error!(
                                    "Couldn't reach edcas api under {} Reason: {}",
                                    url.clone(),
                                    err
                                );
                                None
                            }
                        };
                    });

                match answer {
                    None => {}
                    Some(json) => {
                        buy_price = json["buy_price"].as_f64().unwrap_or(0f64);
                    }
                }
                materials.push(MiningMaterial {
                    name: material_json["Name"].to_string(),
                    name_localised: material_json["Name_Localised"].to_string(),
                    proportion: material_json["Proportion"].as_f64().unwrap_or(-1.0),
                    buy_price,
                });
                material_json = json["Materials"].pop();
            }

            let prospector: Prospector = Prospector {
                timestamp: json["timestamp"].to_string(),
                event: json["event"].to_string(),
                materials,
                content: json["Content"].to_string(),
                content_localised: json["Content_Localised"].to_string(),
                remaining: json["Remaining"].as_f64().unwrap_or(-1.0),
            };
            mining.prospectors.push_front(prospector);
        }
        "CargoTransfer" => {}
        "CollectCargo" => {}

        //Mission and Redeeming
        "Missions" => {}
        "MissionAccepted" => {}
        "MissionRedirected" => {}
        "MissionCompleted" => {}
        "RedeemVoucher" => {}
        "Bounty" => {}
        "NpcCrewPaidWage" => {}
        "PayFines" => {}
        "MissionAbandoned" => {}
        "MissionFailed" => {}
        "PayBounties" => {}
        "SellOrganicData" => {}

        //Carrier
        "CarrierStats" => {}
        "CarrierJumpRequest" => {}
        "CarrierTradeOrder" => {}
        "CarrierFinance" => {}
        "CarrierJumpCancelled" => {}
        "CarrierDepositFuel" => {}
        "CarrierDockingPermission" => {}
        "CarrierCrewServices" => {}
        "CarrierModulePack" => {}
        "CarrierBankTransfer" => {}

        //Dropship
        "BookDropship" => {}
        "DropshipDeploy" => {}

        //Wing
        "WingInvite" => {}
        "WingJoin" => {}
        "WingAdd" => {}
        "WingLeave" => {}

        //Crew
        "CrewMemberQuits" => {}
        "CrewMemberRoleChange" => {}
        "CrewMemberJoins" => {}
        "EndCrewSession" => {}

        "SellMicroResources" => {}
        "TradeMicroResources" => {}
        "FuelScoop" => {}
        "ReceiveText" => {}
        "Friends" => {}
        "Scanned" => {}
        "LoadGame" => {}
        "SquadronStartup" => {}
        "Music" => {}
        "CodexEntry" => {}
        "Rank" => {}
        "Progress" => {}
        "Reputation" => {}
        "Statistics" => {}
        "Commander" => {}
        "PowerplaySalary" => {}
        "Powerplay" => {}
        "CommitCrime" => {}
        "DockingDenied" => {}
        "HeatWarning" => {}
        "FactionKillBond" => {}
        "MultiSellExplorationData" => {}
        "SwitchSuitLoadout" => {}
        "MaterialTrade" => {
            //{ "timestamp":"2023-12-05T19:23:23Z", "event":"MaterialTrade", "MarketID":3223208960, "TraderType":"manufactured",
            // "Paid":{ "Material":"fedcorecomposites", "Material_Localised":"Core Dynamics Kompositwerkstoffe", "Category":"Manufactured", "Quantity":6 },
            // "Received":{ "Material":"protoradiolicalloys", "Material_Localised":"Radiologische Legierungen (Proto)", "Category":"Manufactured", "Quantity":1 } }
            let paid = &json["Paid"];
            let received = &json["Received"];
            match json["TraderType"].to_string().as_str() {
                "manufactured" => {
                    if let Some(paid_material) =
                        materials.manufactured.get(&paid["Material"].to_string())
                    {
                        let mut cloned_paid_material = paid_material.clone();
                        if let Some(received_material) = materials
                            .manufactured
                            .get(&received["Material"].to_string())
                        {
                            let mut cloned_received_material = received_material.clone();
                            materials.manufactured.remove(&paid["Material"].to_string());
                            materials
                                .manufactured
                                .remove(&received["Material"].to_string());
                            cloned_paid_material.count -= paid["Quantity"].as_u64().unwrap();
                            cloned_received_material.count +=
                                received["Quantity"].as_u64().unwrap();
                            materials
                                .manufactured
                                .insert(paid["Material"].to_string(), cloned_paid_material);
                            materials
                                .manufactured
                                .insert(received["Material"].to_string(), cloned_received_material);
                        } else {
                            error!(
                                "Didn't found manufactured material in material list: {}",
                                &json
                            );
                        }
                    } else {
                        error!(
                            "Didn't found manufactured material in material list: {}",
                            &json
                        );
                    }
                }
                "raw" => {
                    if let Some(paid_material) = materials.raw.get(&paid["Material"].to_string()) {
                        let mut cloned_paid_material = paid_material.clone();
                        if let Some(received_material) =
                            materials.raw.get(&received["Material"].to_string())
                        {
                            let mut cloned_received_material = received_material.clone();
                            materials.raw.remove(&paid["Material"].to_string());
                            materials.raw.remove(&received["Material"].to_string());
                            cloned_paid_material.count -= paid["Quantity"].as_u64().unwrap();
                            cloned_received_material.count +=
                                received["Quantity"].as_u64().unwrap();
                            materials
                                .raw
                                .insert(paid["Material"].to_string(), cloned_paid_material);
                            materials
                                .raw
                                .insert(received["Material"].to_string(), cloned_received_material);
                        } else {
                            error!(
                                "Didn't found manufactured material in material list: {}",
                                &json
                            );
                        }
                    } else {
                        error!(
                            "Didn't found manufactured material in material list: {}",
                            &json
                        );
                    }
                }
                "encoded" => {
                    if let Some(paid_material) =
                        materials.encoded.get(&paid["Material"].to_string())
                    {
                        let mut cloned_paid_material = paid_material.clone();
                        if let Some(received_material) =
                            materials.encoded.get(&received["Material"].to_string())
                        {
                            let mut cloned_received_material = received_material.clone();
                            materials.encoded.remove(&paid["Material"].to_string());
                            materials.encoded.remove(&received["Material"].to_string());
                            cloned_paid_material.count -= paid["Quantity"].as_u64().unwrap();
                            cloned_received_material.count +=
                                received["Quantity"].as_u64().unwrap();
                            materials
                                .encoded
                                .insert(paid["Material"].to_string(), cloned_paid_material);
                            materials
                                .encoded
                                .insert(received["Material"].to_string(), cloned_received_material);
                        } else {
                            error!(
                                "Didn't found manufactured material in material list: {}",
                                &json
                            );
                        }
                    } else {
                        error!(
                            "Didn't found manufactured material in material list: {}",
                            &json
                        );
                    }
                }
                &_ => {
                    error!("Unknown material trader: {}", &json);
                }
            }
        }
        "CommunityGoal" => {}
        "ModuleRetrieve" => {}
        "FetchRemoteModule" => {}
        "SendText" => {}
        "SearchAndRescue" => {}
        "HeatDamage" => {}
        "CommunityGoalReward" => {}
        "NavBeaconScan" => {}
        "USSDrop" => {}
        "Interdicted" => {}
        "Promotion" => {}
        "RepairDrone" => {}
        "DataScanned" => {}
        "DatalinkScan" => {}
        "DatalinkVoucher" => {}
        "CockpitBreached" => {}
        "SystemsShutdown" => {}
        "Screenshot" => {}
        "UpgradeWeapon" => {}
        "PowerplayFastTrack" => {}
        "PowerplayCollect" => {}
        "PowerplayDeliver" => {}
        "BookTaxi" => {}
        "SharedBookmarkToSquadron" => {}
        "MaterialDiscovered" => {}
        "SetUserShipName" => {}
        "FCMaterials" => {}
        "CommunityGoalJoin" => {}
        "SupercruiseDestinationDrop" => {}
        "JetConeBoost" => {}
        "AsteroidCracked" => {}
        "EscapeInterdiction" => {}
        "TechnologyBroker" => {}
        "NavBeaconDetail" => {}

        //Jesus
        "Died" => {}
        "Resurrect" => {}
        "SelfDestruct" => {}

        "Fileheader" => {}
        "Shutdown" => {}
        "" => {}
        _ => {
            warn!("Unknown event: {}", &event);
            println!("UNKNOWN EVENT:{}", event);
        }
    }
    if now.elapsed().as_secs() >= 1 {
        warn!(
            "Event took over a second ({}): {}",
            now.elapsed().as_secs(),
            &json
        );
    }
}
