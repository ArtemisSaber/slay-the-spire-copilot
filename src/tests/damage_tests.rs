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

// ===================== generate_plays tests =====================

#[test]
fn generate_plays_targeted_card_returns_per_monster_branches() {
    let monsters = vec![
        ms_with_index(0, 10, 0, vec![]),
        ms_with_index(1, 10, 0, vec![]),
    ];
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(3, monsters, cards);
    let effect = effect_damage(6, HitCount::Fixed(1), TargetType::Targeted);

    let branches = generate_plays(0, &effect, &ctx);
    assert_eq!(branches.len(), 2);
    assert!(branches.iter().all(|b| b.target_index.is_some()));
}

#[test]
fn generate_plays_untargeted_aoe_card_returns_one_branch() {
    let monsters = vec![
        ms_with_index(0, 10, 0, vec![]),
        ms_with_index(1, 10, 0, vec![]),
    ];
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(3, monsters, cards);
    let effect = effect_damage(6, HitCount::Fixed(1), TargetType::AoE);

    let branches = generate_plays(0, &effect, &ctx);
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].target_index, None);
}

#[test]
fn generate_plays_not_enough_energy_returns_empty() {
    let monsters = vec![ms_with_index(0, 10, 0, vec![])];
    let cards = vec![card(5, "s1")];
    let ctx = ctx_with(3, monsters, cards);
    let effect = effect_damage(6, HitCount::Fixed(1), TargetType::Targeted);

    let branches = generate_plays(0, &effect, &ctx);
    assert!(branches.is_empty());
}

#[test]
fn generate_plays_x_cost_with_negative_card_cost_clamps_to_zero() {
    let monsters = vec![ms_with_index(0, 10, 0, vec![])];
    let mut cards = vec![card(-1, "s1")];
    cards[0].cost = -1;
    let ctx = ctx_with(3, monsters, cards);
    let effect = effect_x_cost(5, HitCount::Fixed(1), TargetType::AoE);

    let branches = generate_plays(0, &effect, &ctx);
    assert_eq!(branches.len(), 1);
}

#[test]
fn generate_plays_x_cost_with_zero_cost_uses_zero() {
    let monsters = vec![ms_with_index(0, 10, 0, vec![])];
    let cards = vec![card(0, "s1")];
    let ctx = ctx_with(3, monsters, cards);
    let effect = effect_x_cost(5, HitCount::Fixed(1), TargetType::Targeted);

    let branches = generate_plays(0, &effect, &ctx);
    assert_eq!(branches.len(), 1);
}

#[test]
fn generate_plays_x_cost_with_positive_cost_used_as_required_energy() {
    let monsters = vec![ms_with_index(0, 10, 0, vec![])];
    let cards = vec![card(2, "s1")];
    let ctx = ctx_with(3, monsters, cards);
    let effect = effect_x_cost(5, HitCount::Fixed(1), TargetType::Targeted);

    let branches = generate_plays(0, &effect, &ctx);
    assert_eq!(branches.len(), 1);
}

#[test]
fn generate_plays_x_cost_too_expensive_returns_empty() {
    let monsters = vec![ms_with_index(0, 10, 0, vec![])];
    let cards = vec![card(5, "s1")];
    let ctx = ctx_with(3, monsters, cards);
    let effect = effect_x_cost(5, HitCount::Fixed(1), TargetType::Targeted);

    let branches = generate_plays(0, &effect, &ctx);
    assert!(branches.is_empty());
}

#[test]
fn generate_plays_skips_dead_monsters() {
    let monsters = vec![
        ms_with_index(0, 0, 0, vec![]),
        ms_with_index(1, 10, 0, vec![]),
    ];
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(3, monsters, cards);
    let effect = effect_damage(6, HitCount::Fixed(1), TargetType::Targeted);

    let branches = generate_plays(0, &effect, &ctx);
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].target_index, Some(1));
}

#[test]
fn generate_plays_vulnerable_makes_card_targeted() {
    let monsters = vec![
        ms_with_index(0, 10, 0, vec![]),
        ms_with_index(1, 10, 0, vec![]),
    ];
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(3, monsters, cards);
    let mut effect = effect_no_damage();
    effect.vulnerable = Some(2);

    let branches = generate_plays(0, &effect, &ctx);
    assert_eq!(branches.len(), 2);
}

