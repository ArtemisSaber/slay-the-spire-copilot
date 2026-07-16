use super::*;

fn candidate(kind: &str, action_id: &str, label: &str, targeted: bool) -> ActionCandidate {
    ActionCandidate {
        kind: kind.into(),
        action_id: action_id.into(),
        label: label.into(),
        target_required: targeted.then_some(true),
    }
}

#[test]
fn combat_prompt_replaces_localized_prose_with_complete_structured_context() {
    let secret_uuid = "private-strike-instance";
    let state: NormalizedState = serde_json::from_value(json!({
        "screen_type": "NONE",
        "room_type": "MonsterRoomElite",
        "character": "IRONCLAD",
        "ascension_level": 20,
        "floor": 8,
        "current_hp": 31,
        "max_hp": 80,
        "gold": 155,
        "energy": 2,
        "block": 4,
        "turn_number": 3,
        "stance": "Wrath",
        "incoming_damage": 18,
        "danger": {
            "hp_critical": false,
            "incoming_lethal": false,
            "no_block_against_hit": false,
            "any_monster_attacking": true,
            "wrath_stance": true,
            "level": "Caution"
        },
        "powers": [{"id":"Strength","name":"Strength","amount":2}],
        "orbs": [{"id":"Frost","amount":5}],
        "relics": [{
            "id":"Pen Nib","name":"Pen Nib","description":"Every 10th Attack deals double damage.",
            "counter":9
        }],
        "potions": [{
            "id":"Fire Potion","slot":1,"name":"Fire Potion","description":"Deal 20 damage.",
            "can_use":true,"can_discard":true,"requires_target":true
        }],
        "hand": [{
            "id":"Strike_R","name":"Strike","cost":0,"card_type":"ATTACK","upgraded":true,
            "uuid":secret_uuid,"description":"Deal 9 damage.","playable":true,"has_target":true
        }],
        "monsters": [{
            "monster_id":"GremlinNob","name":"Gremlin Nob","index":2,
            "current_hp":21,"max_hp":90,"block":3,"intent":"ATTACK",
            "damage":6,"hits":3,
            "monster_powers":[{"id":"Enrage","name":"Enrage","amount":2}],
            "can_be_killed":true,"is_scaling":true
        }],
        "draw_pile": [{
            "id":"Defend_R","name":"Defend","cost":1,"card_type":"SKILL","upgraded":false,
            "uuid":"draw-private","description":"Gain 5 Block.","playable":true,"has_target":false
        }],
        "discard_pile": [{
            "id":"Bash","name":"Bash","cost":2,"card_type":"ATTACK","upgraded":false,
            "uuid":"discard-private","description":"Deal 8 damage. Apply 2 Vulnerable.",
            "playable":true,"has_target":true
        }],
        "exhaust_cards": []
    }))
    .unwrap();
    let candidates = vec![
        candidate(
            "play",
            &format!("combat:play:{secret_uuid}"),
            "Play Strike+",
            true,
        ),
        candidate("drink", "combat:potion:1", "Drink Fire Potion", true),
        candidate("end", "combat:end", "End turn", false),
    ];

    let payload = planner_payload(&state, &candidates, false);
    let scenario = &payload["scenario"];
    let combat = &scenario["combat"];
    let action = &payload["available_actions"][0];

    assert!(payload.get("localized_status_context").is_none());
    assert!(payload.get("state").is_none());
    assert_eq!(scenario["kind"], "combat");
    assert_eq!(scenario["run"]["character"], "IRONCLAD");
    assert_eq!(scenario["run"]["ascension_level"], 20);
    assert_eq!(scenario["player"]["current_hp"], 31);
    assert_eq!(scenario["player"]["powers"][0]["id"], "Strength");
    assert_eq!(scenario["inventory"]["relics"][0]["counter"], 9);
    assert_eq!(scenario["threat"]["incoming_damage"], 18);
    assert_eq!(scenario["threat"]["incoming_lethal"], false);
    assert_eq!(combat["turn"], 3);
    assert_eq!(combat["hand"][0]["description"], "Deal 9 damage.");
    assert_eq!(combat["hand"][0]["upgraded"], true);
    assert_eq!(combat["monsters"][0]["damage_per_hit"], 6);
    assert_eq!(combat["monsters"][0]["hits"], 3);
    assert_eq!(combat["monsters"][0]["total_damage"], 18);
    assert_eq!(combat["monsters"][0]["powers"][0]["id"], "Enrage");
    assert_eq!(combat["monsters"][0]["is_scaling"], true);
    assert_eq!(combat["piles"]["draw"][0]["id"], "Defend_R");
    assert_eq!(combat["piles"]["discard"][0]["id"], "Bash");
    assert_eq!(action["ref"], "A0");
    assert_eq!(action["hand_index"], 0);
    assert_eq!(action["card"]["id"], "Strike_R");
    assert_eq!(action["card"]["description"], "Deal 9 damage.");
    assert!(!payload.to_string().contains(secret_uuid));
    assert!(!payload.to_string().to_ascii_lowercase().contains("uuid"));
}

#[test]
fn combat_action_sources_link_potions_without_exposing_slots_as_action_ids() {
    let state: NormalizedState = serde_json::from_value(json!({
        "screen_type":"NONE",
        "potions":[{
            "id":"Weak Potion","slot":2,"name":"Weak Potion","description":"Apply 3 Weak.",
            "can_use":true,"can_discard":true,"requires_target":true
        }],
        "monsters":[{
            "monster_id":"Cultist","name":"Cultist","index":4,"current_hp":40,"max_hp":48,
            "block":0,"intent":"ATTACK","damage":6,"hits":1,"monster_powers":[],
            "can_be_killed":false,"is_scaling":true
        }]
    }))
    .unwrap();
    let candidates = vec![candidate(
        "drink",
        "combat:potion:2",
        "Drink Weak Potion",
        true,
    )];

    let payload = planner_payload(&state, &candidates, false);
    let action = &payload["available_actions"][0];

    assert_eq!(action["potion_slot"], 2);
    assert_eq!(action["potion"]["id"], "Weak Potion");
    assert_eq!(action["potion"]["description"], "Apply 3 Weak.");
    assert!(action.get("action_id").is_none());
}
