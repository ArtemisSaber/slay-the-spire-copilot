use super::*;

#[test]
fn parsed_focus_gain_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            focus_gain_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        focus_gain: Some(2),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_mantra_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            mantra_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        mantra: Some(3),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_channel_orb_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            channel_orb: Some("Lightning".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        channel_orb: Some("Lightning".into()),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_channel_orb_no_match_wrong_type() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            channel_orb: Some("Frost".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        channel_orb: Some("Lightning".into()),
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_orb_slot_expand_gt_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            orb_slot_expand_gt: Some(0),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        orb_slot_expand: 1,
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn parsed_exits_stance_matches() {
    let c = Condition::Parsed(crate::ranker::rules::ParsedCondition {
        parsed: crate::ranker::rules::ParsedPredicates {
            exits_stance: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_parsed_ctx(ParsedEffects {
        exits_stance: true,
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}
