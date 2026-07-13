use super::*;

#[test]
fn postmortem_skips_state_changed_without_normalized_field() {
    let input = event_line(json!({
        "schema_version": 1,
        "event": "state_changed",
        "advice_hash": "hash",
        "observation_hash": "obs"
    }));
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();
    assert!(report.contains("# 本局复盘"));
}

#[test]
fn postmortem_advice_without_hash_uses_question_mark() {
    let input = event_line(json!({
        "schema_version": 1,
        "event": "advice",
        "advice": "建议选A"
    }));
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();
    assert!(report.contains("`?`:"));
    assert!(report.contains("建议选A"));
}

#[test]
fn postmortem_advice_with_state_hash_fallback() {
    let input = event_line(json!({
        "schema_version": 1,
        "event": "advice",
        "state_hash": "state-abc",
        "advice": "建议选B"
    }));
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();
    assert!(report.contains("`state-abc`:"));
}

#[test]
fn postmortem_advice_with_empty_text_produces_entry_without_content() {
    let input = event_line(json!({
        "schema_version": 1,
        "event": "advice",
        "advice_hash": "empty-hash"
    }));
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();
    assert!(report.contains("`empty-hash`:"));
}

#[test]
fn postmortem_deck_with_duplicate_cards_shows_multiplied_count() {
    let mut state = normalized_fixture("combat-state.json");
    state["screen_type"] = json!("NONE");
    state["deck_names"]
        .as_array_mut()
        .unwrap()
        .push(json!("Strike"));
    if let Some(arr) = state["master_cards"].as_array_mut() {
        arr.push(
            json!({"id":"Strike_R","name":"Strike","cost":1,"card_type":"ATTACK","upgraded":false,"uuid":"extra"}),
        );
    }

    let report =
        generate_report_from_jsonl(&state_event(state), crate::test_utils::test_locale()).unwrap();
    assert!(report.contains("Strike ×2"));
}

#[test]
fn postmortem_omits_deck_section_when_deck_names_empty() {
    let state = json!({
        "character": "IRONCLAD",
        "current_hp": 75,
        "max_hp": 75,
        "gold": 99,
        "floor": 1,
        "screen_type": "NONE",
        "monsters": [],
        "master_cards": [],
        "relics": [],
        "potions": [],
        "deck_names": []
    });
    let report =
        generate_report_from_jsonl(&state_event(state), crate::test_utils::test_locale()).unwrap();
    assert!(!report.contains("## 卡组"));
}

#[test]
fn postmortem_deck_section_absent_when_deck_names_missing() {
    let state = json!({
        "character": "IRONCLAD",
        "current_hp": 75,
        "max_hp": 75,
        "gold": 99,
        "floor": 1,
        "screen_type": "NONE",
        "monsters": [],
        "master_cards": [],
        "relics": [],
        "potions": []
    });
    let report =
        generate_report_from_jsonl(&state_event(state), crate::test_utils::test_locale()).unwrap();
    assert!(!report.contains("## 卡组"));
}

#[test]
fn postmortem_death_with_no_monsters_shows_question_mark() {
    let state = json!({
        "character": "IRONCLAD",
        "current_hp": 0,
        "max_hp": 75,
        "gold": 99,
        "floor": 10,
        "screen_type": "GAME_OVER",
        "room_type": "MonsterRoom",
        "monsters": [],
        "master_cards": [],
        "relics": [],
        "potions": [],
        "deck_names": []
    });
    let input = [
        event_line(json!({"schema_version":1,"event":"run_started","ts_ms":123})),
        state_event(state),
        event_line(json!({"schema_version":1,"event":"run_ended","reason":"game_over"})),
    ]
    .join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();
    assert!(report.contains("死于 ?"));
}

#[test]
fn postmortem_run_ended_without_reason_produces_no_ended_label() {
    let input = [
        event_line(json!({"schema_version":1,"event":"run_started","ts_ms":123})),
        event_line(json!({"schema_version":1,"event":"run_ended"})),
    ]
    .join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();
    assert!(!report.contains("结束原因"));
}
