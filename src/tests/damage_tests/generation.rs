use super::*;

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
