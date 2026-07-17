use super::*;

fn card_reward_fixture(
    skip_available: bool,
) -> (
    AutoPlayControl,
    AutoPlaySession,
    CommandState,
    NormalizedState,
) {
    let available_commands = if skip_available {
        json!(["choose", "skip"])
    } else {
        json!(["choose"])
    };
    let raw = json!({
        "available_commands": available_commands,
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "floor": 7,
            "character": "IRONCLAD",
            "deck": [
                {"id": "Strike_R", "name": "Strike", "type": "ATTACK"},
                {"id": "Defend_R", "name": "Defend", "type": "SKILL"},
                {"id": "Bash", "name": "Bash", "type": "ATTACK"}
            ],
            "screen_state": {
                "skip_available": skip_available,
                "cards": [
                    {
                        "id": "Uppercut",
                        "name": "Uppercut",
                        "type": "ATTACK",
                        "description": "Deal damage and apply Weak and Vulnerable."
                    },
                    {
                        "id": "Shrug It Off",
                        "name": "Shrug It Off",
                        "type": "SKILL",
                        "description": "Gain Block. Draw 1 card."
                    }
                ]
            }
        }
    });
    (
        AutoPlayControl::default_enabled(),
        AutoPlaySession::default(),
        command_state(&raw),
        state(raw),
    )
}

fn selector_response(prompt: &str, offered_index: usize, reason: &str) -> anyhow::Result<String> {
    let payload: Value = serde_json::from_str(prompt)?;
    let selected_ref = payload["offered_cards"][offered_index]["ref"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("selector prompt is missing offered card ref"))?;
    Ok(json!({
        "schema_version": 1,
        "selected_ref": selected_ref,
        "reason": reason,
        "risk": "",
        "memory_ids_used": []
    })
    .to_string())
}

fn prefer_larger_deck_response(prompt: &str) -> anyhow::Result<String> {
    let payload: Value = serde_json::from_str(prompt)?;
    let states = payload["resulting_states"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("judge prompt is missing resulting states"))?;
    let preferred_ref = states
        .iter()
        .max_by_key(|resulting_state| {
            resulting_state["deck"]
                .as_array()
                .map_or(0, std::vec::Vec::len)
        })
        .and_then(|resulting_state| resulting_state["ref"].as_str())
        .ok_or_else(|| anyhow::anyhow!("judge prompt is missing resulting state ref"))?;
    Ok(json!({
        "schema_version": 1,
        "verdict": "prefer",
        "preferred_ref": preferred_ref,
        "reason": "The preferred resulting state is stronger.",
        "risk": "",
        "memory_ids_used": []
    })
    .to_string())
}

async fn plan_card_reward_with(
    provider: &LlmProvider,
    skip_available: bool,
) -> (PlannedAction, Vec<crate::llm::RecordedRequest>) {
    let (mut control, mut session, command, state) = card_reward_fixture(skip_available);
    let planned = plan_action_with_memory(
        provider,
        &mut control,
        &mut session,
        &command,
        &state,
        &Locale::load("en"),
        false,
        None,
        &[],
    )
    .await
    .unwrap()
    .unwrap();
    let requests = provider.recorded_requests();
    (planned, requests)
}

#[tokio::test]
async fn card_reward_calls_selector_then_blinded_judge_exactly_once_each() {
    const PRIVATE_SELECTOR_REASON: &str = "selector-only-private-reason";
    let provider = LlmProvider::scripted(|system_prompt, prompt, _effort| {
        if system_prompt.contains("CARD_REWARD_CANDIDATE_SELECTOR_V1") {
            selector_response(prompt, 1, PRIVATE_SELECTOR_REASON)
        } else if system_prompt.contains("CARD_REWARD_RESULTING_STATE_JUDGE_V1") {
            if prompt.contains(PRIVATE_SELECTOR_REASON) {
                anyhow::bail!("judge received selector reasoning");
            }
            prefer_larger_deck_response(prompt)
        } else {
            anyhow::bail!("unexpected generic planner call")
        }
    });

    let (planned, requests) = plan_card_reward_with(&provider, true).await;

    assert_eq!(planned.action, AutoPlayAction::Choose(1));
    assert_eq!(planned.source, DecisionSource::Llm);
    assert_eq!(requests.len(), 2);
    assert!(
        requests[0]
            .system_prompt
            .contains("CARD_REWARD_CANDIDATE_SELECTOR_V1")
    );
    assert_eq!(requests[0].effort.as_str(), "medium");
    assert!(
        requests[1]
            .system_prompt
            .contains("CARD_REWARD_RESULTING_STATE_JUDGE_V1")
    );
    assert_eq!(requests[1].effort.as_str(), "heavy");
    assert!(!requests[1].prompt.contains(PRIVATE_SELECTOR_REASON));
}

#[tokio::test]
async fn card_reward_without_skip_only_calls_the_selector() {
    let provider = LlmProvider::scripted(|system_prompt, prompt, _effort| {
        if system_prompt.contains("CARD_REWARD_CANDIDATE_SELECTOR_V1") {
            selector_response(prompt, 1, "Best forced pick.")
        } else {
            anyhow::bail!("unexpected second call")
        }
    });

    let (planned, requests) = plan_card_reward_with(&provider, false).await;

    assert_eq!(planned.action, AutoPlayAction::Choose(1));
    assert_eq!(planned.source, DecisionSource::Llm);
    assert_eq!(requests.len(), 1);
}

#[tokio::test]
async fn selector_failure_skips_without_generic_retries() {
    let provider = LlmProvider::scripted(|_system_prompt, _prompt, _effort| {
        anyhow::bail!("selector unavailable")
    });

    let (planned, requests) = plan_card_reward_with(&provider, true).await;

    assert_eq!(planned.action, AutoPlayAction::Skip);
    assert_eq!(planned.source, DecisionSource::Fallback);
    assert_eq!(planned.selected_action_id, "card_reward:skip");
    assert_eq!(requests.len(), 1);
}

#[tokio::test]
async fn judge_failure_skips_without_generic_retries() {
    let provider = LlmProvider::scripted(|system_prompt, prompt, _effort| {
        if system_prompt.contains("CARD_REWARD_CANDIDATE_SELECTOR_V1") {
            selector_response(prompt, 0, "Best forced pick.")
        } else {
            anyhow::bail!("judge unavailable")
        }
    });

    let (planned, requests) = plan_card_reward_with(&provider, true).await;

    assert_eq!(planned.action, AutoPlayAction::Skip);
    assert_eq!(planned.source, DecisionSource::Fallback);
    assert_eq!(planned.selected_action_id, "card_reward:skip");
    assert_eq!(requests.len(), 2);
}
