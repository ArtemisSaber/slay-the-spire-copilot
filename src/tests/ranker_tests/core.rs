use super::*;

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
fn rank_keeps_avoids_but_sorts_them_to_bottom() {
    let state: NormalizedState = serde_json::from_value(serde_json::json!({
        "screen_type": "NONE",
        "current_hp": 60,
        "max_hp": 75,
        "energy": 3,
        "block": 5,
        "hand": [
            {
                "id": "Strike_R",
                "name": "Strike",
                "cost": 1,
                "card_type": "ATTACK",
                "uuid": "uuid-strike",
                "description": "Deal 6 damage.",
                "has_target": true,
                "playable": true
            },
            {
                "id": "Limit Break",
                "name": "Limit Break",
                "cost": 1,
                "card_type": "SKILL",
                "uuid": "uuid-limit-break",
                "description": "Double your Strength.",
                "has_target": false,
                "playable": true
            }
        ],
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
    }))
    .expect("should parse");

    let scored = ranker::rank(&state);
    let first_avoid = scored
        .iter()
        .position(|s| s.is_avoid)
        .expect("Limit Break with no Strength should be marked avoid");

    assert!(
        scored[..first_avoid].iter().all(|s| !s.is_avoid),
        "non-avoided legal actions should be before avoided actions"
    );
    assert!(
        scored[first_avoid..].iter().all(|s| s.is_avoid),
        "avoided actions should stay in ranker output, at the bottom"
    );
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
    let json = include_str!("../../ranker/rules.json");
    let _: crate::ranker::rules::RuleSet =
        serde_json::from_str(json).expect("embedded rules.json should be valid");
}

#[test]
fn attack_vs_shifting_scores_effective_block() {
    let state: NormalizedState = serde_json::from_value(serde_json::json!({
        "screen_type": "NONE",
        "current_hp": 50,
        "max_hp": 70,
        "energy": 3,
        "block": 0,
        "hand": [{
            "id": "Strike_R",
            "name": "Strike",
            "cost": 1,
            "card_type": "ATTACK",
            "uuid": "uuid-strike",
            "description": "Deal 12 damage.",
            "has_target": true,
            "playable": true
        }],
        "monsters": [{
            "name": "Transient",
            "monster_id": "Transient",
            "index": 0,
            "current_hp": 999,
            "max_hp": 999,
            "block": 0,
            "intent": "ATTACK",
            "damage": 30,
            "hits": 1,
            "monster_powers": [
                {"id": "Shifting", "name": "Shifting", "amount": -1}
            ],
            "is_scaling": false,
            "can_be_killed": false
        }],
        "incoming_damage": 30,
        "powers": [],
        "relics": [],
        "draw_pile": [],
        "discard_pile": [],
        "potions": [],
        "deck_names": []
    }))
    .expect("should parse");

    let scored = ranker::rank(&state);
    let strike = find_scored(&scored, "Strike");
    let rule = find_breakdown(&strike.breakdown, "core_shifting_attack_block");

    assert!(
        rule.matched,
        "Attack damage vs Shifting should mitigate incoming"
    );
    assert_eq!(
        rule.score, 240,
        "12 effective damage mitigation × 1000 weight / 50 current HP"
    );
}
