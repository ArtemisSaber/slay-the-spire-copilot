use super::*;

#[test]
fn build_combat_elite_room() {
    let locale = test_locale();
    let state = NormalizedState {
        room_type: Some(RoomType::MonsterRoomElite),
        monsters: vec![MonsterInfo {
            name: "大颚虫".into(),
            monster_id: None,
            index: 0,
            current_hp: Some(44),
            max_hp: Some(46),
            block: Some(0),
            intent: Some("ATTACK".into()),
            damage: Some(12),
            hits: Some(1),
            monster_powers: vec![],
            can_be_killed: false,
            is_scaling: false,
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("精英战斗"));
}

#[test]
fn build_combat_boss_room() {
    let locale = test_locale();
    let state = NormalizedState {
        room_type: Some(RoomType::MonsterRoomBoss),
        monsters: vec![MonsterInfo {
            name: "六火亡魂".into(),
            monster_id: None,
            index: 0,
            current_hp: Some(100),
            max_hp: Some(100),
            block: Some(0),
            intent: Some("ATTACK".into()),
            damage: Some(20),
            hits: Some(1),
            monster_powers: vec![],
            can_be_killed: false,
            is_scaling: false,
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("Boss战"));
}

#[test]
fn build_combat_high_incoming_damage_normal() {
    let locale = test_locale();
    let state = NormalizedState {
        room_type: Some(RoomType::MonsterRoom),
        max_hp: Some(75),
        incoming_damage: 20,
        monsters: vec![MonsterInfo {
            name: "大颚虫".into(),
            monster_id: None,
            index: 0,
            current_hp: Some(44),
            max_hp: Some(46),
            block: Some(0),
            intent: Some("ATTACK".into()),
            damage: Some(20),
            hits: Some(1),
            monster_powers: vec![],
            can_be_killed: false,
            is_scaling: false,
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("本回合来伤高"));
}

#[test]
fn build_combat_punish_priority() {
    let locale = test_locale();
    let state = NormalizedState {
        monsters: vec![MonsterInfo {
            name: "带刺史莱姆".into(),
            monster_id: None,
            index: 0,
            current_hp: Some(14),
            max_hp: Some(14),
            block: Some(0),
            intent: Some("ATTACK".into()),
            damage: Some(6),
            hits: Some(1),
            monster_powers: vec![PowerInfo {
                id: "Thorns".into(),
                name: "荆棘".into(),
                amount: 3,
            }],
            can_be_killed: false,
            is_scaling: false,
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("惩罚机制"));
}

#[test]
fn build_combat_default_priority_shows_name() {
    let locale = test_locale();
    let state = NormalizedState {
        monsters: vec![MonsterInfo {
            name: "独特怪物".into(),
            monster_id: None,
            index: 0,
            current_hp: Some(30),
            max_hp: Some(30),
            block: Some(0),
            intent: Some("ATTACK".into()),
            damage: Some(10),
            hits: Some(1),
            monster_powers: vec![],
            can_be_killed: false,
            is_scaling: false,
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("目标优先级：独特怪物"));
}

#[test]
fn build_combat_elite_tradeoff() {
    let locale = test_locale();
    let state = NormalizedState {
        room_type: Some(RoomType::MonsterRoomElite),
        monsters: vec![MonsterInfo {
            name: "大颚虫".into(),
            monster_id: None,
            index: 0,
            current_hp: Some(44),
            max_hp: Some(46),
            block: Some(0),
            intent: Some("ATTACK".into()),
            damage: Some(12),
            hits: Some(1),
            monster_powers: vec![],
            can_be_killed: false,
            is_scaling: false,
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("可接受掉血"));
}

#[test]
fn build_combat_goal_boss() {
    let locale = test_locale();
    let state = NormalizedState {
        room_type: Some(RoomType::MonsterRoomBoss),
        monsters: vec![MonsterInfo {
            name: "六火亡魂".into(),
            monster_id: None,
            index: 0,
            current_hp: Some(100),
            max_hp: Some(100),
            block: Some(0),
            intent: Some("ATTACK".into()),
            damage: Some(20),
            hits: Some(1),
            monster_powers: vec![],
            can_be_killed: false,
            is_scaling: false,
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("长期战"));
}