#[test]
fn generate_plays_execute_makes_card_targeted() {
    let monsters = vec![
        ms_with_index(0, 10, 0, vec![]),
        ms_with_index(1, 10, 0, vec![]),
    ];
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(3, monsters, cards);
    let mut effect = effect_no_damage();
    effect.execute = Some(30);

    let branches = generate_plays(0, &effect, &ctx);
    assert_eq!(branches.len(), 2);
}

#[test]
fn generate_plays_all_monsters_dead_returns_empty() {
    let monsters = vec![ms_with_index(0, 0, 0, vec![])];
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(3, monsters, cards);
    let effect = effect_damage(6, HitCount::Fixed(1), TargetType::Targeted);

    let branches = generate_plays(0, &effect, &ctx);
    assert!(branches.is_empty());
}

// ===================== resolve_play tests (execution path) =====================

#[test]
fn resolve_play_execute_kills_monster() {
    let monster = ms(5, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(3, vec![monster], cards);
    let mut effect = effect_no_damage();
    effect.execute = Some(10);

    let result = resolve_play(0, Some(0), &effect, &ctx);
    assert_eq!(result.monsters[0].hp, 0);
}

#[test]
fn resolve_play_execute_no_kill_when_hp_above_threshold() {
    let monster = ms(15, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(3, vec![monster], cards);
    let mut effect = effect_no_damage();
    effect.execute = Some(10);

    let result = resolve_play(0, Some(0), &effect, &ctx);
    assert_eq!(result.monsters[0].hp, 15);
}

#[test]
fn resolve_play_execute_no_effect_when_hp_already_zero() {
    let monster = ms(0, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(3, vec![monster], cards);
    let mut effect = effect_no_damage();
    effect.execute = Some(10);

    let result = resolve_play(0, Some(0), &effect, &ctx);
    assert_eq!(result.monsters[0].hp, 0);
}

#[test]
fn resolve_play_execute_does_not_trigger_without_target() {
    let monster = ms(5, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(3, vec![monster], cards);
    let mut effect = effect_no_damage();
    effect.execute = Some(10);

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.monsters[0].hp, 5);
}

// ===================== resolve_play (x-cost with chemical X) =====================

#[test]
fn resolve_play_x_cost_fixed_hit_uses_energy_and_bonus() {
    let monster = ms(100, 0, vec![]);
    let cards = vec![card(0, "s1")];
    let mut ctx = ctx_with(3, vec![monster], cards);
    ctx.x_cost_bonus = 2;
    let effect = effect_x_cost(10, HitCount::Fixed(1), TargetType::Targeted);

    let result = resolve_play(0, Some(0), &effect, &ctx);
    assert_eq!(result.energy, 0);
    assert_eq!(result.x_cost_bonus, 2);
}

#[test]
fn resolve_play_x_cost_x_times_uses_energy_and_bonus() {
    let monster = ms(100, 0, vec![]);
    let cards = vec![card(0, "s1")];
    let mut ctx = ctx_with(3, vec![monster], cards);
    ctx.x_cost_bonus = 2;
    let effect = effect_x_cost(5, HitCount::XTimes, TargetType::Targeted);

    let result = resolve_play(0, Some(0), &effect, &ctx);
    assert_eq!(result.energy, 0);
}

#[test]
fn resolve_play_x_cost_x_plus_uses_energy_and_bonus() {
    let monster = ms(100, 0, vec![]);
    let cards = vec![card(0, "s1")];
    let mut ctx = ctx_with(3, vec![monster], cards);
    ctx.x_cost_bonus = 2;
    let effect = effect_x_cost(7, HitCount::XPlus(1), TargetType::Targeted);

    let result = resolve_play(0, Some(0), &effect, &ctx);
    assert_eq!(result.energy, 0);
}

#[test]
fn resolve_play_x_cost_without_bonus_uses_energy_only() {
    let monster = ms(100, 0, vec![]);
    let cards = vec![card(0, "s1")];
    let ctx = ctx_with(3, vec![monster], cards);
    let effect = effect_x_cost(8, HitCount::XTimes, TargetType::Targeted);

    let result = resolve_play(0, Some(0), &effect, &ctx);
    assert_eq!(result.energy, 0);
}

// ===================== resolve_play stance transitions =====================

#[test]
fn resolve_play_enter_wrath_stance() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(3, vec![monster], cards);
    let mut effect = effect_no_damage();
    effect.stance = StanceEffect::EnterWrath;

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.current_stance, Stance::Wrath);
}

#[test]
fn resolve_play_enter_calm_stance() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(3, vec![monster], cards);
    let mut effect = effect_no_damage();
    effect.stance = StanceEffect::EnterCalm;

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.current_stance, Stance::Calm);
}

