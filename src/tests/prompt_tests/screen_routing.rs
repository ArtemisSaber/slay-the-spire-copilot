use super::*;

#[test]
fn prompt_is_structured() {
    let locale = test_locale();
    let prompt = build_prompt(&test_state(), &locale, false);
    assert!(prompt.contains("=== 当前状态 ==="));
    assert!(prompt.starts_with("[mode: generic]"));
}

#[test]
fn generic_prompt_shows_status_and_format() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: None,
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.starts_with("[mode: generic]"));
}

#[test]
fn build_prompt_routes_rest() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Rest),
        rest_options: vec!["rest".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 选项 ==="));
    assert!(!prompt.contains("=== 手牌"));
}

#[test]
fn build_prompt_routes_event_choice() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Event),
        event_choices: vec!["离开".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.starts_with("[mode: event_choice]"));
}

#[test]
fn build_prompt_routes_combat_when_monsters_present() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: None,
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
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 怪物"));
    assert!(!prompt.contains("=== 选牌 ==="));
}

#[test]
fn build_prompt_routes_generic_when_no_monsters() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: None,
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 当前状态 ==="));
    assert!(!prompt.contains("=== 怪物"));
    assert!(!prompt.contains("=== 选牌 ==="));
    assert!(!prompt.contains("=== 选项 ==="));
}
