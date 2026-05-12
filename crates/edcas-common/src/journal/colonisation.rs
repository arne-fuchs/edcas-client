use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColonisationConstructionDepot {
    #[serde(rename = "timestamp")]
    pub timestamp: String,
    #[serde(rename = "MarketID")]
    pub market_id: i64,
    #[serde(rename = "SystemAddress")]
    pub system_address: i64,
    #[serde(rename = "ConstructionProgress")]
    pub construction_progress: f32,
    #[serde(rename = "ConstructionComplete", default)]
    pub construction_complete: bool,
    #[serde(rename = "ConstructionFailed", default)]
    pub construction_failed: bool,
    #[serde(rename = "Resources")]
    pub resources: Vec<ConstructionResource>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConstructionResource {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Name_Localised")]
    pub name_localised: Option<String>,
    #[serde(rename = "RequiredAmount")]
    pub required_amount: i32,
    #[serde(rename = "ProvidedAmount")]
    pub provided_amount: i32,
    #[serde(rename = "Payment")]
    pub payment: i64,
}

impl ConstructionResource {
    pub fn display_name(&self) -> &str {
        self.name_localised.as_deref().unwrap_or_else(|| {
            self.name
                .trim_start_matches('$')
                .trim_end_matches(';')
                .trim_end_matches("_name")
        })
    }

    pub fn fraction(&self) -> f32 {
        if self.required_amount == 0 {
            1.0
        } else {
            self.provided_amount as f32 / self.required_amount as f32
        }
    }

    pub fn remaining(&self) -> i32 {
        (self.required_amount - self.provided_amount).max(0)
    }
}