#[test]
fn resolve_play_enter_divinity_stance() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(3, vec![monster], cards);
    let mut effect = effect_no_damage();
    effect.stance = StanceEffect::EnterDivinity;

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.current_stance, Stance::Divinity);
}

#[test]
fn resolve_play_exit_stance_returns_to_neutral() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let mut ctx = ctx_with(2, vec![monster], cards);
    ctx.current_stance = Stance::Wrath;
    let mut effect = effect_no_damage();
    effect.stance = StanceEffect::ExitStance;

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.current_stance, Stance::Neutral);
}

#[test]
fn resolve_play_calm_to_wrath_gives_energy() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let mut ctx = ctx_with(2, vec![monster], cards);
    ctx.current_stance = Stance::Calm;
    let mut effect = effect_no_damage();
    effect.stance = StanceEffect::EnterWrath;

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.energy, 3);
}

#[test]
fn resolve_play_neutral_to_divinity_gives_energy() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(2, vec![monster], cards);
    let mut effect = effect_no_damage();
    effect.stance = StanceEffect::EnterDivinity;

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.energy, 4);
}

#[test]
fn resolve_play_calm_to_divinity_gives_both_bonuses() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let mut ctx = ctx_with(2, vec![monster], cards);
    ctx.current_stance = Stance::Calm;
    let mut effect = effect_no_damage();
    effect.stance = StanceEffect::EnterDivinity;

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.energy, 6);
}

#[test]
fn resolve_play_calm_to_neutral_exit_gives_energy() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let mut ctx = ctx_with(2, vec![monster], cards);
    ctx.current_stance = Stance::Calm;
    let mut effect = effect_no_damage();
    effect.stance = StanceEffect::ExitStance;

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.energy, 3);
}

#[test]
fn resolve_play_energy_gain_added() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(2, vec![monster], cards);
    let mut effect = effect_no_damage();
    effect.energy_gain = 2;

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.energy, 3);
}

#[test]
fn resolve_play_strength_gain_added() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(2, vec![monster], cards);
    let mut effect = effect_no_damage();
    effect.strength_gain = 3;

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.strength_delta, 3);
}

#[test]
fn resolve_play_remaining_card_plays_decremented() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(2, vec![monster], cards);
    let effect = effect_no_damage();

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.remaining_card_plays, 9);
}

// ===================== apply_damage tests =====================

#[test]
fn apply_damage_fixed_hit_with_x_value() {
    let mut monsters = vec![ms(20, 0, vec![])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), Some(3), &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 5);
}

#[test]
fn apply_damage_x_times_with_outstanding_x_value() {
    let mut monsters = vec![ms(60, 0, vec![])];
    let dmg = DamageEffect {
        amount: 6,
        hits: HitCount::XTimes,
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), Some(3), &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 6);
}

#[test]
fn apply_damage_x_plus_with_offset() {
    let mut monsters = vec![ms(30, 0, vec![])];
    let dmg = DamageEffect {
        amount: 7,
        hits: HitCount::XPlus(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), Some(2), &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 9);
}

#[test]
fn apply_damage_x_times_default_zero_when_no_x_value() {
    let mut monsters = vec![ms(20, 0, vec![])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::XTimes,
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 20);
}

#[test]
fn apply_damage_x_plus_default_zero_when_no_x_value() {
    let mut monsters = vec![ms(20, 0, vec![])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::XPlus(2),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 10);
}

#[test]
fn apply_damage_aoe_skips_dead_monsters() {
    let mut monsters = vec![ms(0, 0, vec![]), ms(10, 0, vec![])];
    let dmg = DamageEffect {
        amount: 6,
        hits: HitCount::Fixed(1),
        target_type: TargetType::AoE,
    };
    apply_damage(&dmg, None, None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 0);
    assert_eq!(monsters[1].hp, 4);
}

#[test]
fn apply_damage_targeted_stops_when_target_dies_mid_hit() {
    let mut monsters = vec![ms(5, 0, vec![])];
    let dmg = DamageEffect {
        amount: 6,
        hits: HitCount::Fixed(3),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 0);
}

#[test]
fn apply_damage_random_target_stops_when_all_dead() {
    let mut monsters = vec![ms(6, 0, vec![])];
    let dmg = DamageEffect {
        amount: 7,
        hits: HitCount::Fixed(1),
        target_type: TargetType::RandomTarget,
    };
    apply_damage(&dmg, None, None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 0);
}

