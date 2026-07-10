use super::*;

#[test]
fn monster_section_shows_index_and_intent() {
    let locale = test_locale();
    let state = NormalizedState {
        monsters: vec![
            MonsterInfo {
                name: "大颚虫".into(),
                monster_id: None,
                index: 0,
                current_hp: Some(44),
                max_hp: Some(46),
                block: Some(0),
                intent: Some("ATTACK".into()),
                damage: Some(12),
                hits: Some(1),
                monster_powers: vec![],
                can_be_killed: false,
                is_scaling: false,
            },
            MonsterInfo {
                name: "邪教徒".into(),
                monster_id: None,
                index: 1,
                current_hp: Some(18),
                max_hp: Some(40),
                block: Some(0),
                intent: Some("BUFF".into()),
                damage: None,
                hits: None,
                monster_powers: vec![],
                can_be_killed: true,
                is_scaling: false,
            },
        ],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[0]"));
    assert!(prompt.contains("[1]"));
    assert!(prompt.contains("意图：攻击"));
    assert!(prompt.contains("意图：增益"));
    assert!(prompt.contains("可斩杀"));
    assert!(prompt.contains("伤害：12"));
    assert!(prompt.contains("伤害：无"));
}

#[test]
fn monster_intent_sleep() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "地精大法师".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(20),
        max_hp: Some(20),
        block: Some(0),
        intent: Some("SLEEP".into()),
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("睡眠"));
}

#[test]
fn monster_intent_stun() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "哨兵".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(30),
        max_hp: Some(30),
        block: Some(0),
        intent: Some("STUN".into()),
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("眩晕"));
}

#[test]
fn monster_intent_unknown() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "神秘生物".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(10),
        max_hp: Some(10),
        block: Some(0),
        intent: Some("UNKNOWN".into()),
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("未知"));
}

#[test]
fn monster_intent_escape() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "盗贼".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(5),
        max_hp: Some(30),
        block: Some(0),
        intent: Some("ESCAPE".into()),
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("逃跑"));
}

#[test]
fn monster_intent_magic() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "六火亡魂".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(60),
        max_hp: Some(60),
        block: Some(0),
        intent: Some("MAGIC".into()),
        damage: Some(8),
        hits: Some(1),
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("法术"));
}

#[test]
fn monster_intent_raw_string_fallback() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "怪异".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(10),
        max_hp: Some(10),
        block: Some(0),
        intent: Some("CUSTOM_INTENT".into()),
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("CUSTOM_INTENT"));
}

#[test]
fn monster_no_intent() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "测试怪物".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(10),
        max_hp: Some(10),
        block: Some(0),
        intent: None,
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(!output.contains("意图"));
}
