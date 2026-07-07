use super::*;
use crate::state::{DangerFlags, RelicInfo};
use crate::test_utils::{load_fixture, test_locale};
use serde_json::{Value, json};

#[test]
fn normalize_combat_state() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(state.screen_type.as_deref(), Some("NONE"));
    assert_eq!(state.character.as_deref(), Some("IRONCLAD"));
    assert_eq!(state.seed, Some(-3047511808784702860));
    assert_eq!(state.ascension_level, Some(20));
    assert_eq!(state.floor, Some(1));
    assert_eq!(state.current_hp, Some(68));
    assert_eq!(state.max_hp, Some(75));
    assert_eq!(state.gold, Some(99));
    assert_eq!(state.energy, Some(3));
    assert_eq!(state.block, Some(6));

    assert_eq!(state.hand.len(), 3);
    assert_eq!(state.monsters.len(), 1);
    assert_eq!(state.relics.len(), 2);
    assert_eq!(state.potions.len(), 1);
    assert!(state.card_reward_choices.is_empty());

    assert_eq!(state.incoming_damage, 12);
    assert_eq!(state.danger.level, DangerLevel::Caution);
    assert!(state.danger.any_monster_attacking);

    let jaw_worm = &state.monsters[0];
    assert_eq!(jaw_worm.name, "Jaw Worm");
    assert_eq!(jaw_worm.index, 0);
    assert_eq!(jaw_worm.intent.as_deref(), Some("ATTACK"));
    assert!(jaw_worm.is_scaling);

    assert_eq!(state.hand_cards.len(), 3);
    assert_eq!(state.draw_pile.len(), 2);
    assert_eq!(state.discard_pile.len(), 0);
    assert_eq!(state.exhaust_cards.len(), 0);
    assert_eq!(state.master_cards.len(), 1);
}

