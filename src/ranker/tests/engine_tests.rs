use std::collections::HashMap;

use crate::ranker::context::{ActionContext, ActionType};
use crate::ranker::engine::{evaluate, rank_contexts};
use crate::ranker::parser::ParsedEffects;
use crate::ranker::rules::{Rule, RuleSet, Weight};

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

#[test]
fn flat_weight_rule_scores_correctly() {
    let rules = make_rules(vec![Rule {
        rule_id: "test_flat".into(),
        priority: 1000,
        weight: Weight::Value(-100),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        per_target: false,
        conditions: vec![],
    }]);
    let ctx = make_ctx(HashMap::new());
    let results = evaluate(&ctx, &rules);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].rule_id, "test_flat");
    assert_eq!(results[0].score, -100);
    assert!(results[0].matched);
}

#[test]
fn formula_rule_uses_weight_and_vars() {
    let mut vars = HashMap::new();
    vars.insert("damage".to_string(), 6.0);
    vars.insert("hits".to_string(), 2.0);

    let rules = make_rules(vec![Rule {
        rule_id: "test_formula".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: Some("@damage * @hits * @weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        per_target: false,
        conditions: vec![],
    }]);
    let ctx = make_ctx(vars);
    let results = evaluate(&ctx, &rules);
    assert_eq!(results[0].rule_id, "test_formula");
    assert_eq!(results[0].score, 120);
    assert!(results[0].matched);
}

#[test]
fn override_suppresses_target_rule() {
    let rules = make_rules(vec![
        Rule {
            rule_id: "override_me".into(),
            priority: 2000,
            weight: Weight::Value(10),
            formula: Some("@damage * @weight".into()),
            score_fn: None,
            override_rule: None,
            applies_to: vec!["play_card".into()],
            per_target: false,
            conditions: vec![],
        },
        Rule {
            rule_id: "the_override".into(),
            priority: 1000,
            weight: Weight::Value(5),
            formula: Some("@damage * @weight".into()),
            score_fn: None,
            override_rule: Some("override_me".into()),
            applies_to: vec!["play_card".into()],
            per_target: false,
            conditions: vec![],
        },
    ]);
    let mut vars = HashMap::new();
    vars.insert("damage".to_string(), 6.0);
    let ctx = make_ctx(vars);

    let results = evaluate(&ctx, &rules);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].rule_id, "the_override");
    assert_eq!(results[0].score, 30);
    assert!(results[0].matched);
    assert_eq!(results[1].rule_id, "override_me");
    assert_eq!(results[1].score, 0);
    assert!(!results[1].matched);
}

#[test]
fn avoid_actions_separated() {
    let rules = make_rules(vec![Rule {
        rule_id: "min".into(),
        priority: 1000,
        weight: Weight::MinI64,
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        per_target: false,
        conditions: vec![],
    }]);
    let ctx = make_ctx(HashMap::new());
    let scored = rank_contexts(&[ctx], &rules);
    assert_eq!(scored.len(), 1);
    assert!(scored[0].is_avoid);
    assert_eq!(scored[0].score, i64::MIN);
}

#[test]
fn unscored_end_turn_gets_total_zero() {
    let rules = make_rules(vec![]);
    let ctx = end_turn_ctx(HashMap::new());
    let results = evaluate(&ctx, &rules);
    let total: i64 = results.iter().map(|r| r.score).sum();
    assert_eq!(total, 0);
}

#[test]
fn sort_by_score_descending() {
    let rules = make_rules(vec![]);
    let ctx1 = make_ctx(HashMap::new());
    let ctx2 = end_turn_ctx(HashMap::new());

    let scored = rank_contexts(&[ctx1, ctx2], &rules);
    assert_eq!(scored.len(), 2);
    assert_eq!(scored[0].score, 0);
    assert_eq!(scored[1].score, 0);
}
