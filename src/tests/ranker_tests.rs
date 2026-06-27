use crate::ranker;
use crate::ranker::context::ActionType;
use crate::state::NormalizedState;
use crate::test_utils;

fn comm_mod_state(filename: &str) -> NormalizedState {
    let raw = test_utils::load_fixture(filename);
    let locale = crate::locales::Locale::load("zh");
    NormalizedState::from_raw(&raw, &locale)
}

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
    let json = include_str!("../ranker/rules.json");
    let _: crate::ranker::rules::RuleSet =
        serde_json::from_str(json).expect("embedded rules.json should be valid");
}

// --- Integration tests from CommunicationMod raw log fixtures ---

fn find_breakdown<'a>(
    breakdown: &'a [crate::ranker::engine::RuleResult],
    rule_id: &str,
) -> &'a crate::ranker::engine::RuleResult {
    breakdown
        .iter()
        .find(|b| b.rule_id == rule_id)
        .unwrap_or_else(|| panic!("rule {rule_id} not in breakdown"))
}

fn find_scored<'a>(
    scored: &'a [crate::ranker::engine::ScoredAction],
    card_name: &str,
) -> &'a crate::ranker::engine::ScoredAction {
    scored
        .iter()
        .find(|s| match &s.action_type {
            ActionType::PlayCard { card_name: cn, .. } => cn == card_name,
            _ => false,
        })
        .unwrap_or_else(|| panic!("card {card_name} not in scored actions"))
}

#[test]
fn defend_does_not_end_fight() {
    let state = comm_mod_state("comm-f16t15-defend-strike-hexaghost.json");
    let scored = ranker::rank(&state);
    let defend = find_scored(&scored, "防御");
    let rule = find_breakdown(&defend.breakdown, "combat_ends_fight");
    assert!(!rule.matched, "Defend should NOT end the fight");
    assert_eq!(rule.score, 0);
}

#[test]
fn strike_can_end_fight() {
    let state = comm_mod_state("comm-f1t1-strikes-vs-slimes.json");
    let scored = ranker::rank(&state);
    let strike = find_scored(&scored, "打击");
    let rule = find_breakdown(&strike.breakdown, "combat_ends_fight");
    assert!(!rule.matched, "Strike 6dmg vs 10HP should NOT end fight");
}

#[test]
fn defend_no_str_gain_false_positive() {
    let state = comm_mod_state("comm-f16t16-hexaghost-turn.json");
    let scored = ranker::rank(&state);
    let defend = find_scored(&scored, "防御");
    let rule = find_breakdown(&defend.breakdown, "setup_self_str_gain");
    assert!(!rule.matched, "Defend should NOT give Strength gain");
    assert_eq!(rule.score, 0);
}

#[test]
fn defend_no_dex_gain_false_positive() {
    let state = comm_mod_state("comm-f16t16-hexaghost-turn.json");
    let scored = ranker::rank(&state);
    let defend = find_scored(&scored, "防御");
    let rule = find_breakdown(&defend.breakdown, "setup_self_dex_gain");
    assert!(!rule.matched, "Defend should NOT give Dexterity gain");
    assert_eq!(rule.score, 0);
}

#[test]
fn bash_scores_vulnerable_apply() {
    let state = comm_mod_state("comm-f16t16-hexaghost-turn.json");
    let scored = ranker::rank(&state);
    let bash = find_scored(&scored, "痛击");
    let rule = find_breakdown(&bash.breakdown, "setup_vulnerable_apply");
    assert!(rule.matched, "Bash should apply Vulnerable");
    assert_eq!(rule.score, 20, "2 stacks × 10 weight = 20");
}

#[test]
fn body_slam_with_zero_block_does_not_end_fight() {
    let state = comm_mod_state("comm-f16t16-bodyslam-vs-hexaghost.json");
    let scored = ranker::rank(&state);
    let slam = find_scored(&scored, "全身撞击");
    let rule = find_breakdown(&slam.breakdown, "combat_ends_fight");
    assert!(
        !rule.matched,
        "Body Slam with 0 block vs 41 HP should NOT end fight"
    );
    assert_eq!(rule.score, 0);
}

#[test]
fn strike_scores_damage() {
    let state = comm_mod_state("comm-f16t15-defend-strike-hexaghost.json");
    let scored = ranker::rank(&state);
    let strike = find_scored(&scored, "打击");
    let rule = find_breakdown(&strike.breakdown, "core_damage");
    assert!(rule.matched, "Strike should deal damage");
    assert_eq!(
        rule.score, 109,
        "6 dmg × 1 hit × 1000 weight / 55 monster hp = 109"
    );
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

#[test]
fn defend_scores_block() {
    let state = comm_mod_state("comm-f16t16-hexaghost-turn.json");
    let scored = ranker::rank(&state);
    let defend = find_scored(&scored, "防御");
    let rule = find_breakdown(&defend.breakdown, "core_block_non_excessive");
    assert!(rule.matched, "Defend should give block");
    let b = defend
        .breakdown
        .iter()
        .find(|b| b.rule_id == "core_block_non_excessive")
        .unwrap();
    assert!(b.matched);
    assert!(b.score > 0, "Block should have positive score");
}

#[test]
fn potion_slot_preserves_raw_index_with_gaps() {
    let state = comm_mod_state("comm-gapped-potions.json");
    let potions = &state.potions;
    assert_eq!(potions.len(), 2, "should have 2 usable potions");

    let energy = potions.iter().find(|p| p.name == "能量药水").unwrap();
    assert_eq!(energy.slot, 1, "能量药水 should be at raw index 1");
    let block = potions.iter().find(|p| p.name == "格挡药水").unwrap();
    assert_eq!(block.slot, 2, "格挡药水 should be at raw index 2");
}

#[test]
fn potion_slot_not_zero_when_first_slot_empty() {
    let state = comm_mod_state("comm-gapped-potions.json");
    let energy = state.potions.iter().find(|p| p.name == "能量药水").unwrap();
    assert_eq!(
        energy.slot, 1,
        "energy potion slot should be 1, not 0 (index 0 is empty Potion Slot)"
    );
    let block = state.potions.iter().find(|p| p.name == "格挡药水").unwrap();
    assert_eq!(block.slot, 2, "block potion slot should be 2");
}

#[test]
fn empty_potion_slots_counted_correctly() {
    let state = comm_mod_state("comm-gapped-potions.json");
    assert_eq!(
        state.empty_potion_slots, 1,
        "one slot (index 0) should be counted as empty"
    );
}
