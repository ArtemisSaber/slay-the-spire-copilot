use super::*;

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
