use super::*;

#[test]
fn flat_weight_rule_scores_correctly() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test_flat".into(),
        priority: 1000,
        weight: Weight::Value(-100),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
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
fn unscored_end_turn_gets_total_zero() {
    let rules = make_rules(vec![]);
    let ctx = end_turn_ctx(HashMap::new());
    let results = evaluate(&ctx, &rules);
    let total: i64 = results.iter().map(|r| r.score).sum();
    assert_eq!(total, 0);
}

#[test]
fn sort_by_score_descending() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "score_by_value".into(),
        priority: 1000,
        weight: Weight::Value(1),
        formula: Some("@value * @weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let mut low_vars = HashMap::new();
    low_vars.insert("value".to_string(), 2.0);
    let mut high_vars = HashMap::new();
    high_vars.insert("value".to_string(), 5.0);
    let low = make_ctx(low_vars);
    let high = make_ctx(high_vars);

    let scored = rank_contexts(&[low, high], &rules);
    assert_eq!(scored.len(), 2);
    assert_eq!(scored[0].score, 5);
    assert_eq!(scored[1].score, 2);
}

#[test]
fn single_context_scores_once_per_context() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: Some("@weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx = ActionContext {
        action_type: ActionType::PlayCard {
            card_id: "test".into(),
            card_name: "Test".into(),
        },
        card: None,
        target_index: Some(0),
        target: None,
        monsters: vec![
            MonsterInfo {
                name: "A".into(),
                monster_id: None,
                index: 0,
                ..Default::default()
            },
            MonsterInfo {
                name: "B".into(),
                monster_id: None,
                index: 1,
                ..Default::default()
            },
        ],
        parsed: ParsedEffects::default(),
        vars: HashMap::new(),
    };
    let results = evaluate(&ctx, &rules);
    assert_eq!(results[0].score, 10, "single context scores once");
    assert!(results[0].matched);
}

#[test]
fn context_with_no_monsters_still_scores_once() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: Some("@weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx = ActionContext {
        action_type: ActionType::PlayCard {
            card_id: "test".into(),
            card_name: "Test".into(),
        },
        card: None,
        target_index: None,
        target: None,
        monsters: vec![],
        parsed: ParsedEffects::default(),
        vars: HashMap::new(),
    };
    let results = evaluate(&ctx, &rules);
    assert_eq!(
        results[0].score, 10,
        "score applies regardless of monster count"
    );
    assert!(results[0].matched);
}

#[test]
fn scored_action_propagates_target_index_from_context() {
    let target = MonsterInfo {
        name: "Enemy".into(),
        monster_id: None,
        index: 3,
        current_hp: Some(20),
        max_hp: Some(20),
        block: Some(0),
        intent: None,
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let ctx = ActionContext {
        action_type: ActionType::PlayCard {
            card_id: "uuid-1".into(),
            card_name: "Strike".into(),
        },
        card: None,
        target_index: Some(3),
        target: Some(target),
        monsters: vec![],
        parsed: ParsedEffects::default(),
        vars: HashMap::new(),
    };
    let rule_set = make_rules(vec![]);
    let scored = rank_contexts(&[ctx], &rule_set);

    assert_eq!(scored.len(), 1);
    assert_eq!(scored[0].target_index, Some(3));
}

#[test]
fn equal_scores_preserve_order() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "equal_score".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: Some("@weight".into()),
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![],
    }]);
    let ctx_a = {
        let mut v = HashMap::new();
        v.insert("tag".to_string(), 1.0);
        make_ctx(v)
    };
    let ctx_b = {
        let mut v = HashMap::new();
        v.insert("tag".to_string(), 2.0);
        make_ctx(v)
    };
    let scored = rank_contexts(&[ctx_a, ctx_b], &rules);
    assert_eq!(scored.len(), 2);
    assert_eq!(scored[0].score, 10);
    assert_eq!(scored[1].score, 10);
}
