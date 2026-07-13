use super::*;

#[test]
fn x_cost_chemical_x_enables_zero_energy_kill() {
    // Chemical X adds 2 to X. With 0 energy, X=2, damage=4*2=8 per hit, 2 hits = 16 total > 8hp
    let result = test_scan(
        &[with_xcost(
            TestCard {
                uuid: "x",
                name: "Whirlwind",
                cost: 0,
                card_type: "ATTACK",
                damage: None,
                vulnerable: None,
                strength_gain: 0,
                energy_gain: 0,
                stance: StanceEffect::None,
                mantra_gain: 0,
                execute_threshold: None,
                x_cost: false,
            },
            4,
        )],
        0,
        &[ms(0, 8, 0, false, vec![])],
        Stance::Neutral,
        0,
        2, // Chemical X bonus
        1,
        10_000_000,
    );
    assert!(
        result.is_some(),
        "Chemical X should enable zero-energy X-cost kill"
    );
}

#[test]
fn x_cost_skewer_x_plus_one_zero_energy() {
    // Skewer: 7 damage X+1 times, 0 energy, X=0, 1 hit, 7 damage vs 5hp → kill
    let result = test_scan(
        &[with_xplus(
            TestCard {
                uuid: "skewer",
                name: "Skewer",
                cost: 1,
                card_type: "ATTACK",
                damage: None,
                vulnerable: None,
                strength_gain: 0,
                energy_gain: 0,
                stance: StanceEffect::None,
                mantra_gain: 0,
                execute_threshold: None,
                x_cost: false,
            },
            7,
            1,
            T,
        )],
        0,
        &[ms(0, 5, 0, false, vec![])],
        Stance::Neutral,
        0,
        0,
        1,
        10_000_000,
    );
    assert!(
        result.is_some(),
        "Skewer with X+1 should deal 1 hit of 7 at 0 energy, killing 5hp"
    );
}

#[test]
fn x_cost_skewer_x_plus_one_with_energy() {
    // Skewer: 7 damage X+1 times, 2 energy, X=2, 3 hits, 21 damage vs 20hp → kill
    let result = test_scan(
        &[with_xplus(
            TestCard {
                uuid: "skewer",
                name: "Skewer",
                cost: 1,
                card_type: "ATTACK",
                damage: None,
                vulnerable: None,
                strength_gain: 0,
                energy_gain: 0,
                stance: StanceEffect::None,
                mantra_gain: 0,
                execute_threshold: None,
                x_cost: false,
            },
            7,
            1,
            T,
        )],
        2,
        &[ms(0, 20, 0, false, vec![])],
        Stance::Neutral,
        0,
        0,
        1,
        10_000_000,
    );
    assert!(
        result.is_some(),
        "Skewer at 2 energy should deal 3 hits of 7 = 21, killing 20hp"
    );
}
