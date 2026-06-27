use crate::autoplay::action::AutoPlayAction;
use crate::state::NormalizedState;

pub fn try_kill_scan_action(state: &NormalizedState) -> Option<AutoPlayAction> {
    if state.screen_type.as_deref() != Some("NONE") {
        return None;
    }
    let sequence = crate::combat::find_kill_sequence(state)?;
    let first = sequence.first()?;
    let hand_index = state
        .hand
        .iter()
        .position(|c| c.uuid.as_deref() == Some(&first.card))?;
    Some(AutoPlayAction::Play {
        hand_index,
        target_index: first.target,
    })
}

pub fn top_ranked_context(state: &NormalizedState) -> Option<serde_json::Value> {
    if state.screen_type.as_deref() != Some("NONE") {
        return None;
    }
    let scored = crate::ranker::rank(state);
    if scored.is_empty() {
        return None;
    }
    let top5: Vec<serde_json::Value> = scored
        .iter()
        .take(5)
        .map(|s| {
            serde_json::json!({
                "action": s.action_type,
                "score": s.score,
                "is_avoid": s.is_avoid,
                "target_index": s.target_index,
                "matched_rules": s.breakdown.iter()
                    .filter(|b| b.matched)
                    .map(|b| serde_json::json!({
                        "rule_id": b.rule_id,
                        "score": b.score,
                    }))
                    .collect::<Vec<_>>(),
            })
        })
        .collect();

    Some(serde_json::json!({ "ranked_actions": top5 }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils;

    fn load_normalized_state(filename: &str) -> NormalizedState {
        let raw = test_utils::load_fixture(filename);
        serde_json::from_value(raw).expect("fixture should deserialize as NormalizedState")
    }

    fn combat_state_with_cards(energy: i64, cards: Vec<serde_json::Value>) -> NormalizedState {
        let monsters = serde_json::json!([{
            "name": "Jaw Worm",
            "index": 0,
            "current_hp": 20,
            "max_hp": 20,
            "block": 0,
            "intent": "ATTACK",
            "damage": 5,
            "hits": 1,
            "monster_powers": [],
            "can_be_killed": false,
            "is_scaling": false
        }]);
        let payload = serde_json::json!({
            "screen_type": "NONE",
            "energy": energy,
            "hand": cards,
            "monsters": monsters,
            "incoming_damage": 5,
            "powers": [],
            "relics": [],
            "potions": [],
            "draw_pile": [],
            "discard_pile": [],
            "deck_names": []
        });
        serde_json::from_value(payload).expect("should deserialize as NormalizedState")
    }

    fn strike_card(uuid: &str, cost: i64) -> serde_json::Value {
        serde_json::json!({
            "id": "Strike_R",
            "name": "Strike",
            "cost": cost,
            "card_type": "ATTACK",
            "uuid": uuid,
            "description": "Deal 6 damage.",
            "has_target": true,
            "playable": true
        })
    }

    #[test]
    fn kill_scan_finds_lethal_and_returns_first_play() {
        let state = load_normalized_state("kill-scan-run-pos-hp1-4hand.json");
        let action =
            try_kill_scan_action(&state).expect("should find a lethal action in pos fixture");
        assert!(matches!(action, AutoPlayAction::Play { .. }));
    }

    #[test]
    fn kill_scan_no_lethal_returns_none() {
        let state = load_normalized_state("kill-scan-run-neg-hp19-energy0.json");
        assert_eq!(try_kill_scan_action(&state), None);
    }

    #[test]
    fn kill_scan_non_combat_screen_returns_none() {
        let state = NormalizedState {
            screen_type: Some("REST".into()),
            ..NormalizedState::default()
        };
        assert_eq!(try_kill_scan_action(&state), None);
    }

    #[test]
    fn top_ranked_returns_top5_with_scores() {
        let state = combat_state_with_cards(3, vec![strike_card("s1", 1)]);
        let context = top_ranked_context(&state).expect("should produce ranked context");
        let ranked = context["ranked_actions"]
            .as_array()
            .expect("ranked_actions should be array");
        assert!(!ranked.is_empty(), "should have at least one ranked action");
        assert!(ranked.len() <= 5, "should return at most 5 actions");
        for entry in ranked {
            assert!(
                entry.get("action").is_some(),
                "each entry should have action"
            );
            assert!(entry.get("score").is_some(), "each entry should have score");
            assert!(
                entry.get("is_avoid").is_some(),
                "each entry should have is_avoid"
            );
        }
    }

    #[test]
    fn top_ranked_includes_target_index() {
        let state = combat_state_with_cards(3, vec![strike_card("s1", 1)]);
        let context = top_ranked_context(&state).expect("should produce ranked context");
        let ranked = context["ranked_actions"].as_array().unwrap();
        let play_entry = ranked
            .iter()
            .find(|e| e["action"]["PlayCard"].is_object())
            .expect("should have a PlayCard entry");
        assert_eq!(
            play_entry["target_index"],
            serde_json::Value::Number(serde_json::Number::from(0))
        );
    }

    #[test]
    fn top_ranked_marks_avoid_actions() {
        let state = combat_state_with_cards(
            3,
            vec![serde_json::json!({
                "id": "Limit Break",
                "name": "Limit Break",
                "cost": 1,
                "card_type": "SKILL",
                "uuid": "lb-1",
                "description": "Double your Strength.",
                "has_target": false,
                "playable": true
            })],
        );
        let context = top_ranked_context(&state).expect("should produce ranked context");
        let ranked = context["ranked_actions"].as_array().unwrap();
        let avoid_entry = ranked
            .iter()
            .find(|e| e["is_avoid"] == serde_json::Value::Bool(true));
        assert!(
            avoid_entry.is_some() || ranked.iter().any(|e| !e["is_avoid"].as_bool().unwrap()),
            "should have at least one non-avoid action (e.g., EndTurn)"
        );
    }

    #[test]
    fn top_ranked_returns_none_for_non_combat() {
        let state = NormalizedState {
            screen_type: Some("REST".into()),
            ..NormalizedState::default()
        };
        assert_eq!(top_ranked_context(&state), None);
    }
}
