use serde::{Deserialize, Serialize};

use crate::edcas::assets::station::StationEconomy;

#[derive(Serialize, Deserialize)]
pub struct Commodities {
    #[serde(rename = "economies")]
    economies: Option<Vec<StationEconomy>>,

    #[serde(rename = "systemName")]
    system_name: String,

    #[serde(rename = "prohibited")]
    prohibited: Option<Vec<String>>,

    #[serde(rename = "commodities")]
    commodities: Vec<Commodity>,

    #[serde(rename = "stationName")]
    station_name: String,

    #[serde(rename = "odyssey")]
    odyssey: bool,

    #[serde(rename = "horizons")]
    horizons: bool,

    #[serde(rename = "marketId")]
    market_id: i64,

    #[serde(rename = "timestamp")]
    timestamp: String,
}
impl Commodities {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        client: &mut postgres::Client,
    ) -> Result<(), crate::eddn::edcas_error::EdcasError> {
        use crate::eddn::edcas_error::EdcasError;

        let Self {
            economies,
            system_name,
            prohibited,
            commodities,
            station_name,
            odyssey: _,
            horizons: _,
            market_id,
            timestamp,
        } = self;

        //TODO Insert station economies
        // TODO Insert prohibited
        let mut transaction = client.transaction()?;
        if let Err(err) = transaction.execute(
            // language=postgresql
            "DELETE FROM commodity_listening WHERE market_id=$1",
            &[&market_id],
        ) {
            return Err(EdcasError::new(format!(
                "Couldn't delete old commodity_listening: {}",
                err
            )));
        }
        for commodity in commodities {
            commodity.insert_into_db(journal_id, market_id, &mut transaction)?;
        }
        transaction.commit()?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Commodity {
    #[serde(rename = "name")]
    name: String,

    #[serde(rename = "buyPrice")]
    buy_price: i32,

    #[serde(rename = "sellPrice")]
    sell_price: i32,

    #[serde(rename = "meanPrice")]
    mean_price: i32,

    #[serde(rename = "demandBracket")]
    demand_bracket: StringOrNumber,

    #[serde(rename = "stockBracket")]
    stock_bracket: StringOrNumber,

    #[serde(rename = "stock")]
    stock: StringOrNumber,

    #[serde(rename = "demand")]
    demand: StringOrNumber,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum StringOrNumber{
    String(String),
    Number(i32)
}
impl Commodity {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        market_id: i64,
        client: &mut postgres::Transaction,
    ) -> Result<(), crate::eddn::edcas_error::EdcasError> {
        use crate::edcas::tables::value_table;
        use crate::eddn::edcas_error::EdcasError;

        let Self {
            buy_price,
            demand_bracket,
            stock_bracket,
            name,
            sell_price,
            stock,
            mean_price,
            demand,
        } = self;
        let name = value_table(
            crate::edcas::tables::Tables::CommodityName,
            name,
            journal_id,
            client,
        )?;
        let demand = if let StringOrNumber::Number(number) = demand{number}else{0};
        let demand_bracket = if let StringOrNumber::Number(number) = demand_bracket{number}else{0};
        let stock = if let StringOrNumber::Number(number) = stock{number}else{0};
        let stock_bracket = if let StringOrNumber::Number(number) = stock_bracket{number}else{0};

        if let Err(err) = client.execute(
            // language=postgresql
            "INSERT INTO commodity_listening (commodity_name, market_id, buy_price, demand, demand_bracket, mean_price, sell_price, stock, stock_bracket, journal_id) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
            &[&name,&market_id,&buy_price,&demand,&demand_bracket,&mean_price,&sell_price,&stock,&stock_bracket,&journal_id]
        ) {
            return Err(EdcasError::new(format!("[Commodities] Couldn't insert commodity_listening: {}",err)))
        }
        Ok(())
    }
}
