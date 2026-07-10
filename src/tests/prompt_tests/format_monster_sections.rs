use super::*;

#[test]
fn monster_shows_multi_hit_damage() {
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
            damage: Some(4),
            hits: Some(3),
            monster_powers: vec![],
            can_be_killed: false,
            is_scaling: false,
        }],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("（×3）"));
}

#[test]
fn monster_with_block() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "大颚虫".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(44),
        max_hp: Some(46),
        block: Some(8),
        intent: Some("ATTACK".into()),
        damage: Some(12),
        hits: Some(1),
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("格挡：8"));
}

#[test]
fn build_monsters_section_empty() {
    let locale = test_locale();
    let state = NormalizedState {
        monsters: vec![],
        ..test_state()
    };
    let output = build_monsters_section(&state, &locale);
    assert!(output.is_empty());
}
