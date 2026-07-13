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
