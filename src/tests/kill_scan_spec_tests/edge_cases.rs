use super::*;

#[test]
fn minion_left_after_leader_kill() {
    assert_kill(
        &[atk("a", "Strike", 1, 5, 1, T)],
        1,
        &[ms(0, 5, 0, false, vec![]), ms(2, 50, 0, true, vec![])],
    );
}

#[test]
fn slow_increases_damage_on_later_cards() {
    assert_kill(
        &[
            atk("a", "Strike A", 1, 5, 1, T),
            atk("b", "Strike B", 1, 5, 1, T),
        ],
        2,
        &[ms(0, 11, 0, false, vec![("Slow", 1)])],
    );
}

#[test]
fn no_kill_when_empty_hand() {
    assert_no_kill(&[], 3, &[ms(0, 1, 0, false, vec![])]);
}

#[test]
fn targeted_overkill_does_not_spill() {
    assert_no_kill(
        &[atk("a", "Big Strike", 1, 100, 1, T)],
        1,
        &[ms(0, 10, 0, false, vec![]), ms(1, 10, 0, false, vec![])],
    );
}

#[test]
fn block_counts_as_durability() {
    assert_no_kill(
        &[atk("a", "Strike", 1, 10, 1, T)],
        1,
        &[ms(0, 6, 5, false, vec![])],
    );
}

#[test]
fn block_exact_damage_kills() {
    assert_kill(
        &[atk("a", "Strike", 1, 10, 1, T)],
        1,
        &[ms(0, 6, 4, false, vec![])],
    );
}

#[test]
fn aoe_kills_multiple_non_minions() {
    assert_kill(
        &[atk("a", "Cleave", 1, 6, 1, A)],
        1,
        &[ms(0, 6, 0, false, vec![]), ms(1, 5, 0, false, vec![])],
    );
}

#[test]
fn aoe_leaves_non_minion_alive() {
    assert_no_kill(
        &[atk("a", "Cleave", 1, 5, 1, A)],
        1,
        &[ms(0, 5, 0, false, vec![]), ms(1, 6, 0, false, vec![])],
    );
}

#[test]
fn random_target_hits_less_than_alive_count() {
    assert_no_kill(
        &[atk("a", "Boomerang", 1, 10, 2, R)],
        1,
        &[
            ms(0, 1, 0, false, vec![]),
            ms(1, 1, 0, false, vec![]),
            ms(2, 1, 0, false, vec![]),
        ],
    );
}

#[test]
fn random_target_with_mutable_power_rejected() {
    assert_no_kill(
        &[atk("a", "Boomerang", 1, 4, 3, R)],
        1,
        &[
            ms(0, 4, 0, false, vec![("Curl Up", 6)]),
            ms(1, 4, 0, false, vec![]),
        ],
    );
}

#[test]
fn artifact_allows_second_vulnerable() {
    assert_kill(
        &[
            with_vuln(skill("a", "Trip A", 0), 2),
            with_vuln(skill("b", "Trip B", 0), 2),
            atk("c", "Strike", 1, 10, 1, T),
        ],
        1,
        &[ms(0, 15, 0, false, vec![("Artifact", 1)])],
    );
}

#[test]
fn x_cost_zero_energy_rejected() {
    assert_no_kill(
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
        &[ms(0, 1, 0, false, vec![])],
    );
}

#[test]
fn dangerous_power_fails_closed() {
    let result = test_scan(
        &[atk("a", "Strike", 1, 20, 1, T)],
        1,
        &[ms(0, 5, 0, false, vec![("Mode Shift", 30)])],
        Stance::Neutral,
        0,
        0,
        1,
        10_000_000,
    );
    assert!(
        result.is_none(),
        "unknown dangerous powers should fail closed, got {:?}",
        result
    );
}
