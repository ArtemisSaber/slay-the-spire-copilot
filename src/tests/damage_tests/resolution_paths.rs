use super::*;

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
