use super::*;

#[test]
fn context_builder_vigor_fails_closed() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);
    s.powers = vec![power("Vigor", "Vigor", 1)];

    assert!(build_context(&s).is_none());
}

#[test]
fn context_builder_wreath_of_flame_fails_closed() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);
    s.powers = vec![power("Wreath of Flame", "Wreath of Flame", 1)];

    assert!(build_context(&s).is_none());
}

#[test]
fn context_builder_shifting_fails_closed() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster(
        "Champ",
        100,
        0,
        vec![power("Shifting", "Shifting", 1)],
        0,
    )];
    s.energy = Some(3);

    assert!(build_context(&s).is_none());
}

#[test]
fn context_builder_time_warp_fails_closed() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster(
        "Time Eater",
        100,
        0,
        vec![power("Time Warp", "Time Warp", 10)],
        0,
    )];
    s.energy = Some(3);

    assert!(build_context(&s).is_none());
}

#[test]
fn context_builder_normality_fails_closed() {
    let mut s = state();
    s.hand = vec![
        strike("打击", "s1"),
        CardInfo {
            id: "Normality".into(),
            name: "凡庸".into(),
            cost: 1,
            card_type: "CURSE".into(),
            upgraded: false,
            uuid: Some("n1".into()),
            description: "".into(),
            price: None,
            playable: true, // Clash is not playable, but Normality is
            has_target: false,
        },
    ];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);

    assert!(build_context(&s).is_none());
}

#[test]
fn context_builder_velvet_choker_fails_closed() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);
    s.relics = vec![RelicInfo {
        id: "Velvet Choker".into(),
        name: "天鹅绒项圈".into(),
        description: "".into(),
        counter: None,
        price: None,
    }];

    assert!(build_context(&s).is_none());
}

#[test]
fn context_builder_unknown_monster_power_fails_closed() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster(
        "Unknown",
        20,
        0,
        vec![power("SomeUnknownPower", "SomeUnknownPower", 1)],
        0,
    )];
    s.energy = Some(3);

    assert!(build_context(&s).is_none());
}

#[test]
fn context_builder_out_of_range_hp_fails_closed() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Huge", 100_000, 0, vec![], 0)];
    s.energy = Some(3);

    assert!(build_context(&s).is_none());
}

#[test]
fn context_builder_out_of_range_energy_fails_closed() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(100_000);

    assert!(build_context(&s).is_none());
}

#[test]
fn context_builder_player_dead_returns_none() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.current_hp = Some(0);
    s.energy = Some(3);

    assert!(build_context(&s).is_none());
}