#[test]
fn normalize_card_reward_state() {
    let raw = load_fixture("card-reward-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(state.screen_type.as_deref(), Some("CARD_REWARD"));
    assert_eq!(state.card_reward_choices.len(), 3);
    assert!(state.card_reward_choices.iter().any(|c| c.id == "Uppercut"));
    assert!(state.card_reward_choices.iter().any(|c| c.id == "Anger"));
    assert!(state.card_reward_choices.iter().any(|c| c.id == "Headbutt"));

    assert!(state.hand.is_empty());
    assert!(state.monsters.is_empty());
    assert_eq!(state.incoming_damage, 0);
    assert_eq!(state.danger.level, DangerLevel::Safe);

    assert_eq!(state.master_cards.len(), 1);
    assert!(state.skip_available);
}

#[test]
fn normalize_rest_state() {
    let raw = load_fixture("rest-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(state.screen_type.as_deref(), Some("REST"));
    assert_eq!(
        state.rest_options,
        vec!["rest".to_string(), "smith".to_string(), "toke".to_string()]
    );
    assert_eq!(state.current_hp, Some(25));
    assert_eq!(state.max_hp, Some(75));
    assert_eq!(state.floor, Some(5));
    assert_eq!(state.gold, Some(120));

    assert_eq!(state.master_cards.len(), 9);
    assert!(state.potions.is_empty());
    assert!(state.monsters.is_empty());
    assert!(state.hand_cards.is_empty());
    assert_eq!(state.danger.level, DangerLevel::Caution);
}

#[test]
fn normalize_boss_relic_choices() {
    let raw = serde_json::json!({
        "in_game": true,
        "game_state": {
            "screen_type": "BOSS_REWARD",
            "screen_state": {
                "relics": [
                    {"id": "Snecko Eye", "name": "Snecko Eye"},
                    {"id": "Runic Dome", "name": "Runic Dome"},
                    {"id": "Cursed Key", "name": "Cursed Key"}
                ]
            },
            "deck": [],
            "relics": [],
            "current_hp": 70,
            "max_hp": 75,
            "floor": 17,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(state.screen_type.as_deref(), Some("BOSS_REWARD"));
    assert_eq!(state.boss_relic_choices.len(), 3);
    assert!(
        state
            .boss_relic_choices
            .iter()
            .any(|r| r.name == "Snecko Eye")
    );
    assert!(
        state
            .boss_relic_choices
            .iter()
            .any(|r| r.name == "Runic Dome")
    );
    assert!(
        state
            .boss_relic_choices
            .iter()
            .any(|r| r.name == "Cursed Key")
    );
}

#[test]
fn normalize_event_choices() {
    let raw = serde_json::json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "screen_state": {
                "event_name": "Golden Idol",
                "body": "A golden idol sits on a pedestal.",
                "options": [
                    {"label": "Take"},
                    {"label": "Leave"}
                ]
            },
            "deck": [],
            "relics": [],
            "current_hp": 52,
            "max_hp": 75,
            "gold": 120,
            "floor": 8,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(state.screen_type.as_deref(), Some("EVENT"));
    assert_eq!(state.event_name.as_deref(), Some("Golden Idol"));
    assert_eq!(
        state.event_body.as_deref(),
        Some("A golden idol sits on a pedestal.")
    );
    assert_eq!(
        state.event_choices,
        vec!["Take".to_string(), "Leave".to_string()]
    );
}

#[test]
fn normalize_event_choices_prefer_option_text_for_full_description() {
    let raw = serde_json::json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "screen_state": {
                "event_name": "World of Goop",
                "body": "You fall into a puddle. It is made of slime goop.",
                "options": [
                    {"label": "Lose Gold", "text": "Lose 11 Gold."},
                    {"label": "Lose HP", "text": "Lose 5 HP."}
                ]
            },
            "deck": [],
            "relics": [],
            "current_hp": 52,
            "max_hp": 75,
            "gold": 120,
            "floor": 8,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(
        state.event_body.as_deref(),
        Some("You fall into a puddle. It is made of slime goop.")
    );
    assert_eq!(
        state.event_choices,
        vec!["Lose 11 Gold.".to_string(), "Lose 5 HP.".to_string()]
    );
}

#[test]
fn normalize_event_choices_replaces_unreadable_locale_garble() {
    let raw = serde_json::json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "screen_name": "??",
            "room_type": "NeowRoom",
            "screen_state": {
                "event_name": "??",
                "body": "?????",
                "options": [
                    {"label": "??? 3 ?????????? 1 ???"},
                    {"label": "????? +7"}
                ]
            },
            "deck": [],
            "relics": [],
            "current_hp": 72,
            "max_hp": 72,
            "gold": 99,
            "floor": 0,
            "class": "WATCHER"
        }
    });
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(state.screen_type.as_deref(), Some("EVENT"));
    assert_eq!(state.room_type.as_deref(), Some("NeowRoom"));
    assert!(state.event_name.is_none());
    assert!(state.event_body.is_none());
    assert_eq!(
        state.event_choices,
        vec![
            "选项 1（事件文本不可读，请在游戏内核对按钮）".to_string(),
            "选项 2（事件文本不可读，请在游戏内核对按钮）".to_string(),
        ]
    );
}

#[test]
fn normalize_event_payload_from_communication_mod_log() {
    let raw = serde_json::json!({
        "available_commands": ["choose", "key", "click", "wait", "state"],
        "ready_for_command": true,
        "in_game": true,
        "game_state": {
            "choice_list": ["??? 3 ?????????? 1 ???", "????? +7"],
            "screen_type": "EVENT",
            "screen_state": {
                "event_id": "Neow Event",
                "body_text": "",
                "options": [
                    {
                        "choice_index": 0,
                        "disabled": false,
                        "text": "[ ??? 3 ?????????? 1 ??? ]",
                        "label": "??? 3 ?????????? 1 ???"
                    },
                    {
                        "choice_index": 1,
                        "disabled": false,
                        "text": "[ ????? +7 ]",
                        "label": "????? +7"
                    }
                ],
                "event_name": "??"
            },
            "screen_name": "NONE",
            "room_type": "NeowRoom",
            "deck": [],
            "relics": [{"name": "????", "id": "PureWater", "counter": -1}],
            "current_hp": 72,
            "max_hp": 72,
            "gold": 99,
            "floor": 0,
            "class": "WATCHER"
        }
    });
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(state.event_id.as_deref(), Some("Neow Event"));
    assert!(state.event_name.is_none());
    assert!(state.event_body.is_none());
    assert_eq!(
        state.event_choices,
        vec![
            "选项 1（事件文本不可读，请在游戏内核对按钮）".to_string(),
            "选项 2（事件文本不可读，请在游戏内核对按钮）".to_string(),
        ]
    );
    assert_eq!(
        state.relics,
        vec![RelicInfo {
            id: "PureWater".into(),
            name: "????".into(),
            description: String::new(),
            counter: None,
            price: None,
        }]
    );
}

#[test]
fn danger_detects_low_hp() {
    let d = compute_danger(10, 80, 0, 0, false, &[]);
    assert!(d.hp_critical);
    assert!(!d.incoming_lethal);
    assert_eq!(d.level, DangerLevel::Danger);
}

#[test]
fn danger_detects_incoming_lethal() {
    let d = compute_danger(20, 80, 5, 30, false, &[]);
    assert!(d.incoming_lethal);
    assert_eq!(d.level, DangerLevel::Danger);
}

#[test]
fn danger_no_block_against_hit_is_caution() {
    let d = compute_danger(60, 80, 0, 10, false, &["ATTACK"]);
    assert!(d.no_block_against_hit);
    assert!(!d.hp_critical);
    assert_eq!(d.level, DangerLevel::Caution);
}

#[test]
fn danger_monster_attacking_is_caution() {
    let d = compute_danger(60, 80, 10, 0, false, &["ATTACK"]);
    assert!(d.any_monster_attacking);
    assert!(!d.hp_critical);
    assert_eq!(d.level, DangerLevel::Caution);
}

#[test]
fn danger_low_hp_is_caution() {
    let d = compute_danger(35, 80, 10, 0, false, &[]);
    assert!(!d.hp_critical);
    assert!(!d.any_monster_attacking);
    assert_eq!(d.level, DangerLevel::Caution);
}

#[test]
fn danger_wrath_stance_detected() {
    let d = compute_danger(60, 80, 10, 0, true, &["ATTACK"]);
    assert!(d.wrath_stance);
    assert!(d.any_monster_attacking);
    assert_eq!(d.level, DangerLevel::Caution);
}

#[test]
fn danger_safe_when_healthy_and_not_threatened() {
    let d = compute_danger(70, 80, 20, 0, false, &[]);
    assert!(!d.hp_critical);
    assert!(!d.incoming_lethal);
    assert!(!d.no_block_against_hit);
    assert!(!d.any_monster_attacking);
    assert_eq!(d.level, DangerLevel::Safe);
}

fn compute_danger(
    hp: i64,
    max_hp: i64,
    block: i64,
    incoming: i64,
    wrath: bool,
    intents: &[&str],
) -> DangerFlags {
    let monsters: Vec<MonsterInfo> = intents
        .iter()
        .map(|intent| MonsterInfo {
            name: "Test".into(),
            monster_id: None,
            index: 0,
            current_hp: None,
            max_hp: None,
            block: None,
            intent: if *intent == "NONE" {
                None
            } else {
                Some(intent.to_string())
            },
            damage: None,
            hits: None,
            monster_powers: vec![],
            can_be_killed: false,
            is_scaling: false,
        })
        .collect();

    let powers: Vec<PowerInfo> = if wrath {
        vec![PowerInfo {
            id: "".into(),
            name: "Wrath".into(),
            amount: 1,
        }]
    } else {
        vec![]
    };

    DangerFlags::compute(
        Some(hp),
        Some(max_hp),
        Some(block),
        incoming,
        &monsters,
        &powers,
    )
}

#[test]
fn stable_hash_same_state_same_hash() {
    let raw = load_fixture("combat-state.json");
    let state1 = NormalizedState::from_raw(&raw, test_locale());
    let state2 = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(state1.stable_hash(), state2.stable_hash());
}

#[test]
fn stable_hash_different_state_different_hash() {
    let combat = load_fixture("combat-state.json");
    let reward = load_fixture("card-reward-state.json");

    let state1 = NormalizedState::from_raw(&combat, test_locale());
    let state2 = NormalizedState::from_raw(&reward, test_locale());

    assert_ne!(state1.stable_hash(), state2.stable_hash());
}

#[test]
fn stable_hash_produces_hex() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    let hash = state.stable_hash();

    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn observation_hash_changes_when_draw_pile_changes() {
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["draw_pile"][0]["uuid"] =
        Value::String("changed-draw".into());

    let state1 = NormalizedState::from_raw(&raw1, test_locale());
    let state2 = NormalizedState::from_raw(&raw2, test_locale());

    assert_ne!(state1.observation_hash(), state2.observation_hash());
}

#[test]
fn observation_hash_changes_when_discard_pile_changes() {
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["discard_pile"] = serde_json::json!([
        {"id":"Strike_R","name":"Strike","cost":1,"type":"ATTACK","uuid":"discarded-card","upgrades":0}
    ]);

    let state1 = NormalizedState::from_raw(&raw1, test_locale());
    let state2 = NormalizedState::from_raw(&raw2, test_locale());

    assert_ne!(state1.observation_hash(), state2.observation_hash());
}

#[test]
fn observation_hash_changes_when_monster_block_changes() {
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["monsters"][0]["block"] = Value::Number(7.into());

    let state1 = NormalizedState::from_raw(&raw1, test_locale());
    let state2 = NormalizedState::from_raw(&raw2, test_locale());

    assert_ne!(state1.observation_hash(), state2.observation_hash());
}

#[test]
fn observation_hash_changes_when_monster_power_changes() {
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["monsters"][0]["powers"][0]["amount"] =
        Value::Number(9.into());

    let state1 = NormalizedState::from_raw(&raw1, test_locale());
    let state2 = NormalizedState::from_raw(&raw2, test_locale());

    assert_ne!(state1.observation_hash(), state2.observation_hash());
}

#[test]
fn observation_hash_changes_when_card_uuid_changes() {
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["hand"][0]["uuid"] =
        Value::String("changed-hand-card".into());

    let state1 = NormalizedState::from_raw(&raw1, test_locale());
    let state2 = NormalizedState::from_raw(&raw2, test_locale());

    assert_ne!(state1.observation_hash(), state2.observation_hash());
}

#[test]
fn card_reward_filters_potion_slot() {
    let raw = load_fixture("card-reward-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    assert!(
        !state
            .potions
            .iter()
            .any(|p| p.name.contains("Potion Slot") || p.name == "?")
    );
}

#[test]
fn card_info_upgraded() {
    let json: Value = serde_json::from_str(
        r#"{"id":"Strike_R","name":"Strike","cost":1,"type":"ATTACK","upgrades":1}"#,
    )
    .unwrap();
    let card = CardInfo::from_json(&json);
    assert!(card.upgraded);
}

#[test]
fn card_info_missing_cost_and_type() {
    let json: Value = serde_json::from_str(r#"{"id":"Strike_R","name":"Strike"}"#).unwrap();
    let card = CardInfo::from_json(&json);
    assert_eq!(card.cost, 0);
    assert_eq!(card.card_type, "?");
}

#[test]
fn card_info_uuid() {
    let json: Value =
        serde_json::from_str(r#"{"id":"Strike_R","name":"Strike","uuid":"abc-123"}"#).unwrap();
    let card = CardInfo::from_json(&json);
    assert_eq!(card.uuid.as_deref(), Some("abc-123"));
}

#[test]
fn danger_none_hp_uses_fallback() {
    let d = DangerFlags::compute(None, None, None, 0, &[], &[]);
    assert!(d.hp_critical);
    assert!(!d.incoming_lethal);
    assert_eq!(d.level, DangerLevel::Danger);
}

#[test]
fn map_nodes_parsed_from_state() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "screen_state": {},
            "map": [
                {
                    "symbol": "M",
                    "x": 0,
                    "y": 0,
                    "children": [{"x": 1, "y": 1}]
                },
                {
                    "symbol": "R",
                    "x": 1,
                    "y": 1,
                    "children": []
                }
            ],
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.map_nodes.len(), 2);
    assert_eq!(state.map_nodes[0].symbol, "M");
    assert_eq!(state.map_nodes[0].x, 0);
    assert_eq!(state.map_nodes[0].y, 0);
    assert_eq!(state.map_nodes[0].children, vec![(1, 1)]);
    assert_eq!(state.map_nodes[1].symbol, "R");
    assert_eq!(state.map_nodes[1].x, 1);
    assert_eq!(state.map_nodes[1].y, 1);
    assert!(state.map_nodes[1].children.is_empty());
}

#[test]
fn map_screen_state_parsed() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "screen_state": {
                "first_node_chosen": true,
                "current_node": {"x": 3, "y": 7}
            },
            "map": [
                {"symbol": "M", "x": 3, "y": 7, "children": []}
            ],
            "current_hp": 70,
            "max_hp": 75,
            "floor": 5,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.map_first_node_chosen, Some(true));
    assert_eq!(state.map_current_x, Some(3));
    assert_eq!(state.map_current_y, Some(7));
}

