use super::*;

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
