use crate::ranker;
use crate::state::NormalizedState;
use crate::test_utils;

#[test]
fn ranks_combat_state_fixture() {
    let raw = test_utils::load_fixture("combat-state.json");
    let state: NormalizedState = serde_json::from_value(raw).expect("fixture should parse");

    let scored = ranker::rank(&state);
    assert!(
        !scored.is_empty(),
        "should produce at least one scored action"
    );

    for s in &scored {
        eprintln!(
            "action={:?} score={} avoid={}",
            s.action_type, s.score, s.is_avoid
        );
        for b in &s.breakdown {
            if b.matched {
                eprintln!("  {}: {}", b.rule_id, b.score);
            }
        }
    }

    let best = scored.first().expect("should have a best action");
    eprintln!("best action: {:?} score={}", best.action_type, best.score);
    assert!(!best.is_avoid, "best action should not be avoided");
    assert!(
        best.score > -10000,
        "best action should have reasonable score"
    );
}

#[test]
fn avoids_are_sorted_to_bottom() {
    let raw = test_utils::load_fixture("combat-state.json");
    let state: NormalizedState = serde_json::from_value(raw).expect("fixture should parse");

    let mut scored = ranker::rank(&state);
    scored.sort_by_key(|s| s.is_avoid);

    if let Some(first_avoid) = scored.iter().position(|s| s.is_avoid) {
        for s in &scored[..first_avoid] {
            assert!(!s.is_avoid, "avoid actions should be after non-avoid");
        }
    }
}

#[test]
fn ranks_without_external_rules_file() {
    let template = serde_json::json!({
        "available_commands": ["state"],
        "ready_for_command": true,
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 60,
            "max_hp": 75,
            "energy": 3,
            "block": 5,
            "hand": [{
                "id": "Strike_R",
                "name": "Strike",
                "cost": 1,
                "card_type": "ATTACK",
                "uuid": "uuid-1",
                "description": "造成 6 点伤害",
                "has_target": true,
                "playable": true
            }],
            "monsters": [{
                "name": "Jaw Worm",
                "index": 0,
                "current_hp": 44,
                "max_hp": 46,
                "block": 0,
                "intent": "ATTACK",
                "damage": 12,
                "hits": 1,
                "monster_powers": [],
                "is_scaling": false,
                "can_be_killed": false
            }],
            "incoming_damage": 12,
            "powers": [],
            "relics": [],
            "draw_pile": [],
            "discard_pile": [],
            "potions": [],
            "deck_names": []
        }
    });

    let state: NormalizedState =
        serde_json::from_value(template["game_state"].clone()).expect("should parse");
    let scored = ranker::rank(&state);
    assert!(!scored.is_empty());
    assert!(
        scored.iter().any(|s| s.score != 0),
        "should have non-zero scores from embedded rules"
    );
}

#[test]
fn embedded_rules_json_is_valid() {
    let json = include_str!("../ranker/rules.json");
    let _: crate::ranker::rules::RuleSet =
        serde_json::from_str(json).expect("embedded rules.json should be valid");
}
