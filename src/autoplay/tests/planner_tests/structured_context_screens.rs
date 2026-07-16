use super::*;

fn candidate(kind: &str, action_id: &str, label: &str) -> ActionCandidate {
    ActionCandidate {
        kind: kind.into(),
        action_id: action_id.into(),
        label: label.into(),
        target_required: None,
    }
}

#[test]
fn reward_prompt_structures_choices_deck_and_action_sources() {
    let state: NormalizedState = serde_json::from_value(json!({
        "screen_type":"CARD_REWARD",
        "floor":14,
        "skip_available":true,
        "card_reward_choices":[{
            "id":"Uppercut","name":"Uppercut","cost":2,"card_type":"ATTACK","upgraded":false,
            "description":"Deal 13 damage. Apply 1 Weak and 1 Vulnerable.",
            "playable":false,"has_target":true
        }],
        "master_cards":[{
            "id":"Bash","name":"Bash","cost":2,"card_type":"ATTACK","upgraded":true,
            "description":"Deal 10 damage. Apply 3 Vulnerable.",
            "playable":false,"has_target":true
        }]
    }))
    .unwrap();
    let candidates = vec![
        candidate("choose", "card_reward:0", "Uppercut"),
        candidate("skip", "card_reward:skip", "Skip"),
    ];

    let payload = planner_payload(&state, &candidates, false);
    let scenario = &payload["scenario"];

    assert_eq!(scenario["kind"], "card_reward");
    assert_eq!(
        scenario["card_reward"]["choices"][0]["description"],
        "Deal 13 damage. Apply 1 Weak and 1 Vulnerable."
    );
    assert_eq!(scenario["card_reward"]["skip_available"], true);
    assert_eq!(scenario["card_reward"]["next_act_full_heal"], false);
    assert_eq!(scenario["card_reward"]["deck_size_before_pick"], 1);
    assert_eq!(
        scenario["card_reward"]["selection_policy"]["baseline"],
        "skip"
    );
    assert_eq!(
        scenario["card_reward"]["selection_policy"]["take_card_only_if"],
        "meaningful_net_improvement"
    );
    assert_eq!(
        scenario["card_reward"]["selection_policy"]["positive_synergy_alone_is_sufficient"],
        false
    );
    assert_eq!(scenario["deck"][0]["id"], "Bash");
    assert_eq!(scenario["deck"][0]["upgraded"], true);
    assert_eq!(
        payload["available_actions"][0]["card"]["description"],
        "Deal 13 damage. Apply 1 Weak and 1 Vulnerable."
    );
    assert_eq!(payload["available_actions"][0]["choice_index"], 0);
    assert_eq!(payload["available_actions"][1]["baseline"], true);
}

#[test]
fn boss_card_reward_context_scopes_full_heal_to_boss_rewards() {
    let state: NormalizedState = serde_json::from_value(json!({
        "screen_type":"CARD_REWARD",
        "floor":16,
        "skip_available":true,
        "card_reward_choices":[{
            "id":"Demon Form","name":"Demon Form","cost":3,"card_type":"POWER","upgraded":false,
            "description":"At the start of each turn, gain 2 Strength.",
            "playable":false,"has_target":false
        }],
        "master_cards":[{
            "id":"Bash","name":"Bash","cost":2,"card_type":"ATTACK","upgraded":true,
            "description":"Deal 10 damage. Apply 3 Vulnerable.",
            "playable":false,"has_target":true
        }]
    }))
    .unwrap();
    let candidates = vec![
        candidate("choose", "boss_card_reward:0", "Demon Form"),
        candidate("skip", "boss_card_reward:skip", "Skip"),
    ];

    let payload = planner_payload(&state, &candidates, false);

    assert_eq!(payload["scenario"]["kind"], "boss_card_reward");
    assert_eq!(
        payload["scenario"]["card_reward"]["next_act_full_heal"],
        true
    );
    assert_eq!(
        payload["scenario"]["card_reward"]["selection_policy"]["baseline"],
        "skip"
    );
    assert_eq!(payload["available_actions"][1]["baseline"], true);
}

