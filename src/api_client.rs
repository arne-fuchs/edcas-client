use edcas_common::api::{
    BodyResponse, CommodityPricePoint, ConstructionDepotResponse,
    ConstructionDepotSubmission, ConstructionQuery, FactionQuery, FactionResponse, InfluencePoint,
    MultiCommodityQuery, MultiCommodityResult, NearestCommodityQuery, NearestCommodityResult,
    ServerTickResponse, StationQuery, StationResponse, TradeLoopResponse, TradeRouteResponse,
};

// ─── Native (async) implementation ────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
use tracing::{debug, warn};

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
    rt: tokio::runtime::Handle,
}

#[cfg(not(target_arch = "wasm32"))]
impl ApiClient {
    pub fn new(base_url: impl Into<String>, rt: tokio::runtime::Handle) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("failed to build HTTP client"),
            rt,
        }
    }

    /// Spawn an async task on the background runtime. Results should be
    /// communicated back via channels or shared state — do not block the UI.
    pub fn spawn<F>(&self, f: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.rt.spawn(f);
    }

    /// Returns a clone of the underlying async HTTP client.
    pub fn http_client(&self) -> reqwest::Client {
        self.client.clone()
    }

    pub async fn get_bodies(&self, system_address: i64) -> anyhow::Result<Vec<BodyResponse>> {
        let url = format!("{}/api/v1/systems/{}/bodies", self.base_url, system_address);
        debug!(url, system_address, "API call: get_bodies");
        let resp = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            let result: Vec<BodyResponse> = resp.json().await?;
            debug!(url, count = result.len(), "API response: get_bodies");
            Ok(result)
        } else {
            warn!(url, %status, "API response: get_bodies failed");
            Ok(vec![])
        }
    }

    pub async fn search_stations(&self, query: &StationQuery) -> anyhow::Result<Vec<StationResponse>> {
        let url = format!("{}/api/v1/stations", self.base_url);
        debug!(url, ?query, "API call: search_stations");
        let resp = self.client.get(&url).query(query).send().await?;
        let status = resp.status();
        if status.is_success() {
            let result: Vec<StationResponse> = resp.json().await?;
            debug!(url, count = result.len(), "API response: search_stations");
            Ok(result)
        } else {
            warn!(url, %status, "API response: search_stations failed");
            Ok(vec![])
        }
    }

    pub async fn search_factions(&self, query: &FactionQuery) -> anyhow::Result<Vec<FactionResponse>> {
        let url = format!("{}/api/v1/factions", self.base_url);
        debug!(url, ?query, "API call: search_factions");
        let resp = self.client.get(&url).query(query).send().await?;
        let status = resp.status();
        if status.is_success() {
            let result: Vec<FactionResponse> = resp.json().await?;
            debug!(url, count = result.len(), "API response: search_factions");
            Ok(result)
        } else {
            warn!(url, %status, "API response: search_factions failed");
            Ok(vec![])
        }
    }

    pub async fn search_construction_depots(
        &self,
        query: &ConstructionQuery,
    ) -> anyhow::Result<Vec<ConstructionDepotResponse>> {
        let url = format!("{}/api/v1/construction-depots", self.base_url);
        debug!(url, ?query, "API call: search_construction_depots");
        let resp = self.client.get(&url).query(query).send().await?;
        let status = resp.status();
        if status.is_success() {
            let result: Vec<ConstructionDepotResponse> = resp.json().await?;
            debug!(url, count = result.len(), "API response: search_construction_depots");
            Ok(result)
        } else {
            warn!(url, %status, "API response: search_construction_depots failed");
            Ok(vec![])
        }
    }

    pub async fn submit_construction_depot(
        &self,
        submission: &ConstructionDepotSubmission,
    ) -> anyhow::Result<()> {
        let url = format!("{}/api/v1/construction-depots", self.base_url);
        debug!(url, ?submission, "API call: submit_construction_depot");
        let resp = self.client.post(&url).json(submission).send().await?;
        let status = resp.status();
        if status.is_success() {
            debug!(url, "API response: submit_construction_depot ok");
        } else {
            warn!(url, %status, "API response: submit_construction_depot failed");
        }
        Ok(())
    }

    pub async fn fetch_trade_routes(&self) -> anyhow::Result<Vec<TradeRouteResponse>> {
        let url = format!("{}/api/v1/trade-routes", self.base_url);
        debug!(url, "API call: fetch_trade_routes");
        let resp = self.client.get(&url).send().await?;
        let status = resp.status();
        if status.is_success() {
            let result: Vec<TradeRouteResponse> = resp.json().await?;
            debug!(url, count = result.len(), "API response: fetch_trade_routes");
            Ok(result)
        } else {
            warn!(url, %status, "API response: fetch_trade_routes failed");
            Ok(vec![])
        }
    }

    pub async fn fetch_trade_loops(&self) -> anyhow::Result<Vec<TradeLoopResponse>> {
        let url = format!("{}/api/v1/trade-loops", self.base_url);
        debug!(url, "API call: fetch_trade_loops");
        let resp = self.client.get(&url).send().await?;
        let status = resp.status();
        if status.is_success() {
            let result: Vec<TradeLoopResponse> = resp.json().await?;
            debug!(url, count = result.len(), "API response: fetch_trade_loops");
            Ok(result)
        } else {
            warn!(url, %status, "API response: fetch_trade_loops failed");
            Ok(vec![])
        }
    }

    pub async fn get_server_tick(&self) -> anyhow::Result<Option<ServerTickResponse>> {
        let url = format!("{}/api/v1/server-tick", self.base_url);
        debug!(url, "API call: get_server_tick");
        let resp = self.client.get(&url).send().await?;
        let status = resp.status();
        if status.is_success() {
            let result: ServerTickResponse = resp.json().await?;
            debug!(url, "API response: get_server_tick ok");
            Ok(Some(result))
        } else {
            warn!(url, %status, "API response: get_server_tick failed");
            Ok(None)
        }
    }

    pub async fn search_nearest_commodity(
        &self,
        query: &NearestCommodityQuery,
    ) -> anyhow::Result<Vec<NearestCommodityResult>> {
        let url = format!("{}/api/v1/nearest-commodity", self.base_url);
        debug!(url, ?query, "API call: search_nearest_commodity");
        let resp = self.client.get(&url).query(query).send().await?;
        let status = resp.status();
        if status.is_success() {
            let result: Vec<NearestCommodityResult> = resp.json().await?;
            debug!(url, count = result.len(), "API response: search_nearest_commodity");
            Ok(result)
        } else {
            warn!(url, %status, "API response: search_nearest_commodity failed");
            Ok(vec![])
        }
    }

    pub async fn search_nearest_multi_commodity(
        &self,
        query: &MultiCommodityQuery,
    ) -> anyhow::Result<Vec<MultiCommodityResult>> {
        let url = format!("{}/api/v1/nearest-multi-commodity", self.base_url);
        debug!(url, "API call: search_nearest_multi_commodity ({} commodities)", query.commodities.len());
        let resp = self.client.post(&url).json(query).send().await?;
        let status = resp.status();
        if status.is_success() {
            let result: Vec<MultiCommodityResult> = resp.json().await?;
            debug!(url, count = result.len(), "API response: search_nearest_multi_commodity");
            Ok(result)
        } else {
            warn!(url, %status, "API response: search_nearest_multi_commodity failed");
            Ok(vec![])
        }
    }

    pub async fn fetch_faction_influence_history(
        &self,
        name: &str,
        system_address: i64,
        days: u32,
    ) -> anyhow::Result<Vec<InfluencePoint>> {
        let url = format!("{}/api/v1/faction-influence-history", self.base_url);
        debug!(url, name, system_address, days, "API call: fetch_faction_influence_history");
        let resp = self
            .client
            .get(&url)
            .query(&[
                ("name", name.to_string()),
                ("system_address", system_address.to_string()),
                ("days", days.to_string()),
            ])
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            let result: Vec<InfluencePoint> = resp.json().await?;
            debug!(url, count = result.len(), "API response: fetch_faction_influence_history");
            Ok(result)
        } else {
            warn!(url, %status, "API response: fetch_faction_influence_history failed");
            Ok(vec![])
        }
    }

    pub async fn fetch_commodity_price_history(
        &self,
        market_id: i64,
        commodity: &str,
        days: u32,
    ) -> anyhow::Result<Vec<CommodityPricePoint>> {
        let url = format!("{}/api/v1/commodity-price-history", self.base_url);
        debug!(url, market_id, commodity, days, "API call: fetch_commodity_price_history");
        let resp = self
            .client
            .get(&url)
            .query(&[
                ("market_id", market_id.to_string()),
                ("commodity", commodity.to_string()),
                ("days", days.to_string()),
            ])
            .send()
            .await?;
        let status = resp.status();
        if status.is_success() {
            let result: Vec<CommodityPricePoint> = resp.json().await?;
            debug!(url, count = result.len(), "API response: fetch_commodity_price_history");
            Ok(result)
        } else {
            warn!(url, %status, "API response: fetch_commodity_price_history failed");
            Ok(vec![])
        }
    }
}

