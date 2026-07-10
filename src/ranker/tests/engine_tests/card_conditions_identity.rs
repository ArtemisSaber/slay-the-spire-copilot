use super::*;

#[test]
fn card_id_exact_match() {
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
                id: Some("Strike_R".into()),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "Strike_R".into(),
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
fn card_id_in_match() {
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
                id_in: Some(vec!["Strike_R".into(), "Strike_G".into()]),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "Strike_R".into(),
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
fn card_ethereal_match() {
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
                ethereal: Some(true),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "AscendersBane".into(),
        name: "Ascender's Bane".into(),
        cost: 0,
        card_type: "CURSE".into(),
        upgraded: false,
        uuid: None,
        description: "Ethereal".into(),
        price: None,
        playable: false,
        has_target: false,
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn card_is_none_fails_card_condition() {
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
    let ctx = make_ctx(HashMap::new());
    assert!(!evaluate(&ctx, &rules)[0].matched);
}
