use super::*;

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
