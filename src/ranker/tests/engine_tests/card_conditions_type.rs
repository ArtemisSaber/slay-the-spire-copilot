use super::*;

#[test]
fn card_type_exact_match_matches() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Card(crate::ranker::rules::CardCondition {
            card: crate::ranker::rules::CardPredicates {
                card_type: Some("ATTACK".into()),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "Strike".into(),
        name: "Strike".into(),
        cost: 1,
        card_type: "ATTACK".into(),
        upgraded: false,
        uuid: None,
        description: "".into(),
        price: None,
        playable: true,
        has_target: true,
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn card_type_exact_mismatch_fails() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Card(crate::ranker::rules::CardCondition {
            card: crate::ranker::rules::CardPredicates {
                card_type: Some("ATTACK".into()),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "Defend".into(),
        name: "Defend".into(),
        cost: 1,
        card_type: "SKILL".into(),
        upgraded: false,
        uuid: None,
        description: "".into(),
        price: None,
        playable: true,
        has_target: false,
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn card_type_in_match() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Card(crate::ranker::rules::CardCondition {
            card: crate::ranker::rules::CardPredicates {
                type_in: Some(vec!["ATTACK".into(), "SKILL".into()]),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "Strike".into(),
        name: "Strike".into(),
        cost: 1,
        card_type: "ATTACK".into(),
        upgraded: false,
        uuid: None,
        description: "".into(),
        price: None,
        playable: true,
        has_target: true,
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn card_type_not_in_match() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Card(crate::ranker::rules::CardCondition {
            card: crate::ranker::rules::CardPredicates {
                type_not_in: Some(vec!["STATUS".into(), "CURSE".into()]),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "Strike".into(),
        name: "Strike".into(),
        cost: 1,
        card_type: "ATTACK".into(),
        upgraded: false,
        uuid: None,
        description: "".into(),
        price: None,
        playable: true,
        has_target: true,
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn card_type_not_in_fails_when_type_is_excluded() {
    let rules = make_rules(vec![Rule {
        category: RuleCategory::PerTarget,
        rule_id: "test".into(),
        priority: 1000,
        weight: Weight::Value(10),
        formula: None,
        score_fn: None,
        override_rule: None,
        applies_to: vec!["play_card".into()],
        conditions: vec![Condition::Card(crate::ranker::rules::CardCondition {
            card: crate::ranker::rules::CardPredicates {
                type_not_in: Some(vec!["STATUS".into(), "CURSE".into()]),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(status_card());
    assert!(!evaluate(&ctx, &rules)[0].matched);
}
