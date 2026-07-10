use super::*;

#[test]
fn normalize_event_choices() {
    let raw = serde_json::json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "screen_state": {
                "event_name": "Golden Idol",
                "body": "A golden idol sits on a pedestal.",
                "options": [
                    {"label": "Take"},
                    {"label": "Leave"}
                ]
            },
            "deck": [],
            "relics": [],
            "current_hp": 52,
            "max_hp": 75,
            "gold": 120,
            "floor": 8,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(
        state.screen_type.as_ref().map(|st| st.as_str()),
        Some("EVENT")
    );
    assert_eq!(state.event_name.as_deref(), Some("Golden Idol"));
    assert_eq!(
        state.event_body.as_deref(),
        Some("A golden idol sits on a pedestal.")
    );
    assert_eq!(
        state.event_choices,
        vec!["Take".to_string(), "Leave".to_string()]
    );
}

#[test]
fn normalize_event_choices_prefer_option_text_for_full_description() {
    let raw = serde_json::json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "screen_state": {
                "event_name": "World of Goop",
                "body": "You fall into a puddle. It is made of slime goop.",
                "options": [
                    {"label": "Lose Gold", "text": "Lose 11 Gold."},
                    {"label": "Lose HP", "text": "Lose 5 HP."}
                ]
            },
            "deck": [],
            "relics": [],
            "current_hp": 52,
            "max_hp": 75,
            "gold": 120,
            "floor": 8,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(
        state.event_body.as_deref(),
        Some("You fall into a puddle. It is made of slime goop.")
    );
    assert_eq!(
        state.event_choices,
        vec!["Lose 11 Gold.".to_string(), "Lose 5 HP.".to_string()]
    );
}

#[test]
fn normalize_event_choices_replaces_unreadable_locale_garble() {
    let raw = serde_json::json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "screen_name": "??",
            "room_type": "NeowRoom",
            "screen_state": {
                "event_name": "??",
                "body": "?????",
                "options": [
                    {"label": "??? 3 ?????????? 1 ???"},
                    {"label": "????? +7"}
                ]
            },
            "deck": [],
            "relics": [],
            "current_hp": 72,
            "max_hp": 72,
            "gold": 99,
            "floor": 0,
            "class": "WATCHER"
        }
    });
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(
        state.screen_type.as_ref().map(|st| st.as_str()),
        Some("EVENT")
    );
    assert_eq!(
        state.room_type.as_ref().map(|rt| rt.as_str()),
        Some("NeowRoom")
    );
    assert!(state.event_name.is_none());
    assert!(state.event_body.is_none());
    assert_eq!(
        state.event_choices,
        vec![
            "选项 1（事件文本不可读，请在游戏内核对按钮）".to_string(),
            "选项 2（事件文本不可读，请在游戏内核对按钮）".to_string(),
        ]
    );
}

#[test]
fn normalize_event_payload_from_communication_mod_log() {
    let raw = serde_json::json!({
        "available_commands": ["choose", "key", "click", "wait", "state"],
        "ready_for_command": true,
        "in_game": true,
        "game_state": {
            "choice_list": ["??? 3 ?????????? 1 ???", "????? +7"],
            "screen_type": "EVENT",
            "screen_state": {
                "event_id": "Neow Event",
                "body_text": "",
                "options": [
                    {
                        "choice_index": 0,
                        "disabled": false,
                        "text": "[ ??? 3 ?????????? 1 ??? ]",
                        "label": "??? 3 ?????????? 1 ???"
                    },
                    {
                        "choice_index": 1,
                        "disabled": false,
                        "text": "[ ????? +7 ]",
                        "label": "????? +7"
                    }
                ],
                "event_name": "??"
            },
            "screen_name": "NONE",
            "room_type": "NeowRoom",
            "deck": [],
            "relics": [{"name": "????", "id": "PureWater", "counter": -1}],
            "current_hp": 72,
            "max_hp": 72,
            "gold": 99,
            "floor": 0,
            "class": "WATCHER"
        }
    });
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(state.event_id.as_deref(), Some("Neow Event"));
    assert!(state.event_name.is_none());
    assert!(state.event_body.is_none());
    assert_eq!(
        state.event_choices,
        vec![
            "选项 1（事件文本不可读，请在游戏内核对按钮）".to_string(),
            "选项 2（事件文本不可读，请在游戏内核对按钮）".to_string(),
        ]
    );
    assert_eq!(
        state.relics,
        vec![RelicInfo {
            id: "PureWater".into(),
            name: "????".into(),
            description: String::new(),
            counter: None,
            price: None,
        }]
    );
}
