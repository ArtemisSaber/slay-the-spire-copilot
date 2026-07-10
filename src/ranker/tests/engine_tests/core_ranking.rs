use super::*;

#[test]
fn override_suppresses_target_rule() {
    let rules = make_rules(vec![
        Rule {
            category: RuleCategory::PerTarget,
            rule_id: "override_me".into(),
            priority: 2000,
            weight: Weight::Value(10),
            formula: Some("@damage * @weight".into()),
            score_fn: None,
            override_rule: None,
            applies_to: vec!["play_card".into()],
            conditions: vec![],
        },
        Rule {
            category: RuleCategory::PerTarget,
            rule_id: "the_override".into(),
            priority: 1000,
            weight: Weight::Value(5),
            formula: Some("@damage * @weight".into()),
            score_fn: None,
            override_rule: Some("override_me".into()),
            applies_to: vec!["play_card".into()],
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
        category: RuleCategory::PerTarget,
        rule_id: "min".into(),
        priority: 1000,
        weight: Weight::MinI64,
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx = make_ctx(HashMap::new());
    let scored = rank_contexts(&[ctx], &rules);
    assert_eq!(scored.len(), 1);
    assert!(scored[0].is_avoid);
    assert_eq!(scored[0].score, i64::MIN);
}

#[test]
fn rank_contexts_empty_returns_empty() {
    let rules = make_rules(vec![]);
    let scored = rank_contexts(&[], &rules);
    assert!(scored.is_empty());
}

#[test]
fn mixed_avoid_and_normal_in_rank() {
    let rules = make_rules(vec![
        Rule {
            category: RuleCategory::PerTarget,
            rule_id: "avoid_on_turn_1".into(),
            priority: 1000,
            weight: Weight::MinI64,
            formula: None,
            score_fn: None,
            override_rule: None,
            applies_to: vec!["play_card".into()],
            conditions: vec![Condition::State(crate::ranker::rules::StateCondition {
                state: crate::ranker::rules::StatePredicates {
                    turn_eq: Some(1),
                    ..Default::default()
                },
            })],
        },
        Rule {
            category: RuleCategory::PerTarget,
            rule_id: "normal_score".into(),
            priority: 1000,
            weight: Weight::Value(50),
            formula: None,
            score_fn: None,
            override_rule: None,
            applies_to: vec!["play_card".into()],
            conditions: vec![],
        },
    ]);
    let mut v1 = HashMap::new();
    v1.insert("turn".to_string(), 1.0);
    let mut v2 = HashMap::new();
    v2.insert("turn".to_string(), 2.0);
    let scored = rank_contexts(&[make_ctx(v1), make_ctx(v2)], &rules);
    assert_eq!(scored.len(), 2);
    let avoids: Vec<_> = scored.iter().filter(|s| s.is_avoid).collect();
    assert_eq!(avoids.len(), 1);
    assert_eq!(avoids[0].score, i64::MIN);
    let normals: Vec<_> = scored.iter().filter(|s| !s.is_avoid).collect();
    assert_eq!(normals.len(), 1);
    assert_eq!(normals[0].score, 50);
}

#[test]
fn end_turn_through_rank_contexts() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "end_turn_rule".into(),
        priority: 1000,
        weight: Weight::Value(30),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["end_turn".into()],
        conditions: vec![],
    }]);
    let ctx = end_turn_ctx(HashMap::new());
    let scored = rank_contexts(&[ctx], &rules);
    assert_eq!(scored.len(), 1);
    assert_eq!(scored[0].score, 30);
    assert!(!scored[0].is_avoid);
}

#[test]
fn play_card_rule_does_not_apply_to_end_turn() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "only_play".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx = end_turn_ctx(HashMap::new());
    let results = evaluate(&ctx, &rules);
    assert!(!results[0].matched);
    assert_eq!(results[0].score, 0);
}
