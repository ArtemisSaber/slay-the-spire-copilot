use super::*;

#[test]
fn curl_up_lethal_single_hit() {
    // Curl Up block is added AFTER the card, so the single lethal hit kills first
    assert_kill(
        &[atk("a", "Big Strike", 1, 12, 1, T)],
        1,
        &[ms(0, 12, 0, false, vec![("Curl Up", 6)])],
    );
}

#[test]
fn curl_up_nonlethal_blocks_later_card() {
    assert_no_kill(
        &[
            atk("a", "Strike A", 1, 5, 1, T),
            atk("b", "Strike B", 1, 5, 1, T),
        ],
        2,
        &[ms(0, 8, 0, false, vec![("Curl Up", 6)])],
    );
}

#[test]
fn curl_up_multihit_same_card_kills() {
    // Curl Up block is added AFTER the card, so all 5 hits resolve before block
    assert_kill(
        &[atk("a", "Five Cuts", 1, 2, 5, T)],
        1,
        &[ms(0, 9, 0, false, vec![("Curl Up", 5)])],
    );
}

#[test]
fn artifact_blocks_vulnerable_setup() {
    assert_no_kill(
        &[
            with_vuln(atk("a", "Bash", 2, 8, 1, T), 2),
            atk("b", "Strike", 1, 6, 1, T),
        ],
        3,
        &[ms(0, 17, 0, false, vec![("Artifact", 1)])],
    );
}

#[test]
fn bash_vulnerable_does_not_buff_self() {
    assert_no_kill(
        &[with_vuln(atk("a", "Bash", 2, 8, 1, T), 2)],
        2,
        &[ms(0, 12, 0, false, vec![])],
    );
}

#[test]
fn vulnerable_setup_enables_later_attack() {
    assert_kill(
        &[
            with_vuln(skill("a", "Trip", 0), 2),
            atk("b", "Heavy Strike", 1, 10, 1, T),
        ],
        1,
        &[ms(0, 15, 0, false, vec![])],
    );
}

#[test]
fn malleable_blocks_second_attack() {
    assert_no_kill(
        &[
            atk("a", "Strike A", 1, 6, 1, T),
            atk("b", "Strike B", 1, 6, 1, T),
        ],
        2,
        &[ms(0, 12, 0, false, vec![("Malleable", 3)])],
    );
}

#[test]
fn intangible_caps_single_hit() {
    assert_no_kill(
        &[atk("a", "Big Strike", 1, 20, 1, T)],
        1,
        &[ms(0, 3, 0, false, vec![("Intangible", 1)])],
    );
}

#[test]
fn flight_halves_attack_damage() {
    assert_no_kill(
        &[atk("a", "Strike", 1, 10, 1, T)],
        1,
        &[ms(0, 10, 0, false, vec![("Flight", 3)])],
    );
}

#[test]
fn invincible_caps_turn_damage() {
    // 3 attacks of 40 each vs 100hp + Invincible(30)
    // Total damage should be capped at 30, not enough to kill 100hp
    let result = test_scan(
        &[
            atk("a", "Heavy Strike A", 1, 40, 1, T),
            atk("b", "Heavy Strike B", 1, 40, 1, T),
            atk("c", "Heavy Strike C", 1, 40, 1, T),
        ],
        3,
        &[ms(0, 100, 0, false, vec![("Invincible", 30)])],
        Stance::Neutral,
        0,
        0,
        3,
        10_000_000,
    );
    assert!(result.is_none(), "expected no kill but got {:?}", result);
}

#[test]
fn play_limit_prevents_many_small_attacks() {
    assert_no_kill_plays(
        &[
            atk("1", "Strike 1", 0, 3, 1, T),
            atk("2", "Strike 2", 0, 3, 1, T),
            atk("3", "Strike 3", 0, 3, 1, T),
            atk("4", "Strike 4", 0, 3, 1, T),
        ],
        0,
        &[ms(0, 10, 0, false, vec![])],
        3,
    );
}

#[test]
fn random_target_guaranteed_two_enemies() {
    assert_kill(
        &[atk("a", "Sword Boomerang", 1, 4, 3, R)],
        1,
        &[ms(0, 4, 0, false, vec![]), ms(1, 4, 0, false, vec![])],
    );
}

#[test]
fn random_target_not_guaranteed_two_enemies() {
    assert_no_kill(
        &[atk("a", "Sword Boomerang", 1, 4, 3, R)],
        1,
        &[ms(0, 5, 0, false, vec![]), ms(1, 5, 0, false, vec![])],
    );
}

#[test]
fn random_target_guaranteed_after_setup() {
    assert_kill(
        &[
            atk("a", "Strike", 1, 6, 1, T),
            atk("b", "Boomerang", 1, 4, 3, R),
        ],
        2,
        &[ms(0, 10, 0, false, vec![]), ms(1, 4, 0, false, vec![])],
    );
}
