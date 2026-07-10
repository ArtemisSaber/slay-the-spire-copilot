use super::*;

#[test]
fn monster_section_shows_scaling() {
    let locale = test_locale();
    let state = NormalizedState {
        monsters: vec![MonsterInfo {
            name: "大颚虫".into(),
            monster_id: None,
            index: 0,
            current_hp: Some(44),
            max_hp: Some(46),
            block: Some(0),
            intent: Some("ATTACK".into()),
            damage: Some(12),
            hits: None,
            monster_powers: vec![PowerInfo {
                id: "Strength".into(),
                name: "力量".into(),
                amount: 2,
            }],
            can_be_killed: false,
            is_scaling: true,
        }],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("成长中"));
    assert!(prompt.contains("力量(2)"));
}

#[test]
fn monster_is_scaling_with_powers() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "邪教徒".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(18),
        max_hp: Some(40),
        block: Some(0),
        intent: Some("BUFF".into()),
        damage: None,
        hits: None,
        monster_powers: vec![PowerInfo {
            id: "Strength".into(),
            name: "力量".into(),
            amount: 1,
        }],
        can_be_killed: false,
        is_scaling: true,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("成长中"));
    assert!(output.contains("力量(1)"));
}

#[test]
fn format_monster_powers_without_scaling() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "邪教徒".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(18),
        max_hp: Some(40),
        block: Some(0),
        intent: Some("BUFF".into()),
        damage: None,
        hits: None,
        monster_powers: vec![PowerInfo {
            id: "Strength".into(),
            name: "力量".into(),
            amount: 1,
        }],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("力量(1)"));
    assert!(!output.contains("成长中"));
}
