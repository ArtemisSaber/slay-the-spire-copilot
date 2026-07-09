use crate::combat::damage::combat_ended;
use crate::combat::effects::{HitCount, StanceEffect, TargetType};
use crate::combat::kill_scan::{TestCard, random_target_guaranteed, test_scan};
use crate::combat::{MonsterSnapshot, PowerState, Stance};

fn ms(
    index: usize,
    hp: i16,
    block: i16,
    is_minion: bool,
    powers: Vec<(&str, i16)>,
) -> MonsterSnapshot {
    MonsterSnapshot {
        command_index: index,
        hp,
        block,
        is_minion,
        powers: powers
            .into_iter()
            .map(|(id, amount)| PowerState {
                id: id.to_string(),
                amount,
                triggered: false,
            })
            .collect(),
    }
}

fn atk(
    uuid: &'static str,
    name: &'static str,
    cost: i16,
    dmg: i16,
    hits: i16,
    tt: TargetType,
) -> TestCard {
    TestCard {
        uuid,
        name,
        cost,
        card_type: "ATTACK",
        damage: Some((dmg, HitCount::Fixed(hits), tt)),
        vulnerable: None,
        strength_gain: 0,
        energy_gain: 0,
        stance: StanceEffect::None,
        mantra_gain: 0,
        execute_threshold: None,
        x_cost: false,
    }
}

fn skill(uuid: &'static str, name: &'static str, cost: i16) -> TestCard {
    TestCard {
        uuid,
        name,
        cost,
        card_type: "SKILL",
        damage: None,
        vulnerable: None,
        strength_gain: 0,
        energy_gain: 0,
        stance: StanceEffect::None,
        mantra_gain: 0,
        execute_threshold: None,
        x_cost: false,
    }
}

fn with_vuln(mut tc: TestCard, amount: i16) -> TestCard {
    tc.vulnerable = Some(amount);
    tc
}

fn with_strength(mut tc: TestCard, amount: i16) -> TestCard {
    tc.strength_gain = amount;
    tc
}

fn with_energy(mut tc: TestCard, amount: i16) -> TestCard {
    tc.energy_gain = amount;
    tc
}

fn with_stance(mut tc: TestCard, stance: StanceEffect) -> TestCard {
    tc.stance = stance;
    tc
}

fn with_mantra(mut tc: TestCard, amount: i16) -> TestCard {
    tc.mantra_gain = amount;
    tc
}

fn with_xcost(mut tc: TestCard, dmg_per_x: i16) -> TestCard {
    tc.x_cost = true;
    tc.damage = Some((dmg_per_x, HitCount::XTimes, TargetType::AoE));
    tc
}

fn with_xplus(mut tc: TestCard, dmg: i16, offset: i16, tt: TargetType) -> TestCard {
    tc.x_cost = true;
    tc.damage = Some((dmg, HitCount::XPlus(offset), tt));
    tc
}

const T: TargetType = TargetType::Targeted;
const A: TargetType = TargetType::AoE;
const R: TargetType = TargetType::RandomTarget;

fn assert_kill(hand: &[TestCard], energy: i16, monsters: &[MonsterSnapshot]) {
    assert!(
        test_scan(
            hand,
            energy,
            monsters,
            Stance::Neutral,
            0,
            0,
            hand.len(),
            10_000_000
        )
        .is_some(),
        "expected kill but got None"
    );
}

fn assert_no_kill(hand: &[TestCard], energy: i16, monsters: &[MonsterSnapshot]) {
    assert!(
        test_scan(
            hand,
            energy,
            monsters,
            Stance::Neutral,
            0,
            0,
            hand.len(),
            10_000_000
        )
        .is_none(),
        "expected no kill but got Some"
    );
}

fn assert_kill_with_stance(
    hand: &[TestCard],
    energy: i16,
    monsters: &[MonsterSnapshot],
    stance: Stance,
) {
    assert!(
        test_scan(hand, energy, monsters, stance, 0, 0, hand.len(), 10_000_000).is_some(),
        "expected kill but got None"
    );
}

fn assert_no_kill_plays(
    hand: &[TestCard],
    energy: i16,
    monsters: &[MonsterSnapshot],
    remaining_plays: usize,
) {
    assert!(
        test_scan(
            hand,
            energy,
            monsters,
            Stance::Neutral,
            0,
            0,
            remaining_plays,
            10_000_000
        )
        .is_none(),
        "expected no kill but got Some"
    );
}

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

fn dm(hp: i16, block: i16) -> MonsterSnapshot {
    MonsterSnapshot {
        command_index: 0,
        hp,
        block,
        is_minion: false,
        powers: vec![],
    }
}

#[test]
fn combat_ended_all_dead() {
    let monsters = vec![MonsterSnapshot {
        command_index: 0,
        hp: 0,
        block: 0,
        powers: vec![],
        is_minion: false,
    }];
    assert!(combat_ended(&monsters));
}

