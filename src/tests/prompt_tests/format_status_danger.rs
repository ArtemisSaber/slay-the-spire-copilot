use super::*;

#[test]
fn danger_prefix_shows_wrath_warning() {
    let locale = test_locale();
    let state = NormalizedState {
        powers: vec![PowerInfo {
            id: "".into(),
            name: "Wrath".into(),
            amount: 1,
        }],
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
        danger: DangerFlags {
            wrath_stance: true,
            any_monster_attacking: true,
            level: DangerLevel::Danger,
            ..test_state().danger
        },
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("愤怒姿态下受到双倍伤害"));
}

#[test]
fn danger_prefix_empty_reasons() {
    let locale = test_locale();
    let state = NormalizedState {
        danger: DangerFlags {
            hp_critical: false,
            incoming_lethal: false,
            no_block_against_hit: false,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Danger,
        },
        ..test_state()
    };
    let output = danger_prefix(&state, &locale);
    assert_eq!(output, "危险！");
}

#[test]
fn danger_prefix_multiple_reasons() {
    let locale = test_locale();
    let state = NormalizedState {
        danger: DangerFlags {
            incoming_lethal: true,
            hp_critical: true,
            no_block_against_hit: true,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Danger,
        },
        ..test_state()
    };
    let output = danger_prefix(&state, &locale);
    assert!(output.contains("危险！"));
    assert!(output.contains("致命伤害"));
    assert!(output.contains("血量危急"));
    assert!(output.contains("无格挡"));
}

#[test]
fn danger_prefix_caution_level() {
    let locale = test_locale();
    let state = NormalizedState {
        danger: DangerFlags {
            hp_critical: false,
            incoming_lethal: false,
            no_block_against_hit: false,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Caution,
        },
        ..test_state()
    };
    let output = danger_prefix(&state, &locale);
    assert_eq!(output, "小心行事。");
}

#[test]
fn danger_prefix_safe_level() {
    let locale = test_locale();
    let state = NormalizedState {
        danger: DangerFlags {
            hp_critical: false,
            incoming_lethal: false,
            no_block_against_hit: false,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Safe,
        },
        ..test_state()
    };
    let output = danger_prefix(&state, &locale);
    assert_eq!(output, "形势不错。");
}

#[test]
fn danger_prefix_incoming_lethal_alone() {
    let locale = test_locale();
    let state = NormalizedState {
        danger: DangerFlags {
            incoming_lethal: true,
            hp_critical: false,
            no_block_against_hit: false,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Danger,
        },
        ..test_state()
    };
    let output = danger_prefix(&state, &locale);
    assert!(output.contains("危险！"));
    assert!(output.contains("致命伤害"));
}

#[test]
fn danger_prefix_hp_critical_alone() {
    let locale = test_locale();
    let state = NormalizedState {
        danger: DangerFlags {
            hp_critical: true,
            incoming_lethal: false,
            no_block_against_hit: false,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Danger,
        },
        ..test_state()
    };
    let output = danger_prefix(&state, &locale);
    assert!(output.contains("危险！"));
    assert!(output.contains("血量危急"));
}

#[test]
fn danger_prefix_no_block_alone() {
    let locale = test_locale();
    let state = NormalizedState {
        danger: DangerFlags {
            hp_critical: false,
            incoming_lethal: false,
            no_block_against_hit: true,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Danger,
        },
        ..test_state()
    };
    let output = danger_prefix(&state, &locale);
    assert!(output.contains("危险！"));
    assert!(output.contains("无格挡"));
}