#[test]
fn apply_damage_random_target_no_living_breaks() {
    let mut monsters = vec![ms(0, 0, vec![])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(5),
        target_type: TargetType::RandomTarget,
    };
    apply_damage(&dmg, None, None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 0);
}

// ===================== calc_effective_damage edge cases =====================

#[test]
fn intangible_caps_damage_at_1_even_with_wrath_and_strength() {
    let monster = ms(100, 0, vec![("Intangible", 1)]);
    let effective = super::calc_effective_damage(10, &monster, Stance::Wrath, 5);
    assert_eq!(effective, 1);
}

#[test]
fn intangible_caps_at_1_even_with_vulnerable() {
    let monster = ms(100, 0, vec![("Intangible", 1), ("Vulnerable", 1)]);
    let effective = super::calc_effective_damage(6, &monster, Stance::Neutral, 0);
    assert_eq!(effective, 1);
}

#[test]
fn slow_increases_damage() {
    let monster = ms(20, 0, vec![("Slow", 5)]);
    let effective = super::calc_effective_damage(10, &monster, Stance::Neutral, 0);
    assert!(effective > 10);
}

#[test]
fn slow_chinese_id_applies() {
    let monster = ms(20, 0, vec![("缓慢", 5), ("Slow", 1)]);
    let effective = super::calc_effective_damage(10, &monster, Stance::Neutral, 0);
    assert!(effective > 10);
}

#[test]
fn vulnerable_multiplies_damage() {
    let monster = ms(20, 0, vec![("Vulnerable", 1)]);
    let effective = super::calc_effective_damage(10, &monster, Stance::Neutral, 0);
    assert_eq!(effective, 15);
}

#[test]
fn vulnerable_chinese_id_multiplies_damage() {
    let monster = ms(20, 0, vec![("易伤", 1)]);
    let effective = super::calc_effective_damage(10, &monster, Stance::Neutral, 0);
    assert_eq!(effective, 15);
}

#[test]
fn flight_halves_damage() {
    let monster = ms(20, 0, vec![("Flight", 3)]);
    let effective = super::calc_effective_damage(10, &monster, Stance::Neutral, 0);
    assert_eq!(effective, 5);
}

#[test]
fn strength_adds_before_multipliers() {
    let monster = ms(20, 0, vec![("Vulnerable", 1)]);
    let effective = super::calc_effective_damage(10, &monster, Stance::Neutral, 4);
    assert_eq!(effective, 21);
}

#[test]
fn wrath_doubles_base_damage() {
    let monster = ms(20, 0, vec![]);
    let effective = super::calc_effective_damage(6, &monster, Stance::Wrath, 0);
    assert_eq!(effective, 12);
}

#[test]
fn divinity_triples_base_damage() {
    let monster = ms(20, 0, vec![]);
    let effective = super::calc_effective_damage(6, &monster, Stance::Divinity, 0);
    assert_eq!(effective, 18);
}

#[test]
fn damage_floor_at_zero() {
    let monster = ms(20, 0, vec![]);
    let effective = super::calc_effective_damage(1, &monster, Stance::Neutral, -10);
    assert_eq!(effective, 0);
}

// ===================== apply_block tests =====================

#[test]
fn block_exact_match_zero_overflow() {
    // Test via a targeted hit where block == damage
    let mut monsters = vec![ms(20, 5, vec![])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 20);
    assert_eq!(monsters[0].block, 0);
}

#[test]
fn block_overflow_hp_reduction() {
    let mut monsters = vec![ms(20, 3, vec![])];
    let dmg = DamageEffect {
        amount: 10,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 13);
    assert_eq!(monsters[0].block, 0);
}

#[test]
fn block_completely_absorbs_damage() {
    let mut monsters = vec![ms(20, 20, vec![])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 20);
    assert_eq!(monsters[0].block, 15);
}

// ===================== invincible tests =====================

#[test]
fn invincible_caps_damage_per_hit() {
    let mut monsters = vec![ms(50, 0, vec![("Invincible", 5)])];
    let dmg = DamageEffect {
        amount: 20,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 45);
}

