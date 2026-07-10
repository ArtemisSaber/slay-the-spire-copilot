use super::*;
use crate::state::{DangerFlags, RelicInfo};
use crate::test_utils::{load_fixture, test_locale};
use serde_json::{Value, json};

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

mod core;

mod danger;

mod extract_cards;
mod extract_combat_state;
mod extract_events;
mod extract_inventory;
mod extract_monsters_core;
mod extract_monsters_intent;
mod extract_monsters_killable;
mod extract_monsters_scaling_core;
mod extract_monsters_scaling_powers;
mod extract_normalize_event;
mod extract_normalize_screens;
mod extract_selection;

mod map_nodes;
mod screen_room_types;

mod stable_hash_combat;
mod stable_hash_core;
mod stable_hash_screens;
