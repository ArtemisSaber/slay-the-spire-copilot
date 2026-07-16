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

pub(crate) fn mock_advice_response(system_prompt: &str, user_prompt: &str) -> String {
    if system_prompt.contains("LEARNING_CRITIC_ENVELOPE_V2") {
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
        .rsplit_once("LEARNING_CRITIC_ENVELOPE_V2")
        .and_then(|(_, appendix)| serde_json::from_str::<serde_json::Value>(appendix.trim()).ok());
    let primary_case_id = appendix
        .as_ref()
        .and_then(|value| value.get("primary_case_id"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("missing");
    let outcome = appendix
        .as_ref()
        .and_then(|value| value.get("run_outcome"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("victory");
    let primary_case = appendix.as_ref().and_then(|value| {
        value
            .get("eligible_cases")?
            .as_array()?
            .iter()
            .find(|case| {
                case.get("case_id").and_then(serde_json::Value::as_str) == Some(primary_case_id)
            })
            .cloned()
    });
    let proposal = primary_case.and_then(|case| mock_lesson_proposal(&case));
    serde_json::json!({
        "schema_version": 2,
        "report_markdown": "# Mock Review\n\nThis human-readable postmortem was transported inside the JSON envelope.",
        "run_analysis": {
            "outcome": outcome,
            "primary_case_id": primary_case_id,
            "contributing_case_ids": [],
            "explanation": "The deterministic primary case is the closest observed decision to the run outcome.",
            "confidence_millis": 700
        },
        "lesson_proposals": proposal.into_iter().collect::<Vec<_>>(),
    })
    .to_string()
}

fn mock_lesson_proposal(case: &serde_json::Value) -> Option<serde_json::Value> {
    let situation = case.get("situation")?;
    let action = case.get("selected_action")?;
    let (kind, card_ids, potion_ids) = match action.get("kind")?.as_str()? {
        "play_card" => (
            "play_card",
            vec![action.get("card_id")?.as_str()?.to_string()],
            Vec::new(),
        ),
        "use_potion" => (
            "use_potion",
            Vec::new(),
            vec![action.get("potion_id")?.as_str()?.to_string()],
        ),
        "end_turn" => ("end_turn", Vec::new(), Vec::new()),
        _ => return None,
    };
    let outcome_code = match case
        .pointer("/later_outcome/combat_won")
        .and_then(serde_json::Value::as_bool)
    {
        Some(true) => "combat_win",
        Some(false) => "combat_death",
        None => return None,
    };
    Some(serde_json::json!({
        "scope": {
            "character": situation.get("character")?,
            "objective": situation.get("objective")?,
            "ascension_bands": [situation.get("ascension_band")?],
            "encounter_ids": situation.get("encounter_ids")?,
        },
        "trigger": {
            "turn_buckets": [situation.get("turn_bucket")?],
            "block_threat_buckets": [situation.get("block_threat_bucket")?],
            "required_card_ids": [],
            "required_enemy_power_ids": [],
            "required_ranker_tags": [],
        },
        "action_pattern": {
            "kind": kind,
            "card_types": [],
            "card_ids": card_ids,
            "potion_ids": potion_ids,
        },
        "outcome_code": outcome_code,
        "guidance": {
            "kind": if outcome_code == "combat_death" { "avoid" } else { "prefer" },
            "text": "Treat this as observational evidence and re-check the current state.",
        },
        "rationale": "The cited case matched this action and observed combat outcome.",
        "source_case_ids": [case.get("case_id")?],
        "confidence_millis": 700,
    }))
}
