use super::*;

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
