use super::*;

#[test]
fn context_builder_safe_monster_powers_pass() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster(
        "Darkling",
        20,
        0,
        vec![
            power("Fading", "消逝", 3),
            power("Life Link", "生命链接", -1),
            power("Shackled", "镣铐", 1),
            power("Weakened", "虚弱", 1),
        ],
        0,
    )];
    s.energy = Some(3);

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.monsters.len(), 1);
}

#[test]
fn context_builder_modeled_monster_powers_pass() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster(
        "Test",
        20,
        0,
        vec![
            power("Artifact", "人工制品", 2),
            power("Slow", "缓慢", 1),
            power("Vulnerable", "易伤", 2),
            power("Strength", "力量", 3),
            power("Intangible", "Intangible", 1),
            power("Flight", "Flight", 3),
        ],
        0,
    )];
    s.energy = Some(3);

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.monsters.len(), 1);
}

#[test]
fn context_builder_monster_snapshot_fields() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![
        monster(
            "Jaw Worm",
            20,
            5,
            vec![
                power("Vulnerable", "易伤", 2),
                power("Artifact", "人工制品", 1),
            ],
            0,
        ),
        monster("Minion", 10, 0, vec![power("Minion", "爪牙", 1)], 1),
    ];
    s.energy = Some(3);

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.monsters.len(), 2);

    let m0 = &ctx.monsters[0];
    assert_eq!(m0.command_index, 0);
    assert_eq!(m0.hp, 20);
    assert_eq!(m0.block, 5);
    assert!(!m0.is_minion);
    assert_eq!(m0.powers.len(), 2);
    assert_eq!(m0.powers[0].id, "Vulnerable");
    assert_eq!(m0.powers[0].amount, 2);

    let m1 = &ctx.monsters[1];
    assert_eq!(m1.command_index, 1);
    assert_eq!(m1.hp, 10);
    assert_eq!(m1.block, 0);
    assert!(m1.is_minion);
    assert_eq!(m1.powers[0].id, "Minion");
}

#[test]
fn context_builder_minion_power_presence_ignores_negative_amount() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster(
        "Torch Head",
        40,
        0,
        vec![power("Minion", "爪牙", -1)],
        0,
    )];
    s.energy = Some(1);

    let ctx = build_context(&s).unwrap();
    assert!(ctx.monsters[0].is_minion);
}

#[test]
fn kill_scan_collector_ends_when_only_negative_amount_minions_remain() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![
        monster("Torch Head", 40, 0, vec![power("Minion", "爪牙", -1)], 0),
        monster("Torch Head", 40, 0, vec![power("Minion", "爪牙", -1)], 1),
        monster("Collector", 6, 0, vec![], 2),
    ];
    s.energy = Some(1);

    assert!(can_end_fight(&s));
}
