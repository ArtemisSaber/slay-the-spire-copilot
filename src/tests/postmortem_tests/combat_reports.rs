use super::*;

#[test]
fn postmortem_uses_elite_label_for_elite_combats() {
    let mut combat = normalized_fixture("combat-state.json");
    combat["room_type"] = json!("MonsterRoomElite");
    combat["screen_type"] = json!("NONE");
    combat["monsters"] = json!([
        {"name": "地精大法师", "current_hp": 60, "max_hp": 60, "block": 0, "intent": "ATTACK", "damage": 10, "hits": 1, "monster_powers": [], "can_be_killed": false, "is_scaling": false}
    ]);
    combat["current_hp"] = json!(68);

    let mut combat_end = combat.clone();
    combat_end["screen_type"] = json!("COMBAT_REWARD");
    combat_end["monsters"] = json!([]);
    combat_end["current_hp"] = json!(42);

    let input = [state_event(combat), state_event(combat_end)].join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("(精英)"));
    assert!(report.contains("地精大法师"));
    assert!(report.contains("(普通:0 精英:1 Boss:0)"));
}

#[test]
fn postmortem_uses_boss_label_for_boss_combats() {
    let mut combat = normalized_fixture("combat-state.json");
    combat["room_type"] = json!("MonsterRoomBoss");
    combat["screen_type"] = json!("NONE");
    combat["monsters"] = json!([
        {"name": "六火亡魂", "current_hp": 250, "max_hp": 250, "block": 0, "intent": "ATTACK", "damage": 20, "hits": 1, "monster_powers": [], "can_be_killed": false, "is_scaling": false}
    ]);
    combat["current_hp"] = json!(75);

    let mut combat_end = combat.clone();
    combat_end["screen_type"] = json!("COMBAT_REWARD");
    combat_end["monsters"] = json!([]);
    combat_end["current_hp"] = json!(0);

    let input = [state_event(combat), state_event(combat_end)].join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("(Boss)"));
    assert!(report.contains("六火亡魂"));
    assert!(report.contains("(普通:0 精英:0 Boss:1)"));
}

#[test]
fn postmortem_derives_death_cause_from_final_state() {
    let mut combat = normalized_fixture("combat-state.json");
    combat["room_type"] = json!("MonsterRoom");
    combat["screen_type"] = json!("NONE");
    combat["monsters"] = json!([
        {"name": "邪教徒", "current_hp": 10, "max_hp": 10, "block": 0, "intent": "ATTACK", "damage": 6, "hits": 1, "monster_powers": [], "can_be_killed": false, "is_scaling": false},
        {"name": "邪教徒", "current_hp": 12, "max_hp": 12, "block": 0, "intent": "ATTACK", "damage": 6, "hits": 1, "monster_powers": [], "can_be_killed": false, "is_scaling": false}
    ]);
    combat["current_hp"] = json!(5);
    combat["floor"] = json!(22);

    let mut game_over = combat.clone();
    game_over["screen_type"] = json!("GAME_OVER");
    game_over["current_hp"] = json!(0);

    let input = [
        event_line(json!({"schema_version":1,"event":"run_started","ts_ms":123})),
        state_event(combat),
        state_event(game_over),
        event_line(json!({"schema_version":1,"event":"run_ended","reason":"game_over"})),
    ]
    .join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("阵亡原因: 第22层 MonsterRoom — 死于 邪教徒×2"));
}

#[test]
fn postmortem_shows_per_combat_monsters_in_output() {
    let mut combat = normalized_fixture("combat-state.json");
    combat["room_type"] = json!("MonsterRoom");
    combat["screen_type"] = json!("NONE");
    combat["monsters"] = json!([
        {"name": "虱虫", "current_hp": 11, "max_hp": 11, "block": 0, "intent": "ATTACK", "damage": 6, "hits": 1, "monster_powers": [], "can_be_killed": false, "is_scaling": false},
        {"name": "虱虫", "current_hp": 12, "max_hp": 12, "block": 0, "intent": "ATTACK", "damage": 6, "hits": 1, "monster_powers": [], "can_be_killed": false, "is_scaling": false}
    ]);
    combat["current_hp"] = json!(68);

    let mut combat_end = combat.clone();
    combat_end["screen_type"] = json!("COMBAT_REWARD");
    combat_end["monsters"] = json!([]);
    combat_end["current_hp"] = json!(62);

    let input = [state_event(combat), state_event(combat_end)].join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("虱虫×2"));
}