#[test]
fn combat_ended_all_minions() {
    let monsters = vec![MonsterSnapshot {
        command_index: 0,
        hp: 10,
        block: 0,
        powers: vec![],
        is_minion: true,
    }];
    assert!(combat_ended(&monsters));
}

#[test]
fn combat_ended_one_alive_non_minion() {
    assert!(!combat_ended(&[
        ms(0, 0, 0, false, vec![]),
        ms(1, 10, 0, false, vec![])
    ]));
}

#[test]
fn combat_ended_dead_and_minion() {
    assert!(combat_ended(&[
        ms(0, 0, 0, false, vec![]),
        ms(1, 10, 0, true, vec![])
    ]));
}

#[test]
fn combat_ended_empty_monsters() {
    assert!(!combat_ended(&[]));
}

#[test]
fn random_target_4x3_vs_two_4hp() {
    assert!(random_target_guaranteed(3, 4, &[dm(4, 0), dm(4, 0)]));
}

#[test]
fn random_target_4x3_vs_two_5hp_6hp_rejected() {
    assert!(!random_target_guaranteed(3, 4, &[dm(6, 0), dm(5, 0)]));
}

#[test]
fn random_target_3x5_vs_three_3hp() {
    assert!(random_target_guaranteed(
        3,
        5,
        &[dm(3, 0), dm(3, 0), dm(3, 0)]
    ));
}

#[test]
fn random_target_single_monster_guaranteed() {
    assert!(random_target_guaranteed(3, 4, &[dm(3, 0)]));
}

#[test]
fn random_target_single_monster_rejected() {
    assert!(!random_target_guaranteed(3, 4, &[dm(100, 0)]));
}

#[test]
fn random_target_empty_monsters() {
    assert!(random_target_guaranteed(3, 4, &[dm(0, 0)]));
}

#[test]
fn kill_scan_supports_more_than_sixteen_cards_in_hand() {
    // Modded games may exceed the vanilla 10-card hand limit. The bit mask
    // tracking remaining cards must not overflow at index >= 16 (u16 wraps).
    let mut hand: Vec<TestCard> = (0..16).map(|_| skill("filler", "filler", 99)).collect();
    // Index 16 — the only playable card, lethal to a 10hp monster.
    hand.push(atk("lethal", "strike", 1, 10, 1, TargetType::AoE));
    let monsters = vec![ms(0, 10, 0, false, vec![])];
    assert!(
        test_scan(&hand, 1, &monsters, Stance::Neutral, 0, 0, 1, 10_000_000).is_some(),
        "expected lethal sequence with 17-card hand (index 16 mask bit)"
    );
}

mod fixture_tests {

    use crate::combat::{can_end_fight, find_kill_sequence};
    use crate::locales::Locale;
    use crate::state::NormalizedState;

    fn load_fixture(name: &str) -> serde_json::Value {
        let path = format!("tests/fixtures/{name}");
        let content = std::fs::read_to_string(&path).unwrap();
        serde_json::from_str(&content).unwrap()
    }

    fn fixture_list() -> Vec<String> {
        let content = std::fs::read_to_string("tests/fixtures/kill-scan-fixtures.json").unwrap();
        serde_json::from_str(&content).unwrap()
    }

    fn fixture_by_line(line: &str) -> String {
        fixture_list()
            .into_iter()
            .find(|n| n.contains(line))
            .expect("fixture not found")
    }

    #[test]
    fn all_fixtures_parse() {
        let names = fixture_list();
        assert!(!names.is_empty());
        for name in &names {
            let raw = load_fixture(name);
            let state = NormalizedState::from_raw(&raw, &Locale::load("zh"));
            assert_eq!(
                state.screen_type.as_ref().map(|st| st.as_str()),
                Some("NONE"),
                "{name}"
            );
            assert!(!state.hand.is_empty(), "{name}");
            assert!(!state.monsters.is_empty(), "{name}");
        }
    }

    #[test]
    fn l52_two_card_kill() {
        let raw = load_fixture(&fixture_by_line("L52"));
        let state = NormalizedState::from_raw(&raw, &Locale::load("zh"));

        // Strike(13) + Twin Strike+(14*2=28) = 41 > 26hp, energy=2
        assert!(can_end_fight(&state));
        let seq = find_kill_sequence(&state).unwrap();
        assert!(!seq.is_empty());
    }

    #[test]
    fn l156_xcost_aoe_kill() {
        let raw = load_fixture(&fixture_by_line("L156"));
        let state = NormalizedState::from_raw(&raw, &Locale::load("zh"));

        // Whirlwind+ X-cost (25 per X), energy=5, 36hp Giant Head
        assert!(can_end_fight(&state));
        let seq = find_kill_sequence(&state).unwrap();
        assert!(!seq.is_empty());
    }

