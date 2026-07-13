use std::collections::HashMap;

use crate::state::{CardInfo, MonsterInfo, NormalizedState};

use super::parser::ParsedEffects;

mod builder;
mod cost;
mod variables;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum ActionType {
    PlayCard { card_id: String, card_name: String },
    UsePotion { potion_name: String },
    EndTurn,
}

#[derive(Debug, Clone)]
pub struct ActionContext {
    pub action_type: ActionType,
    pub card: Option<CardInfo>,
    pub target_index: Option<usize>,
    pub target: Option<MonsterInfo>,
    pub monsters: Vec<MonsterInfo>,
    pub parsed: ParsedEffects,
    pub vars: HashMap<String, f64>,
}

impl ActionContext {
    pub fn build_all(state: &NormalizedState) -> Vec<ActionContext> {
        builder::build_all(state)
    }
}

#[cfg(test)]
#[path = "tests/context_tests.rs"]
mod tests;
