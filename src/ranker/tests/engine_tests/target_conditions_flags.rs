use super::*;

#[test]
fn target_power_matches() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            power: Some("Thorns".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        monster_id: None,
        index: 0,
        monster_powers: vec![crate::state::PowerInfo {
            id: "Thorns".into(),
            name: "Thorns".into(),
            amount: 1,
        }],
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_power_not_matches_when_absent() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            power_not: Some("Thorns".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_any_power_in_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            any_power_in: Some(vec!["Thorns".into(), "Metallicize".into()]),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        monster_powers: vec![crate::state::PowerInfo {
            id: "Thorns".into(),
            name: "Thorns".into(),
            amount: 1,
        }],
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_is_scaling_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            is_scaling: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        is_scaling: true,
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_is_minion_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            is_minion: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        monster_powers: vec![crate::state::PowerInfo {
            id: "Minion".into(),
            name: "Minion".into(),
            amount: 1,
        }],
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_is_minion_no_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            is_minion: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_can_be_killed_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            can_be_killed: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        can_be_killed: true,
        ..Default::default()
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_power_not_fails_when_present() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            power_not: Some("Thorns".into()),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        monster_powers: vec![crate::state::PowerInfo {
            id: "Thorns".into(),
            name: "Thorns".into(),
            amount: 1,
        }],
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_is_scaling_no_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            is_scaling: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        is_scaling: false,
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_can_be_killed_no_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            can_be_killed: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        can_be_killed: false,
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_any_power_in_no_match() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            any_power_in: Some(vec!["Thorns".into(), "Metallicize".into()]),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        monster_powers: vec![crate::state::PowerInfo {
            id: "Strength".into(),
            name: "Strength".into(),
            amount: 3,
        }],
        ..Default::default()
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}
