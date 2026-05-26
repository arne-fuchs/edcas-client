use edcas_common::api::{
    BodyResponse, CarrierQuery, CarrierResponse, ConstructionDepotResponse,
    ConstructionDepotSubmission, ConstructionQuery, FactionQuery, FactionResponse,
    NearestCommodityQuery, NearestCommodityResult, ServerTickResponse, StationQuery,
    StationResponse, TradeLoopResponse, TradeRouteResponse,
};

// ─── Native (async) implementation ────────────────────────────────────────────

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
        let resp = self.client.get(&url).send().await?;
        if resp.status().is_success() { Ok(resp.json().await?) } else { Ok(vec![]) }
    }

    pub async fn search_stations(&self, query: &StationQuery) -> anyhow::Result<Vec<StationResponse>> {
        let url = format!("{}/api/v1/stations", self.base_url);
        let resp = self.client.get(&url).query(query).send().await?;
        if resp.status().is_success() { Ok(resp.json().await?) } else { Ok(vec![]) }
    }

    pub async fn search_carriers(&self, query: &CarrierQuery) -> anyhow::Result<Vec<CarrierResponse>> {
        let url = format!("{}/api/v1/carriers", self.base_url);
        let resp = self.client.get(&url).query(query).send().await?;
        if resp.status().is_success() { Ok(resp.json().await?) } else { Ok(vec![]) }
    }

    pub async fn search_factions(&self, query: &FactionQuery) -> anyhow::Result<Vec<FactionResponse>> {
        let url = format!("{}/api/v1/factions", self.base_url);
        let resp = self.client.get(&url).query(query).send().await?;
        if resp.status().is_success() { Ok(resp.json().await?) } else { Ok(vec![]) }
    }

    pub async fn search_construction_depots(
        &self,
        query: &ConstructionQuery,
    ) -> anyhow::Result<Vec<ConstructionDepotResponse>> {
        let url = format!("{}/api/v1/construction-depots", self.base_url);
        let resp = self.client.get(&url).query(query).send().await?;
        if resp.status().is_success() { Ok(resp.json().await?) } else { Ok(vec![]) }
    }

    pub async fn submit_construction_depot(
        &self,
        submission: &ConstructionDepotSubmission,
    ) -> anyhow::Result<()> {
        let url = format!("{}/api/v1/construction-depots", self.base_url);
        self.client.post(&url).json(submission).send().await?;
        Ok(())
    }

    pub async fn fetch_trade_routes(&self) -> anyhow::Result<Vec<TradeRouteResponse>> {
        let url = format!("{}/api/v1/trade-routes", self.base_url);
        let resp = self.client.get(&url).send().await?;
        if resp.status().is_success() { Ok(resp.json().await?) } else { Ok(vec![]) }
    }

    pub async fn fetch_trade_loops(&self) -> anyhow::Result<Vec<TradeLoopResponse>> {
        let url = format!("{}/api/v1/trade-loops", self.base_url);
        let resp = self.client.get(&url).send().await?;
        if resp.status().is_success() { Ok(resp.json().await?) } else { Ok(vec![]) }
    }

    pub async fn get_server_tick(&self) -> anyhow::Result<Option<ServerTickResponse>> {
        let url = format!("{}/api/v1/server-tick", self.base_url);
        let resp = self.client.get(&url).send().await?;
        if resp.status().is_success() {
            Ok(Some(resp.json().await?))
        } else {
            Ok(None)
        }
    }

    pub async fn search_nearest_commodity(
        &self,
        query: &NearestCommodityQuery,
    ) -> anyhow::Result<Vec<NearestCommodityResult>> {
        let url = format!("{}/api/v1/nearest-commodity", self.base_url);
        let resp = self.client.get(&url).query(query).send().await?;
        if resp.status().is_success() { Ok(resp.json().await?) } else { Ok(vec![]) }
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

    pub async fn search_carriers(&self, query: CarrierQuery) -> Vec<CarrierResponse> {
        let url = format!("{}/api/v1/carriers", self.base_url);
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
}
