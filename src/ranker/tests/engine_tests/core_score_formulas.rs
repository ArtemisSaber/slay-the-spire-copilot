use super::*;

#[test]
fn formula_rule_uses_weight_and_vars() {
    let mut vars = HashMap::new();
    vars.insert("damage".to_string(), 6.0);
    vars.insert("hits".to_string(), 2.0);

    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test_formula".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: Some("@damage * @hits * @weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx = make_ctx(vars);
    let results = evaluate(&ctx, &rules);
    assert_eq!(results[0].rule_id, "test_formula");
    assert_eq!(results[0].score, 120);
    assert!(results[0].matched);
}

#[test]
fn score_fn_dispatch_path_used() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test_score_fn".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: Some("hp_cost_penalty".into()),
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let mut vars = HashMap::new();
    vars.insert("self_damage".to_string(), 12.0);
    vars.insert("current_hp".to_string(), 60.0);
    let ctx = make_ctx(vars);
    let results = evaluate(&ctx, &rules);
    assert!(results[0].matched);
    assert_eq!(results[0].score, -58);
}

#[test]
fn formula_eval_failure_returns_zero_and_matches() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "bad_formula".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: Some("max(@x)".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx = make_ctx(HashMap::new());
    let results = evaluate(&ctx, &rules);
    assert!(results[0].matched);
    assert_eq!(results[0].score, 0);
}

#[test]
fn weight_only_no_formula_no_score_fn() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "weight_only".into(),
        priority: 1000,
        weight: Weight::Value(42),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx = make_ctx(HashMap::new());
    let results = evaluate(&ctx, &rules);
    assert!(results[0].matched);
    assert_eq!(results[0].score, 42);
}

#[test]
fn compute_condition_formula_error_returns_false() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "bad_compute".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Compute(crate::ranker::rules::ComputeCondition {
            compute: crate::ranker::rules::ComputePredicates {
                formula: Some("max(@x)".into()),
            },
        })],
    }]);
    let ctx = make_ctx(HashMap::new());
    let results = evaluate(&ctx, &rules);
    assert!(!results[0].matched);
    assert_eq!(results[0].score, 0);
}