    #[test]
    fn l157_single_strike_kill() {
        let raw = load_fixture(&fixture_by_line("L157"));
        let state = NormalizedState::from_raw(&raw, &Locale::load("zh"));

        // Any Strike (40 dmg) kills 33hp Giant Head; DFS may find multi-card path first
        assert!(can_end_fight(&state));
        let seq = find_kill_sequence(&state).unwrap();
        assert!(!seq.is_empty(), "should find some kill sequence");
    }

    #[test]
    fn l208_minion_end() {
        let raw = load_fixture(&fixture_by_line("L208"));
        let state = NormalizedState::from_raw(&raw, &Locale::load("zh"));

        // Strike kills 6hp snake leader → minion dagger remains but combat ends
        assert!(can_end_fight(&state));
        let seq = find_kill_sequence(&state).unwrap();
        assert!(!seq.is_empty());
    }

    #[test]
    fn non_combat_screen_returns_none() {
        let raw = load_fixture(&fixture_by_line("L52"));
        let mut state = NormalizedState::from_raw(&raw, &Locale::load("zh"));
        state.screen_type = Some(crate::state::ScreenType::CardReward);

        assert!(!can_end_fight(&state));
        assert!(find_kill_sequence(&state).is_none());
    }

    #[test]
    fn empty_hand_returns_none() {
        let raw = load_fixture(&fixture_by_line("L52"));
        let mut state = NormalizedState::from_raw(&raw, &Locale::load("zh"));
        state.hand.clear();

        assert!(!can_end_fight(&state));
        assert!(find_kill_sequence(&state).is_none());
    }

    #[test]
    fn pure_block_hand_returns_none() {
        let raw = load_fixture(&fixture_by_line("L52"));
        let mut state = NormalizedState::from_raw(&raw, &Locale::load("zh"));

        state.hand = vec![crate::state::CardInfo {
            id: "Defend_R".into(),
            name: "防御".into(),
            cost: 1,
            card_type: "SKILL".into(),
            upgraded: false,
            uuid: Some("defend-1".into()),
            description: "获得 5 点 格挡 。".into(),
            price: None,
            playable: true,
            has_target: false,
        }];

        assert!(!can_end_fight(&state));
        assert!(find_kill_sequence(&state).is_none());
    }

    #[test]
    fn command_target_index_gap_preserved() {
        use crate::combat::MonsterSnapshot;
        use crate::combat::Stance;
        use crate::combat::effects::{HitCount, TargetType};
        use crate::combat::kill_scan::TestCard;
        use crate::combat::kill_scan::test_scan;

        let tc = TestCard {
            uuid: "kill-cmd-idx-two",
            name: "Strike",
            cost: 1,
            card_type: "ATTACK",
            damage: Some((5, HitCount::Fixed(1), TargetType::Targeted)),
            vulnerable: None,
            strength_gain: 0,
            energy_gain: 0,
            stance: crate::combat::effects::StanceEffect::None,
            mantra_gain: 0,
            execute_threshold: None,
            x_cost: false,
        };

        let monsters = vec![
            MonsterSnapshot {
                command_index: 0,
                hp: 50,
                block: 0,
                is_minion: true,
                powers: vec![],
            },
            MonsterSnapshot {
                command_index: 2,
                hp: 5,
                block: 0,
                is_minion: false,
                powers: vec![],
            },
        ];

        let seq = test_scan(&[tc], 1, &monsters, Stance::Neutral, 0, 0, 1, 10_000_000).unwrap();

        assert_eq!(seq.len(), 1);
        assert_eq!(seq[0].card, "kill-cmd-idx-two");
        assert_eq!(
            seq[0].target,
            Some(2),
            "target should be command index 2, not living vector position 1"
        );
    }

