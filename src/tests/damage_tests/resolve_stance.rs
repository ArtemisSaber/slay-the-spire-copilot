use super::*;

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
