use super::*;

#[test]
fn parsed_heal_gt_matches() {
    let rules = make_rules(vec![rule_with_condition(Condition::Parsed(
        crate::ranker::rules::ParsedCondition {
            parsed: crate::ranker::rules::ParsedPredicates {
                heal_gt: Some(0),
                ..Default::default()
            },
        },
    ))]);
    let ctx = make_parsed_ctx(ParsedEffects {
        heal: Some(5),
        ..Default::default()
    });
    let results = evaluate(&ctx, &rules);
    assert!(results[0].matched);
}

#[test]
fn parsed_draw_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            draw_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        draw: Some(2),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_str_gain_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            str_gain_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        str_gain: Some(3),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_dex_gain_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            dex_gain_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        dex_gain: Some(2),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_poison_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            poison_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        poison: Some(5),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_vulnerable_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            vulnerable_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        vulnerable: Some(2),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_weak_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            weak_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        weak: Some(3),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_energy_gain_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            energy_gain_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        energy_gain: Some(2),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_str_loss_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            str_loss_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        str_loss: Some(4),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_str_loss_temp_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            str_loss_temp: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        str_loss: Some(2),
        str_loss_temp: true,
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}