#[test]
fn map_screen_state_defaults_to_none() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.map_first_node_chosen, None);
    assert_eq!(state.map_current_x, None);
    assert_eq!(state.map_current_y, None);
}

// --- monster_id ---

#[test]
fn monster_id_from_combat_fixture() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    let jaw_worm = &state.monsters[0];
    assert_eq!(jaw_worm.monster_id.as_deref(), Some("JawWorm"));
}

#[test]
fn monster_id_defaults_to_none_when_missing() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Test Enemy",
                    "current_hp": 10,
                    "max_hp": 20,
                    "block": 0
                }],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.monsters[0].monster_id, None);
}

// --- turn_number ---

#[test]
fn turn_number_from_combat_fixture() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    assert_eq!(state.turn_number, Some(1));
}

#[test]
fn turn_number_defaults_to_none_when_missing() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Test Enemy",
                    "id": "TestEnemy",
                    "current_hp": 10,
                    "max_hp": 20,
                    "block": 0
                }],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.turn_number, None);
}

// --- orbs ---

#[test]
fn orbs_from_raw_json() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Test Enemy",
                    "id": "TestEnemy",
                    "current_hp": 10,
                    "max_hp": 20,
                    "block": 0
                }],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": [
                        {"id": "Lightning", "amount": 2},
                        {"id": "Frost", "amount": 1}
                    ]
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.orbs.len(), 2);
    assert_eq!(state.orbs[0].id, "Lightning");
    assert_eq!(state.orbs[0].amount, 2);
    assert_eq!(state.orbs[1].id, "Frost");
    assert_eq!(state.orbs[1].amount, 1);
}

#[test]
fn orbs_empty_when_missing() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    assert!(state.orbs.is_empty());
}

// --- stance ---

#[test]
fn stance_from_powers_wrath() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Test Enemy",
                    "id": "TestEnemy",
                    "current_hp": 10,
                    "max_hp": 20,
                    "block": 0
                }],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [
                        {"id": "Wrath", "name": "Wrath", "amount": 1}
                    ],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.stance.as_deref(), Some("Wrath"));
}

#[test]
fn stance_from_powers_calm() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Test Enemy",
                    "id": "TestEnemy",
                    "current_hp": 10,
                    "max_hp": 20,
                    "block": 0
                }],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [
                        {"id": "Calm", "name": "Calm", "amount": 1}
                    ],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.stance.as_deref(), Some("Calm"));
}

