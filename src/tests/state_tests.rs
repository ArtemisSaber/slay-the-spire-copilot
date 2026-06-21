use super::*;
use crate::state::{DangerFlags, RelicInfo};
use crate::test_utils::{load_fixture, load_i18n};
use serde_json::Value;

#[test]
fn normalize_combat_state() {
    let i18n = load_i18n();
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, &i18n);

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
    assert_eq!(jaw_worm.name, "大颚虫");
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
    let i18n = load_i18n();
    let raw = load_fixture("card-reward-state.json");
    let state = NormalizedState::from_raw(&raw, &i18n);

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
    let i18n = load_i18n();
    let raw = load_fixture("rest-state.json");
    let state = NormalizedState::from_raw(&raw, &i18n);

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
    let i18n = load_i18n();
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
    let state = NormalizedState::from_raw(&raw, &i18n);

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
    let i18n = load_i18n();
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
    let state = NormalizedState::from_raw(&raw, &i18n);

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
    let i18n = load_i18n();
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
    let state = NormalizedState::from_raw(&raw, &i18n);

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
    let i18n = load_i18n();
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
    let state = NormalizedState::from_raw(&raw, &i18n);

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
    let i18n = load_i18n();
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
    let state = NormalizedState::from_raw(&raw, &i18n);

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
            name: "????".into(),
            description: String::new()
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
    let i18n = load_i18n();
    let raw = load_fixture("combat-state.json");
    let state1 = NormalizedState::from_raw(&raw, &i18n);
    let state2 = NormalizedState::from_raw(&raw, &i18n);

    assert_eq!(state1.stable_hash(), state2.stable_hash());
}

#[test]
fn stable_hash_different_state_different_hash() {
    let i18n = load_i18n();
    let combat = load_fixture("combat-state.json");
    let reward = load_fixture("card-reward-state.json");

    let state1 = NormalizedState::from_raw(&combat, &i18n);
    let state2 = NormalizedState::from_raw(&reward, &i18n);

    assert_ne!(state1.stable_hash(), state2.stable_hash());
}

#[test]
fn stable_hash_produces_hex() {
    let i18n = load_i18n();
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, &i18n);
    let hash = state.stable_hash();

    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn observation_hash_changes_when_draw_pile_changes() {
    let i18n = load_i18n();
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["draw_pile"][0]["uuid"] =
        Value::String("changed-draw".into());

    let state1 = NormalizedState::from_raw(&raw1, &i18n);
    let state2 = NormalizedState::from_raw(&raw2, &i18n);

    assert_ne!(state1.observation_hash(), state2.observation_hash());
}

#[test]
fn observation_hash_changes_when_discard_pile_changes() {
    let i18n = load_i18n();
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["discard_pile"] = serde_json::json!([
        {"id":"Strike_R","name":"Strike","cost":1,"type":"ATTACK","uuid":"discarded-card","upgrades":0}
    ]);

    let state1 = NormalizedState::from_raw(&raw1, &i18n);
    let state2 = NormalizedState::from_raw(&raw2, &i18n);

    assert_ne!(state1.observation_hash(), state2.observation_hash());
}

#[test]
fn observation_hash_changes_when_monster_block_changes() {
    let i18n = load_i18n();
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["monsters"][0]["block"] = Value::Number(7.into());

    let state1 = NormalizedState::from_raw(&raw1, &i18n);
    let state2 = NormalizedState::from_raw(&raw2, &i18n);

    assert_ne!(state1.observation_hash(), state2.observation_hash());
}

#[test]
fn observation_hash_changes_when_monster_power_changes() {
    let i18n = load_i18n();
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["monsters"][0]["powers"][0]["amount"] =
        Value::Number(9.into());

    let state1 = NormalizedState::from_raw(&raw1, &i18n);
    let state2 = NormalizedState::from_raw(&raw2, &i18n);

    assert_ne!(state1.observation_hash(), state2.observation_hash());
}

#[test]
fn observation_hash_changes_when_card_uuid_changes() {
    let i18n = load_i18n();
    let raw1 = load_fixture("combat-state.json");
    let mut raw2 = raw1.clone();
    raw2["game_state"]["combat_state"]["hand"][0]["uuid"] =
        Value::String("changed-hand-card".into());

    let state1 = NormalizedState::from_raw(&raw1, &i18n);
    let state2 = NormalizedState::from_raw(&raw2, &i18n);

    assert_ne!(state1.observation_hash(), state2.observation_hash());
}

#[test]
fn card_reward_filters_potion_slot() {
    let i18n = load_i18n();
    let raw = load_fixture("card-reward-state.json");
    let state = NormalizedState::from_raw(&raw, &i18n);
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
