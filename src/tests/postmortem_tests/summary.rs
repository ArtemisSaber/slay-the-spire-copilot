use super::*;

#[test]
fn postmortem_includes_run_metadata_character_and_ascension() {
    let input = [
        event_line(json!({"schema_version":1,"event":"run_started","ts_ms":123})),
        event_line(json!({"schema_version":1,"event":"run_metadata","character":"DEFECT","ascension_level":5,"seed":1,"floor":0})),
    ]
    .join("\n");

    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("角色: DEFECT"));
    assert!(report.contains("进阶: A5"));
}

#[test]
fn postmortem_includes_character_from_final_state_when_no_metadata() {
    let state = normalized_fixture("combat-state.json");
    let report =
        generate_report_from_jsonl(&state_event(state), crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("角色: IRONCLAD"));
}

#[test]
fn postmortem_lists_relics_with_descriptions() {
    let state = normalized_fixture("combat-state.json");
    let report =
        generate_report_from_jsonl(&state_event(state), crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("## 遗物"));
    assert!(report.contains("Burning Blood"));
    assert!(report.contains("Neow's Lament"));
}

#[test]
fn postmortem_lists_potions_when_present() {
    let state = normalized_fixture("combat-state.json");
    let report =
        generate_report_from_jsonl(&state_event(state), crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("## 药水"));
    assert!(report.contains("Fear Potion"));
}

#[test]
fn postmortem_omits_potions_section_when_empty() {
    let mut state = normalized_fixture("combat-state.json");
    state["potions"] = json!([]);

    let report =
        generate_report_from_jsonl(&state_event(state), crate::test_utils::test_locale()).unwrap();

    assert!(!report.contains("## 药水"));
}

#[test]
fn postmortem_lists_deck_cards_with_duplicates_counted() {
    let state = normalized_fixture("combat-state.json");
    let report =
        generate_report_from_jsonl(&state_event(state), crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("## 卡组"));
    assert!(report.contains("Strike"));
}

#[test]
fn postmortem_tracks_unique_monsters_and_combat_hp() {
    let mut combat1 = normalized_fixture("combat-state.json");
    combat1["room_type"] = json!("MonsterRoom");
    combat1["screen_type"] = json!("BATTLE");
    combat1["monsters"] = json!([
        {"name": "邪教徒", "current_hp": 48, "max_hp": 48, "block": 0, "intent": "ATTACK", "damage": 6, "hits": 1, "monster_powers": [], "can_be_killed": false, "is_scaling": false}
    ]);
    combat1["current_hp"] = json!(68);

    let mut combat1_end = combat1.clone();
    combat1_end["screen_type"] = json!("NONE");
    combat1_end["monsters"] = json!([]);
    combat1_end["current_hp"] = json!(62);

    let input = [state_event(combat1), state_event(combat1_end)].join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("## 战斗记录"));
    assert!(report.contains("邪教徒"));
    assert!(report.contains("战斗: 1场"));
}

#[test]
fn postmortem_advice_includes_full_multiline_text() {
    let input = event_line(json!({
        "schema_version": 1,
        "event": "advice",
        "advice_hash": "abc123",
        "advice": "推荐：选A\n理由：非常强\n风险：可能会卡手\n吐槽：勇敢的人才敢选"
    }));

    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("abc123"));
    assert!(report.contains("推荐：选A"));
    assert!(report.contains("理由：非常强"));
    assert!(report.contains("理由：非常强"));
    assert!(report.contains("风险：可能会卡手"));
    assert!(report.contains("吐槽：勇敢的人才敢选"));
}
