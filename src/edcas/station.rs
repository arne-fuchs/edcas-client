use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::edcas::backend::evm::edcas_contract;
use crate::edcas::request_handler::EvmRequest;
use chrono::{DateTime, Utc};

use crate::edcas::settings::Settings;

pub struct StationState {
    pub stations: Vec<Station>,
    pub search: String,
    pub evm_request_writer: Sender<EvmRequest>,
    pub settings: Arc<Settings>,
}

#[derive(Clone)]
pub struct Station {
    pub market_id: u64,
    pub name: String,
    pub _type: String,
    pub requested_meta_data: bool,
    pub meta_data: Option<StationMetaData>,
    pub requested_market: bool,
    pub market: Option<Vec<CommodityListening>>,
}
#[derive(Clone)]
pub struct StationMetaData {
    pub timestamp: DateTime<Utc>,
    pub services: String,
    pub system_name: String,
    pub faction: edcas_contract::Faction,
    pub government: String,
    pub economy: String,
    pub distance: edcas_contract::Floating,
    pub landingpads: String,
}
#[derive(Clone)]
pub struct CommodityListening {
    pub name: String,
    pub buy_price: u32,
    pub mean_price: u32,
    pub demand: u32,
    pub demand_bracket: u32,
    pub stock: u32,
    pub stock_bracket: u32,
}
