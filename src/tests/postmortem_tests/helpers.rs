use super::*;

#[test]
fn combine_postmortem_report_combines_all_sections() {
    let ai = "# AI Report\nA sufficiently long AI-generated postmortem body for the test.";
    let combined = combine_postmortem_report(
        ai,
        "## Deterministic\nMore content",
        "## Machine Summary\nData",
    );
    assert!(combined.starts_with("# AI Report"));
    assert!(combined.contains("\n\n---\n\n"));
    assert!(combined.contains("## Machine Summary\nData"));
    assert!(combined.contains("## Deterministic\nMore content"));
}

#[test]
fn combine_postmortem_report_drops_empty_ai_report() {
    let combined = combine_postmortem_report(
        "   \n  ",
        "## Deterministic\nMore content",
        "## Machine Summary\nData",
    );
    assert!(!combined.contains("---"));
    assert!(combined.contains("## Machine Summary\nData"));
    assert!(combined.contains("## Deterministic\nMore content"));
}

#[test]
fn combine_postmortem_report_drops_too_short_ai_report() {
    let combined = combine_postmortem_report(
        "ok",
        "## Deterministic\nMore content",
        "## Machine Summary\nData",
    );
    assert!(!combined.contains("---"));
    assert!(!combined.contains("ok"));
    assert!(combined.contains("## Deterministic\nMore content"));
}

#[test]
fn combine_postmortem_report_keeps_valid_long_ai_report() {
    let ai = "# 本局复盘\n\n这是一段足够长的 AI 复盘内容，用于验证长度阈值不会误杀正常报告输出，此处补充更多正文以超过阈值。";
    let combined = combine_postmortem_report(
        ai,
        "## Deterministic\nMore content",
        "## Machine Summary\nData",
    );
    assert!(combined.starts_with("# 本局复盘"));
    assert!(combined.contains("---"));
}

#[test]
fn postmortem_display_i64_missing_shows_question_mark() {
    let state = json!({"current_hp": 75});
    let result = display_i64(&state, "gold");
    assert_eq!(result, "?");
}

#[test]
fn postmortem_deck_counts_absent_master_cards_returns_empty() {
    let state = json!({"character": "IRONCLAD"});
    let counts = deck_counts(&state);
    assert!(counts.is_empty());
}

#[test]
fn postmortem_deck_counts_skips_cards_without_id() {
    let state = json!({
        "master_cards": [
            {"name": "Strike"},
            {"id": "Strike_R", "name": "Strike"}
        ]
    });
    let counts = deck_counts(&state);
    assert_eq!(counts.len(), 1);
    assert_eq!(counts.get("Strike_R").copied(), Some(1));
}

#[test]
fn postmortem_reward_snapshot_none_when_no_choices() {
    let state = json!({
        "character": "IRONCLAD",
        "master_cards": []
    });
    let snap = RewardSnapshot::from_state(&state);
    assert!(snap.is_none());
}

#[test]
fn postmortem_reward_snapshot_filters_choices_without_id() {
    let state = json!({
        "character": "IRONCLAD",
        "master_cards": [],
        "card_reward_choices": [
            {"name": "BadCard"},
            {"id": "GoodCard", "name": "GoodCard"}
        ]
    });
    let snap = RewardSnapshot::from_state(&state).unwrap();
    assert_eq!(snap.choices.len(), 1);
    assert_eq!(snap.choices[0].0, "GoodCard");
}

#[test]
fn postmortem_relic_without_name_is_filtered() {
    let state = json!({
        "character": "IRONCLAD",
        "current_hp": 75,
        "max_hp": 75,
        "gold": 99,
        "floor": 1,
        "screen_type": "NONE",
        "monsters": [],
        "master_cards": [],
        "potions": [],
        "deck_names": [],
        "relics": [
            {"description": "no name relic"},
            {"name": "Burning Blood", "description": "Heal after combat"}
        ]
    });
    let report =
        generate_report_from_jsonl(&state_event(state), crate::test_utils::test_locale()).unwrap();
    assert!(report.contains("Burning Blood"));
    assert!(!report.contains("no name relic"));
}

#[test]
fn postmortem_potion_without_name_is_filtered() {
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
        "deck_names": [],
        "potions": [
            {"description": "nameless potion"},
            {"name": "Fear Potion", "description": "Apply 3 Vulnerable"}
        ]
    });
    let report =
        generate_report_from_jsonl(&state_event(state), crate::test_utils::test_locale()).unwrap();
    assert!(report.contains("Fear Potion"));
    assert!(!report.contains("nameless potion"));
}

#[test]
fn postmortem_relics_section_absent_when_empty() {
    let mut state = normalized_fixture("combat-state.json");
    state["relics"] = json!([]);
    let report =
        generate_report_from_jsonl(&state_event(state), crate::test_utils::test_locale()).unwrap();
    assert!(!report.contains("## 遗物"));
}

#[test]
fn postmortem_run_ended_without_reason_with_victory_room() {
    let state = json!({
        "character": "IRONCLAD",
        "current_hp": 1,
        "max_hp": 75,
        "gold": 99,
        "floor": 50,
        "screen_type": "NONE",
        "room_type": "VictoryRoom",
        "monsters": [],
        "master_cards": [],
        "relics": [],
        "potions": [],
        "deck_names": []
    });
    let input = [
        event_line(json!({"schema_version":1,"event":"run_started","ts_ms":123})),
        state_event(state),
        event_line(json!({"schema_version":1,"event":"run_ended"})),
    ]
    .join("\n");
    // hp > 0 but run_ended without reason — is_victory is false, no ended label
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();
    assert!(!report.contains("结束原因"));
    assert!(!report.contains("胜利"));
}

#[test]
fn postmortem_death_with_duplicate_monsters_counts_them() {
    let state = json!({
        "character": "IRONCLAD",
        "current_hp": 0,
        "max_hp": 75,
        "gold": 99,
        "floor": 22,
        "screen_type": "GAME_OVER",
        "room_type": "MonsterRoom",
        "monsters": [
            {"name": "邪教徒"},
            {"name": "邪教徒"},
            {"name": "虱虫"}
        ],
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
    // Sorted by name: 虱虫, 邪教徒×2 → "虱虫、邪教徒×2"
    assert!(report.contains("邪教徒×2"));
    assert!(report.contains("虱虫"));
}
