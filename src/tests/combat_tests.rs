use crate::combat::context::build_context;
use crate::combat::{Stance, can_end_fight};
use crate::state::{
    CardInfo, DangerFlags, DangerLevel, MonsterInfo, NormalizedState, PowerInfo, RelicInfo,
    RoomType, ScreenType,
};

fn state() -> NormalizedState {
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

fn strike(name: &str, uuid: &str) -> CardInfo {
    CardInfo {
        id: "Strike_R".into(),
        name: name.into(),
        cost: 1,
        card_type: "ATTACK".into(),
        upgraded: false,
        uuid: Some(uuid.into()),
        description: "造成 6 点伤害。".into(),
        price: None,
        playable: true,
        has_target: true,
    }
}

fn defend(name: &str, uuid: &str) -> CardInfo {
    CardInfo {
        id: "Defend_R".into(),
        name: name.into(),
        cost: 1,
        card_type: "SKILL".into(),
        upgraded: false,
        uuid: Some(uuid.into()),
        description: "获得 5 点 格挡 。".into(),
        price: None,
        playable: true,
        has_target: false,
    }
}

fn monster(name: &str, hp: i64, block: i64, powers: Vec<PowerInfo>, index: usize) -> MonsterInfo {
    MonsterInfo {
        name: name.into(),
        monster_id: None,
        index,
        current_hp: Some(hp),
        max_hp: Some(hp),
        block: Some(block),
        intent: None,
        damage: None,
        hits: None,
        monster_powers: powers,
        can_be_killed: false,
        is_scaling: false,
    }
}

fn power(id: &str, name: &str, amount: i64) -> PowerInfo {
    PowerInfo {
        id: id.into(),
        name: name.into(),
        amount,
    }
}

#[path = "combat_tests/context_basics.rs"]
mod context_basics;
#[path = "combat_tests/context_safety.rs"]
mod context_safety;
#[path = "combat_tests/fight_end.rs"]
mod fight_end;
#[path = "combat_tests/monster_handling.rs"]
mod monster_handling;
