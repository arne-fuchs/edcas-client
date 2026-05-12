use edcas_common::api::{
    BodyResponse, CarrierQuery, CarrierResponse, ConstructionDepotResponse,
    ConstructionDepotSubmission, ConstructionQuery, FactionQuery, FactionResponse,
    StationQuery, StationResponse,
};

pub struct ApiClient {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("failed to build HTTP client"),
        }
    }

    pub fn get_bodies(&self, system_address: i64) -> anyhow::Result<Vec<BodyResponse>> {
        let url = format!("{}/api/v1/systems/{}/bodies", self.base_url, system_address);
        let resp = self.client.get(&url).send()?;
        if resp.status().is_success() {
            Ok(resp.json()?)
        } else {
            Ok(vec![])
        }
    }

    pub fn search_stations(&self, query: &StationQuery) -> anyhow::Result<Vec<StationResponse>> {
        let url = format!("{}/api/v1/stations", self.base_url);
        let resp = self.client.get(&url).query(query).send()?;
        if resp.status().is_success() {
            Ok(resp.json()?)
        } else {
            Ok(vec![])
        }
    }

    pub fn search_carriers(&self, query: &CarrierQuery) -> anyhow::Result<Vec<CarrierResponse>> {
        let url = format!("{}/api/v1/carriers", self.base_url);
        let resp = self.client.get(&url).query(query).send()?;
        if resp.status().is_success() {
            Ok(resp.json()?)
        } else {
            Ok(vec![])
        }
    }

    pub fn search_factions(&self, query: &FactionQuery) -> anyhow::Result<Vec<FactionResponse>> {
        let url = format!("{}/api/v1/factions", self.base_url);
        let resp = self.client.get(&url).query(query).send()?;
        if resp.status().is_success() {
            Ok(resp.json()?)
        } else {
            Ok(vec![])
        }
    }

    pub fn search_construction_depots(
        &self,
        query: &ConstructionQuery,
    ) -> anyhow::Result<Vec<ConstructionDepotResponse>> {
        let url = format!("{}/api/v1/construction-depots", self.base_url);
        let resp = self.client.get(&url).query(query).send()?;
        if resp.status().is_success() {
            Ok(resp.json()?)
        } else {
            Ok(vec![])
        }
    }

    pub fn submit_construction_depot(&self, submission: &ConstructionDepotSubmission) -> anyhow::Result<()> {
        let url = format!("{}/api/v1/construction-depots", self.base_url);
        self.client.post(&url).json(submission).send()?;
        Ok(())
    }
}
