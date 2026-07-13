use crate::ranker::context::{ActionContext, ActionType};
use crate::state::{
    CardInfo, MonsterInfo, NormalizedState, PotionInfo, PowerInfo, RelicInfo, ScreenType,
};

fn make_state(hand: Vec<CardInfo>, monsters: Vec<MonsterInfo>) -> NormalizedState {
    NormalizedState {
        hand,
        monsters,
        energy: Some(3),
        block: Some(5),
        current_hp: Some(60),
        max_hp: Some(75),
        incoming_damage: 6,
        screen_type: Some(ScreenType::None),
        ..Default::default()
    }
}

fn strike() -> CardInfo {
    CardInfo {
        id: "Strike_R".into(),
        name: "Strike".into(),
        cost: 1,
        card_type: "ATTACK".into(),
        description: "造成 6 点伤害".into(),
        uuid: Some("uuid-strike".into()),
        has_target: true,
        playable: true,
        upgraded: false,
        price: None,
    }
}

fn defend() -> CardInfo {
    CardInfo {
        id: "Defend_R".into(),
        name: "Defend".into(),
        cost: 1,
        card_type: "SKILL".into(),
        description: "获得 5 点 格挡".into(),
        uuid: Some("uuid-defend".into()),
        has_target: false,
        playable: true,
        upgraded: false,
        price: None,
    }
}

fn jaw_worm() -> MonsterInfo {
    MonsterInfo {
        name: "Jaw Worm".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(44),
        max_hp: Some(46),
        block: Some(0),
        intent: Some("ATTACK".into()),
        damage: Some(12),
        hits: Some(1),
        is_scaling: false,
        can_be_killed: false,
        monster_powers: vec![],
    }
}

fn find_play_context(contexts: &[ActionContext]) -> &ActionContext {
    contexts
        .iter()
        .find(|c| matches!(c.action_type, ActionType::PlayCard { .. }))
        .unwrap()
}

fn card_with_desc(desc: &str) -> CardInfo {
    CardInfo {
        id: "Test".into(),
        name: "Test".into(),
        cost: 1,
        card_type: "SKILL".into(),
        description: desc.into(),
        uuid: Some("uuid-test".into()),
        has_target: false,
        playable: true,
        upgraded: false,
        price: None,
    }
}

#[path = "context_tests/aoe.rs"]
mod aoe;
#[path = "context_tests/core.rs"]
mod core;
#[path = "context_tests/detection.rs"]
mod detection;
#[path = "context_tests/end_turn.rs"]
mod end_turn;
#[path = "context_tests/parsed_effects.rs"]
mod parsed_effects;
#[path = "context_tests/potions.rs"]
mod potions;
#[path = "context_tests/state_vars.rs"]
mod state_vars;
