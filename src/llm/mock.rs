pub(crate) fn mock_autoplay_action_response(prompt: &str) -> String {
    let prompt_json: serde_json::Value = serde_json::from_str(prompt).unwrap_or_default();
    let test_marker = prompt_json
        .pointer("/scenario/test_marker")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let has_rejections = prompt_json
        .get("rejected_attempts")
        .and_then(|value| value.as_array())
        .is_some_and(|attempts| !attempts.is_empty());

    if test_marker.contains("fallback_test_marker") {
        return r#"{"schema_version":2,"actions":[{"ref":"A99","reason":"","risk":""}]}"#
            .to_string();
    }
    if test_marker.contains("retry_test_marker") && !has_rejections {
        return r#"{"schema_version":2,"actions":[{"ref":"A99","reason":"","risk":""}]}"#
            .to_string();
    }

    let actions = prompt_json
        .get("available_actions")
        .and_then(|value| value.as_array())
        .cloned()
        .unwrap_or_default();
    let preferred = actions
        .iter()
        .find(|action| action.get("kind").and_then(|value| value.as_str()) == Some("skip"))
        .or_else(|| {
            actions
                .iter()
                .find(|action| action.get("kind").and_then(|value| value.as_str()) == Some("play"))
        })
        .or_else(|| actions.first());
    let Some(action) = preferred else {
        return r#"{"schema_version":2,"actions":[]}"#.to_string();
    };

    let action_ref = action
        .get("ref")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let target_index = if action
        .get("target_required")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        prompt_json
            .pointer("/scenario/combat/monsters/0/index")
            .cloned()
            .unwrap_or_else(|| serde_json::json!(0))
    } else {
        serde_json::Value::Null
    };

    serde_json::json!({
        "schema_version": 2,
        "actions": [{
            "ref": action_ref,
            "target_index": target_index,
            "reason": "Mock auto-play planner selected the first preferred available action.",
            "risk": ""
        }]
    })
    .to_string()
}

pub(crate) fn mock_advice_response(system_prompt: &str, user_prompt: &str) -> String {
    if system_prompt.contains("LEARNING_CRITIC_ENVELOPE_V3") {
        mock_learning_postmortem_response(user_prompt)
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

fn mock_learning_postmortem_response(user_prompt: &str) -> String {
    let appendix = user_prompt
        .rsplit_once("LEARNING_CRITIC_ENVELOPE_V3")
        .and_then(|(_, appendix)| serde_json::from_str::<serde_json::Value>(appendix.trim()).ok());
    let mode = appendix
        .as_ref()
        .and_then(|value| value.get("mode"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("report_only");
    let run_id = appendix
        .as_ref()
        .and_then(|value| value.pointer("/origin_benchmark/origin_run_id"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("missing");
    let decisions: Vec<_> = appendix
        .as_ref()
        .and_then(|value| value.pointer(&format!("/run_evidence/{run_id}")))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|case| case.get("decision_id").and_then(serde_json::Value::as_str))
        .take(3)
        .collect();
    let lesson = (mode != "report_only" && !decisions.is_empty()).then(|| {
        serde_json::json!({
            "text": if mode == "regenerate" {
                "Preserve enough HP to establish the deck's setup before committing to an extended damage sequence."
            } else {
                "Prioritize the opponent whose continued presence creates the greatest near-term pressure."
            },
            "applies_when": "Several actions or targets are available and the choice affects later turns.",
            "expected_effect": "This may preserve more options and improve progression in comparable runs.",
            "evidence": [{
                "run_id": run_id,
                "decision_ids": decisions,
                "observed_chain": "The cited ordered decisions were followed by the recorded combat and run outcome."
            }],
            "uncertainty": "The alternatives were not played, so their outcomes remain untested.",
            "confidence_millis": 700
        })
    });
    serde_json::json!({
        "schema_version": 3,
        "report_markdown": "# Mock Review\n\nThis human-readable postmortem was transported inside the JSON envelope.",
        "result": if lesson.is_some() { "lesson" } else { "no_lesson" },
        "lesson": lesson,
        "rejected_lesson_analysis": (mode == "regenerate").then_some(
            "The replacement changes the policy dimension instead of restating the rejected strategy."
        ),
    })
    .to_string()
}
