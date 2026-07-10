use super::*;

#[test]
fn monster_exclude_target_excludes_target() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                exclude_target: Some(true),
                is_scaling: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![
        MonsterInfo {
            name: "Target".into(),
            monster_id: None,
            index: 0,
            is_scaling: true,
            ..Default::default()
        },
        MonsterInfo {
            name: "Other".into(),
            monster_id: None,
            index: 1,
            is_scaling: false,
            ..Default::default()
        },
    ];
    let ctx = monsters_ctx(monsters, 0);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monster_exclude_target_finds_other() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                exclude_target: Some(true),
                is_scaling: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![
        MonsterInfo {
            name: "Target".into(),
            monster_id: None,
            index: 0,
            is_scaling: false,
            ..Default::default()
        },
        MonsterInfo {
            name: "Other".into(),
            monster_id: None,
            index: 1,
            is_scaling: true,
            ..Default::default()
        },
    ];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monsters_none_matches_when_no_monster_satisfies() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            none: Some(crate::ranker::rules::MonsterSubPredicates {
                monster_id: Some("AwakenedOne".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "Jaw Worm".into(),
        monster_id: Some("JawWorm".into()),
        index: 0,
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monsters_none_fails_when_monster_satisfies() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            none: Some(crate::ranker::rules::MonsterSubPredicates {
                monster_id: Some("AwakenedOne".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "Awakened One".into(),
        monster_id: Some("AwakenedOne".into()),
        index: 0,
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monsters_none_exclude_target_finds_none_other() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            none: Some(crate::ranker::rules::MonsterSubPredicates {
                exclude_target: Some(true),
                is_scaling: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![
        MonsterInfo {
            name: "Target".into(),
            index: 0,
            is_scaling: true,
            ..Default::default()
        },
        MonsterInfo {
            name: "Other".into(),
            index: 1,
            is_scaling: false,
            ..Default::default()
        },
    ];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monsters_none_with_intent_fails_when_monster_has_intent() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            none: Some(crate::ranker::rules::MonsterSubPredicates {
                intent: Some("ATTACK".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "A".into(),
        index: 0,
        intent: Some("ATTACK".into()),
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monsters_none_with_intent_matches_when_no_monster_has_intent() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            none: Some(crate::ranker::rules::MonsterSubPredicates {
                intent: Some("ATTACK".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "A".into(),
        index: 0,
        intent: Some("BUFF".into()),
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monsters_both_any_and_none_combined() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                is_scaling: Some(true),
                ..Default::default()
            }),
            none: Some(crate::ranker::rules::MonsterSubPredicates {
                intent: Some("ATTACK".into()),
                ..Default::default()
            }),
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![
        MonsterInfo {
            name: "Scaler".into(),
            index: 0,
            is_scaling: true,
            intent: Some("BUFF".into()),
            ..Default::default()
        },
        MonsterInfo {
            name: "Attacker".into(),
            index: 1,
            is_scaling: false,
            intent: Some("BUFF".into()),
            ..Default::default()
        },
    ];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}