#[test]
fn stance_from_powers_divinity() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Test Enemy",
                    "id": "TestEnemy",
                    "current_hp": 10,
                    "max_hp": 20,
                    "block": 0
                }],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [
                        {"id": "Divinity", "name": "Divinity", "amount": 1}
                    ],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.stance.as_deref(), Some("Divinity"));
}

#[test]
fn stance_none_when_no_stance_powers() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    assert_eq!(state.stance, None);
}

#[test]
fn stance_none_when_stance_amount_zero() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Test Enemy",
                    "id": "TestEnemy",
                    "current_hp": 10,
                    "max_hp": 20,
                    "block": 0
                }],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [
                        {"id": "Wrath", "name": "Wrath", "amount": 0}
                    ],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.stance, None);
}

// --- default state ---

#[test]
fn default_state_all_fields_empty() {
    let state = NormalizedState::default();
    assert!(state.screen_type.is_none());
    assert!(state.room_type.is_none());
    assert!(state.character.is_none());
    assert!(state.seed.is_none());
    assert!(state.ascension_level.is_none());
    assert!(state.floor.is_none());
    assert!(state.current_hp.is_none());
    assert!(state.max_hp.is_none());
    assert!(state.gold.is_none());
    assert!(state.energy.is_none());
    assert!(state.block.is_none());
    assert!(state.powers.is_empty());
    assert!(state.hand.is_empty());
    assert!(state.monsters.is_empty());
    assert!(state.card_reward_choices.is_empty());
    assert!(state.boss_relic_choices.is_empty());
    assert!(state.event_id.is_none());
    assert!(state.event_name.is_none());
    assert!(state.event_body.is_none());
    assert!(state.event_choices.is_empty());
    assert!(state.relics.is_empty());
    assert!(state.potions.is_empty());
    assert!(state.deck_names.is_empty());
    assert_eq!(state.incoming_damage, 0);
    assert!(state.rest_options.is_empty());
    assert!(!state.skip_available);
    assert!(state.shop_cards.is_empty());
    assert!(state.shop_relics.is_empty());
    assert!(state.shop_potions.is_empty());
    assert!(!state.purge_available);
    assert!(state.purge_cost.is_none());
    assert!(state.turn_number.is_none());
    assert!(state.orbs.is_empty());
    assert!(state.stance.is_none());
    assert!(state.hand_cards.is_empty());
    assert!(state.draw_pile.is_empty());
    assert!(state.discard_pile.is_empty());
    assert!(state.exhaust_cards.is_empty());
    assert!(state.master_cards.is_empty());
    assert!(state.map_nodes.is_empty());
    assert!(state.map_first_node_chosen.is_none());
    assert!(state.map_current_x.is_none());
    assert!(state.map_current_y.is_none());
    assert!(state.hand_select_max_cards.is_none());
    assert!(!state.hand_select_can_pick_zero);
    assert!(state.hand_select_selected.is_empty());
    assert!(state.current_action.is_none());
    assert!(state.card_in_play.is_none());
    assert!(state.grid_cards.is_empty());
    assert!(state.grid_selected_cards.is_empty());
    assert!(!state.grid_for_upgrade);
    assert!(!state.grid_for_transform);
    assert!(!state.grid_for_purge);
    assert!(state.grid_num_cards.is_none());
    assert_eq!(state.empty_potion_slots, 0);
    assert_eq!(state.danger.level, DangerLevel::Safe);
}

// --- is_boss_card_reward ---

#[test]
fn is_boss_card_reward_true_for_floors_16_33_50() {
    for floor in [16, 33, 50] {
        let raw = json!({
            "in_game": true,
            "game_state": {
                "screen_type": "CARD_REWARD",
                "screen_state": {
                    "cards": [
                        {"id": "Immolate", "name": "Immolate", "cost": 2, "type": "ATTACK"}
                    ]
                },
                "current_hp": 60,
                "max_hp": 75,
                "floor": floor,
                "class": "IRONCLAD"
            }
        });
        let state = NormalizedState::from_raw(&raw, test_locale());
        assert!(
            state.is_boss_card_reward(),
            "floor {floor} should be boss card reward"
        );
    }
}

#[test]
fn is_boss_card_reward_false_for_other_floors() {
    for floor in [1, 14, 17, 32, 34, 49, 51] {
        let raw = json!({
            "in_game": true,
            "game_state": {
                "screen_type": "CARD_REWARD",
                "screen_state": {
                    "cards": [
                        {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                    ]
                },
                "current_hp": 60,
                "max_hp": 75,
                "floor": floor,
                "class": "IRONCLAD"
            }
        });
        let state = NormalizedState::from_raw(&raw, test_locale());
        assert!(
            !state.is_boss_card_reward(),
            "floor {floor} should not be boss card reward"
        );
    }
}

#[test]
fn is_boss_card_reward_false_non_card_reward_screen() {
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "BOSS_REWARD",
            "screen_state": {
                "relics": [
                    {"id": "Snecko Eye", "name": "Snecko Eye"}
                ]
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 16,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, test_locale());
    assert!(!state.is_boss_card_reward());
}

// --- has_active_monsters ---

#[test]
fn has_active_monsters_true_with_monsters() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    assert!(state.has_active_monsters());
}

