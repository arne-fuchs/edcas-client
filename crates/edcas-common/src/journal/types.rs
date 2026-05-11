use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Faction {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Allegiance", default)]
    pub allegiance: String,
    #[serde(rename = "Government", default)]
    pub government: String,
    #[serde(rename = "Happiness", default)]
    pub happiness: String,
    #[serde(rename = "Influence", default)]
    pub influence: f32,
    #[serde(rename = "MyReputation", default)]
    pub my_reputation: f32,
    #[serde(rename = "FactionState", default)]
    pub faction_state: String,
    #[serde(rename = "ActiveStates")]
    pub active_states: Option<Vec<FactionState>>,
    #[serde(rename = "PendingStates")]
    pub pending_states: Option<Vec<FactionState>>,
    #[serde(rename = "RecoveringStates")]
    pub recovering_states: Option<Vec<FactionState>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FactionState {
    #[serde(rename = "State")]
    pub state: String,
    #[serde(rename = "Trend", default)]
    pub trend: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemFaction {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "FactionState", default)]
    pub faction_state: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Conflict {
    #[serde(rename = "WarType")]
    pub war_type: String,
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Faction1")]
    pub faction1: ConflictFaction,
    #[serde(rename = "Faction2")]
    pub faction2: ConflictFaction,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConflictFaction {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Stake", default)]
    pub stake: String,
    #[serde(rename = "WonDays", default)]
    pub won_days: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StationEconomy {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Proportion")]
    pub proportion: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StationFaction {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "FactionState", default)]
    pub faction_state: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LandingPads {
    #[serde(rename = "Small", default)]
    pub small: i32,
    #[serde(rename = "Medium", default)]
    pub medium: i32,
    #[serde(rename = "Large", default)]
    pub large: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ring {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "RingClass")]
    pub ring_class: String,
    #[serde(rename = "MassMT")]
    pub mass_mt: f64,
    #[serde(rename = "InnerRad")]
    pub inner_rad: f64,
    #[serde(rename = "OuterRad")]
    pub outer_rad: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Parent {
    #[serde(rename = "Star")]
    pub star: Option<i32>,
    #[serde(rename = "Planet")]
    pub planet: Option<i32>,
    #[serde(rename = "Ring")]
    pub ring: Option<i32>,
    #[serde(rename = "Null")]
    pub null: Option<i32>,
}

impl Parent {
    pub fn parent_id(&self) -> Option<i32> {
        self.star.or(self.planet).or(self.ring).or(self.null)
    }

    pub fn parent_type(&self) -> &'static str {
        if self.star.is_some() {
            "Star"
        } else if self.planet.is_some() {
            "Planet"
        } else if self.ring.is_some() {
            "Ring"
        } else {
            "Null"
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AtmosphereComposition {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Percent")]
    pub percent: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Material {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Percent")]
    pub percent: f64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Composition {
    #[serde(rename = "Rock", default)]
    pub rock: f32,
    #[serde(rename = "Ice", default)]
    pub ice: f32,
    #[serde(rename = "Metal", default)]
    pub metal: f32,
}
