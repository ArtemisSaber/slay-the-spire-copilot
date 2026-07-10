use super::*;

#[test]
fn target_is_max_hp_true() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            is_max_hp: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(20),
        max_hp: Some(20),
        block: Some(0),
        intent: None,
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    });
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_is_max_hp_false() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            is_max_hp: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = target_ctx(MonsterInfo {
        name: "A".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(10),
        max_hp: Some(20),
        block: Some(0),
        intent: None,
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    });
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn target_is_none_fails_target_condition() {
    let c = Condition::Target(crate::ranker::rules::TargetCondition {
        target: crate::ranker::rules::TargetPredicates {
            is_scaling: Some(true),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let ctx = make_ctx(HashMap::new());
    assert!(!evaluate(&ctx, &rules)[0].matched);
}