#[test]
fn has_active_monsters_false_without_monsters() {
    let raw = load_fixture("card-reward-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    assert!(!state.has_active_monsters());
}

// --- CardInfo::from_json edge cases ---

#[test]
fn card_info_has_target_true() {
    let json: Value = serde_json::from_str(
        r#"{"id":"Bash","name":"Bash","cost":2,"type":"ATTACK","has_target":true}"#,
    )
    .unwrap();
    let card = CardInfo::from_json(&json);
    assert!(card.has_target);
}

#[test]
fn card_info_has_target_defaults_to_false() {
    let json: Value =
        serde_json::from_str(r#"{"id":"Defend_R","name":"Defend","cost":1,"type":"SKILL"}"#)
            .unwrap();
    let card = CardInfo::from_json(&json);
    assert!(!card.has_target);
}

#[test]
fn card_info_upgraded_false_when_zero() {
    let json: Value = serde_json::from_str(
        r#"{"id":"Strike_R","name":"Strike","cost":1,"type":"ATTACK","upgrades":0}"#,
    )
    .unwrap();
    let card = CardInfo::from_json(&json);
    assert!(!card.upgraded);
}

#[test]
fn card_info_missing_id() {
    let json: Value =
        serde_json::from_str(r#"{"name":"Mystery","cost":1,"type":"SKILL"}"#).unwrap();
    let card = CardInfo::from_json(&json);
    assert_eq!(card.id, "?");
    assert_eq!(card.name, "Mystery");
}

#[test]
fn card_info_missing_name() {
    let json: Value = serde_json::from_str(r#"{"id":"TestCard","cost":2,"type":"POWER"}"#).unwrap();
    let card = CardInfo::from_json(&json);
    assert_eq!(card.id, "TestCard");
    assert_eq!(card.name, "?");
}

#[test]
fn card_info_price_parsed() {
    let json: Value = serde_json::from_str(
        r#"{"id":"Armaments","name":"Armaments","cost":1,"type":"SKILL","price":51}"#,
    )
    .unwrap();
    let card = CardInfo::from_json(&json);
    assert_eq!(card.price, Some(51));
}

#[test]
fn card_info_price_none_when_missing() {
    let json: Value =
        serde_json::from_str(r#"{"id":"Strike_R","name":"Strike","cost":1,"type":"ATTACK"}"#)
            .unwrap();
    let card = CardInfo::from_json(&json);
    assert_eq!(card.price, None);
}

#[test]
fn card_info_not_playable() {
    let json: Value = serde_json::from_str(
        r#"{"id":"Barricade","name":"Barricade","cost":3,"type":"POWER","is_playable":false}"#,
    )
    .unwrap();
    let card = CardInfo::from_json(&json);
    assert!(!card.playable);
}

// --- shop parsing ---

#[test]
fn normalize_shop_state() {
    let raw = load_fixture("shop-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(state.screen_type.as_deref(), Some("SHOP_SCREEN"));
    assert_eq!(state.floor, Some(5));
    assert_eq!(state.gold, Some(190));
    assert!(state.purge_available);
    assert_eq!(state.purge_cost, Some(75));
    assert!(!state.shop_cards.is_empty());
    assert!(!state.shop_potions.is_empty());

    let card_ids: Vec<&str> = state.shop_cards.iter().map(|c| c.id.as_str()).collect();
    assert!(card_ids.contains(&"Sever Soul"));
    assert!(card_ids.contains(&"Armaments"));
    assert!(card_ids.contains(&"Fire Breathing"));

    for card in &state.shop_cards {
        assert!(
            card.price.is_some(),
            "shop card {} should have a price",
            card.id
        );
    }

    assert!(!state.master_cards.is_empty());
    assert!(state.hand.is_empty());
    assert!(state.monsters.is_empty());
}

// --- grid screen parsing ---

#[test]
fn grid_for_upgrade_parsed() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "GRID",
            "screen_state": {
                "cards": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"},
                    {"id": "Defend_R", "name": "Defend", "cost": 1, "type": "SKILL"}
                ],
                "for_upgrade": true,
                "for_transform": false,
                "for_purge": false,
                "num_cards": 2
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert!(state.grid_for_upgrade);
    assert!(!state.grid_for_transform);
    assert!(!state.grid_for_purge);
    assert_eq!(state.grid_cards.len(), 2);
    assert_eq!(state.grid_num_cards, Some(2));
}

#[test]
fn grid_for_transform_parsed() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "GRID",
            "screen_state": {
                "cards": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ],
                "for_upgrade": false,
                "for_transform": true,
                "for_purge": false,
                "num_cards": 1
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert!(!state.grid_for_upgrade);
    assert!(state.grid_for_transform);
    assert!(!state.grid_for_purge);
    assert_eq!(state.grid_cards.len(), 1);
    assert_eq!(state.grid_num_cards, Some(1));
}

#[test]
fn grid_for_purge_parsed() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "GRID",
            "screen_state": {
                "cards": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"},
                    {"id": "Defend_R", "name": "Defend", "cost": 1, "type": "SKILL"},
                    {"id": "Bash", "name": "Bash", "cost": 2, "type": "ATTACK"}
                ],
                "for_upgrade": false,
                "for_transform": false,
                "for_purge": true,
                "num_cards": 3,
                "selected_cards": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ]
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert!(!state.grid_for_upgrade);
    assert!(!state.grid_for_transform);
    assert!(state.grid_for_purge);
    assert_eq!(state.grid_cards.len(), 3);
    assert_eq!(state.grid_num_cards, Some(3));
    assert_eq!(state.grid_selected_cards.len(), 1);
    assert_eq!(state.grid_selected_cards[0].id, "Strike_R");
}

#[test]
fn grid_all_flags_false_by_default() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "GRID",
            "screen_state": {
                "cards": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ]
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert!(!state.grid_for_upgrade);
    assert!(!state.grid_for_transform);
    assert!(!state.grid_for_purge);
    assert!(state.grid_num_cards.is_none());
    assert!(state.grid_selected_cards.is_empty());
}

// --- hand select ---

#[test]
fn hand_select_can_pick_zero_parsed() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "HAND_SELECT",
            "screen_state": {
                "max_cards": 1,
                "can_pick_zero": true,
                "selected": []
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.hand_select_max_cards, Some(1));
    assert!(state.hand_select_can_pick_zero);
    assert!(state.hand_select_selected.is_empty());
}

#[test]
fn hand_select_can_pick_zero_defaults_to_false() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "HAND_SELECT",
            "screen_state": {
                "max_cards": 2,
                "selected": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ]
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.hand_select_max_cards, Some(2));
    assert!(!state.hand_select_can_pick_zero);
    assert_eq!(state.hand_select_selected.len(), 1);
    assert_eq!(state.hand_select_selected[0].id, "Strike_R");
}

#[test]
fn card_in_play_parsed_from_combat() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "HAND_SELECT",
            "screen_state": {
                "max_cards": 1,
                "can_pick_zero": false
            },
            "combat_state": {
                "hand": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "monsters": [],
                "card_in_play": {
                    "id": "Burning Pact",
                    "name": "Burning Pact",
                    "cost": 1,
                    "type": "SKILL",
                    "upgrades": 0
                },
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert!(state.card_in_play.is_some());
    let card = state.card_in_play.unwrap();
    assert_eq!(card.id, "Burning Pact");
    assert_eq!(card.name, "Burning Pact");
    assert_eq!(card.cost, 1);
    assert_eq!(card.card_type, "SKILL");
}

#[test]
fn card_in_play_none_when_not_present() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    assert!(state.card_in_play.is_none());
}

// --- event choices text extraction ---

