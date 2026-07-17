use super::*;

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

#[test]
fn in_combat_uses_room_phase_even_without_active_monsters() {
    let raw = json!({
        "game_state": {
            "screen_type": "CARD_REWARD",
            "room_phase": "COMBAT",
            "combat_state": {"monsters": []}
        }
    });
    let state = NormalizedState::from_raw(&raw, test_locale());

    assert!(!state.has_active_monsters());
    assert!(state.is_in_combat());
}
