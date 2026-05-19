use serde::{Deserialize, Serialize};

use crate::journal::types::{LandingPads, StationEconomy, StationFaction};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Docked {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "StationName")]
    pub station_name: String,
    #[serde(rename = "StationType")]
    pub station_type: String,
    #[serde(rename = "MarketID")]
    pub market_id: i64,
    #[serde(rename = "SystemAddress")]
    pub system_address: i64,
    #[serde(rename = "StarSystem")]
    pub star_system: String,
    #[serde(rename = "StationFaction")]
    pub station_faction: Option<StationFaction>,
    #[serde(rename = "StationGovernment", default)]
    pub station_government: String,
    #[serde(rename = "StationGovernment_Localised", default)]
    pub station_government_localised: String,
    #[serde(rename = "StationAllegiance", default)]
    pub station_allegiance: String,
    #[serde(rename = "StationServices")]
    pub station_services: Option<Vec<String>>,
    #[serde(rename = "StationEconomy", default)]
    pub station_economy: String,
    #[serde(rename = "StationEconomy_Localised", default)]
    pub station_economy_localised: String,
    #[serde(rename = "StationEconomies")]
    pub station_economies: Option<Vec<StationEconomy>>,
    #[serde(rename = "LandingPads")]
    pub landing_pads: Option<LandingPads>,
    #[serde(rename = "DistFromStarLS")]
    pub dist_from_star_ls: Option<f32>,
    #[serde(rename = "Wanted", default)]
    pub wanted: bool,
    #[serde(rename = "ActiveFine", default)]
    pub active_fine: bool,
    #[serde(rename = "Taxi", default)]
    pub taxi: bool,
    #[serde(rename = "horizons", default)]
    pub horizons: bool,
    #[serde(rename = "odyssey", default)]
    pub odyssey: bool,
}

/// Emitted when the player views their fleet carrier stats panel.
/// Contains the custom owner-given name separate from the callsign.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CarrierStats {
    #[serde(rename = "CarrierID")]
    pub carrier_id: i64,
    #[serde(rename = "Callsign")]
    pub callsign: String,
    #[serde(rename = "Name")]
    pub name: String,
}

/// EDDN commodities schema (not a journal event; sent separately by the game)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Commodities {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "marketId")]
    pub market_id: i64,
    #[serde(rename = "stationName")]
    pub station_name: String,
    #[serde(rename = "systemName")]
    pub system_name: String,
    #[serde(rename = "horizons", default)]
    pub horizons: bool,
    #[serde(rename = "odyssey", default)]
    pub odyssey: bool,
    #[serde(rename = "commodities")]
    pub commodities: Vec<Commodity>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Commodity {
    pub name: String,
    #[serde(rename = "meanPrice")]
    pub mean_price: i32,
    #[serde(rename = "buyPrice")]
    pub buy_price: i32,
    #[serde(rename = "stock")]
    pub stock: i32,
    #[serde(rename = "stockBracket")]
    pub stock_bracket: i32,
    #[serde(rename = "sellPrice")]
    pub sell_price: i32,
    #[serde(rename = "demand")]
    pub demand: i32,
    #[serde(rename = "demandBracket")]
    pub demand_bracket: i32,
    #[serde(rename = "statusFlags")]
    pub status_flags: Option<Vec<String>>,
}

/// EDDN outfitting / modules schema
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Outfitting {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "marketId")]
    pub market_id: i64,
    #[serde(rename = "stationName")]
    pub station_name: String,
    #[serde(rename = "systemName")]
    pub system_name: String,
    #[serde(rename = "horizons", default)]
    pub horizons: bool,
    #[serde(rename = "odyssey", default)]
    pub odyssey: bool,
    pub modules: Vec<OutfittingModule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutfittingModule {
    pub id: i64,
    pub category: String,
    pub name: String,
    pub cost: Option<i64>,
    pub ship: Option<String>,
}

/// EDDN shipyard schema
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Shipyard {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "marketId")]
    pub market_id: i64,
    #[serde(rename = "stationName")]
    pub station_name: String,
    #[serde(rename = "systemName")]
    pub system_name: String,
    #[serde(rename = "horizons", default)]
    pub horizons: bool,
    #[serde(rename = "odyssey", default)]
    pub odyssey: bool,
    pub ships: Vec<ShipyardShip>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShipyardShip {
    pub id: i64,
    pub name: String,
    #[serde(rename = "basevalue")]
    pub base_value: Option<i64>,
}