// ─── WASM (async) implementation ──────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub struct ApiClient {
    base_url: Rc<String>,
    client: Rc<reqwest::Client>,
}

#[cfg(target_arch = "wasm32")]
impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: Rc::new(base_url.into()),
            client: Rc::new(reqwest::Client::new()),
        }
    }

    pub async fn search_stations(&self, query: StationQuery) -> Vec<StationResponse> {
        let url = format!("{}/api/v1/stations", self.base_url);
        match self.client.get(&url).query(&query).send().await {
            Ok(resp) if resp.status().is_success() => resp.json().await.unwrap_or_default(),
            _ => vec![],
        }
    }

    pub async fn search_factions(&self, query: FactionQuery) -> Vec<FactionResponse> {
        let url = format!("{}/api/v1/factions", self.base_url);
        match self.client.get(&url).query(&query).send().await {
            Ok(resp) if resp.status().is_success() => resp.json().await.unwrap_or_default(),
            _ => vec![],
        }
    }

    pub async fn search_construction_depots(&self, query: ConstructionQuery) -> Vec<ConstructionDepotResponse> {
        let url = format!("{}/api/v1/construction-depots", self.base_url);
        match self.client.get(&url).query(&query).send().await {
            Ok(resp) if resp.status().is_success() => resp.json().await.unwrap_or_default(),
            _ => vec![],
        }
    }

    pub async fn fetch_trade_routes(&self) -> Vec<TradeRouteResponse> {
        let url = format!("{}/api/v1/trade-routes", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => resp.json().await.unwrap_or_default(),
            _ => vec![],
        }
    }

    pub async fn fetch_trade_loops(&self) -> Vec<TradeLoopResponse> {
        let url = format!("{}/api/v1/trade-loops", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => resp.json().await.unwrap_or_default(),
            _ => vec![],
        }
    }

    pub async fn get_server_tick(&self) -> Option<ServerTickResponse> {
        let url = format!("{}/api/v1/server-tick", self.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => resp.json().await.ok(),
            _ => None,
        }
    }

    pub async fn search_nearest_commodity(
        &self,
        query: NearestCommodityQuery,
    ) -> Vec<NearestCommodityResult> {
        let url = format!("{}/api/v1/nearest-commodity", self.base_url);
        match self.client.get(&url).query(&query).send().await {
            Ok(resp) if resp.status().is_success() => resp.json().await.unwrap_or_default(),
            _ => vec![],
        }
    }

    pub async fn search_nearest_multi_commodity(
        &self,
        query: MultiCommodityQuery,
    ) -> Vec<MultiCommodityResult> {
        let url = format!("{}/api/v1/nearest-multi-commodity", self.base_url);
        match self.client.post(&url).json(&query).send().await {
            Ok(resp) if resp.status().is_success() => resp.json().await.unwrap_or_default(),
            _ => vec![],
        }
    }

    pub async fn fetch_faction_influence_history(
        &self,
        _name: &str,
        _system_address: i64,
        _days: u32,
    ) -> Vec<InfluencePoint> {
        vec![]
    }

    pub async fn fetch_commodity_price_history(
        &self,
        _market_id: i64,
        _commodity: &str,
        _days: u32,
    ) -> Vec<CommodityPricePoint> {
        vec![]
    }
}
