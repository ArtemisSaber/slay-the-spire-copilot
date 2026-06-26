use super::*;

#[test]
fn deserialize_minimal_ruleset() {
    let json = r#"{
        "version": "1.0",
        "available_score_fns": ["hp_cost_penalty"],
        "rules": []
    }"#;
    let set: RuleSet = serde_json::from_str(json).unwrap();
    assert_eq!(set.version, "1.0");
    assert_eq!(set.available_score_fns, vec!["hp_cost_penalty"]);
    assert!(set.rules.is_empty());
}

#[test]
fn deserialize_flat_weight_rule() {
    let json = r#"{
        "version": "1.0",
        "rules": [{
            "rule_id": "base_status_curse",
            "priority": 1011,
            "weight": -100,
            "applies_to": ["play_card"],
            "conditions": [{"card": {"type_in": ["STATUS", "CURSE"]}}]
        }]
    }"#;
    let set: RuleSet = serde_json::from_str(json).unwrap();
    let rule = &set.rules[0];
    assert_eq!(rule.rule_id, "base_status_curse");
    assert_eq!(rule.priority, 1011);
    assert_eq!(rule.weight.resolve(), -100);
    assert!(rule.formula.is_none());
    assert!(rule.score_fn.is_none());
    assert!(rule.override_rule.is_none());
    assert_eq!(rule.applies_to, vec!["play_card"]);
    assert!(!rule.per_target);
    assert_eq!(rule.conditions.len(), 1);
}

#[test]
fn deserialize_formula_rule() {
    let json = r#"{
        "version": "1.0",
        "rules": [{
            "rule_id": "core_damage",
            "priority": 1052,
            "weight": 10,
            "applies_to": ["play_card"],
            "formula": "@damage * @hits * @weight",
            "per_target": true,
            "conditions": [
                {"parsed": {"damage_gt": 0}},
                {"target": {"power_not": "Intangible"}}
            ]
        }]
    }"#;
    let set: RuleSet = serde_json::from_str(json).unwrap();
    let rule = &set.rules[0];
    assert_eq!(rule.formula.as_deref(), Some("@damage * @hits * @weight"));
    assert!(rule.per_target);
    assert_eq!(rule.conditions.len(), 2);
}

#[test]
fn deserialize_score_fn_rule() {
    let json = r#"{
        "version": "1.0",
        "available_score_fns": ["hp_cost_penalty"],
        "rules": [{
            "rule_id": "danger_hp_cost",
            "priority": 1174,
            "weight": 0,
            "score_fn": "hp_cost_penalty",
            "applies_to": ["play_card"],
            "conditions": [{"parsed": {"self_damage_gt": 0}}]
        }]
    }"#;
    let set: RuleSet = serde_json::from_str(json).unwrap();
    let rule = &set.rules[0];
    assert_eq!(rule.score_fn.as_deref(), Some("hp_cost_penalty"));
}

#[test]
fn deserialize_override_rule() {
    let json = r#"{
        "version": "1.0",
        "rules": [{
            "rule_id": "core_block_retain",
            "priority": 1060,
            "weight": 10,
            "applies_to": ["play_card"],
            "formula": "@block * @weight",
            "override": "core_block_non_excessive",
            "conditions": [
                {"parsed": {"block_gt": 0}},
                {"player": {"power": "Barricade"}}
            ]
        }]
    }"#;
    let set: RuleSet = serde_json::from_str(json).unwrap();
    let rule = &set.rules[0];
    assert_eq!(
        rule.override_rule.as_deref(),
        Some("core_block_non_excessive")
    );
}

#[test]
fn deserialize_monsters_condition() {
    let json = r#"{
        "version": "1.0",
        "rules": [{
            "rule_id": "punishment_skill_vs_nob",
            "priority": 1160,
            "weight": -100,
            "applies_to": ["play_card"],
            "conditions": [
                {"monsters": {"any": {"monster_id": "GremlinNob"}}},
                {"card": {"type": "SKILL"}}
            ]
        }]
    }"#;
    let set: RuleSet = serde_json::from_str(json).unwrap();
    assert_eq!(set.rules.len(), 1);
}

#[test]
fn deserialize_weight_i64_min() {
    let json = r#"{
        "version": "1.0",
        "rules": [{
            "rule_id": "manual_filter",
            "priority": 1020,
            "weight": "i64::MIN",
            "applies_to": ["play_card"],
            "conditions": []
        }]
    }"#;
    let set: RuleSet = serde_json::from_str(json).unwrap();
    assert_eq!(set.rules[0].weight.resolve(), i64::MIN);
}

#[test]
fn deserialize_compute_condition() {
    let json = r#"{
        "version": "1.0",
        "rules": [{
            "rule_id": "test",
            "priority": 2000,
            "weight": 80,
            "applies_to": ["end_turn"],
            "conditions": [{"compute": {"formula": "@useful_cards_in_hand == 0"}}]
        }]
    }"#;
    let set: RuleSet = serde_json::from_str(json).unwrap();
    assert_eq!(set.rules.len(), 1);
}

#[test]
fn deserialize_relic_condition() {
    let json = r#"{
        "version": "1.0",
        "rules": [{
            "rule_id": "core_block_retain_relic",
            "priority": 1061,
            "weight": 10,
            "applies_to": ["play_card"],
            "formula": "@block * @weight",
            "override": "core_block_non_excessive",
            "conditions": [
                {"parsed": {"block_gt": 0}},
                {"player": {"relic": "Calipers"}}
            ]
        }]
    }"#;
    let set: RuleSet = serde_json::from_str(json).unwrap();
    let rule = &set.rules[0];
    assert_eq!(rule.conditions.len(), 2);
}
