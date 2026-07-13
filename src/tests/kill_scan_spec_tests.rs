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

#[path = "kill_scan_spec_tests/edge_cases.rs"]
mod edge_cases;
#[path = "kill_scan_spec_tests/fixture_tests.rs"]
mod fixture_tests;
#[path = "kill_scan_spec_tests/mechanics.rs"]
mod mechanics;
#[path = "kill_scan_spec_tests/transitions.rs"]
mod transitions;
#[path = "kill_scan_spec_tests/utilities.rs"]
mod utilities;
#[path = "kill_scan_spec_tests/x_cost.rs"]
mod x_cost;
