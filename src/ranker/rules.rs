#![allow(
    dead_code,
    reason = "types deserialized from JSON; constructed in tests and engine"
)]

use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, Deserialize)]
pub struct RuleSet {
    pub version: String,
    #[serde(default)]
    pub available_score_fns: Vec<String>,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub rule_id: String,
    pub priority: i64,
    pub weight: Weight,
    #[serde(default)]
    pub formula: Option<String>,
    #[serde(default)]
    pub score_fn: Option<String>,
    #[serde(default)]
    #[serde(rename = "override")]
    pub override_rule: Option<String>,
    #[serde(default)]
    pub applies_to: Vec<String>,
    #[serde(default)]
    pub per_target: bool,
    #[serde(default)]
    pub conditions: Vec<Condition>,
}

#[derive(Debug, Clone)]
pub enum Weight {
    Value(i64),
    MinI64,
}

impl Weight {
    pub fn resolve(&self) -> i64 {
        match self {
            Weight::Value(v) => *v,
            Weight::MinI64 => i64::MIN,
        }
    }
}

impl<'de> Deserialize<'de> for Weight {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value {
            serde_json::Value::Number(n) => n
                .as_i64()
                .map(Weight::Value)
                .ok_or_else(|| serde::de::Error::custom("weight must be a valid i64")),
            serde_json::Value::String(s) if s == "i64::MIN" => Ok(Weight::MinI64),
            _ => Err(serde::de::Error::custom(
                "weight must be an integer or the string \"i64::MIN\"",
            )),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Condition {
    Card(CardCondition),
    Parsed(ParsedCondition),
    Target(TargetCondition),
    Monsters(MonstersCondition),
    Player(PlayerCondition),
    State(StateCondition),
    Compute(ComputeCondition),
}

#[derive(Debug, Clone, Deserialize)]
pub struct CardCondition {
    #[serde(default)]
    pub card: CardPredicates,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CardPredicates {
    #[serde(default)]
    #[serde(rename = "type")]
    pub card_type: Option<String>,
    #[serde(default)]
    pub type_in: Option<Vec<String>>,
    #[serde(default)]
    pub type_not_in: Option<Vec<String>>,
    #[serde(default)]
    pub cost_eq: Option<i64>,
    #[serde(default)]
    pub ethereal: Option<bool>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub id_in: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParsedCondition {
    #[serde(default)]
    pub parsed: ParsedPredicates,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ParsedPredicates {
    #[serde(default)]
    pub damage_gt: Option<i64>,
    #[serde(default)]
    pub damage_eq: Option<i64>,
    #[serde(default)]
    pub block_gt: Option<i64>,
    #[serde(default)]
    pub block_eq: Option<i64>,
    #[serde(default)]
    pub heal_gt: Option<i64>,
    #[serde(default)]
    pub draw_gt: Option<i64>,
    #[serde(default)]
    pub self_damage_gt: Option<i64>,
    #[serde(default)]
    pub str_gain_gt: Option<i64>,
    #[serde(default)]
    pub dex_gain_gt: Option<i64>,
    #[serde(default)]
    pub poison_gt: Option<i64>,
    #[serde(default)]
    pub vulnerable_gt: Option<i64>,
    #[serde(default)]
    pub weak_gt: Option<i64>,
    #[serde(default)]
    pub energy_gain_gt: Option<i64>,
    #[serde(default)]
    pub focus_gain_gt: Option<i64>,
    #[serde(default)]
    pub str_loss_gt: Option<i64>,
    #[serde(default)]
    pub str_loss_temp: Option<bool>,
    #[serde(default)]
    pub mantra_gt: Option<i64>,
    #[serde(default)]
    pub exhausts_cards: Option<bool>,
    #[serde(default)]
    pub exhausts_status_curse: Option<bool>,
    #[serde(default)]
    pub channel_orb: Option<String>,
    #[serde(default)]
    pub orb_slot_expand_gt: Option<i64>,
    #[serde(default)]
    pub exits_stance: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TargetCondition {
    #[serde(default)]
    pub target: TargetPredicates,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TargetPredicates {
    #[serde(default)]
    pub power: Option<String>,
    #[serde(default)]
    pub power_not: Option<String>,
    #[serde(default)]
    pub any_power_in: Option<Vec<String>>,
    #[serde(default)]
    pub is_scaling: Option<bool>,
    #[serde(default)]
    pub is_minion: Option<bool>,
    #[serde(default)]
    pub is_max_hp: Option<bool>,
    #[serde(default)]
    pub can_be_killed: Option<bool>,
    #[serde(default)]
    pub intent: Option<String>,
    #[serde(default)]
    pub intent_not: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonstersCondition {
    #[serde(default)]
    pub monsters: MonstersPredicates,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MonstersPredicates {
    #[serde(default)]
    pub any: Option<MonsterSubPredicates>,
    #[serde(default)]
    pub none: Option<MonsterSubPredicates>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct MonsterSubPredicates {
    #[serde(default)]
    pub monster_id: Option<String>,
    #[serde(default)]
    pub is_scaling: Option<bool>,
    #[serde(default)]
    pub is_not_minion: Option<bool>,
    #[serde(default)]
    pub exclude_target: Option<bool>,
    #[serde(default)]
    pub power: Option<String>,
    #[serde(default)]
    pub intent: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlayerCondition {
    #[serde(default)]
    pub player: PlayerPredicates,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PlayerPredicates {
    #[serde(default)]
    pub power: Option<String>,
    #[serde(default)]
    pub power_not: Option<String>,
    #[serde(default)]
    pub relic: Option<String>,
    #[serde(default)]
    pub relic_not: Option<String>,
    #[serde(default)]
    pub stance: Option<String>,
    #[serde(default)]
    pub stance_not: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StateCondition {
    #[serde(default)]
    pub state: StatePredicates,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct StatePredicates {
    #[serde(default)]
    pub turn_eq: Option<i64>,
    #[serde(default)]
    pub remaining_energy_gt: Option<i64>,
    #[serde(default)]
    pub incoming_damage_gt: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ComputeCondition {
    #[serde(default)]
    pub compute: ComputePredicates,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ComputePredicates {
    #[serde(default)]
    pub formula: Option<String>,
}

#[cfg(test)]
#[path = "tests/rules_tests.rs"]
mod tests;