#[test]
fn event_choices_prefer_choice_text_field() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "screen_state": {
                "event_name": "Wing Statue",
                "body": "A statue of a bird.",
                "options": [
                    {"label": "Pray", "text": "Pray for strength.", "description": "Gain 1 Strength."},
                    {"label": "Leave", "text": "Walk away silently."}
                ]
            },
            "current_hp": 52,
            "max_hp": 75,
            "floor": 8,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(
        state.event_choices,
        vec![
            "Pray for strength.".to_string(),
            "Walk away silently.".to_string()
        ]
    );
}

#[test]
fn event_choices_fallback_to_label_when_text_unreadable() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "screen_state": {
                "event_name": "Test Event",
                "body": "Hello world.",
                "options": [
                    {"label": "Choose me", "text": "????"},
                    {"label": "No, choose me", "text": "??????"}
                ]
            },
            "current_hp": 52,
            "max_hp": 75,
            "floor": 8,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(
        state.event_choices,
        vec!["Choose me".to_string(), "No, choose me".to_string()]
    );
}

#[test]
fn event_choices_extract_from_string_array() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "screen_state": {
                "event_name": "Test",
                "body": "Test body.",
                "options": [
                    "Take the relic",
                    "Leave it alone"
                ]
            },
            "current_hp": 52,
            "max_hp": 75,
            "floor": 8,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(
        state.event_choices,
        vec!["Take the relic".to_string(), "Leave it alone".to_string()]
    );
}

#[test]
fn event_choices_from_choice_list_fallback() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "screen_state": {},
            "choice_list": ["Option A", "Option B"],
            "current_hp": 52,
            "max_hp": 75,
            "floor": 8,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(
        state.event_choices,
        vec!["Option A".to_string(), "Option B".to_string()]
    );
}

// --- monster parsing ---

#[test]
fn monster_is_gone_filtered_out() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [
                    {
                        "name": "Active Enemy",
                        "id": "ActiveEnemy",
                        "current_hp": 10,
                        "max_hp": 20,
                        "block": 0,
                        "intent": "ATTACK",
                        "is_gone": false
                    },
                    {
                        "name": "Gone Enemy",
                        "id": "GoneEnemy",
                        "current_hp": 0,
                        "max_hp": 15,
                        "block": 0,
                        "intent": "NONE",
                        "is_gone": true
                    }
                ],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.monsters.len(), 1);
    assert_eq!(state.monsters[0].name, "Active Enemy");
}

#[test]
fn monster_is_gone_defaults_to_false() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [
                    {
                        "name": "Test Enemy",
                        "current_hp": 10,
                        "max_hp": 20,
                        "block": 0
                    }
                ],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.monsters.len(), 1);
}

#[test]
fn monster_is_scaling_detects_strength() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Scaling Enemy",
                    "id": "ScalingEnemy",
                    "current_hp": 30,
                    "max_hp": 40,
                    "block": 0,
                    "intent": "ATTACK",
                    "powers": [
                        {"id": "Strength", "name": "Strength", "amount": 3}
                    ]
                }],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert!(state.monsters[0].is_scaling);
}

#[test]
fn monster_is_scaling_detects_metallicize() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Armored Enemy",
                    "id": "ArmoredEnemy",
                    "current_hp": 30,
                    "max_hp": 40,
                    "block": 0,
                    "intent": "DEFEND",
                    "powers": [
                        {"id": "Metallicize", "name": "Metallicize", "amount": 4}
                    ]
                }],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert!(state.monsters[0].is_scaling);
}

#[test]
fn monster_is_scaling_detects_regeneration() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Regen Enemy",
                    "id": "RegenEnemy",
                    "current_hp": 30,
                    "max_hp": 40,
                    "block": 0,
                    "intent": "BUFF",
                    "powers": [
                        {"id": "Regeneration", "name": "Regeneration", "amount": 5}
                    ]
                }],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert!(state.monsters[0].is_scaling);
}

#[test]
fn monster_is_scaling_detects_plated_armor() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Shelled Enemy",
                    "id": "ShelledEnemy",
                    "current_hp": 30,
                    "max_hp": 40,
                    "block": 0,
                    "intent": "ATTACK_BUFF",
                    "powers": [
                        {"id": "Plated Armor", "name": "Plated Armor", "amount": 10}
                    ]
                }],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert!(state.monsters[0].is_scaling);
}

#[test]
fn monster_not_scaling_without_scaling_powers() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Normal Enemy",
                    "id": "NormalEnemy",
                    "current_hp": 30,
                    "max_hp": 40,
                    "block": 0,
                    "intent": "ATTACK",
                    "powers": [
                        {"id": "Vulnerable", "name": "Vulnerable", "amount": 1}
                    ]
                }],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert!(!state.monsters[0].is_scaling);
}

#[test]
fn monster_can_be_killed_with_enough_hand_attack() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Weak Enemy",
                    "id": "WeakEnemy",
                    "current_hp": 6,
                    "max_hp": 40,
                    "block": 0,
                    "intent": "ATTACK"
                }],
                "hand": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert!(state.monsters[0].can_be_killed);
}

#[test]
fn monster_cannot_be_killed_with_insufficient_hand_attack() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Tough Enemy",
                    "id": "ToughEnemy",
                    "current_hp": 100,
                    "max_hp": 100,
                    "block": 0,
                    "intent": "ATTACK"
                }],
                "hand": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert!(!state.monsters[0].can_be_killed);
}

#[test]
fn monster_cannot_be_killed_when_hp_missing() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Mystery Enemy",
                    "id": "MysteryEnemy",
                    "max_hp": 40,
                    "block": 0,
                    "intent": "ATTACK"
                }],
                "hand": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert!(!state.monsters[0].can_be_killed);
}

#[test]
fn monster_intent_parsed() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [{
                    "name": "Test Enemy",
                    "id": "TestEnemy",
                    "current_hp": 10,
                    "max_hp": 20,
                    "block": 0,
                    "intent": "ATTACK_BUFF",
                    "move_adjusted_damage": 15,
                    "move_hits": 2
                }],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.monsters[0].intent.as_deref(), Some("ATTACK_BUFF"));
    assert_eq!(state.monsters[0].damage, Some(15));
    assert_eq!(state.monsters[0].hits, Some(2));
}

// --- potions ---

#[test]
fn potion_slot_counting_with_gaps() {
    let raw = load_fixture("comm-gapped-potions.json");
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert_eq!(state.potions.len(), 2);
    assert_eq!(state.empty_potion_slots, 1);

    let potion_names: Vec<&str> = state.potions.iter().map(|p| p.name.as_str()).collect();
    assert!(!potion_names.contains(&"Potion Slot"));
    assert!(!potion_names.contains(&"?"));
    assert!(!potion_names.is_empty());
}

