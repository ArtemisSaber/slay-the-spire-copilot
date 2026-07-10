use super::builder::*;
use super::routing::*;
use crate::locales::Locale;
use crate::state::{
    CardInfo, DangerFlags, DangerLevel, MapCoord, MonsterInfo, NormalizedState, PotionInfo,
    PowerInfo, RelicInfo, RoomType, ScreenType,
};
use crate::test_utils::card;

fn test_locale() -> Locale {
    Locale::load("zh")
}

fn path_node(symbol: &str) -> MapCoord {
    MapCoord {
        symbol: symbol.to_string(),
        x: 0,
        y: 0,
        children: vec![],
    }
}

fn path_of(symbols: &[&str]) -> Vec<MapCoord> {
    symbols.iter().map(|s| path_node(s)).collect()
}

fn make_path_evaluation(score: f64, pros: Vec<&str>, cons: Vec<&str>) -> PathEvaluation {
    PathEvaluation {
        score,
        pros: pros.into_iter().map(|s| s.to_string()).collect(),
        cons: cons.into_iter().map(|s| s.to_string()).collect(),
        description: PathDescription {
            route_chain: String::new(),
            counts: String::new(),
            annotations: vec![],
            metrics: PathMetrics {
                counts: PathCounts {
                    monsters: 0,
                    elites: 0,
                    events: 0,
                    shops: 0,
                    rests: 0,
                    treasures: 0,
                },
                shop_timing: ShopTiming::None,
                rest_before_first_elite: false,
                double_elite_without_rest: false,
                max_elite_to_rest_risk: 0.0,
            },
        },
    }
}

fn danger_safe() -> DangerFlags {
    DangerFlags {
        hp_critical: false,
        incoming_lethal: false,
        no_block_against_hit: false,
        any_monster_attacking: false,
        wrath_stance: false,
        level: DangerLevel::Safe,
    }
}

fn test_state() -> NormalizedState {
    NormalizedState {
        screen_type: Some(ScreenType::None),
        room_type: Some(RoomType::MonsterRoom),
        character: Some("IRONCLAD".to_string()),
        seed: Some(-3047511808784702860),
        ascension_level: Some(20),
        floor: Some(1),
        current_hp: Some(68),
        max_hp: Some(75),
        gold: Some(99),
        energy: Some(3),
        block: Some(6),
        powers: vec![],
        hand: vec![],
        monsters: vec![],
        card_reward_choices: vec![],
        boss_relic_choices: vec![],
        event_id: None,
        event_name: None,
        event_body: None,
        event_choices: vec![],
        relics: vec![],
        potions: vec![],
        deck_names: vec![],
        incoming_damage: 0,
        rest_options: vec![],
        danger: DangerFlags {
            hp_critical: false,
            incoming_lethal: false,
            no_block_against_hit: false,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Safe,
        },
        hand_cards: vec![],
        draw_pile: vec![],
        discard_pile: vec![],
        exhaust_cards: vec![],
        master_cards: vec![],
        map_nodes: vec![],
        map_first_node_chosen: None,
        map_current_x: None,
        map_current_y: None,
        skip_available: false,
        shop_cards: vec![],
        shop_relics: vec![],
        shop_potions: vec![],
        purge_available: false,
        purge_cost: None,
        hand_select_max_cards: None,
        hand_select_can_pick_zero: false,
        hand_select_selected: vec![],
        current_action: None,
        card_in_play: None,
        grid_cards: vec![],
        grid_selected_cards: vec![],
        grid_for_upgrade: false,
        grid_for_transform: false,
        grid_for_purge: false,
        grid_num_cards: None,
        empty_potion_slots: 0,
        ..Default::default()
    }
}

fn make_node(symbol: &str, x: i64, y: i64, children: Vec<(i64, i64)>) -> MapCoord {
    MapCoord {
        symbol: symbol.to_string(),
        x,
        y,
        children,
    }
}

mod combat_build;
mod combat_prompt;

mod format_cards_deck;
mod format_cards_piles;
mod format_cards_single;
mod format_cleaning;
mod format_inventory;
mod format_labels;
mod format_monster_intent_combat;
mod format_monster_intent_other;
mod format_monster_powers;
mod format_monster_sections;
mod format_status_combat_profile;
mod format_status_danger;
mod format_status_line;
mod format_status_turn;

mod map_crossroad;
mod map_describe;
mod map_evaluate;
mod map_labels;
mod map_routes;
mod map_suggestion;

mod screen_event;
mod screen_rest;
mod screen_rewards;
mod screen_routing;
mod screen_selection;
mod screen_shop;
