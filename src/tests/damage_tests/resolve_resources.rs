use super::*;

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
