use super::*;

#[test]
fn combat_ended_hp_exactly_zero_counts_as_dead() {
    let monsters = vec![MonsterSnapshot {
        command_index: 0,
        hp: 0,
        block: 10,
        is_minion: false,
        powers: vec![],
    }];
    assert!(combat_ended(&monsters));
}

#[test]
fn combat_ended_hp_negative_counts_as_dead() {
    let monsters = vec![MonsterSnapshot {
        command_index: 0,
        hp: -1,
        block: 0,
        is_minion: false,
        powers: vec![],
    }];
    assert!(combat_ended(&monsters));
}

#[test]
fn combat_ended_one_alive_dead_and_minion() {
    let monsters = vec![
        MonsterSnapshot {
            command_index: 0,
            hp: 5,
            block: 0,
            is_minion: true,
            powers: vec![],
        },
        MonsterSnapshot {
            command_index: 1,
            hp: 0,
            block: 0,
            is_minion: false,
            powers: vec![],
        },
    ];
    assert!(combat_ended(&monsters));
}

#[test]
fn combat_ended_multiple_minions_all_alive() {
    let monsters = vec![ms_minion(0, 10), ms_minion(1, 20)];
    assert!(combat_ended(&monsters));
}

#[test]
fn combat_ended_empty_returns_false() {
    assert!(!combat_ended(&[]));
}

#[test]
fn combat_ended_one_alive_non_minion_returns_false() {
    let monsters = vec![ms(10, 0, vec![])];
    assert!(!combat_ended(&monsters));
}
