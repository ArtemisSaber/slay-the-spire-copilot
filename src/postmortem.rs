use serde_json::Value;
use std::collections::HashMap;

pub fn generate_report_from_jsonl(input: &str) -> Result<String, String> {
    let mut events = Vec::new();
    let mut malformed = 0usize;

    for line in input.lines().map(str::trim).filter(|line| !line.is_empty()) {
        match serde_json::from_str::<Value>(line) {
            Ok(value) => events.push(value),
            Err(_) => malformed += 1,
        }
    }

    if events.is_empty() {
        return Err(if malformed > 0 {
            "No valid journal events found".to_string()
        } else {
            "No journal events found".to_string()
        });
    }

    let mut run_started = None;
    let mut run_ended = None;
    let mut final_state = None;
    let mut advice_lines = Vec::new();
    let mut reward_lines = Vec::new();
    let mut pending_reward: Option<RewardSnapshot> = None;

    for event in &events {
        match event.get("event").and_then(|v| v.as_str()) {
            Some("run_started") => run_started = event.get("ts_ms").and_then(|v| v.as_i64()),
            Some("run_ended") => {
                run_ended = event
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            }
            Some("advice") => {
                let hash = event
                    .get("advice_hash")
                    .or_else(|| event.get("state_hash"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                let advice = event.get("advice").and_then(|v| v.as_str()).unwrap_or("");
                advice_lines.push(format!("- `{hash}`: {}", first_line(advice)));
            }
            Some("state_changed") => {
                if let Some(state) = event.get("normalized") {
                    if let Some(snapshot) = pending_reward.take()
                        && let Some(line) = infer_reward_choice(&snapshot, state)
                    {
                        reward_lines.push(line);
                    }

                    if state.get("screen_type").and_then(|v| v.as_str()) == Some("CARD_REWARD") {
                        pending_reward = RewardSnapshot::from_state(state);
                    }
                    final_state = Some(state.clone());
                }
            }
            _ => {}
        }
    }

    let mut report = Vec::new();
    report.push("# Slay the Spire Postmortem".to_string());
    report.push(String::new());
    report.push("## Run".to_string());
    if let Some(ts) = run_started {
        report.push(format!("- Started: {ts}"));
    }
    if let Some(reason) = run_ended {
        report.push(format!("- Ended: {reason}"));
    }
    if malformed > 0 {
        report.push(format!("- Ignored malformed lines: {malformed}"));
    }

    if let Some(state) = final_state.as_ref() {
        report.push(String::new());
        report.push("## Final State".to_string());
        report.push(format!(
            "- Floor: {}",
            state
                .get("floor")
                .and_then(|v| v.as_i64())
                .map_or("?".to_string(), |v| v.to_string())
        ));
        report.push(format!(
            "- HP: {}/{}",
            display_i64(state, "current_hp"),
            display_i64(state, "max_hp")
        ));
        report.push(format!("- Gold: {}", display_i64(state, "gold")));
        report.push(format!(
            "- Deck: {} cards",
            state
                .get("master_cards")
                .and_then(|v| v.as_array())
                .map_or(0, |arr| arr.len())
        ));
        report.push(format!(
            "- Relics: {}",
            state
                .get("relics")
                .and_then(|v| v.as_array())
                .map_or(0, |arr| arr.len())
        ));
    }

    if !advice_lines.is_empty() {
        report.push(String::new());
        report.push("## Advice".to_string());
        report.extend(advice_lines);
    }

    if !reward_lines.is_empty() {
        report.push(String::new());
        report.push("## Card Rewards".to_string());
        report.extend(reward_lines);
    }

    Ok(report.join("\n"))
}

pub fn build_ai_postmortem_prompt(deterministic_report: &str) -> String {
    format!(
        "\
请根据下面的机器生成复盘摘要，写一份更适合玩家阅读的中文复盘报告。

要求：
- 保留事实和数字，不要补充日志里没有的内容
- 解释建议记录代表什么，而不是只复制原文
- 给出 2-4 条下次改进建议
- 如果摘要信息不足，请指出缺失信息

=== 机器摘要 ===
{deterministic_report}
"
    )
}

#[derive(Debug)]
struct RewardSnapshot {
    choices: Vec<(String, String)>,
    deck_counts: HashMap<String, usize>,
}

impl RewardSnapshot {
    fn from_state(state: &Value) -> Option<Self> {
        let choices = state
            .get("card_reward_choices")?
            .as_array()?
            .iter()
            .filter_map(|card| {
                let id = card.get("id")?.as_str()?.to_string();
                let name = card
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&id)
                    .to_string();
                Some((id, name))
            })
            .collect();

        Some(RewardSnapshot {
            choices,
            deck_counts: deck_counts(state),
        })
    }
}

fn infer_reward_choice(snapshot: &RewardSnapshot, state: &Value) -> Option<String> {
    let next_counts = deck_counts(state);

    for (id, name) in &snapshot.choices {
        let before = snapshot.deck_counts.get(id).copied().unwrap_or(0);
        let after = next_counts.get(id).copied().unwrap_or(0);
        if after > before {
            return Some(format!("- Picked: {name}"));
        }
    }

    if next_counts == snapshot.deck_counts {
        return Some("- Likely skipped".to_string());
    }

    None
}

fn deck_counts(state: &Value) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    if let Some(cards) = state.get("master_cards").and_then(|v| v.as_array()) {
        for card in cards {
            if let Some(id) = card.get("id").and_then(|v| v.as_str()) {
                *counts.entry(id.to_string()).or_insert(0) += 1;
            }
        }
    }
    counts
}

fn display_i64(state: &Value, key: &str) -> String {
    state
        .get(key)
        .and_then(|v| v.as_i64())
        .map_or("?".to_string(), |v| v.to_string())
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or(text)
}

#[cfg(test)]
#[path = "tests/postmortem_tests.rs"]
mod tests;