#[test]
fn shop_and_event_prompts_preserve_descriptions_prices_and_event_text() {
    let shop: NormalizedState = serde_json::from_value(json!({
        "screen_type":"SHOP_SCREEN",
        "gold":180,
        "purge_available":true,
        "purge_cost":75,
        "shop_cards":[{
            "id":"Armaments","name":"Armaments","cost":1,"card_type":"SKILL","upgraded":false,
            "description":"Gain 5 Block. Upgrade a card in your hand.","price":51,
            "playable":false,"has_target":false
        }],
        "shop_relics":[{
            "id":"Shovel","name":"Shovel","description":"You can Dig for relics at Rest Sites.",
            "counter":null,"price":286
        }],
        "shop_potions":[{
            "id":"Regen Potion","slot":0,"name":"Regen Potion","description":"Gain 5 Regeneration.",
            "price":79,"can_use":false,"can_discard":true,"requires_target":false
        }],
        "master_cards":[{
            "id":"Strike_R","name":"Strike","cost":1,"card_type":"ATTACK","upgraded":false,
            "description":"Deal 6 damage.","playable":false,"has_target":true
        }]
    }))
    .unwrap();
    let shop_payload = planner_payload(
        &shop,
        &[candidate("choose", "shop:choice:0", "Buy Armaments")],
        true,
    );

    assert_eq!(shop_payload["scenario"]["kind"], "shop");
    assert_eq!(shop_payload["scenario"]["shop"]["cards"][0]["price"], 51);
    assert_eq!(
        shop_payload["scenario"]["shop"]["relics"][0]["description"],
        "You can Dig for relics at Rest Sites."
    );
    assert_eq!(shop_payload["scenario"]["shop"]["purge_cost"], 75);
    assert_eq!(shop_payload["scenario"]["shop"]["visited"], true);
    assert_eq!(
        shop_payload["scenario"]["deck"][0]["description"],
        "Deal 6 damage."
    );

    let event: NormalizedState = serde_json::from_value(json!({
        "screen_type":"EVENT",
        "event_id":"Golden Idol",
        "event_name":"Golden Idol",
        "event_body":"You find a golden idol on a pedestal.",
        "event_choices":["Take it","Leave"]
    }))
    .unwrap();
    let event_payload =
        planner_payload(&event, &[candidate("choose", "event:0", "Take it")], false);

    assert_eq!(event_payload["scenario"]["kind"], "event");
    assert_eq!(event_payload["scenario"]["event"]["id"], "Golden Idol");
    assert_eq!(
        event_payload["scenario"]["event"]["body"],
        "You find a golden idol on a pedestal."
    );
    assert_eq!(event_payload["scenario"]["event"]["choices"][0], "Take it");
    assert_eq!(event_payload["available_actions"][0]["choice_index"], 0);
}

#[test]
fn selection_prompt_structures_purpose_limits_and_card_descriptions() {
    let state: NormalizedState = serde_json::from_value(json!({
        "screen_type":"HAND_SELECT",
        "current_action":"ExhaustAction",
        "hand_select_max_cards":1,
        "hand_select_can_pick_zero":false,
        "card_in_play":{
            "id":"True Grit","name":"True Grit","cost":1,"card_type":"SKILL","upgraded":true,
            "description":"Gain 9 Block. Exhaust a card.","playable":true,"has_target":false
        },
        "hand":[{
            "id":"Burn","name":"Burn","cost":-2,"card_type":"STATUS","upgraded":false,
            "uuid":"private-burn","description":"At end of turn, take 2 damage.",
            "playable":false,"has_target":false
        }],
        "hand_select_selected":[]
    }))
    .unwrap();
    let payload = planner_payload(
        &state,
        &[candidate("choose", "hand_select:0", "Burn")],
        false,
    );

    assert_eq!(payload["scenario"]["kind"], "hand_select");
    assert_eq!(
        payload["scenario"]["hand_select"]["current_action"],
        "ExhaustAction"
    );
    assert_eq!(payload["scenario"]["hand_select"]["max_cards"], 1);
    assert_eq!(
        payload["scenario"]["hand_select"]["card_in_play"]["description"],
        "Gain 9 Block. Exhaust a card."
    );
    assert_eq!(
        payload["scenario"]["hand_select"]["available_cards"][0]["description"],
        "At end of turn, take 2 damage."
    );
    assert_eq!(payload["available_actions"][0]["card"]["id"], "Burn");
    assert!(!payload.to_string().contains("private-burn"));
}

#[test]
fn map_prompt_includes_graph_and_scored_route_options() {
    let state: NormalizedState = serde_json::from_value(json!({
        "screen_type":"MAP",
        "floor":8,
        "current_hp":35,
        "max_hp":80,
        "gold":190,
        "map_first_node_chosen":true,
        "map_current_x":0,
        "map_current_y":0,
        "map_nodes":[
            {"symbol":"M","x":0,"y":0,"children":[[1,1],[2,1]]},
            {"symbol":"E","x":1,"y":1,"children":[[1,2]]},
            {"symbol":"$","x":2,"y":1,"children":[[2,2]]},
            {"symbol":"R","x":1,"y":2,"children":[]},
            {"symbol":"?","x":2,"y":2,"children":[]}
        ]
    }))
    .unwrap();
    let candidates = vec![
        candidate("choose", "map:choice:0", "x=1"),
        candidate("choose", "map:choice:1", "x=2"),
    ];

    let payload = planner_payload(&state, &candidates, false);
    let map = &payload["scenario"]["map"];
    let routes = map["route_options"].as_array().unwrap();

    assert_eq!(payload["scenario"]["kind"], "map_crossroad");
    assert_eq!(map["current"]["x"], 0);
    assert_eq!(map["nodes"].as_array().unwrap().len(), 5);
    assert!(!routes.is_empty());
    assert!(routes[0]["score"].is_number());
    assert!(routes[0]["counts"]["elites"].is_number());
    assert!(routes[0]["nodes"].is_array());
    assert_eq!(payload["available_actions"][0]["choice_index"], 0);
}