// ── Journal companion-file format (PascalCase / game-native) ─────────────────

/// Market.json companion-file format (written by the game when opening the market UI).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketJournal {
    pub timestamp: String,
    #[serde(rename = "MarketID")]
    pub market_id: i64,
    #[serde(rename = "StationName")]
    pub station_name: String,
    #[serde(rename = "StarSystem")]
    pub star_system: String,
    #[serde(rename = "Items", default)]
    pub items: Vec<MarketJournalItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketJournalItem {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "MeanPrice")]
    pub mean_price: i32,
    #[serde(rename = "BuyPrice")]
    pub buy_price: i32,
    #[serde(rename = "Stock")]
    pub stock: i32,
    #[serde(rename = "StockBracket")]
    pub stock_bracket: i32,
    #[serde(rename = "SellPrice")]
    pub sell_price: i32,
    #[serde(rename = "Demand")]
    pub demand: i32,
    #[serde(rename = "DemandBracket")]
    pub demand_bracket: i32,
}

impl From<MarketJournal> for Commodities {
    fn from(m: MarketJournal) -> Self {
        let commodities = m.items.into_iter().map(|item| {
            // "$platinum_name;" → "platinum"
            let name = item.name
                .trim_start_matches('$')
                .trim_end_matches("_name;")
                .to_owned();
            Commodity {
                name,
                mean_price: item.mean_price,
                buy_price: item.buy_price,
                stock: item.stock,
                stock_bracket: item.stock_bracket,
                sell_price: item.sell_price,
                demand: item.demand,
                demand_bracket: item.demand_bracket,
                status_flags: None,
            }
        }).collect();
        Commodities {
            timestamp: m.timestamp,
            market_id: m.market_id,
            station_name: m.station_name,
            system_name: m.star_system,
            horizons: false,
            odyssey: false,
            commodities,
        }
    }
}

/// Outfitting.json companion-file format.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutfittingJournal {
    pub timestamp: String,
    #[serde(rename = "MarketID")]
    pub market_id: i64,
    #[serde(rename = "StationName")]
    pub station_name: String,
    #[serde(rename = "StarSystem")]
    pub star_system: String,
    #[serde(rename = "Items", default)]
    pub items: Vec<OutfittingJournalItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OutfittingJournalItem {
    pub id: i64,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "BuyPrice")]
    pub buy_price: i32,
}

impl From<OutfittingJournal> for Outfitting {
    fn from(o: OutfittingJournal) -> Self {
        let modules = o.items.into_iter().map(|item| {
            let category = if item.name.starts_with("hpt_") {
                "hardpoint"
            } else if item.name.starts_with("int_") {
                "internal"
            } else {
                ""
            };
            OutfittingModule {
                id: item.id,
                category: category.to_owned(),
                name: item.name,
                cost: Some(item.buy_price as i64),
                ship: None,
            }
        }).collect();
        Outfitting {
            timestamp: o.timestamp,
            market_id: o.market_id,
            station_name: o.station_name,
            system_name: o.star_system,
            horizons: false,
            odyssey: false,
            modules,
        }
    }
}

/// Shipyard.json companion-file format.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShipyardJournal {
    pub timestamp: String,
    #[serde(rename = "MarketID")]
    pub market_id: i64,
    #[serde(rename = "StationName")]
    pub station_name: String,
    #[serde(rename = "StarSystem")]
    pub star_system: String,
    #[serde(rename = "PriceList", default)]
    pub price_list: Vec<ShipyardJournalItem>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShipyardJournalItem {
    #[serde(rename = "ShipType")]
    pub ship_type: String,
    #[serde(rename = "ShipPrice")]
    pub ship_price: i64,
}

impl From<ShipyardJournal> for Shipyard {
    fn from(s: ShipyardJournal) -> Self {
        // The companion file always has id=0 for all ships, so we use a sequential
        // index as the unique key within this market to avoid DB conflicts.
        let ships = s.price_list.into_iter().enumerate().map(|(i, item)| {
            ShipyardShip {
                id: i as i64,
                name: item.ship_type,
                base_value: Some(item.ship_price),
            }
        }).collect();
        Shipyard {
            timestamp: s.timestamp,
            market_id: s.market_id,
            station_name: s.station_name,
            system_name: s.star_system,
            horizons: false,
            odyssey: false,
            ships,
        }
    }
}
