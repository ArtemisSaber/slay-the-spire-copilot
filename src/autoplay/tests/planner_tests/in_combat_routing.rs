use super::*;

fn planner_response(prompt: &str, action_index: usize) -> anyhow::Result<String> {
    let payload: Value = serde_json::from_str(prompt)?;
    let selected_ref = payload["available_actions"][action_index]["ref"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("planner prompt is missing action ref"))?;
    Ok(json!({
        "schema_version": 2,
        "actions": [{
            "ref": selected_ref,
            "reason": "Best tactical choice for this combat.",
            "risk": ""
        }]
    })
    .to_string())
}

fn specialized_card_response(system_prompt: &str, prompt: &str) -> anyhow::Result<String> {
    if system_prompt.contains("CARD_REWARD_CANDIDATE_SELECTOR_V1") {
        return super::card_reward_pipeline::selector_response(
            prompt,
            1,
            "Best permanent-deck card.",
        );
    }
    if system_prompt.contains("CARD_REWARD_RESULTING_STATE_JUDGE_V1") {
        return super::card_reward_pipeline::prefer_larger_deck_response(prompt);
    }
    planner_response(prompt, 1)
}

#[tokio::test]
async fn combat_phase_card_choice_uses_one_fast_generic_planner_call() {
    let raw = json!({
        "available_commands": ["choose", "skip"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "CARD_REWARD",
            "room_phase": "COMBAT",
            "room_type": "MonsterRoomElite",
            "current_action": "CodexAction",
            "floor": 33,
            "deck": [
                {"id": "Strike_R", "name": "Strike", "type": "ATTACK"},
                {"id": "Defend_R", "name": "Defend", "type": "SKILL"}
            ],
            "screen_state": {
                "skip_available": true,
                "cards": [
                    {"id": "Rampage", "name": "Rampage", "type": "ATTACK"},
                    {"id": "Barricade", "name": "Barricade", "type": "POWER"}
                ]
            },
            "combat_state": {
                "turn": 2,
                "player": {"energy": 0, "block": 4, "powers": []},
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "monsters": [{
                    "id": "Cultist",
                    "name": "Cultist",
                    "current_hp": 42,
                    "max_hp": 48,
                    "block": 0,
                    "intent": "BUFF",
                    "move_adjusted_damage": -1,
                    "move_hits": 1,
                    "is_gone": false,
                    "powers": []
                }]
            }
        }
    });
    let provider = LlmProvider::scripted(|system_prompt, prompt, _effort| {
        specialized_card_response(system_prompt, prompt)
    });
    let mut control = AutoPlayControl::default_enabled();
    let mut session = AutoPlaySession::default();
    let command = command_state(&raw);
    let state = state(raw);

    let planned = plan_action_with_memory(
        &provider,
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

    assert_eq!(planned.action, AutoPlayAction::Choose(1));
    assert_eq!(planned.source, DecisionSource::Llm);
    assert_eq!(planned.selected_action_id, "card_reward:1");
    assert_eq!(requests.len(), 1);
    assert!(
        requests[0]
            .system_prompt
            .contains("AUTO_PLAY_ACTION_PLANNER")
    );
    assert_eq!(requests[0].effort.as_str(), "fast");

    let prompt: Value = serde_json::from_str(&requests[0].prompt).unwrap();
    let scenario = &prompt["scenario"];
    assert_eq!(scenario["kind"], "combat_card_choice");
    assert_eq!(scenario["card_choice"]["current_action"], "CodexAction");
    assert_eq!(scenario["card_choice"]["scope"], "current_combat");
    assert_eq!(scenario["card_choice"]["permanent_deck_change"], false);
    assert_eq!(scenario["card_choice"]["choices"][1]["id"], "Barricade");
    assert_eq!(scenario["combat"]["turn"], 2);
    assert_eq!(scenario["combat"]["monsters"][0]["id"], "Cultist");
    assert!(scenario.get("deck").is_none());
    assert!(scenario.get("card_reward").is_none());
}

#[test]
fn every_combat_modal_keeps_its_screen_context_and_adds_combat_context() {
    for (screen_type, context_key) in [
        ("GRID", "grid"),
        ("HAND_SELECT", "hand_select"),
        ("EVENT", "event"),
    ] {
        let state: NormalizedState = serde_json::from_value(json!({
            "screen_type": screen_type,
            "room_phase": "COMBAT",
            "current_action": "ModalAction",
            "monsters": [{
                "monster_id": "Cultist",
                "name": "Cultist",
                "index": 0,
                "current_hp": 42,
                "max_hp": 48,
                "block": 0,
                "intent": "BUFF",
                "damage": null,
                "hits": 1,
                "monster_powers": [],
                "can_be_killed": false,
                "is_scaling": false
            }],
            "event_name": "Combat Event",
            "event_choices": ["Continue"]
        }))
        .unwrap();

        let payload = planner_payload(&state, &[], false);
        let scenario = &payload["scenario"];

        assert!(
            scenario.get(context_key).is_some(),
            "{screen_type} lost its modal context"
        );
        assert_eq!(scenario["combat"]["monsters"][0]["id"], "Cultist");
        if screen_type == "GRID" {
            assert_eq!(scenario["grid"]["current_action"], "ModalAction");
            assert!(scenario.get("deck").is_none());
        }
    }
}