#[test]
fn potion_slot_counting_with_no_empty_slots() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD",
            "potions": [
                {"requires_target": false, "can_use": true, "can_discard": true, "name": "Energy Potion", "id": "Energy Potion"},
                {"requires_target": false, "can_use": true, "can_discard": true, "name": "Block Potion", "id": "Block Potion"},
                {"requires_target": false, "can_use": true, "can_discard": true, "name": "Fire Potion", "id": "Fire Potion"}
            ]
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.potions.len(), 3);
    assert_eq!(state.empty_potion_slots, 0);
}

#[test]
fn potions_empty_array_produces_no_slots() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD",
            "potions": []
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert!(state.potions.is_empty());
    assert_eq!(state.empty_potion_slots, 0);
}

// --- incoming damage ---

#[test]
fn incoming_damage_excludes_none_intent() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [
                    {
                        "name": "Attacker",
                        "id": "Attacker",
                        "current_hp": 10,
                        "max_hp": 20,
                        "block": 0,
                        "intent": "ATTACK",
                        "move_adjusted_damage": 12
                    },
                    {
                        "name": "Sleeper",
                        "id": "Sleeper",
                        "current_hp": 10,
                        "max_hp": 20,
                        "block": 0,
                        "intent": "NONE",
                        "move_adjusted_damage": 0
                    },
                    {
                        "name": "Buffer",
                        "id": "Buffer",
                        "current_hp": 10,
                        "max_hp": 20,
                        "block": 0,
                        "intent": "BUFF",
                        "move_adjusted_damage": 0
                    }
                ],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.incoming_damage, 12);
}

#[test]
fn incoming_damage_excludes_zero_damage() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [
                    {
                        "name": "Tickle Monster",
                        "id": "TickleMonster",
                        "current_hp": 10,
                        "max_hp": 20,
                        "block": 0,
                        "intent": "ATTACK",
                        "move_adjusted_damage": 0
                    },
                    {
                        "name": "Real Threat",
                        "id": "RealThreat",
                        "current_hp": 10,
                        "max_hp": 20,
                        "block": 0,
                        "intent": "ATTACK",
                        "move_adjusted_damage": 15
                    }
                ],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.incoming_damage, 15);
}

#[test]
fn incoming_damage_sums_multiple_monsters() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "combat_state": {
                "monsters": [
                    {
                        "name": "Monster A",
                        "id": "MonsterA",
                        "current_hp": 10,
                        "max_hp": 20,
                        "block": 0,
                        "intent": "ATTACK",
                        "move_adjusted_damage": 12
                    },
                    {
                        "name": "Monster B",
                        "id": "MonsterB",
                        "current_hp": 10,
                        "max_hp": 20,
                        "block": 0,
                        "intent": "ATTACK",
                        "move_adjusted_damage": 8
                    }
                ],
                "hand": [],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.incoming_damage, 20);
}

// --- DangerFlags max_hp=0 and current_hp=0 ---

#[test]
fn danger_max_hp_zero_fallback() {
    let d = DangerFlags::compute(Some(50), Some(0), Some(0), 0, &[], &[]);
    assert!(!d.hp_critical);
    assert!(!d.incoming_lethal);
    assert_eq!(d.level, DangerLevel::Safe);
}

#[test]
fn danger_current_hp_zero_with_max_hp() {
    let d = DangerFlags::compute(Some(0), Some(80), Some(0), 0, &[], &[]);
    assert!(d.hp_critical);
    assert!(!d.incoming_lethal);
    assert_eq!(d.level, DangerLevel::Danger);
}

#[test]
fn danger_block_absorbs_incoming_lethal() {
    let d = compute_danger(20, 80, 15, 30, false, &["ATTACK"]);
    assert!(!d.incoming_lethal);
    assert!(d.any_monster_attacking);
}

#[test]
fn danger_incoming_lethal_when_damage_exceeds_hp_plus_block() {
    let d = compute_danger(20, 80, 5, 30, false, &[]);
    assert!(d.incoming_lethal);
    assert_eq!(d.level, DangerLevel::Danger);
}

// --- map nodes missing children and coords ---

#[test]
fn map_nodes_missing_children_defaults_to_empty() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "screen_state": {},
            "map": [
                {"symbol": "M", "x": 3, "y": 7}
            ],
            "current_hp": 70,
            "max_hp": 75,
            "floor": 5,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.map_nodes.len(), 1);
    assert_eq!(state.map_nodes[0].symbol, "M");
    assert_eq!(state.map_nodes[0].x, 3);
    assert_eq!(state.map_nodes[0].y, 7);
    assert!(state.map_nodes[0].children.is_empty());
}

#[test]
fn map_nodes_missing_coords_default_to_zero() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "screen_state": {},
            "map": [
                {"symbol": "R"}
            ],
            "current_hp": 70,
            "max_hp": 75,
            "floor": 5,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.map_nodes.len(), 1);
    assert_eq!(state.map_nodes[0].x, 0);
    assert_eq!(state.map_nodes[0].y, 0);
    assert_eq!(state.map_nodes[0].symbol, "R");
}

#[test]
fn map_nodes_missing_symbol_defaults_to_question() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "screen_state": {},
            "map": [
                {"x": 1, "y": 2}
            ],
            "current_hp": 70,
            "max_hp": 75,
            "floor": 5,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.map_nodes[0].symbol, "?");
}

#[test]
fn map_nodes_child_missing_coords_filtered_out() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "screen_state": {},
            "map": [
                {
                    "symbol": "M",
                    "x": 0,
                    "y": 0,
                    "children": [
                        {"x": 1, "y": 1},
                        {"y": 2},
                        {"x": 3}
                    ]
                }
            ],
            "current_hp": 70,
            "max_hp": 75,
            "floor": 5,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.map_nodes.len(), 1);
    assert_eq!(state.map_nodes[0].children, vec![(1, 1)]);
}

// --- relic extraction edge cases ---

#[test]
fn relic_info_from_string_array() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD",
            "relics": ["Burning Blood", "Neow's Lament"]
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.relics.len(), 2);
    assert_eq!(state.relics[0].name, "Burning Blood");
    assert_eq!(state.relics[0].id, "");
    assert_eq!(state.relics[1].name, "Neow's Lament");
}

