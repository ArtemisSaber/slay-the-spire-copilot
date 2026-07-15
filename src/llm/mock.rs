pub(crate) fn mock_autoplay_action_response(prompt: &str) -> String {
    let prompt_json: serde_json::Value = serde_json::from_str(prompt).unwrap_or_default();
    let localized_status_context = prompt_json
        .get("localized_status_context")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let has_rejections = prompt_json
        .get("rejected_attempts")
        .and_then(|value| value.as_array())
        .is_some_and(|attempts| !attempts.is_empty());

    if localized_status_context.contains("fallback_test_marker") {
        return r#"{"schema_version":1,"actions":[{"kind":"choose","action_id":"event:99","label":"Invalid","reason":"","risk":""}]}"#.to_string();
    }
    if localized_status_context.contains("retry_test_marker") && !has_rejections {
        return r#"{"schema_version":1,"actions":[{"kind":"choose","action_id":"event:99","label":"Invalid","reason":"","risk":""}]}"#.to_string();
    }

    let actions = prompt_json
        .get("available_actions")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let preferred = actions
        .iter()
        .find(|action| {
            action.get("action_id").and_then(|value| value.as_str()) == Some("card_reward:skip")
        })
        .or_else(|| {
            actions
                .iter()
                .find(|action| action.get("kind").and_then(|value| value.as_str()) == Some("play"))
        })
        .or_else(|| actions.first());
    let Some(action) = preferred else {
        return r#"{"schema_version":1,"actions":[]}"#.to_string();
    };

    let kind = action
        .get("kind")
        .and_then(|value| value.as_str())
        .unwrap_or("choose");
    let action_id = action
        .get("action_id")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let target_index = if action
        .get("target_required")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        serde_json::json!(0)
    } else {
        serde_json::Value::Null
    };
    let label = action
        .get("label")
        .and_then(|value| value.as_str())
        .unwrap_or("Mock action");

    serde_json::json!({
        "schema_version": 1,
        "actions": [{
            "kind": kind,
            "action_id": action_id,
            "target_index": target_index,
            "label": label,
            "reason": "Mock auto-play planner selected the first preferred available action.",
            "risk": ""
        }]
    })
    .to_string()
}

pub(crate) fn mock_advice_response(system_prompt: &str) -> String {
    if system_prompt.contains("LEARNING_CRITIC_ENVELOPE_V1") {
        serde_json::json!({
            "schema_version": 1,
            "report_markdown": "# Mock Review\n\nThis mock postmortem summarizes the completed run and leaves structured lessons empty.",
            "lesson_proposals": []
        })
        .to_string()
    } else if system_prompt.contains("## 总览") || system_prompt.contains("## Overview") {
        "# 本局复盘\n## 总览\n这是 mock 复盘。\n## 关键决策\n回看选牌、篝火和战斗入口建议。\n## 风险与转折\n关注血量变化和卡组膨胀。\n## 下次改进\n优先保证生存，再贪长期收益。"
            .to_string()
    } else {
        "推荐：出防御牌，注意格挡。\n\
         理由：怪物意图攻击且你HP较低。\n\
         风险：如果不出防御牌可能被斩杀。\n\
         吐槽：这手牌是真的烂。"
            .to_string()
    }
}
