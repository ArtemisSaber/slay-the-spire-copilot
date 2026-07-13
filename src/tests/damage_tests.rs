use crate::combat::damage::{
    apply_after_card_powers, apply_damage, apply_vulnerable_through_artifact, combat_ended,
    generate_plays, resolve_play,
};
use crate::combat::effects::{
    CardEffect, DamageEffect, ExhaustKind, HitCount, StanceEffect, TargetType,
};
use crate::combat::{CombatScanContext, MonsterSnapshot, PowerState, Stance};
use crate::state::CardInfo;

fn effect_no_damage() -> CardEffect {
    CardEffect {
        damage: None,
        energy_gain: 0,
        strength_gain: 0,
        vulnerable: None,
        stance: StanceEffect::None,
        mantra_gain: 0,
        execute: None,
        exhaust: ExhaustKind::None,
        x_cost: false,
    }
}

fn effect_damage(amount: i16, hits: HitCount, target_type: TargetType) -> CardEffect {
    let mut e = effect_no_damage();
    e.damage = Some(DamageEffect {
        amount,
        hits,
        target_type,
    });
    e
}

fn effect_x_cost(amount: i16, hits: HitCount, target_type: TargetType) -> CardEffect {
    let mut e = effect_damage(amount, hits, target_type);
    e.x_cost = true;
    e
}

fn ctx_with(
    energy: i16,
    monsters: Vec<MonsterSnapshot>,
    cards: Vec<CardInfo>,
) -> CombatScanContext {
    CombatScanContext {
        cards,
        energy,
        initial_stance: Stance::Neutral,
        current_stance: Stance::Neutral,
        strength_delta: 0,
        x_cost_bonus: 0,
        monsters,
        remaining_card_plays: 10,
    }
}

fn card(cost: i64, uuid: &str) -> CardInfo {
    CardInfo {
        id: "TestCard".into(),
        name: "Test".into(),
        cost,
        card_type: "ATTACK".into(),
        upgraded: false,
        uuid: Some(uuid.into()),
        description: "".into(),
        price: None,
        playable: true,
        has_target: true,
    }
}

fn ms(hp: i16, block: i16, powers: Vec<(&str, i16)>) -> MonsterSnapshot {
    MonsterSnapshot {
        command_index: 0,
        hp,
        block,
        is_minion: false,
        powers: powers
            .into_iter()
            .map(|(id, amount)| PowerState {
                id: id.into(),
                amount,
                triggered: false,
            })
            .collect(),
    }
}

fn ms_triggered(hp: i16, block: i16, powers: Vec<(&str, i16, bool)>) -> MonsterSnapshot {
    MonsterSnapshot {
        command_index: 0,
        hp,
        block,
        is_minion: false,
        powers: powers
            .into_iter()
            .map(|(id, amount, triggered)| PowerState {
                id: id.into(),
                amount,
                triggered,
            })
            .collect(),
    }
}

fn ms_with_index(
    command_index: usize,
    hp: i16,
    block: i16,
    powers: Vec<(&str, i16)>,
) -> MonsterSnapshot {
    MonsterSnapshot {
        command_index,
        hp,
        block,
        is_minion: false,
        powers: powers
            .into_iter()
            .map(|(id, amount)| PowerState {
                id: id.into(),
                amount,
                triggered: false,
            })
            .collect(),
    }
}

fn ms_minion(command_index: usize, hp: i16) -> MonsterSnapshot {
    MonsterSnapshot {
        command_index,
        hp,
        block: 0,
        is_minion: true,
        powers: vec![],
    }
}

#[path = "damage_tests/apply_damage.rs"]
mod apply_damage;
#[path = "damage_tests/combat_end.rs"]
mod combat_end;
#[path = "damage_tests/defense.rs"]
mod defense;
#[path = "damage_tests/generation.rs"]
mod generation;
#[path = "damage_tests/interactions.rs"]
mod interactions;
#[path = "damage_tests/modifiers.rs"]
mod modifiers;
#[path = "damage_tests/multi_target.rs"]
mod multi_target;
#[path = "damage_tests/reactive_powers.rs"]
mod reactive_powers;
#[path = "damage_tests/resolution_paths.rs"]
mod resolution_paths;
#[path = "damage_tests/resolve_execute.rs"]
mod resolve_execute;
#[path = "damage_tests/resolve_resources.rs"]
mod resolve_resources;
#[path = "damage_tests/resolve_stance.rs"]
mod resolve_stance;
#[path = "damage_tests/stance_edges.rs"]
mod stance_edges;
#[path = "damage_tests/vulnerable.rs"]
mod vulnerable;
