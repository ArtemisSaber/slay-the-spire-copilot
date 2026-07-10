use super::*;

#[test]
fn event_prompt_lists_event_text_and_choices() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Event),
        event_id: None,
        event_name: Some("金神像".into()),
        event_body: Some("一个金色神像闪闪发光。".into()),
        event_choices: vec!["拿走神像".into(), "离开".into()],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 事件 ==="));
    assert!(prompt.contains("金神像"));
    assert!(prompt.contains("一个金色神像闪闪发光。"));
    assert!(prompt.contains("A. 拿走神像"));
    assert!(prompt.contains("B. 离开"));
}

#[test]
fn event_prompt_does_not_emit_question_mark_garble() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Event),
        room_type: Some(RoomType::NeowRoom),
        event_id: None,
        event_name: None,
        event_body: None,
        event_choices: vec![
            "选项 1（事件文本不可读，请在游戏内核对按钮）".into(),
            "选项 2（事件文本不可读，请在游戏内核对按钮）".into(),
        ],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("事件文本不可读（房间：NeowRoom）"));
    assert!(prompt.contains("A. 选项 1（事件文本不可读，请在游戏内核对按钮）"));
    assert!(prompt.contains("B. 选项 2（事件文本不可读，请在游戏内核对按钮）"));
    assert!(!prompt.contains("???"));
}

#[test]
fn build_event_choice_with_id_and_name() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Event),
        event_id: Some("Big Fish".into()),
        event_name: Some("大鲸".into()),
        event_body: Some("一个巨大的鲸鱼挡住了去路。".into()),
        event_choices: vec!["香蕉".into(), "甜甜圈".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("大鲸"));
    assert!(prompt.contains("Big Fish"));
    assert!(prompt.contains("一个巨大的鲸鱼挡住了去路"));
    assert!(prompt.contains("A. 香蕉"));
    assert!(prompt.contains("B. 甜甜圈"));
}

#[test]
fn build_event_choice_body_only_no_event_info() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some(ScreenType::Event),
        event_id: None,
        event_name: None,
        room_type: None,
        event_body: Some("一段描述文本。".into()),
        event_choices: vec!["选项A".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("一段描述文本"));
    assert!(prompt.contains("A. 选项A"));
    assert!(!prompt.contains("???"));
}
