use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardInfo {
    pub id: String,
    pub name: String,
    pub cost: i64,
    pub card_type: String,
    #[serde(default)]
    pub upgraded: bool,
    pub uuid: Option<String>,
    pub description: String,
    pub price: Option<i64>,
    #[serde(default = "default_true")]
    pub playable: bool,
    #[serde(default)]
    pub has_target: bool,
}

fn default_true() -> bool {
    true
}

impl CardInfo {
    pub(super) fn from_json(card: &Value) -> Self {
        CardInfo {
            id: card
                .get("id")
                .and_then(|value| value.as_str())
                .unwrap_or("?")
                .to_string(),
            name: card
                .get("name")
                .and_then(|value| value.as_str())
                .unwrap_or("?")
                .to_string(),
            cost: card
                .get("cost")
                .and_then(|number| number.as_i64())
                .unwrap_or(0),
            card_type: card
                .get("type")
                .and_then(|value| value.as_str())
                .unwrap_or("?")
                .to_string(),
            upgraded: card
                .get("upgrades")
                .and_then(|number| number.as_i64())
                .map(|upgrades| upgrades > 0)
                .unwrap_or(false),
            uuid: card
                .get("uuid")
                .and_then(|value| value.as_str())
                .map(str::to_string),
            description: card
                .get("description")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string(),
            price: card.get("price").and_then(|value| value.as_i64()),
            playable: card
                .get("is_playable")
                .and_then(|value| value.as_bool())
                .unwrap_or(true),
            has_target: card
                .get("has_target")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonsterInfo {
    pub name: String,
    #[serde(default)]
    pub monster_id: Option<String>,
    pub index: usize,
    pub current_hp: Option<i64>,
    pub max_hp: Option<i64>,
    pub block: Option<i64>,
    pub intent: Option<String>,
    pub damage: Option<i64>,
    pub hits: Option<i64>,
    pub monster_powers: Vec<PowerInfo>,
    pub can_be_killed: bool,
    pub is_scaling: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerInfo {
    pub id: String,
    pub name: String,
    pub amount: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrbInfo {
    pub id: String,
    #[serde(default)]
    pub amount: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelicInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub counter: Option<i64>,
    pub price: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PotionInfo {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub slot: usize,
    pub name: String,
    pub description: String,
    pub price: Option<i64>,
    #[serde(default)]
    pub can_use: bool,
    #[serde(default)]
    pub can_discard: bool,
    #[serde(default)]
    pub requires_target: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapCoord {
    pub symbol: String,
    pub x: i64,
    pub y: i64,
    pub children: Vec<(i64, i64)>,
}
