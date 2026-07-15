use super::*;

#[test]
fn status_line_shows_block_warning() {
    let locale = test_locale();
    let state = NormalizedState {
        block: Some(5),
        incoming_damage: 12,
        monsters: vec![MonsterInfo {
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
        }],
        ..test_state()
    };
    let output = status_line(&state, &locale);
    assert!(output.contains("需格挡！"));
}

#[test]
fn status_line_no_block_warning_when_block_sufficient() {
    let locale = test_locale();
    let state = NormalizedState {
        block: Some(20),
        incoming_damage: 12,
        monsters: vec![MonsterInfo {
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
        }],
        ..test_state()
    };
    let output = status_line(&state, &locale);
    assert!(!output.contains("需格挡！"));
}

#[test]
fn status_line_full_kitchen_sink() {
    let locale = test_locale();
    let state = NormalizedState {
        character: Some("DEFECT".into()),
        floor: Some(20),
        current_hp: Some(40),
        max_hp: Some(60),
        block: Some(12),
        energy: Some(4),
        gold: Some(250),
        powers: vec![PowerInfo {
            id: "Focus".into(),
            name: "集中".into(),
            amount: 2,
        }],
        relics: vec![RelicInfo {
            id: "Snecko Eye".into(),
            name: "蛇眼".into(),
            description: "".into(),
            counter: None,
            price: None,
        }],
        potions: vec![PotionInfo {
            id: None,
            slot: 0,
            name: "恐惧药水".into(),
            description: "".into(),
            price: None,
            can_use: true,
            can_discard: false,
            requires_target: false,
        }],
        incoming_damage: 15,
        danger: danger_safe(),
        ..test_state()
    };
    let output = status_line(&state, &locale);
    assert!(output.contains("机器人"));
    assert!(output.contains("40/60"));
    assert!(output.contains("格挡：12"));
    assert!(output.contains("能量：4"));
    assert!(output.contains("250"));
    assert!(output.contains("集中(2)"));
    assert!(output.contains("蛇眼"));
    assert!(output.contains("恐惧药水"));
    assert!(output.contains("15"));
    assert!(output.contains("需格挡"));
}

#[test]
fn status_line_no_energy_or_block() {
    let locale = test_locale();
    let state = NormalizedState {
        character: Some("WATCHER".into()),
        floor: Some(1),
        current_hp: Some(63),
        max_hp: Some(70),
        block: None,
        energy: None,
        gold: Some(50),
        danger: danger_safe(),
        ..test_state()
    };
    let output = status_line(&state, &locale);
    assert!(output.contains("观者"));
    assert!(output.contains("63/70"));
    assert!(!output.contains("格挡："));
    assert!(!output.contains("能量："));
}

#[test]
fn status_line_zero_max_hp() {
    let locale = test_locale();
    let state = NormalizedState {
        current_hp: Some(0),
        max_hp: Some(0),
        block: None,
        energy: None,
        danger: danger_safe(),
        ..test_state()
    };
    let output = status_line(&state, &locale);
    assert!(output.contains("0%"));
}

#[test]
fn status_line_block_none_incoming_no_warning() {
    let locale = test_locale();
    let state = NormalizedState {
        current_hp: Some(50),
        max_hp: Some(75),
        block: None,
        energy: Some(3),
        incoming_damage: 15,
        danger: danger_safe(),
        ..test_state()
    };
    let output = status_line(&state, &locale);
    assert!(output.contains("15"));
    assert!(!output.contains("需格挡"));
}
