use super::*;

#[test]
fn integration_can_end_fight_with_strike() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 6, 0, vec![], 0)];
    s.energy = Some(3);
    s.danger = DangerFlags {
        hp_critical: false,
        incoming_lethal: false,
        no_block_against_hit: false,
        any_monster_attacking: false,
        wrath_stance: false,
        level: DangerLevel::Safe,
    };
    assert!(crate::combat::can_end_fight(&s));
}

#[test]
fn integration_cannot_kill_with_insufficient_damage() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 7, 0, vec![], 0)];
    s.energy = Some(3);
    s.danger = DangerFlags {
        hp_critical: false,
        incoming_lethal: false,
        no_block_against_hit: false,
        any_monster_attacking: false,
        wrath_stance: false,
        level: DangerLevel::Safe,
    };
    assert!(!crate::combat::can_end_fight(&s));
}
