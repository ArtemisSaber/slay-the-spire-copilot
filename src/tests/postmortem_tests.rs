use super::*;
use crate::state::NormalizedState;
use serde_json::{Value, json};

fn event_line(value: Value) -> String {
    serde_json::to_string(&value).unwrap()
}

fn normalized_fixture(name: &str) -> Value {
    let content = std::fs::read_to_string(format!("tests/fixtures/{name}")).unwrap();
    let raw: Value = serde_json::from_str(&content).unwrap();
    let state = NormalizedState::from_raw(&raw, &crate::test_utils::test_locale());
    serde_json::to_value(state).unwrap()
}

fn state_event(state: Value) -> String {
    event_line(json!({
        "schema_version": 1,
        "event": "state_changed",
        "advice_hash": "hash",
        "observation_hash": "obs",
        "normalized": state
    }))
}

#[test]
fn postmortem_path_sits_next_to_journal() {
    let path = postmortem_path_for_journal(std::path::Path::new("runs/run-1/events.jsonl"));

    assert_eq!(path, std::path::PathBuf::from("runs/run-1/postmortem.md"));
}

#[test]
fn write_report_for_journal_writes_markdown_file() {
    let dir = tempfile::tempdir().unwrap();
    let journal_path = dir.path().join("events.jsonl");
    std::fs::write(
        &journal_path,
        event_line(json!({"schema_version":1,"event":"run_started","ts_ms":123})),
    )
    .unwrap();

    let report =
        generate_report_from_journal_file(&journal_path, &crate::test_utils::test_locale())
            .unwrap();
    let report_path = write_report_for_journal(&journal_path, &report).unwrap();

    assert_eq!(report_path, dir.path().join("postmortem.md"));
    assert!(
        std::fs::read_to_string(report_path)
            .unwrap()
            .contains("# 本局复盘")
    );
}

#[test]
fn postmortem_summarizes_run_start_and_end() {
    let input = [
        event_line(json!({"schema_version":1,"event":"run_started","ts_ms":123})),
        event_line(json!({"schema_version":1,"event":"run_ended","reason":"stdin_closed"})),
    ]
    .join("\n");

    let report = generate_report_from_jsonl(&input, &crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("开始时间: 123"));
    assert!(report.contains("结束原因: stdin_closed"));
}

#[test]
fn postmortem_summarizes_last_observed_state() {
    let state = normalized_fixture("combat-state.json");
    let report =
        generate_report_from_jsonl(&state_event(state), &crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("层数: 1"));
    assert!(report.contains("血量: 68/75"));
    assert!(report.contains("金币: 99"));
    assert!(report.contains("卡组: 1张"));
    assert!(report.contains("遗物: 2个"));
}

#[test]
fn postmortem_lists_advice_events() {
    let input = event_line(json!({
        "schema_version": 1,
        "event": "advice",
        "advice_hash": "abc123",
        "advice": "推荐：选A\n理由：强"
    }));

    let report = generate_report_from_jsonl(&input, &crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("## 关键决策"));
    assert!(report.contains("abc123"));
    assert!(report.contains("推荐：选A"));
}

#[test]
fn postmortem_infers_card_reward_pick_from_deck_diff() {
    let reward = normalized_fixture("card-reward-state.json");
    let mut after = reward.clone();
    after["screen_type"] = Value::String("NONE".into());
    after["card_reward_choices"] = Value::Array(vec![]);
    after["master_cards"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"Uppercut","name":"Uppercut","cost":2,"card_type":"ATTACK","upgraded":false,"uuid":"new-card"}));

    let input = [state_event(reward), state_event(after)].join("\n");
    let report = generate_report_from_jsonl(&input, &crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("已选: Uppercut"));
}

#[test]
fn postmortem_infers_skip_when_deck_unchanged() {
    let reward = normalized_fixture("card-reward-state.json");
    let mut after = reward.clone();
    after["screen_type"] = Value::String("NONE".into());
    after["card_reward_choices"] = Value::Array(vec![]);

    let input = [state_event(reward), state_event(after)].join("\n");
    let report = generate_report_from_jsonl(&input, &crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("可能跳过"));
}

