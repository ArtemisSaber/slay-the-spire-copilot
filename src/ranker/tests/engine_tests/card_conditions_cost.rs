use super::*;

#[test]
fn card_cost_eq_matches() {
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
                cost_eq: Some(2),
                ..Default::default()
            },
        })],
    }]);
    let mut ctx = make_ctx(HashMap::new());
    ctx.card = Some(crate::state::CardInfo {
        id: "Carnage".into(),
        name: "Carnage".into(),
        cost: 2,
        card_type: "ATTACK".into(),
        upgraded: false,
        uuid: None,
        description: "".into(),
        price: None,
        playable: true,
        has_target: false,
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn card_cost_eq_no_match() {
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
                cost_eq: Some(2),
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
    assert!(!evaluate(&ctx, &rules)[0].matched);
}