#[test]
fn relic_info_counter_filtered_when_negative() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD",
            "relics": [
                {"id": "Burning Blood", "name": "Burning Blood", "counter": -1},
                {"id": "Ornamental Fan", "name": "Ornamental Fan", "counter": 3},
                {"id": "NeowsBlessing", "name": "Neow's Blessing", "counter": -2}
            ]
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    let burning = state
        .relics
        .iter()
        .find(|r| r.id == "Burning Blood")
        .unwrap();
    assert_eq!(burning.counter, None);

    let fan = state
        .relics
        .iter()
        .find(|r| r.id == "Ornamental Fan")
        .unwrap();
    assert_eq!(fan.counter, Some(3));

    let neow = state
        .relics
        .iter()
        .find(|r| r.id == "NeowsBlessing")
        .unwrap();
    assert_eq!(neow.counter, None);
}

#[test]
fn relic_info_defaults_for_missing_fields() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD",
            "relics": [
                {"id": "TestRelic"}
            ]
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.relics.len(), 1);
    assert_eq!(state.relics[0].id, "TestRelic");
    assert_eq!(state.relics[0].name, "?");
    assert_eq!(state.relics[0].description, "");
    assert_eq!(state.relics[0].counter, None);
    assert_eq!(state.relics[0].price, None);
}

#[test]
fn relic_info_with_description() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD",
            "relics": [
                {
                    "id": "Burning Blood",
                    "name": "Burning Blood",
                    "description": "At the end of combat, heal 6 HP."
                }
            ]
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(
        state.relics[0].description,
        "At the end of combat, heal 6 HP."
    );
}

#[test]
fn relic_info_shop_relic_with_price() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "SHOP_SCREEN",
            "screen_state": {
                "cards": [],
                "potions": [],
                "relics": [
                    {
                        "id": "Bronze Scales",
                        "name": "Bronze Scales",
                        "description": "When you are attacked, deal 3 damage back.",
                        "price": 150
                    },
                    {
                        "id": "Oddly Smooth Stone",
                        "name": "Oddly Smooth Stone",
                        "description": "Gain 1 Dexterity.",
                        "price": 251
                    }
                ]
            },
            "current_hp": 70,
            "max_hp": 75,
            "gold": 300,
            "floor": 10,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);

    assert_eq!(state.shop_relics.len(), 2);
    assert_eq!(state.shop_relics[0].name, "Bronze Scales");
    assert_eq!(state.shop_relics[0].price, Some(150));
    assert_eq!(state.shop_relics[1].name, "Oddly Smooth Stone");
    assert_eq!(state.shop_relics[1].price, Some(251));
}

// --- stable hash with various screen types ---

#[test]
fn stable_hash_includes_boss_relic_fields() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "BOSS_REWARD",
            "screen_state": {
                "relics": [
                    {"id": "Snecko Eye", "name": "Snecko Eye", "description": "Draw 7, confuse."},
                    {"id": "Runic Dome", "name": "Runic Dome", "description": "+1 energy, no intents."}
                ]
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 16,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
}

#[test]
fn stable_hash_includes_shop_fields() {
    let raw = load_fixture("shop-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
}

#[test]
fn stable_hash_includes_grid_fields() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "GRID",
            "screen_state": {
                "cards": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ],
                "for_upgrade": true,
                "for_transform": false,
                "for_purge": false,
                "num_cards": 1
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
}

#[test]
fn stable_hash_includes_hand_select_fields() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "HAND_SELECT",
            "screen_state": {
                "max_cards": 1,
                "can_pick_zero": true
            },
            "combat_state": {
                "hand": [
                    {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK"}
                ],
                "draw_pile": [],
                "discard_pile": [],
                "exhaust_pile": [],
                "monsters": [],
                "card_in_play": {
                    "id": "Burning Pact",
                    "name": "Burning Pact",
                    "cost": 1,
                    "type": "SKILL"
                },
                "player": {
                    "energy": 3,
                    "block": 0,
                    "current_hp": 60,
                    "max_hp": 75,
                    "powers": [],
                    "orbs": []
                }
            },
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
}

#[test]
fn stable_hash_includes_event_fields() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "EVENT",
            "screen_state": {
                "event_id": "GoldenIdol",
                "event_name": "Golden Idol",
                "body": "A golden idol sits on a pedestal.",
                "options": [
                    {"label": "Take", "text": "Take the idol and become cursed."},
                    {"label": "Leave", "text": "Leave it alone."}
                ]
            },
            "current_hp": 52,
            "max_hp": 75,
            "floor": 8,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
}

#[test]
fn stable_hash_includes_map_fields() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "MAP",
            "screen_state": {
                "first_node_chosen": true,
                "current_node": {"x": 3, "y": 7}
            },
            "map": [
                {"symbol": "M", "x": 3, "y": 7, "children": [{"x": 4, "y": 8}]}
            ],
            "current_hp": 70,
            "max_hp": 75,
            "floor": 5,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
}

#[test]
fn stable_hash_includes_empty_potion_slots() {
    let raw = load_fixture("comm-gapped-potions.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
    assert!(state.empty_potion_slots > 0);
}

#[test]
fn stable_hash_empty_potion_slots_zero_not_included() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 70,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD",
            "potions": [
                {"id": "Energy Potion", "name": "Energy Potion"}
            ]
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.empty_potion_slots, 0);
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
}

#[test]
fn stable_hash_includes_current_action() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "HAND_SELECT",
            "screen_state": {},
            "current_action": "PlayCard",
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
    assert_eq!(state.current_action.as_deref(), Some("PlayCard"));
}

#[test]
fn stable_hash_excludes_current_action_when_none() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "NONE",
            "current_hp": 60,
            "max_hp": 75,
            "floor": 1,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert!(state.current_action.is_none());
    let hash = state.stable_hash();
    assert_eq!(hash.len(), 64);
}

// --- current_action ---

#[test]
fn current_action_parsed_from_game_state() {
    let locale = test_locale();
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "HAND_SELECT",
            "screen_state": {
                "max_cards": 1
            },
            "current_action": "PlayCard",
            "current_hp": 60,
            "max_hp": 75,
            "floor": 7,
            "class": "IRONCLAD"
        }
    });
    let state = NormalizedState::from_raw(&raw, locale);
    assert_eq!(state.current_action.as_deref(), Some("PlayCard"));
}

#[test]
fn current_action_none_when_missing() {
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, test_locale());
    assert!(state.current_action.is_none());
}

// --- danger wrath_stance ---

#[test]
fn danger_wrath_stance_increases_risk() {
    let d = compute_danger(70, 80, 20, 0, true, &[]);
    assert!(d.wrath_stance);
}

#[test]
fn danger_no_wrath_without_stance_power() {
    let d = compute_danger(70, 80, 20, 0, false, &["ATTACK"]);
    assert!(!d.wrath_stance);
    assert_eq!(d.level, DangerLevel::Caution);
}