#[test]
fn postmortem_handles_empty_or_partial_journal() {
    assert!(
        generate_report_from_jsonl("", &crate::test_utils::test_locale())
            .unwrap_err()
            .contains("No journal events")
    );

    let report = generate_report_from_jsonl(
        "not json\n{\"event\":\"run_started\",\"ts_ms\":1}",
        &crate::test_utils::test_locale(),
    )
    .unwrap();
    assert!(report.contains("忽略格式错误: 1行"));
}

#[test]
fn ai_postmortem_prompt_wraps_deterministic_report() {
    let prompt = build_ai_postmortem_prompt(
        "# Slay the Spire Postmortem\n- Floor: 5",
        &crate::test_utils::test_locale(),
        "Victory",
    );

    assert!(prompt.contains("中文复盘报告"));
    assert!(prompt.contains("不要补充日志里没有的内容"));
    assert!(prompt.contains("# Slay the Spire Postmortem"));
    assert!(prompt.contains("Floor: 5"));
    assert!(prompt.contains("Outcome: Victory"));
}

#[test]
fn postmortem_includes_run_metadata_character_and_ascension() {
    let input = [
        event_line(json!({"schema_version":1,"event":"run_started","ts_ms":123})),
        event_line(json!({"schema_version":1,"event":"run_metadata","character":"DEFECT","ascension_level":5,"seed":1,"floor":0})),
    ]
    .join("\n");

    let report = generate_report_from_jsonl(&input, &crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("角色: DEFECT"));
    assert!(report.contains("进阶: A5"));
}

#[test]
fn postmortem_includes_character_from_final_state_when_no_metadata() {
    let state = normalized_fixture("combat-state.json");
    let report =
        generate_report_from_jsonl(&state_event(state), &crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("角色: IRONCLAD"));
}

#[test]
fn postmortem_lists_relics_with_descriptions() {
    let state = normalized_fixture("combat-state.json");
    let report =
        generate_report_from_jsonl(&state_event(state), &crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("## 遗物"));
    assert!(report.contains("Burning Blood"));
    assert!(report.contains("Neow's Lament"));
}

#[test]
fn postmortem_lists_potions_when_present() {
    let state = normalized_fixture("combat-state.json");
    let report =
        generate_report_from_jsonl(&state_event(state), &crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("## 药水"));
    assert!(report.contains("Fear Potion"));
}

#[test]
fn postmortem_omits_potions_section_when_empty() {
    let mut state = normalized_fixture("combat-state.json");
    state["potions"] = json!([]);

    let report =
        generate_report_from_jsonl(&state_event(state), &crate::test_utils::test_locale()).unwrap();

    assert!(!report.contains("## 药水"));
}

#[test]
fn postmortem_lists_deck_cards_with_duplicates_counted() {
    let state = normalized_fixture("combat-state.json");
    let report =
        generate_report_from_jsonl(&state_event(state), &crate::test_utils::test_locale()).unwrap();

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
    let report = generate_report_from_jsonl(&input, &crate::test_utils::test_locale()).unwrap();

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

    let report = generate_report_from_jsonl(&input, &crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("abc123"));
    assert!(report.contains("推荐：选A"));
    assert!(report.contains("理由：非常强"));
    assert!(report.contains("风险：可能会卡手"));
    assert!(report.contains("吐槽：勇敢的人才敢选"));
}

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
    let report = generate_report_from_jsonl(&input, &crate::test_utils::test_locale()).unwrap();

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
    let report = generate_report_from_jsonl(&input, &crate::test_utils::test_locale()).unwrap();

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
    let report = generate_report_from_jsonl(&input, &crate::test_utils::test_locale()).unwrap();

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
    let report = generate_report_from_jsonl(&input, &crate::test_utils::test_locale()).unwrap();

    assert!(report.contains("虱虫×2"));
}
