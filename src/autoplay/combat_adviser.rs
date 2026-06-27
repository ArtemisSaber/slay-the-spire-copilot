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

fn resolve_target(
    state: &NormalizedState,
    target_index: Option<usize>,
) -> Option<serde_json::Value> {
    let idx = target_index?;
    let monster = state.monsters.iter().find(|m| m.index == idx)?;
    Some(serde_json::json!({
        "index": monster.index,
        "name": monster.name,
        "hp": monster.current_hp,
        "block": monster.block,
    }))
}

fn derive_tags(breakdown: &[crate::ranker::engine::RuleResult]) -> Vec<String> {
    let mut tags = Vec::new();
    for r in breakdown {
        if !r.matched {
            continue;
        }
        let tag = match r.rule_id.as_str() {
            "core_damage" | "core_damage_vulnerable" | "core_damage_intangible" => "damage",
            "core_block_non_excessive" | "core_block_retain" | "core_block_retain_relic" => "block",
            "core_block_excessive" => "excessive",
            "target_killable" => "killable",
            "combat_ends_fight" => "lethal",
            "combat_priority_kill" | "combat_minion_kill" => "priority_kill",
            "core_heal" => "heal",
            "setup_power_card" => "power",
            "core_draw_has_energy" => "draw",
            "potion_base" => "potion",
            _ => continue,
        };
        if !tags.contains(&tag.to_string()) {
            tags.push(tag.to_string());
        }
    }
    tags
}

fn action_entry(
    s: &crate::ranker::engine::ScoredAction,
    state: &NormalizedState,
) -> serde_json::Value {
    let target = resolve_target(state, s.target_index);
    let tags = derive_tags(&s.breakdown);
    serde_json::json!({
        "action": s.action_type,
        "target": target,
        "score": s.score,
        "tags": tags,
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

    let actions: Vec<serde_json::Value> = scored
        .iter()
        .filter(|s| !s.is_avoid)
        .map(|s| action_entry(s, state))
        .collect();

    Some(serde_json::json!({
        "ranked_suggestions": actions
    }))
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

    fn thunderclap_card() -> serde_json::Value {
        serde_json::json!({
            "id": "Thunderclap",
            "name": "Thunderclap",
            "cost": 1,
            "card_type": "ATTACK",
            "uuid": "tc-1",
            "description": "Deal 4 damage and apply 1 Vulnerable to ALL enemies.",
            "has_target": false,
            "playable": true
        })
    }

    fn two_monster_state(energy: i64, cards: Vec<serde_json::Value>) -> NormalizedState {
        let monsters = serde_json::json!([
            {
                "name": "Slime A",
                "index": 0,
                "current_hp": 15,
                "max_hp": 20,
                "block": 0,
                "intent": "ATTACK",
                "damage": 5,
                "hits": 1,
                "monster_powers": [],
                "can_be_killed": false,
                "is_scaling": false
            },
            {
                "name": "Slime B",
                "index": 1,
                "current_hp": 15,
                "max_hp": 20,
                "block": 0,
                "intent": "ATTACK",
                "damage": 5,
                "hits": 1,
                "monster_powers": [],
                "can_be_killed": false,
                "is_scaling": false
            }
        ]);
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
    fn top_ranked_returns_flat_array_with_score_and_tags() {
        let state = combat_state_with_cards(3, vec![strike_card("s1", 1)]);
        let context = top_ranked_context(&state).expect("should produce ranked context");
        let rs = context["ranked_suggestions"]
            .as_array()
            .expect("should be array");
        assert!(!rs.is_empty(), "should have ranked actions");
        let first = &rs[0];
        assert!(first["score"].is_i64(), "should have numeric score");
        assert!(first["tags"].is_array(), "should have tags array");
        assert!(
            first["tags"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("damage")),
            "Strike should have damage tag"
        );
    }

    #[test]
    fn top_ranked_includes_target_resolution() {
        let state = combat_state_with_cards(3, vec![strike_card("s1", 1)]);
        let context = top_ranked_context(&state).expect("should produce ranked context");
        let rs = context["ranked_suggestions"].as_array().unwrap();
        let first = &rs[0];
        let target = &first["target"];
        assert_eq!(target["name"], "Jaw Worm");
        assert_eq!(target["hp"], 20);

        let strike_entries: Vec<_> = rs
            .iter()
            .filter(|e| e["action"]["PlayCard"]["card_name"].as_str() == Some("Strike"))
            .collect();
        assert_eq!(strike_entries.len(), 1);
        assert_eq!(strike_entries[0]["target"]["index"], 0);
    }

    #[test]
    fn top_ranked_excludes_avoided_actions() {
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
        let rs = context["ranked_suggestions"].as_array().unwrap();
        let limit_break_entries: Vec<_> = rs
            .iter()
            .filter(|e| e["action"]["PlayCard"]["card_name"].as_str() == Some("Limit Break"))
            .collect();
        assert!(
            limit_break_entries.is_empty(),
            "Limit Break with 0 Strength should be excluded"
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

    #[test]
    fn aoe_merged_into_single_entry_with_null_target() {
        let state = two_monster_state(3, vec![thunderclap_card()]);
        let context = top_ranked_context(&state).expect("should produce ranked context");
        let rs = context["ranked_suggestions"].as_array().unwrap();
        let aoe_entries: Vec<_> = rs
            .iter()
            .filter(|e| e["action"]["PlayCard"]["card_name"].as_str() == Some("Thunderclap"))
            .collect();
        assert_eq!(
            aoe_entries.len(),
            1,
            "AoE card should produce one merged entry"
        );
        assert!(
            aoe_entries[0]["target"].is_null(),
            "merged AoE entry should have null target"
        );
    }

    #[test]
    fn targeted_keeps_separate_entries_per_monster() {
        let state = two_monster_state(3, vec![strike_card("s1", 1)]);
        let context = top_ranked_context(&state).expect("should produce ranked context");
        let rs = context["ranked_suggestions"].as_array().unwrap();
        let strike_entries: Vec<_> = rs
            .iter()
            .filter(|e| e["action"]["PlayCard"]["card_name"].as_str() == Some("Strike"))
            .collect();
        assert_eq!(
            strike_entries.len(),
            2,
            "targeted Strike vs 2 monsters should produce 2 entries"
        );
        assert_eq!(strike_entries[0]["target"]["index"], 0);
        assert_eq!(strike_entries[1]["target"]["index"], 1);
    }
}
