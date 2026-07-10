use super::*;

#[test]
fn parsed_damage_gt_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            damage_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        damage: Some(6),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_damage_gt_no_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            damage_gt: Some(10),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        damage: Some(5),
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_damage_eq_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            damage_eq: Some(6),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        damage: Some(6),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_block_gt_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            block_gt: Some(4),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        block: Some(8),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_self_damage_gt_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            self_damage_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        self_damage: Some(3),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_exhausts_cards_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            exhausts_cards: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        exhaust_count: 2,
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_exhausts_cards_no_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            exhausts_cards: Some(false),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        exhaust_count: 2,
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_exhausts_status_curse_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            exhausts_status_curse: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut ctx = make_parsed_ctx(ParsedEffects {
        exhaust_count: 1,
        ..Default::default()
    });
    ctx.card = Some(status_card());
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_exhausts_status_curse_no_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            exhausts_status_curse: Some(false),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let mut ctx = make_parsed_ctx(ParsedEffects {
        exhaust_count: 1,
        ..Default::default()
    });
    ctx.card = Some(status_card());
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_damage_eq_no_match() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            damage_eq: Some(6),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        damage: Some(8),
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_exhausts_cards_true_fails_when_count_zero() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            exhausts_cards: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        exhaust_count: 0,
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_exhausts_cards_false_matches_when_count_zero() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            exhausts_cards: Some(false),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        exhaust_count: 0,
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}