#[test]
fn invincible_amount_decreases_after_capping() {
    let mut monsters = vec![ms(50, 0, vec![("Invincible", 8)])];
    let dmg = DamageEffect {
        amount: 15,
        hits: HitCount::Fixed(2),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    let inv = monsters[0]
        .powers
        .iter()
        .find(|p| p.id == "Invincible")
        .unwrap();
    assert!(inv.amount < 8);
}

#[test]
fn invincible_does_not_affect_zero_damage_hits() {
    let mut monsters = vec![ms(50, 0, vec![("Invincible", 5)])];
    let dmg = DamageEffect {
        amount: 1,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    monsters[0].block = 5;
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    let inv = monsters[0]
        .powers
        .iter()
        .find(|p| p.id == "Invincible")
        .unwrap();
    assert_eq!(inv.amount, 5);
}

// ===================== curl up tests =====================

#[test]
fn curl_up_triggers_and_adds_block_after_hits() {
    let mut monsters = vec![ms_triggered(20, 0, vec![("Curl Up", 10, false)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(3),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 5);
    assert_eq!(monsters[0].block, 10);
}

#[test]
fn curl_up_applies_block_after_hit_then_zeroes() {
    let mut monsters = vec![ms_triggered(20, 0, vec![("Curl Up", 5, false)])];
    let dmg = DamageEffect {
        amount: 8,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 12);
    assert_eq!(monsters[0].block, 5);
}

#[test]
fn curl_up_already_triggered_still_adds_block_once() {
    let mut monsters = vec![ms_triggered(20, 0, vec![("Curl Up", 10, true)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(3),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 5);
    assert_eq!(monsters[0].block, 10);
}

#[test]
fn curl_up_with_zero_amount_does_not_trigger() {
    let mut monsters = vec![ms_triggered(20, 0, vec![("Curl Up", 0, false)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 15);
    assert_eq!(monsters[0].block, 0);
}

// ===================== malleable tests =====================

#[test]
fn malleable_adds_block_on_unblocked_damage() {
    let mut monsters = vec![ms(30, 0, vec![("Malleable", 3)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].block, 3);
    let m = monsters[0]
        .powers
        .iter()
        .find(|p| p.id == "Malleable")
        .unwrap();
    assert_eq!(m.amount, 4);
}

#[test]
fn malleable_does_not_trigger_when_all_blocked() {
    let mut monsters = vec![ms(30, 10, vec![("Malleable", 3)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].block, 5);
    let m = monsters[0]
        .powers
        .iter()
        .find(|p| p.id == "Malleable")
        .unwrap();
    assert_eq!(m.amount, 3);
}

#[test]
fn malleable_with_zero_amount_not_found() {
    let mut monsters = vec![ms(30, 0, vec![("Malleable", 0)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].block, 0);
}

// ===================== post-hit effects (Flight) =====================

#[test]
fn flight_decrements_on_hit_whether_unblocked_or_not() {
    let mut monsters = vec![ms(30, 10, vec![("Flight", 4)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    let f = monsters[0]
        .powers
        .iter()
        .find(|p| p.id == "Flight")
        .unwrap();
    assert_eq!(f.amount, 3);
}

#[test]
fn flight_floor_at_zero_does_not_go_negative() {
    let mut monsters = vec![ms(30, 0, vec![("Flight", 0)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    let f = monsters[0]
        .powers
        .iter()
        .find(|p| p.id == "Flight")
        .unwrap();
    assert_eq!(f.amount, 0);
}

// ===================== apply_vulnerable_through_artifact tests =====================

#[test]
fn artifact_blocks_vulnerable_and_decrements() {
    let mut monster = ms(20, 0, vec![("Artifact", 2)]);
    apply_vulnerable_through_artifact(&mut monster, 3);
    let a = monster.powers.iter().find(|p| p.id == "Artifact").unwrap();
    assert_eq!(a.amount, 1);
    let has_vuln = monster.powers.iter().any(|p| p.id == "Vulnerable");
    assert!(!has_vuln);
}

#[test]
fn artifact_floor_at_zero_when_amount_would_go_negative() {
    let mut monster = ms(20, 0, vec![("Artifact", 0)]);
    apply_vulnerable_through_artifact(&mut monster, 3);
    let a = monster.powers.iter().find(|p| p.id == "Artifact").unwrap();
    assert_eq!(a.amount, 0);
}

#[test]
fn vulnerable_stacking_on_existing() {
    let mut monster = ms(20, 0, vec![("Vulnerable", 2)]);
    apply_vulnerable_through_artifact(&mut monster, 3);
    let v = monster
        .powers
        .iter()
        .find(|p| p.id == "Vulnerable")
        .unwrap();
    assert_eq!(v.amount, 5);
}

#[test]
fn vulnerable_creates_new_stack_when_not_present() {
    let mut monster = ms(20, 0, vec![]);
    apply_vulnerable_through_artifact(&mut monster, 3);
    let v = monster
        .powers
        .iter()
        .find(|p| p.id == "Vulnerable")
        .unwrap();
    assert_eq!(v.amount, 3);
}

#[test]
fn vulnerable_chinese_id_stacking() {
    let mut monster = ms(20, 0, vec![("易伤", 2)]);
    apply_vulnerable_through_artifact(&mut monster, 1);
    let v = monster.powers.iter().find(|p| p.id == "易伤").unwrap();
    assert_eq!(v.amount, 3);
}

// ===================== apply_after_card_powers tests =====================

#[test]
fn slow_scales_up_after_card() {
    let mut monsters = vec![ms(20, 0, vec![("Slow", 3)])];
    apply_after_card_powers(&mut monsters);
    let s = monsters[0].powers.iter().find(|p| p.id == "Slow").unwrap();
    assert_eq!(s.amount, 4);
}

#[test]
fn slow_chinese_id_scales_up() {
    let mut monsters = vec![ms(20, 0, vec![("缓慢", 1)])];
    apply_after_card_powers(&mut monsters);
    let s = monsters[0].powers.iter().find(|p| p.id == "缓慢").unwrap();
    assert_eq!(s.amount, 2);
}

#[test]
fn slow_zero_amount_not_incremented() {
    let mut monsters = vec![ms(20, 0, vec![("Slow", 0)])];
    apply_after_card_powers(&mut monsters);
    let s = monsters[0].powers.iter().find(|p| p.id == "Slow").unwrap();
    assert_eq!(s.amount, 0);
}

// ===================== combat_ended edge cases =====================

#[test]
fn combat_ended_hp_exactly_zero_counts_as_dead() {
    let monsters = vec![MonsterSnapshot {
        command_index: 0,
        hp: 0,
        block: 10,
        is_minion: false,
        powers: vec![],
    }];
    assert!(combat_ended(&monsters));
}

#[test]
fn combat_ended_hp_negative_counts_as_dead() {
    let monsters = vec![MonsterSnapshot {
        command_index: 0,
        hp: -1,
        block: 0,
        is_minion: false,
        powers: vec![],
    }];
    assert!(combat_ended(&monsters));
}

#[test]
fn combat_ended_one_alive_dead_and_minion() {
    let monsters = vec![
        MonsterSnapshot {
            command_index: 0,
            hp: 5,
            block: 0,
            is_minion: true,
            powers: vec![],
        },
        MonsterSnapshot {
            command_index: 1,
            hp: 0,
            block: 0,
            is_minion: false,
            powers: vec![],
        },
    ];
    assert!(combat_ended(&monsters));
}

#[test]
fn combat_ended_multiple_minions_all_alive() {
    let monsters = vec![ms_minion(0, 10), ms_minion(1, 20)];
    assert!(combat_ended(&monsters));
}

#[test]
fn combat_ended_empty_returns_false() {
    assert!(!combat_ended(&[]));
}

#[test]
fn combat_ended_one_alive_non_minion_returns_false() {
    let monsters = vec![ms(10, 0, vec![])];
    assert!(!combat_ended(&monsters));
}

// ===================== resolve_play vulnerable path =====================

#[test]
fn resolve_play_vulnerable_with_target_applies() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(2, vec![monster], cards);
    let mut effect = effect_no_damage();
    effect.vulnerable = Some(2);

    let result = resolve_play(0, Some(0), &effect, &ctx);
    let v = result.monsters[0]
        .powers
        .iter()
        .find(|p| p.id == "Vulnerable")
        .unwrap();
    assert_eq!(v.amount, 2);
}

#[test]
fn resolve_play_vulnerable_without_target_skips() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(2, vec![monster], cards);
    let mut effect = effect_no_damage();
    effect.vulnerable = Some(2);

    let result = resolve_play(0, None, &effect, &ctx);
    let has_vuln = result.monsters[0]
        .powers
        .iter()
        .any(|p| p.id == "Vulnerable");
    assert!(!has_vuln);
}

// ===================== resolve_play x_cost energy =====================

#[test]
fn resolve_play_x_cost_consumes_all_energy() {
    let monster = ms(20, 0, vec![]);
    let cards = vec![card(2, "s1")];
    let ctx = ctx_with(5, vec![monster], cards);
    let effect = effect_x_cost(6, HitCount::Fixed(1), TargetType::Targeted);

    let result = resolve_play(0, Some(0), &effect, &ctx);
    assert_eq!(result.energy, 0);
}

#[test]
fn resolve_play_non_x_cost_deducts_energy() {
    let monster = ms(20, 0, vec![]);
    let cards = vec![card(2, "s1")];
    let ctx = ctx_with(5, vec![monster], cards);
    let effect = effect_damage(6, HitCount::Fixed(1), TargetType::Targeted);

    let result = resolve_play(0, Some(0), &effect, &ctx);
    assert_eq!(result.energy, 3);
}

// ===================== apply_damage random target with multiple hits =====================

#[test]
fn apply_damage_random_target_first_hit_kills_next_hits_next() {
    let mut monsters = vec![
        // Note: random target always picks living[0] (first living)
        ms(10, 0, vec![]),
        ms(10, 0, vec![]),
    ];
    let dmg = DamageEffect {
        amount: 6,
        hits: HitCount::Fixed(4),
        target_type: TargetType::RandomTarget,
    };
    apply_damage(&dmg, None, None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 0);
    assert_eq!(monsters[1].hp, 0);
}

// ===================== apply_damage AoE multi-hit =====================

#[test]
fn apply_damage_aoe_multi_hit_hits_all_living() {
    let mut monsters = vec![ms(20, 0, vec![]), ms(20, 0, vec![])];
    let dmg = DamageEffect {
        amount: 6,
        hits: HitCount::Fixed(2),
        target_type: TargetType::AoE,
    };
    apply_damage(&dmg, None, None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 8);
    assert_eq!(monsters[1].hp, 8);
}

#[test]
fn apply_damage_aoe_mid_hit_death_excludes_from_later_hits() {
    let mut monsters = vec![ms(5, 0, vec![]), ms(20, 0, vec![])];
    let dmg = DamageEffect {
        amount: 6,
        hits: HitCount::Fixed(2),
        target_type: TargetType::AoE,
    };
    apply_damage(&dmg, None, None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 0);
    assert_eq!(monsters[1].hp, 8);
}

// ===================== calc_effective with multiple status effects =====================

#[test]
fn slow_and_vulnerable_stacking() {
    let monster = ms(20, 0, vec![("Slow", 3), ("Vulnerable", 1)]);
    let effective = super::calc_effective_damage(10, &monster, Stance::Neutral, 0);
    assert!(effective > 15);
}

#[test]
fn flight_and_vulnerable_cancel_partially() {
    let monster = ms(20, 0, vec![("Flight", 3), ("Vulnerable", 1)]);
    let effective = super::calc_effective_damage(10, &monster, Stance::Neutral, 0);
    assert_eq!(effective, 7);
}

// ===================== resolve_play energy and stance edge cases =====================

#[test]
fn resolve_play_wrath_to_wrath_no_energy_bonus() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let mut ctx = ctx_with(3, vec![monster], cards);
    ctx.current_stance = Stance::Wrath;
    let mut effect = effect_no_damage();
    effect.stance = StanceEffect::EnterWrath;

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.current_stance, Stance::Wrath);
    assert_eq!(result.energy, 2);
}

#[test]
fn resolve_play_calm_stays_calm_no_energy() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let mut ctx = ctx_with(3, vec![monster], cards);
    ctx.current_stance = Stance::Calm;
    let mut effect = effect_no_damage();
    effect.stance = StanceEffect::EnterCalm;

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.current_stance, Stance::Calm);
    assert_eq!(result.energy, 2);
}

#[test]
fn resolve_play_divinity_to_wrath() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let mut ctx = ctx_with(3, vec![monster], cards);
    ctx.current_stance = Stance::Divinity;
    let mut effect = effect_no_damage();
    effect.stance = StanceEffect::EnterWrath;

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.current_stance, Stance::Wrath);
    assert_eq!(result.energy, 2);
}

#[test]
fn resolve_play_stance_none_keeps_current_stance() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let mut ctx = ctx_with(3, vec![monster], cards);
    ctx.current_stance = Stance::Wrath;
    let effect = effect_no_damage();

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.current_stance, Stance::Wrath);
}

// ===================== apply_damage targeted curlup multi-hit =====================

#[test]
fn targeted_damage_with_curl_up_block_applied_after_hits() {
    let mut monsters = vec![ms_triggered(15, 0, vec![("Curl Up", 8, false)])];
    let dmg = DamageEffect {
        amount: 10,
        hits: HitCount::Fixed(2),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 0);
    assert_eq!(monsters[0].block, 8);
}

// ===================== resolve_play complex interactions =====================

#[test]
fn resolve_play_damage_then_execute() {
    let monster = ms(6, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(3, vec![monster], cards);
    let mut effect = effect_damage(4, HitCount::Fixed(1), TargetType::Targeted);
    effect.execute = Some(5);

    let result = resolve_play(0, Some(0), &effect, &ctx);
    assert_eq!(result.monsters[0].hp, 0);
}

#[test]
fn resolve_play_damage_not_enough_for_execute_doesnt_execute() {
    let monster = ms(6, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(3, vec![monster], cards);
    let mut effect = effect_damage(4, HitCount::Fixed(1), TargetType::Targeted);
    effect.execute = Some(1);

    let result = resolve_play(0, Some(0), &effect, &ctx);
    assert_ne!(result.monsters[0].hp, 0);
}

#[test]
fn resolve_play_vulnerable_and_damage_together() {
    let monster = ms(20, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let ctx = ctx_with(3, vec![monster], cards);
    let mut effect = effect_damage(6, HitCount::Fixed(1), TargetType::Targeted);
    effect.vulnerable = Some(2);

    let result = resolve_play(0, Some(0), &effect, &ctx);
    assert_eq!(result.monsters[0].hp, 14);
    let v = result.monsters[0]
        .powers
        .iter()
        .find(|p| p.id == "Vulnerable")
        .unwrap();
    assert_eq!(v.amount, 2);
}

#[test]
fn resolve_play_energy_gain_and_cost_net() {
    let monster = ms(20, 0, vec![]);
    let cards = vec![card(2, "s1")];
    let ctx = ctx_with(5, vec![monster], cards);
    let mut effect = effect_damage(6, HitCount::Fixed(1), TargetType::Targeted);
    effect.energy_gain = 2;

    let result = resolve_play(0, Some(0), &effect, &ctx);
    assert_eq!(result.energy, 5);
}

#[test]
fn resolve_play_remaining_plays_zero_saturates() {
    let monster = ms(10, 0, vec![]);
    let cards = vec![card(1, "s1")];
    let mut ctx = ctx_with(2, vec![monster], cards);
    ctx.remaining_card_plays = 0;
    let effect = effect_no_damage();

    let result = resolve_play(0, None, &effect, &ctx);
    assert_eq!(result.remaining_card_plays, 0);
}

// ===================== apply_damage block + invincible interaction =====================

#[test]
fn block_then_invincible_caps_overflow() {
    let mut monsters = vec![ms(50, 10, vec![("Invincible", 5)])];
    let dmg = DamageEffect {
        amount: 20,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 45);
    assert_eq!(monsters[0].block, 0);
}

#[test]
fn block_below_damage_invincible_hasnt_reached_cap() {
    let mut monsters = vec![ms(50, 3, vec![("Invincible", 8)])];
    let dmg = DamageEffect {
        amount: 10,
        hits: HitCount::Fixed(1),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 43);
}

// ===================== resolve_play x_cost bonus propagation =====================

#[test]
fn resolve_play_x_cost_bonus_is_preserved_in_result() {
    let monster = ms(20, 0, vec![]);
    let cards = vec![card(0, "s1")];
    let mut ctx = ctx_with(3, vec![monster], cards);
    ctx.x_cost_bonus = 2;
    let effect = effect_x_cost(5, HitCount::XTimes, TargetType::Targeted);

    let result = resolve_play(0, Some(0), &effect, &ctx);
    assert_eq!(result.x_cost_bonus, 2);
}

// ===================== generate_plays with bonus edge cases =====================

#[test]
fn generate_plays_x_cost_bonus_not_used_for_energy_check() {
    let monsters = vec![ms_with_index(0, 20, 0, vec![])];
    let cards = vec![card(4, "s1")];
    let mut ctx = ctx_with(3, monsters, cards);
    ctx.x_cost_bonus = 2;
    let effect = effect_x_cost(5, HitCount::Fixed(1), TargetType::Targeted);

    let branches = generate_plays(0, &effect, &ctx);
    assert!(branches.is_empty());
}

// ===================== apply_damage combined status on multi-hit =====================

#[test]
fn vulnerable_multi_hit_applies_to_each_hit() {
    let mut monsters = vec![ms(30, 0, vec![("Vulnerable", 1)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(2),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 16);
}

#[test]
fn flight_multi_hit_applies_to_each_hit() {
    let mut monsters = vec![ms(20, 0, vec![("Flight", 3)])];
    let dmg = DamageEffect {
        amount: 5,
        hits: HitCount::Fixed(2),
        target_type: TargetType::Targeted,
    };
    apply_damage(&dmg, Some(0), None, &mut monsters, Stance::Neutral, 0);
    assert_eq!(monsters[0].hp, 16);
}
