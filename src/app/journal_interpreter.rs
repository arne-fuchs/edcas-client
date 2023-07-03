use std::collections::HashSet;
use json::{JsonValue, Null};
use log::{debug, error, info, warn};
use serde_json::Value;

use crate::app::explorer::{Body, BodySignal, Explorer, Page, Signal, SystemSignal};
use crate::app::materials::{MaterialState, Material};
use crate::app::mining::{Mining, Prospector, MiningMaterial};

pub fn interpret_json(json: JsonValue, explorer: &mut Explorer, material_inventory: &mut MaterialState, mining: &mut Mining) {
    let event = json["event"].as_str().unwrap();
    info!("Interpreter event received: {}", event);

    match event {
        //Navigation
        //{ "timestamp":"2022-10-16T20:54:45Z", "event":"Location", "DistFromStarLS":1007.705243, "Docked":true, "StationName":"Q2K-BHB", "StationType":"FleetCarrier", "MarketID":3704402432, "StationFaction":{ "Name":"FleetCarrier" }, "StationGovernment":"$government_Carrier;", "StationGovernment_Localised":"Privateigentum", "StationServices":[ "dock", "autodock", "commodities", "contacts", "exploration", "outfitting", "crewlounge", "rearm", "refuel", "repair", "shipyard", "engineer", "flightcontroller", "stationoperations", "stationMenu", "carriermanagement", "carrierfuel", "livery", "voucherredemption", "socialspace", "bartender", "vistagenomics" ], "StationEconomy":"$economy_Carrier;", "StationEconomy_Localised":"Privatunternehmen", "StationEconomies":[ { "Name":"$economy_Carrier;", "Name_Localised":"Privatunternehmen", "Proportion":1.000000 } ], "Taxi":false, "Multicrew":false, "StarSystem":"Colonia", "SystemAddress":3238296097059, "StarPos":[-9530.50000,-910.28125,19808.12500], "SystemAllegiance":"Independent", "SystemEconomy":"$economy_Tourism;", "SystemEconomy_Localised":"Tourismus", "SystemSecondEconomy":"$economy_HighTech;", "SystemSecondEconomy_Localised":"Hightech", "SystemGovernment":"$government_Cooperative;", "SystemGovernment_Localised":"Kooperative", "SystemSecurity":"$SYSTEM_SECURITY_low;", "SystemSecurity_Localised":"Geringe Sicherheit", "Population":583869, "Body":"Colonia 2 c", "BodyID":18, "BodyType":"Planet", "Factions":[ { "Name":"Jaques", "FactionState":"Investment", "Government":"Cooperative", "Influence":0.454092, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand1;", "Happiness_Localised":"In Hochstimmung", "MyReputation":100.000000, "RecoveringStates":[ { "State":"PublicHoliday", "Trend":0 } ], "ActiveStates":[ { "State":"Investment" }, { "State":"CivilLiberty" } ] }, { "Name":"Colonia Council", "FactionState":"Boom", "Government":"Cooperative", "Influence":0.331337, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":100.000000, "ActiveStates":[ { "State":"Boom" } ] }, { "Name":"People of Colonia", "FactionState":"None", "Government":"Cooperative", "Influence":0.090818, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":27.956400 }, { "Name":"Holloway Bioscience Institute", "FactionState":"None", "Government":"Corporate", "Influence":0.123752, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":-9.420000, "RecoveringStates":[ { "State":"PirateAttack", "Trend":0 } ] } ], "SystemFaction":{ "Name":"Jaques", "FactionState":"Investment" } }
        //{ "timestamp":"2022-10-16T23:25:31Z", "event":"FSDJump", "Taxi":false, "Multicrew":false, "StarSystem":"Ogmar", "SystemAddress":84180519395914, "StarPos":[-9534.00000,-905.28125,19802.03125], "SystemAllegiance":"Independent", "SystemEconomy":"$economy_HighTech;", "SystemEconomy_Localised":"Hightech", "SystemSecondEconomy":"$economy_Military;", "SystemSecondEconomy_Localised":"Militär", "SystemGovernment":"$government_Confederacy;", "SystemGovernment_Localised":"Konföderation", "SystemSecurity":"$SYSTEM_SECURITY_medium;", "SystemSecurity_Localised":"Mittlere Sicherheit", "Population":151752, "Body":"Ogmar A", "BodyID":1, "BodyType":"Star", "JumpDist":8.625, "FuelUsed":0.024493, "FuelLevel":31.975506, "Factions":[ { "Name":"Jaques", "FactionState":"Election", "Government":"Cooperative", "Influence":0.138384, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand1;", "Happiness_Localised":"In Hochstimmung", "MyReputation":100.000000, "PendingStates":[ { "State":"Outbreak", "Trend":0 } ], "ActiveStates":[ { "State":"Election" } ] }, { "Name":"ICU Colonial Corps", "FactionState":"War", "Government":"Communism", "Influence":0.119192, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":96.402496, "PendingStates":[ { "State":"Expansion", "Trend":0 } ], "ActiveStates":[ { "State":"War" } ] }, { "Name":"Societas Eruditorum de Civitas Dei", "FactionState":"War", "Government":"Dictatorship", "Influence":0.119192, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":46.414799, "ActiveStates":[ { "State":"War" } ] }, { "Name":"GalCop Colonial Defence Commission", "FactionState":"Boom", "Government":"Confederacy", "Influence":0.406061, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":-75.000000, "ActiveStates":[ { "State":"Boom" } ] }, { "Name":"Likedeeler of Colonia", "FactionState":"None", "Government":"Democracy", "Influence":0.068687, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":4.002500 }, { "Name":"Colonia Tech Combine", "FactionState":"Election", "Government":"Cooperative", "Influence":0.138384, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":4.850000, "ActiveStates":[ { "State":"Election" } ] }, { "Name":"Milanov's Reavers", "FactionState":"Bust", "Government":"Anarchy", "Influence":0.010101, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":0.000000, "RecoveringStates":[ { "State":"Terrorism", "Trend":0 } ], "ActiveStates":[ { "State":"Bust" } ] } ], "SystemFaction":{ "Name":"GalCop Colonial Defence Commission", "FactionState":"Boom" }, "Conflicts":[ { "WarType":"election", "Status":"active", "Faction1":{ "Name":"Jaques", "Stake":"Guerrero Military Base", "WonDays":1 }, "Faction2":{ "Name":"Colonia Tech Combine", "Stake":"", "WonDays":0 } }, { "WarType":"war", "Status":"active", "Faction1":{ "Name":"ICU Colonial Corps", "Stake":"Boulaid Command Facility", "WonDays":1 }, "Faction2":{ "Name":"Societas Eruditorum de Civitas Dei", "Stake":"Chatterjee's Respite", "WonDays":0 } } ] }
        "FSDJump" | "Location" | "CarrierJump" => {
            let page = Page{
                system: explorer.system.clone(),
                body_list: explorer.body_list.clone(),
                body_signal_list: explorer.body_signal_list.clone(),
                system_signal_list: explorer.system_signal_list.clone(),
                body: explorer.body.clone(),
            };


            explorer.pages.push(page);

            explorer.index = explorer.pages.len();

            //if explorer.index == explorer.pages.len()-1{
            //    explorer.index = explorer.pages.len();
            //}

            explorer.system.name = json["StarSystem"].to_string();
            explorer.system.allegiance = json["SystemAllegiance"].to_string();
            explorer.system.economy_localised = json["SystemEconomy_Localised"].to_string();
            explorer.system.second_economy_localised = json["SystemSecondEconomy_Localised"].to_string();
            explorer.system.government_localised = json["SystemGovernment_Localised"].to_string();
            explorer.system.security_localised = json["SystemSecurity_Localised"].to_string();
            explorer.system.population = json["Population"].to_string();
            explorer.system.body_count = "N/A".to_string();
            explorer.system.non_body_count = "N/A".to_string();

            explorer.body_list.clear();
            explorer.body_signal_list.clear();
            explorer.system_signal_list.clear();
            explorer.body = Body::default();

            info!("Found system: {}",explorer.system.name.clone());
        }
        "SupercruiseEntry" => {}
        "SupercruiseExit" => {}
        //{ "timestamp":"2022-10-16T23:25:05Z", "event":"StartJump", "JumpType":"Hyperspace", "StarSystem":"Ogmar", "SystemAddress":84180519395914, "StarClass":"K" }
        "StartJump" => {} //If jump has been initialised
        //{ "timestamp":"2022-10-16T23:24:46Z", "event":"FSDTarget", "Name":"Ogmar", "SystemAddress":84180519395914, "StarClass":"K", "RemainingJumpsInRoute":1 }
        "FSDTarget" => {} //If system has been targeted
        "NavRoute" => {} //If route has been set -> check json for further information
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
            explorer.system.body_count = json["BodyCount"].to_string();
            explorer.system.non_body_count = json["NonBodyCount"].to_string();
        }//Honk
        //{ "timestamp":"2022-07-07T20:58:06Z", "event":"SAASignalsFound", "BodyName":"IC 2391 Sector YE-A d103 B 1", "SystemAddress":3549631072611, "BodyID":15, "Signals":[ { "Type":"$SAA_SignalType_Guardian;", "Type_Localised":"Guardian", "Count":1 }, { "Type":"$SAA_SignalType_Human;", "Type_Localised":"Menschlich", "Count":9 } ] }
        "FSSBodySignals" | "SAASignalsFound" => {
            //TODO Implement NFT
            //{ "timestamp":"2022-09-07T17:50:41Z", "event":"FSSBodySignals", "BodyName":"Synuefe EN-H d11-106 6 a", "BodyID":31, "SystemAddress":3652777380195, "Signals":[ { "Type":"$SAA_SignalType_Biological;", "Type_Localised":"Biologisch", "Count":1 }, { "Type":"$SAA_SignalType_Geological;", "Type_Localised":"Geologisch", "Count":3 } ] }
            let mut signals: Vec<Signal> = Vec::new();

            for i in 0..json["Signals"].len() {
                signals.push(Signal {
                    r#type: json["Signals"][i]["Type"].to_string(),
                    type_localised: json["Signals"][i]["Type_Localised"].to_string(),
                    count: json["Signals"][i]["Count"].as_i64().unwrap_or(-1),
                })
            }
            info!("Body {} number of signals: {}",json["BodyName"].to_string(),signals.len().clone());

            let body_signal = BodySignal {
                timestamp: json["timestamp"].to_string(),
                event: json["event"].to_string(),
                body_name: json["BodyName"].to_string(),
                body_id: json["BodyID"].as_i64().unwrap_or(-1),
                system_address: json["SystemAddress"].as_i64().unwrap(),
                signals,
            };

            let id = body_signal.body_id;

            if !explorer.body_signal_list.iter().any(|x| x.body_id == id) {
                explorer.body_signal_list.push(body_signal);

                explorer.body_signal_list.sort_by(|signal_a, signal_b| {
                    //TODO Better sorting -> All signals have to be looked at. Sort by largest in the list maybe
                    signal_a.signals.first().unwrap().count.cmp(&signal_b.signals.first().unwrap().count).reverse()
                });
            }
        }
        "FSSSignalDiscovered" => {
            //{ "timestamp":"2023-05-29T22:40:26Z", "event":"FSSSignalDiscovered", "SystemAddress":672296347049, "SignalName":"$MULTIPLAYER_SCENARIO80_TITLE;", "SignalName_Localised":"Unbewachtes Navigationssignal" }
            // { "timestamp":"2023-05-29T22:40:26Z", "event":"FSSSignalDiscovered", "SystemAddress":672296347049, "SignalName":"THE GENERAL MELCHETT X5W-0XL", "IsStation":true }
            //{ "timestamp":"2023-05-29T22:40:42Z", "event":"FSSSignalDiscovered", "SystemAddress":672296347049, "SignalName":"$USS_HighGradeEmissions;", "SignalName_Localised":"Unidentifizierte Signalquelle",
            // "USSType":"$USS_Type_ValuableSalvage;", "USSType_Localised":"Verschlüsselte Emissionen", "SpawningState":"", "SpawningFaction":"Murus Major Industry", "ThreatLevel":0, "TimeRemaining":707.545837 }
            let mut name = json["SignalName_Localised"].to_string();
            if name == "null".to_string()  {
                name = json["SignalName"].to_string();
                if name == "null".to_string()  {
                    name = json["USSType_Localised"].to_string();
                }
            }

            let mut thread =  json["ThreatLevel"].to_string();
            if thread == "null".to_string()  {
                thread = "".to_string();
            }

            let system_signal = SystemSignal {
                timestamp: json["timestamp"].to_string(),
                event: json["event"].to_string(),
                name,
                thread: "".to_string(),
            };
            explorer.system_signal_list.push(system_signal);
            explorer.system_signal_list.sort_by(|a,b|{
                if a.name == b.name{
                    a.thread.cmp(&b.thread)
                }else {
                    a.name.cmp(&b.name)
                }
            });
        }
        "SAAScanComplete" => {}
        //{ "timestamp":"2022-10-16T23:51:17Z", "event":"Scan", "ScanType":"Detailed", "BodyName":"Ogmar A 6", "BodyID":40, "Parents":[ {"Star":1}, {"Null":0} ], "StarSystem":"Ogmar", "SystemAddress":84180519395914, "DistanceFromArrivalLS":3376.246435, "TidalLock":false, "TerraformState":"", "PlanetClass":"Sudarsky class I gas giant", "Atmosphere":"", "AtmosphereComposition":[ { "Name":"Hydrogen", "Percent":73.044167 }, { "Name":"Helium", "Percent":26.955832 } ], "Volcanism":"", "MassEM":24.477320, "Radius":22773508.000000, "SurfaceGravity":18.811067, "SurfaceTemperature":62.810730, "SurfacePressure":0.000000, "Landable":false, "SemiMajorAxis":1304152250289.916992, "Eccentricity":0.252734, "OrbitalInclination":156.334694, "Periapsis":269.403039, "OrbitalPeriod":990257555.246353, "AscendingNode":-1.479320, "MeanAnomaly":339.074691, "RotationPeriod":37417.276422, "AxialTilt":0.018931, "WasDiscovered":true, "WasMapped":true }
        "Scan" => {
            info!("Body found: {}",json["BodyName"].to_string());

            let body = Body {
                name: json["BodyName"].to_string(),
                body_id: json["BodyID"].to_string(),
                parents: json["Parents"].to_string().to_owned(),
                star_system: json["StarSystem"].to_string(),
                system_address: json["SystemAddress"].to_string(),
                distance_from_arrival_ls: json["DistanceFromArrivalLS"].to_string(),
                tidal_lock: json["TidalLock"].to_string(),
                terraform_state: json["TerraformState"].to_string(),
                planet_class: json["PlanetClass"].to_string(),
                atmosphere: json["Atmosphere"].to_string(),
                atmosphere_composition: json["AtmosphereComposition"].to_string(),
                volcanism: json["Volcanism"].to_string(),
                mass_em: json["MassEM"].to_string(),
                radius: json["Radius"].to_string(),
                surface_gravity: json["SurfaceGravity"].to_string(),
                surface_temperature: json["SurfaceTemperature"].to_string(),
                surface_pressure: json["SurfacePressure"].to_string(),
                landable: json["Landable"].to_string(),
                semi_major_axis: json["SemiMajorAxis"].to_string(),
                eccentricity: json["Eccentricity"].to_string(),
                orbital_inclination: json["OrbitalInclination"].to_string(),
                periapsis: json["Periapsis"].to_string(),
                orbital_period: json["OrbitalPeriod"].to_string(),
                ascending_node: json["AscendingNode"].to_string(),
                mean_anomaly: json["MeanAnomaly"].to_string(),
                rotation_period: json["RotationPeriod"].to_string(),
                axial_tilt: json["AxialTilt"].to_string(),
                was_discovered: json["WasDiscovered"].to_string(),
                was_mapped: json["WasMapped"].to_string(),
            };

            explorer.body = body.clone();
            explorer.body_list.push(body);

            explorer.body_list.sort_by(|body_a, body_b| {
                let id_a: i32 = body_a.body_id.parse().unwrap();
                let id_b: i32 = body_b.body_id.parse().unwrap();
                id_a.cmp(&id_b)
            });
            explorer.body_list.dedup_by(|body_a, body_b| {
                body_a.name.eq(&body_b.name)
            });
        }//Planet scan with fss
        "ScanBaryCentre" => {}

        //Maintenance
        "RefuelAll" => {}
        "Resupply" => {}
        "Repair" => {}
        "BuyDrones" => {}
        "SellDrones" => {}
        "BuyAmmo" => {}
        //{ "timestamp":"2022-10-16T23:55:55Z", "event":"ReservoirReplenished", "FuelMain":30.905506, "FuelReservoir":1.070000 }
        "ReservoirReplenished" => {}//If reservoir needs to drain more fuel from main tank
        "RepairAll" => {}
        "RebootRepair" => {}
        "RestockVehicle" => {}

        //Docking
        "DockingRequested" => {}
        "DockingGranted" => {}
        "Docked" => {}
        "Undocked" => {}

        //Engineer
        "EngineerProgress" => {}
        "EngineerCraft" => {}
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

            let mut raw: Vec<Material> = vec![];
            let mut manufactured: Vec<Material> = vec![];
            let mut encoded: Vec<Material> = vec![];

            {
                let mut material_json = json["Raw"].pop();
                while material_json != Null {
                    raw.push(Material {
                        name: material_json["Name"].to_string(),
                        name_localised: material_json["Name_Localised"].to_string(),
                        count: material_json["Count"].as_u64().unwrap(),
                    });
                    material_json = json["Raw"].pop();
                }
            }

            {
                let mut material_json = json["Manufactured"].pop();
                while material_json != Null {
                    manufactured.push(Material {
                        name: material_json["Name"].to_string(),
                        name_localised: material_json["Name_Localised"].to_string(),
                        count: material_json["Count"].as_u64().unwrap(),
                    });
                    material_json = json["Manufactured"].pop();
                }
            }

            {
                let mut material_json = json["Encoded"].pop();
                while material_json != Null {
                    encoded.push(Material {
                        name: material_json["Name"].to_string(),
                        name_localised: material_json["Name_Localised"].to_string(),
                        count: material_json["Count"].as_u64().unwrap(),
                    });
                    material_json = json["Encoded"].pop();
                }
            }

            raw.sort_by(|a, b| {
                let mut a_name = a.name_localised.to_string();
                let mut b_name = b.name_localised.to_string();

                if a_name == "null" {
                    a_name = a.name.to_string();
                }
                if b_name == "null" {
                    b_name = b.name.to_string();
                }
                a_name.cmp(&b_name)
            });

            encoded.sort_by(|a, b| {
                let mut a_name = a.name_localised.to_string();
                let mut b_name = b.name_localised.to_string();

                if a_name == "null" {
                    a_name = a.name.to_string();
                }
                if b_name == "null" {
                    b_name = b.name.to_string();
                }
                a_name.cmp(&b_name)
            });

            manufactured.sort_by(|a, b| {
                let mut a_name = a.name_localised.to_string();
                let mut b_name = b.name_localised.to_string();

                if a_name == "null" {
                    a_name = a.name.to_string();
                }
                if b_name == "null" {
                    b_name = b.name.to_string();
                }
                a_name.cmp(&b_name)
            });

            material_inventory.raw = raw;
            material_inventory.manufactured = manufactured;
            material_inventory.encoded = encoded;
        }
        "Cargo" => {}
        "MaterialCollected" => {}
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
                        let url = format!("https://api.edcas.de/data/commodity/{}",material_json["Name"].to_string().to_lowercase());
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
                        buy_price = json["highest_sell_price"]["sell_price"].as_f64().unwrap_or(0f64);
                    }
                }
                materials.push(MiningMaterial {
                    name: material_json["Name"].to_string(),
                    name_localised: material_json["Name_Localised"].to_string(),
                    proportion: material_json["Proportion"].as_f64().unwrap_or(-1.0),
                    buy_price
                });
                material_json = json["Materials"].pop();
            }

            let prospector: Prospector = Prospector{
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
        "MaterialTrade" => {}
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
}