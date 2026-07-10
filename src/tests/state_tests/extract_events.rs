use super::*;

#[test]
fn event_choices_prefer_choice_text_field() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "screen_state": {
                "event_name": "Wing Statue",
                "body": "A statue of a bird.",
                "options": [
                    {"label": "Pray", "text": "Pray for strength.", "description": "Gain 1 Strength."},
                    {"label": "Leave", "text": "Walk away silently."}
                ]
            },
            "current_hp": 52,
            "max_hp": 75,
            "floor": 8,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(
        state.event_choices,
        vec![
            "Pray for strength.".to_string(),
            "Walk away silently.".to_string()
        ]
    );
}

#[test]
fn event_choices_fallback_to_label_when_text_unreadable() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "screen_state": {
                "event_name": "Test Event",
                "body": "Hello world.",
                "options": [
                    {"label": "Choose me", "text": "????"},
                    {"label": "No, choose me", "text": "??????"}
                ]
            },
            "current_hp": 52,
            "max_hp": 75,
            "floor": 8,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(
        state.event_choices,
        vec!["Choose me".to_string(), "No, choose me".to_string()]
    );
}

#[test]
fn event_choices_extract_from_string_array() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "screen_state": {
                "event_name": "Test",
                "body": "Test body.",
                "options": [
                    "Take the relic",
                    "Leave it alone"
                ]
            },
            "current_hp": 52,
            "max_hp": 75,
            "floor": 8,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(
        state.event_choices,
        vec!["Take the relic".to_string(), "Leave it alone".to_string()]
    );
}

#[test]
fn event_choices_from_choice_list_fallback() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "screen_state": {},
            "choice_list": ["Option A", "Option B"],
            "current_hp": 52,
            "max_hp": 75,
            "floor": 8,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(
        state.event_choices,
        vec!["Option A".to_string(), "Option B".to_string()]
    );
}