    #[test]
    fn context_builder_time_warp_fails_closed() {
        use crate::state::{
            CardInfo as Sc, DangerFlags, DangerLevel, MonsterInfo, NormalizedState, PowerInfo,
        };

        let state = NormalizedState {
            screen_type: Some(crate::state::ScreenType::None),
            room_type: None,
            character: None,
            seed: None,
            ascension_level: None,
            floor: None,
            current_hp: Some(50),
            max_hp: Some(50),
            gold: None,
            energy: Some(0),
            block: None,
            powers: vec![],
            hand: vec![
                Sc {
                    id: "Strike_R".into(),
                    name: "打击".into(),
                    cost: 0,
                    card_type: "ATTACK".into(),
                    upgraded: false,
                    uuid: Some("tw-s1".into()),
                    description: "造成 3 点伤害。".into(),
                    price: None,
                    playable: true,
                    has_target: true,
                },
                Sc {
                    id: "Strike_R".into(),
                    name: "打击".into(),
                    cost: 0,
                    card_type: "ATTACK".into(),
                    upgraded: false,
                    uuid: Some("tw-s2".into()),
                    description: "造成 3 点伤害。".into(),
                    price: None,
                    playable: true,
                    has_target: true,
                },
                Sc {
                    id: "Strike_R".into(),
                    name: "打击".into(),
                    cost: 0,
                    card_type: "ATTACK".into(),
                    upgraded: false,
                    uuid: Some("tw-s3".into()),
                    description: "造成 3 点伤害。".into(),
                    price: None,
                    playable: true,
                    has_target: true,
                },
                Sc {
                    id: "Strike_R".into(),
                    name: "打击".into(),
                    cost: 0,
                    card_type: "ATTACK".into(),
                    upgraded: false,
                    uuid: Some("tw-s4".into()),
                    description: "造成 3 点伤害。".into(),
                    price: None,
                    playable: true,
                    has_target: true,
                },
            ],
            monsters: vec![MonsterInfo {
                name: "Time Eater".into(),
                monster_id: None,
                index: 0,
                current_hp: Some(10),
                max_hp: Some(10),
                block: Some(0),
                intent: None,
                damage: None,
                hits: None,
                monster_powers: vec![PowerInfo {
                    id: "Time Warp".into(),
                    name: "Time Warp".into(),
                    amount: 0,
                }],
                can_be_killed: false,
                is_scaling: false,
            }],
            card_reward_choices: vec![],
            boss_relic_choices: vec![],
            event_id: None,
            event_name: None,
            event_body: None,
            event_choices: vec![],
            relics: vec![],
            potions: vec![],
            deck_names: vec![],
            incoming_damage: 0,
            rest_options: vec![],
            danger: DangerFlags {
                hp_critical: false,
                incoming_lethal: false,
                no_block_against_hit: false,
                any_monster_attacking: false,
                wrath_stance: false,
                level: DangerLevel::Safe,
            },
            skip_available: false,
            hand_cards: vec![],
            draw_pile: vec![],
            discard_pile: vec![],
            exhaust_cards: vec![],
            master_cards: vec![],
            map_nodes: vec![],
            map_first_node_chosen: None,
            map_current_x: None,
            map_current_y: None,
            shop_cards: vec![],
            shop_relics: vec![],
            shop_potions: vec![],
            purge_available: false,
            purge_cost: None,
            hand_select_max_cards: None,
            hand_select_can_pick_zero: false,
            hand_select_selected: vec![],
            current_action: None,
            card_in_play: None,
            grid_cards: vec![],
            grid_selected_cards: vec![],
            grid_for_upgrade: false,
            grid_for_transform: false,
            grid_for_purge: false,
            grid_num_cards: None,
            empty_potion_slots: 0,
            ..Default::default()
        };

        // 4 Strike cards (3 dmg each) vs 10hp, but Time Warp present → fail closed
        assert!(!can_end_fight(&state));
        assert!(find_kill_sequence(&state).is_none());
    }

    fn run_fixture_list() -> Vec<serde_json::Value> {
        let content =
            std::fs::read_to_string("tests/fixtures/kill-scan-run-fixtures.json").unwrap();
        serde_json::from_str(&content).unwrap()
    }

    #[test]
    fn run_fixtures_deterministic_results() {
        let fixtures: Vec<serde_json::Value> = run_fixture_list();
        assert!(
            !fixtures.is_empty(),
            "run fixtures list should not be empty"
        );

        let mut passes = 0usize;
        let mut failures = 0usize;

        for item in &fixtures {
            let label = item["label"].as_str().unwrap_or("?");
            let expected_kill = item["expected_kill"].as_bool().unwrap_or(false);
            let filename = item["filename"].as_str().unwrap_or("");

            let raw = load_fixture(filename);
            let state: NormalizedState =
                serde_json::from_value(raw.clone()).expect("failed to deserialize fixture");
            let actual_kill = can_end_fight(&state);

            if actual_kill == expected_kill {
                passes += 1;
            } else {
                failures += 1;
                eprintln!("FAIL {label}: expected_kill={expected_kill} actual_kill={actual_kill}");
                let hand = &state.hand;
                eprintln!(
                    "  Hand: {} cards, energy={}",
                    hand.len(),
                    state.energy.unwrap_or(-1)
                );
                for c in hand {
                    eprintln!(
                        "    {} cost={} type={} playable={}",
                        c.name, c.cost, c.card_type, c.playable
                    );
                }
                for m in &state.monsters {
                    eprintln!(
                        "    Monster: {} hp={:?} block={:?}",
                        m.name, m.current_hp, m.block
                    );
                }
            }
        }

        assert_eq!(
            failures, 0,
            "expected 0 mismatches, got {failures} (passed {passes})"
        );
        assert!(passes > 0, "should have tested at least one fixture");
    }
}
