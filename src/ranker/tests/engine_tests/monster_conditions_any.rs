use super::*;

#[test]
fn monster_is_scaling_any() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                is_scaling: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "A".into(),
        monster_id: None,
        index: 0,
        is_scaling: true,
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monster_is_not_minion_any() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                is_not_minion: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "A".into(),
        monster_id: None,
        index: 0,
        monster_powers: vec![],
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monster_is_not_minion_any_false_when_minion() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                is_not_minion: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "A".into(),
        monster_id: None,
        index: 0,
        monster_powers: vec![crate::state::PowerInfo {
            id: "Minion".into(),
            name: "Minion".into(),
            amount: 1,
        }],
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monster_power_any() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                power: Some("Thorns".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "A".into(),
        monster_id: None,
        index: 0,
        monster_powers: vec![crate::state::PowerInfo {
            id: "Thorns".into(),
            name: "Thorns".into(),
            amount: 1,
        }],
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monster_intent_any() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                intent: Some("ATTACK".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "A".into(),
        monster_id: None,
        index: 0,
        intent: Some("ATTACK".into()),
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monster_id_exact_match() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                monster_id: Some("GremlinNob".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "Gremlin Nob Leader".into(),
        monster_id: Some("GremlinNob".into()),
        index: 0,
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(evaluate(&ctx, &rules)[0].matched);
}

#[test]
fn monster_id_exact_no_false_match() {
    let c = Condition::Monsters(crate::ranker::rules::MonstersCondition {
        monsters: crate::ranker::rules::MonstersPredicates {
            any: Some(crate::ranker::rules::MonsterSubPredicates {
                monster_id: Some("GremlinNob".into()),
                ..Default::default()
            }),
            ..Default::default()
        },
    });
    let rules = make_rules(vec![rule_with_condition(c)]);
    let monsters = vec![MonsterInfo {
        name: "Gremlin Nob Leader".into(),
        monster_id: Some("GremlinNobLeader".into()),
        index: 0,
        ..Default::default()
    }];
    let ctx = monsters_ctx(monsters, 0);
    assert!(!evaluate(&ctx, &rules)[0].matched);
}
