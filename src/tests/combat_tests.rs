use crate::combat::Stance;
use crate::combat::context::build_context;
use crate::state::{
    CardInfo, DangerFlags, DangerLevel, MonsterInfo, NormalizedState, PowerInfo, RelicInfo,
};

fn state() -> NormalizedState {
    NormalizedState {
        screen_type: Some("NONE".to_string()),
        room_type: Some("MonsterRoom".to_string()),
        character: Some("IRONCLAD".to_string()),
        seed: Some(-3047511808784702860),
        ascension_level: Some(20),
        floor: Some(1),
        current_hp: Some(68),
        max_hp: Some(75),
        gold: Some(99),
        energy: Some(3),
        block: Some(6),
        powers: vec![],
        hand: vec![],
        monsters: vec![],
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
        hand_cards: vec![],
        draw_pile: vec![],
        discard_pile: vec![],
        exhaust_cards: vec![],
        master_cards: vec![],
        map_nodes: vec![],
        map_first_node_chosen: None,
        map_current_x: None,
        map_current_y: None,
        skip_available: false,
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
    }
}

fn strike(name: &str, uuid: &str) -> CardInfo {
    CardInfo {
        id: "Strike_R".into(),
        name: name.into(),
        cost: 1,
        card_type: "ATTACK".into(),
        upgraded: false,
        uuid: Some(uuid.into()),
        description: "造成 6 点伤害。".into(),
        price: None,
        playable: true,
        has_target: true,
    }
}

fn defend(name: &str, uuid: &str) -> CardInfo {
    CardInfo {
        id: "Defend_R".into(),
        name: name.into(),
        cost: 1,
        card_type: "SKILL".into(),
        upgraded: false,
        uuid: Some(uuid.into()),
        description: "获得 5 点 格挡 。".into(),
        price: None,
        playable: true,
        has_target: false,
    }
}

fn monster(name: &str, hp: i64, block: i64, powers: Vec<PowerInfo>, index: usize) -> MonsterInfo {
    MonsterInfo {
        name: name.into(),
        index,
        current_hp: Some(hp),
        max_hp: Some(hp),
        block: Some(block),
        intent: None,
        damage: None,
        hits: None,
        monster_powers: powers,
        can_be_killed: false,
        is_scaling: false,
    }
}

fn power(id: &str, name: &str, amount: i64) -> PowerInfo {
    PowerInfo {
        id: id.into(),
        name: name.into(),
        amount,
    }
}

#[test]
fn context_builder_happy_path() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1"), strike("打击", "s2")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.cards.len(), 2);
    assert_eq!(ctx.energy, 3);
    assert_eq!(ctx.initial_stance, Stance::Neutral);
    assert_eq!(ctx.current_stance, Stance::Neutral);
    assert_eq!(ctx.strength_delta, 0);
    assert_eq!(ctx.x_cost_bonus, 0);
    assert_eq!(ctx.monsters.len(), 1);
    assert_eq!(ctx.remaining_card_plays, 2);
}

#[test]
fn context_builder_empty_hand_returns_none() {
    let mut s = state();
    s.energy = Some(3);
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    assert!(build_context(&s).is_none());
}

#[test]
fn context_builder_non_none_screen_returns_none() {
    let mut s = state();
    s.screen_type = Some("CARD_REWARD".to_string());
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);
    assert!(build_context(&s).is_none());
}

#[test]
fn context_builder_stance_detection_wrath() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);
    s.powers = vec![power("Wrath", "Wrath", 1)];

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.initial_stance, Stance::Wrath);
    assert_eq!(ctx.current_stance, Stance::Wrath);
}

#[test]
fn context_builder_stance_detection_calm() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);
    s.powers = vec![power("Calm", "Calm", 1)];

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.initial_stance, Stance::Calm);
}

#[test]
fn context_builder_stance_detection_divinity() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);
    s.powers = vec![power("Divinity", "Divinity", 1)];

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.initial_stance, Stance::Divinity);
}

#[test]
fn context_builder_stance_defaults_to_neutral() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.initial_stance, Stance::Neutral);
}

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

#[test]
fn context_builder_cards_without_uuid_filtered() {
    let mut s = state();
    s.hand = vec![
        strike("打击", "s1"),
        CardInfo {
            id: "Strike_R".into(),
            name: "打击".into(),
            cost: 1,
            card_type: "ATTACK".into(),
            upgraded: false,
            uuid: None,
            description: "造成 6 点伤害。".into(),
            price: None,
            playable: true,
            has_target: true,
        },
    ];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.cards.len(), 1);
    assert_eq!(ctx.cards[0].uuid.as_deref(), Some("s1"));
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
    assert_eq!(m0.is_minion, false);
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
fn context_builder_remaining_card_plays_equals_hand_len() {
    let mut s = state();
    s.hand = vec![
        strike("打击", "s1"),
        strike("打击", "s2"),
        strike("打击", "s3"),
        defend("防御", "d1"),
        defend("防御", "d2"),
    ];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.remaining_card_plays, 5);
}

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
