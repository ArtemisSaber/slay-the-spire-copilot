use super::*;
use crate::autoplay::action::ActionCandidate;
use crate::state::{MonsterInfo, PotionInfo, ScreenType};

fn duplicate_card_fixture() -> (
    AutoPlayControl,
    CommandState,
    NormalizedState,
    Vec<ActionCandidate>,
) {
    let raw = json!({
        "available_commands": ["play", "end"],
        "ready_for_command": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "player": {"energy": 3, "block": 0, "powers": []},
                "hand": [
                    {"id":"Strike_R","name":"Strike","cost":1,"type":"ATTACK","uuid":"first","has_target":true,"is_playable":true},
                    {"id":"Strike_R","name":"Strike","cost":1,"type":"ATTACK","uuid":"second","has_target":true,"is_playable":true}
                ],
                "monsters": [{
                    "id":"JawWorm","name":"Jaw Worm","current_hp":40,"max_hp":40,
                    "block":0,"intent":"ATTACK","is_gone":false
                }]
            }
        }
    });
    let control = AutoPlayControl::default_enabled();
    let command = command_state(&raw);
    let normalized = state(raw);
    let candidates =
        available_action_candidates(&control, &AutoPlaySession::default(), &command, &normalized);
    (control, command, normalized, candidates)
}

#[test]
fn reference_selects_the_exact_duplicate_card_candidate() {
    let (control, command, state, candidates) = duplicate_card_fixture();

    let action = parse_planner_response(
        r#"{"schema_version":2,"actions":[{"ref":"A1","target_index":0}]}"#,
        &control,
        &command,
        &state,
        &candidates,
    )
    .unwrap();

    assert_eq!(
        action,
        Some(AutoPlayAction::Play {
            hand_index: 1,
            target_index: Some(0)
        })
    );
}

#[test]
fn target_presence_must_match_the_referenced_candidate() {
    let (control, command, state, candidates) = duplicate_card_fixture();

    let missing = parse_planner_response(
        r#"{"schema_version":2,"actions":[{"ref":"A0"}]}"#,
        &control,
        &command,
        &state,
        &candidates,
    )
    .unwrap_err();
    assert!(missing.to_string().contains("omitted target_index"));

    let unexpected = parse_planner_response(
        r#"{"schema_version":2,"actions":[{"ref":"A2","target_index":0}]}"#,
        &control,
        &command,
        &state,
        &candidates,
    )
    .unwrap_err();
    assert!(unexpected.to_string().contains("supplied target_index"));
}

#[test]
fn reference_resolves_potion_kind_without_model_serialization() {
    let raw = json!({
        "available_commands": ["potion", "end"],
        "ready_for_command": true,
        "game_state": {"screen_type":"NONE","combat_state":{"player":{"energy":0},"hand":[],"monsters":[]}}
    });
    let command = command_state(&raw);
    let state = NormalizedState {
        screen_type: Some(ScreenType::None),
        potions: vec![PotionInfo {
            id: Some("Fire Potion".into()),
            slot: 2,
            name: "Fire Potion".into(),
            description: "Deal 20 damage.".into(),
            price: None,
            can_use: true,
            can_discard: true,
            requires_target: true,
        }],
        monsters: vec![MonsterInfo {
            name: "Jaw Worm".into(),
            index: 4,
            current_hp: Some(20),
            max_hp: Some(40),
            block: Some(0),
            intent: Some("ATTACK".into()),
            damage: Some(8),
            hits: Some(1),
            monster_powers: vec![],
            can_be_killed: false,
            is_scaling: false,
            monster_id: Some("JawWorm".into()),
        }],
        ..NormalizedState::default()
    };
    let control = AutoPlayControl::default_enabled();
    let candidates =
        available_action_candidates(&control, &AutoPlaySession::default(), &command, &state);

    let action = parse_planner_response(
        r#"{"schema_version":2,"actions":[{"ref":"A0","target_index":4}]}"#,
        &control,
        &command,
        &state,
        &candidates,
    )
    .unwrap();

    assert_eq!(
        action,
        Some(AutoPlayAction::Drink {
            slot_index: 2,
            target_index: Some(4)
        })
    );
}
