use super::*;
use crate::state::NormalizedState;
use serde_json::{Value, json};

fn event_line(value: Value) -> String {
    serde_json::to_string(&value).unwrap()
}

fn normalized_fixture(name: &str) -> Value {
    let content = std::fs::read_to_string(format!("tests/fixtures/{name}")).unwrap();
    let raw: Value = serde_json::from_str(&content).unwrap();
    let state = NormalizedState::from_raw(&raw);
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

    let report = generate_report_from_journal_file(&journal_path).unwrap();
    let report_path = write_report_for_journal(&journal_path, &report).unwrap();

    assert_eq!(report_path, dir.path().join("postmortem.md"));
    assert!(
        std::fs::read_to_string(report_path)
            .unwrap()
            .contains("# Slay the Spire Postmortem")
    );
}

#[test]
fn postmortem_summarizes_run_start_and_end() {
    let input = [
        event_line(json!({"schema_version":1,"event":"run_started","ts_ms":123})),
        event_line(json!({"schema_version":1,"event":"run_ended","reason":"stdin_closed"})),
    ]
    .join("\n");

    let report = generate_report_from_jsonl(&input).unwrap();

    assert!(report.contains("Started: 123"));
    assert!(report.contains("Ended: stdin_closed"));
}

#[test]
fn postmortem_summarizes_last_observed_state() {
    let state = normalized_fixture("combat-state.json");
    let report = generate_report_from_jsonl(&state_event(state)).unwrap();

    assert!(report.contains("Floor: 1"));
    assert!(report.contains("HP: 68/75"));
    assert!(report.contains("Gold: 99"));
    assert!(report.contains("Deck: 1 cards"));
    assert!(report.contains("Relics: 2"));
}

#[test]
fn postmortem_lists_advice_events() {
    let input = event_line(json!({
        "schema_version": 1,
        "event": "advice",
        "advice_hash": "abc123",
        "advice": "推荐：选A\n理由：强"
    }));

    let report = generate_report_from_jsonl(&input).unwrap();

    assert!(report.contains("## Advice"));
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
    let report = generate_report_from_jsonl(&input).unwrap();

    assert!(report.contains("Picked: Uppercut"));
}

#[test]
fn postmortem_infers_skip_when_deck_unchanged() {
    let reward = normalized_fixture("card-reward-state.json");
    let mut after = reward.clone();
    after["screen_type"] = Value::String("NONE".into());
    after["card_reward_choices"] = Value::Array(vec![]);

    let input = [state_event(reward), state_event(after)].join("\n");
    let report = generate_report_from_jsonl(&input).unwrap();

    assert!(report.contains("Likely skipped"));
}

#[test]
fn postmortem_handles_empty_or_partial_journal() {
    assert!(
        generate_report_from_jsonl("")
            .unwrap_err()
            .contains("No journal events")
    );

    let report =
        generate_report_from_jsonl("not json\n{\"event\":\"run_started\",\"ts_ms\":1}").unwrap();
    assert!(report.contains("Ignored malformed lines: 1"));
}

#[test]
fn ai_postmortem_prompt_wraps_deterministic_report() {
    let prompt = build_ai_postmortem_prompt("# Slay the Spire Postmortem\n- Floor: 5");

    assert!(prompt.contains("中文复盘报告"));
    assert!(prompt.contains("不要补充日志里没有的内容"));
    assert!(prompt.contains("# Slay the Spire Postmortem"));
    assert!(prompt.contains("Floor: 5"));
}
