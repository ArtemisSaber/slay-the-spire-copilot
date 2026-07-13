use super::*;

#[test]
fn energy_gain_enables_big_attack() {
    assert_kill(
        &[
            with_energy(skill("a", "Seeing Red", 0), 2),
            atk("b", "Big Attack", 2, 18, 1, T),
        ],
        1,
        &[ms(0, 18, 0, false, vec![])],
    );
}

#[test]
fn strength_setup_enables_multihit() {
    assert_kill(
        &[
            with_strength(skill("a", "Inflame", 1), 2),
            atk("b", "Twin Strike", 1, 5, 2, T),
        ],
        2,
        &[ms(0, 14, 0, false, vec![])],
    );
}

#[test]
fn calm_exit_energy_enables_big_attack() {
    assert_kill_with_stance(
        &[
            with_stance(atk("a", "Empty Fist", 1, 9, 1, T), StanceEffect::ExitStance),
            atk("b", "Big Attack", 2, 20, 1, T),
        ],
        1,
        &[ms(0, 29, 0, false, vec![])],
        Stance::Calm,
    );
}

#[test]
fn calm_to_wrath_energy_and_damage_enable_kill() {
    assert_kill_with_stance(
        &[
            with_stance(atk("a", "Eruption", 2, 9, 1, T), StanceEffect::EnterWrath),
            atk("b", "Strike", 1, 6, 1, T),
        ],
        2,
        &[ms(0, 21, 0, false, vec![])],
        Stance::Calm,
    );
}

#[test]
fn neutral_to_wrath_damage_enables_kill() {
    assert_kill_with_stance(
        &[
            with_stance(atk("a", "Eruption", 2, 9, 1, T), StanceEffect::EnterWrath),
            atk("b", "Strike", 1, 6, 1, T),
        ],
        3,
        &[ms(0, 21, 0, false, vec![])],
        Stance::Neutral,
    );
}

#[test]
fn neutral_to_wrath_still_not_enough_damage() {
    assert_no_kill(
        &[
            with_stance(atk("a", "Eruption", 2, 9, 1, T), StanceEffect::EnterWrath),
            atk("b", "Strike", 1, 6, 1, T),
        ],
        3,
        &[ms(0, 22, 0, false, vec![])],
    );
}

#[test]
fn neutral_to_divinity_damage_and_energy_enable_kill() {
    assert_kill_with_stance(
        &[
            with_mantra(skill("a", "Pray", 1), 10),
            atk("b", "Strike", 1, 6, 1, T),
        ],
        1,
        &[ms(0, 18, 0, false, vec![])],
        Stance::Neutral,
    );
}

#[test]
fn neutral_to_divinity_still_not_enough_damage() {
    assert_no_kill(
        &[
            with_mantra(skill("a", "Pray", 1), 10),
            atk("b", "Strike", 1, 6, 1, T),
        ],
        1,
        &[ms(0, 19, 0, false, vec![])],
    );
}

#[test]
fn calm_to_divinity_energy_and_damage_enable_kill() {
    assert_kill_with_stance(
        &[
            with_mantra(skill("a", "Pray", 1), 10),
            atk("b", "Big Strike", 3, 10, 1, T),
        ],
        1,
        &[ms(0, 30, 0, false, vec![])],
        Stance::Calm,
    );
}

#[test]
fn calm_to_divinity_still_not_enough_damage() {
    assert_no_kill(
        &[
            with_mantra(skill("a", "Pray", 1), 10),
            atk("b", "Big Strike", 3, 10, 1, T),
        ],
        1,
        &[ms(0, 31, 0, false, vec![])],
    );
}