#[test]
fn postmortem_path_without_parent_falls_back_to_bare_name() {
    let path = postmortem_path_for_journal(std::path::Path::new("events.jsonl"));
    assert_eq!(path, std::path::PathBuf::from("postmortem.md"));
}

#[test]
fn generate_report_from_file_missing_results_in_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nonexistent.jsonl");
    let err =
        generate_report_from_journal_file(&path, crate::test_utils::test_locale()).unwrap_err();
    assert!(!err.is_empty());
}

#[test]
fn postmortem_all_lines_malformed_gives_no_valid_events_error() {
    let input = "not json\nstill not json\n";
    let err = generate_report_from_jsonl(input, crate::test_utils::test_locale()).unwrap_err();
    assert!(err.contains("No valid journal events"));
}

#[test]
fn postmortem_general_victory_shows_label_without_heart() {
    let input = [
        event_line(json!({"schema_version":1,"event":"run_started","ts_ms":123})),
        state_event(json!({
            "character": "IRONCLAD",
            "current_hp": 72,
            "max_hp": 75,
            "gold": 99,
            "floor": 50,
            "screen_type": "NONE",
            "room_type": "MonsterRoomBoss",
            "monsters": [],
            "master_cards": [],
            "relics": [],
            "potions": [],
            "deck_names": []
        })),
        event_line(json!({"schema_version":1,"event":"run_ended","reason":"game_over"})),
    ]
    .join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();
    assert!(report.contains("本局胜利！"));
    assert!(!report.contains("Heart defeated!"));
}

#[test]
fn postmortem_heart_victory_shows_heart_defeated() {
    let input = [
        event_line(json!({"schema_version":1,"event":"run_started","ts_ms":123})),
        state_event(json!({
            "character": "IRONCLAD",
            "current_hp": 72,
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
        })),
        event_line(json!({"schema_version":1,"event":"run_ended","reason":"game_over"})),
    ]
    .join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();
    assert!(report.contains("Heart defeated!"));
}

#[test]
fn postmortem_fatal_combat_shows_skull_prefix() {
    let mut combat = normalized_fixture("combat-state.json");
    combat["room_type"] = json!("MonsterRoom");
    combat["screen_type"] = json!("NONE");
    combat["monsters"] = json!([
        {"name": "邪教徒", "current_hp": 10, "max_hp": 10, "block": 0, "intent": "ATTACK", "damage": 6, "hits": 1, "monster_powers": [], "can_be_killed": false, "is_scaling": false}
    ]);
    combat["current_hp"] = json!(75);

    let mut combat_end = combat.clone();
    combat_end["screen_type"] = json!("COMBAT_REWARD");
    combat_end["monsters"] = json!([]);
    combat_end["current_hp"] = json!(0);

    let input = [state_event(combat), state_event(combat_end)].join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("💀"));
    assert!(report.contains("(-75)"));
}

#[test]
fn postmortem_combat_with_healing_shows_positive_delta() {
    let mut combat = normalized_fixture("combat-state.json");
    combat["room_type"] = json!("MonsterRoom");
    combat["screen_type"] = json!("NONE");
    combat["monsters"] = json!([
        {"name": "虱虫", "current_hp": 11, "max_hp": 11, "block": 0, "intent": "ATTACK", "damage": 6, "hits": 1, "monster_powers": [], "can_be_killed": false, "is_scaling": false}
    ]);
    combat["current_hp"] = json!(50);

    let mut combat_end = combat.clone();
    combat_end["screen_type"] = json!("COMBAT_REWARD");
    combat_end["monsters"] = json!([]);
    combat_end["current_hp"] = json!(62);

    let input = [state_event(combat), state_event(combat_end)].join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("(+12)"));
}

#[test]
fn postmortem_normal_combat_shows_normal_count() {
    let mut combat = normalized_fixture("combat-state.json");
    combat["room_type"] = json!("MonsterRoom");
    combat["screen_type"] = json!("NONE");
    combat["monsters"] = json!([
        {"name": "虱虫", "current_hp": 11, "max_hp": 11, "block": 0, "intent": "ATTACK", "damage": 6, "hits": 1, "monster_powers": [], "can_be_killed": false, "is_scaling": false}
    ]);
    combat["current_hp"] = json!(68);

    let mut combat_end = combat.clone();
    combat_end["screen_type"] = json!("COMBAT_REWARD");
    combat_end["monsters"] = json!([]);
    combat_end["current_hp"] = json!(62);

    let input = [state_event(combat), state_event(combat_end)].join("\n");
    let report = generate_report_from_jsonl(&input, crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("(普通:1 精英:0 Boss:0)"));
}
