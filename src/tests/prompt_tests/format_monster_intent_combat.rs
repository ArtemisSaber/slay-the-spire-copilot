use super::*;

#[test]
fn monster_intent_attack_buff() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "拜蛇术士".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(30),
        max_hp: Some(30),
        block: Some(0),
        intent: Some("ATTACK_BUFF".into()),
        damage: Some(10),
        hits: Some(1),
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("攻击+增益"));
}

#[test]
fn monster_intent_defend() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "哨兵".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(30),
        max_hp: Some(30),
        block: Some(10),
        intent: Some("DEFEND".into()),
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("防御"));
    assert!(output.contains("伤害：无"));
}

#[test]
fn format_monster_attack_debuff_intent() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "震荡波".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(30),
        max_hp: Some(30),
        block: Some(0),
        intent: Some("ATTACK_DEBUFF".into()),
        damage: Some(8),
        hits: Some(1),
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("攻击+减益"));
}

#[test]
fn format_monster_defend_buff_intent() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "哨兵".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(30),
        max_hp: Some(30),
        block: Some(10),
        intent: Some("DEFEND_BUFF".into()),
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("防御+增益"));
}
