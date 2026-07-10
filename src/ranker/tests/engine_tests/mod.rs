use std::collections::HashMap;

use crate::ranker::context::{ActionContext, ActionType};
use crate::ranker::engine::{evaluate, rank_contexts};
use crate::ranker::parser::ParsedEffects;
use crate::ranker::rules::{Condition, Rule, RuleCategory, RuleSet, Weight};
use crate::state::MonsterInfo;

fn make_rules(rules: Vec<Rule>) -> RuleSet {
    RuleSet {
        version: "1.0".into(),
        available_score_fns: vec!["hp_cost_penalty".into()],
        rules,
    }
}

fn make_ctx(vars: HashMap<String, f64>) -> ActionContext {
    ActionContext {
        action_type: ActionType::PlayCard {
            card_id: "test-1".into(),
            card_name: "Test".into(),
        },
        card: None,
        target_index: None,
        target: None,
        monsters: vec![],
        parsed: ParsedEffects::default(),
        vars,
    }
}

fn end_turn_ctx(vars: HashMap<String, f64>) -> ActionContext {
    ActionContext {
        action_type: ActionType::EndTurn,
        card: None,
        target_index: None,
        target: None,
        monsters: vec![],
        parsed: ParsedEffects::default(),
        vars,
    }
}

mod support;
use support::*;

mod aoe_edge_cases;
mod aoe_merge_core;
mod aoe_wrath;

mod core_ranking;
mod core_score_conditions;
mod core_score_formulas;
mod core_score_weights;

mod parsed_basic_effects;
mod parsed_damage_block;
mod parsed_orb_stance;

mod card_conditions_cost;
mod card_conditions_identity;
mod card_conditions_type;
mod player_conditions;
mod state_conditions;

mod monster_conditions_any;
mod monster_conditions_none;
mod target_conditions_core;
mod target_conditions_flags;
mod target_conditions_intent;
