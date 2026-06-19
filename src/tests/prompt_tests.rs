use super::*;
use crate::i18n::I18n;
use crate::state::{DangerFlags, DangerLevel, MonsterInfo, PowerInfo};

fn load_i18n() -> I18n {
    I18n::load()
}

fn test_state() -> NormalizedState {
    NormalizedState {
        screen_type: Some("NONE".to_string()),
        room_type: Some("MonsterRoom".to_string()),
        character: Some("IRONCLAD".to_string()),
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
    }
}

#[test]
fn combat_prompt_shows_all_three_piles() {
    let i18n = load_i18n();
    let state = NormalizedState {
        monsters: vec![MonsterInfo {
            name: "大颚虫".into(),
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
        hand_cards: vec![CardInfo {
            name: "打击".into(),
            cost: 1,
            card_type: "ATTACK".into(),
            upgraded: false,
            uuid: None,
        }],
        draw_pile: vec![CardInfo {
            name: "防御".into(),
            cost: 1,
            card_type: "SKILL".into(),
            upgraded: false,
            uuid: None,
        }],
        discard_pile: vec![CardInfo {
            name: "痛击".into(),
            cost: 2,
            card_type: "ATTACK".into(),
            upgraded: false,
            uuid: None,
        }],
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("=== 手牌"));
    assert!(prompt.contains("=== 抽牌堆"));
    assert!(prompt.contains("=== 弃牌堆"));
    assert!(prompt.contains("打击"));
    assert!(prompt.contains("防御"));
    assert!(prompt.contains("痛击"));
}

#[test]
fn monster_section_shows_index_and_intent() {
    let i18n = load_i18n();
    let state = NormalizedState {
        monsters: vec![
            MonsterInfo {
                name: "大颚虫".into(),
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
            },
            MonsterInfo {
                name: "邪教徒".into(),
                index: 1,
                current_hp: Some(18),
                max_hp: Some(40),
                block: Some(0),
                intent: Some("BUFF".into()),
                damage: None,
                hits: None,
                monster_powers: vec![],
                can_be_killed: true,
                is_scaling: false,
            },
        ],
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("[0]"));
    assert!(prompt.contains("[1]"));
    assert!(prompt.contains("意图：攻击"));
    assert!(prompt.contains("意图：增益"));
    assert!(prompt.contains("可斩杀"));
    assert!(prompt.contains("伤害：12"));
    assert!(prompt.contains("伤害：无"));
}

#[test]
fn monster_section_shows_scaling() {
    let i18n = load_i18n();
    let state = NormalizedState {
        monsters: vec![MonsterInfo {
            name: "大颚虫".into(),
            index: 0,
            current_hp: Some(44),
            max_hp: Some(46),
            block: Some(0),
            intent: Some("ATTACK".into()),
            damage: Some(12),
            hits: None,
            monster_powers: vec![PowerInfo {
                name: "力量".into(),
                amount: 2,
            }],
            can_be_killed: false,
            is_scaling: true,
        }],
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("成长中"));
    assert!(prompt.contains("力量(2)"));
}

#[test]
fn card_reward_shows_card_choices() {
    let i18n = load_i18n();
    let state = NormalizedState {
        screen_type: Some("CARD_REWARD".into()),
        character: Some("IRONCLAD".into()),
        floor: Some(3),
        current_hp: Some(62),
        max_hp: Some(75),
        card_reward_choices: vec![
            CardInfo {
                name: "上勾拳".into(),
                cost: 2,
                card_type: "ATTACK".into(),
                upgraded: false,
                uuid: None,
            },
            CardInfo {
                name: "愤怒".into(),
                cost: 1,
                card_type: "ATTACK".into(),
                upgraded: false,
                uuid: None,
            },
        ],
        master_cards: vec![
            CardInfo {
                name: "打击".into(),
                cost: 1,
                card_type: "ATTACK".into(),
                upgraded: false,
                uuid: None,
            };
            4
        ],
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("A. 上勾拳"));
    assert!(prompt.contains("B. 愤怒"));
    assert!(prompt.contains("2费"));
    assert!(prompt.contains("1费"));
}

#[test]
fn deck_analysis_counts_from_cards() {
    let cards = vec![
        CardInfo {
            name: "打击".into(),
            cost: 1,
            card_type: "ATTACK".into(),
            upgraded: false,
            uuid: None,
        };
        5
    ];
    let deck = DeckAnalysis::from_cards(&cards);
    assert_eq!(deck.attack_count, 5);
    assert_eq!(deck.skill_count, 0);
    assert_eq!(deck.total, 5);
}

#[test]
fn deck_analysis_detects_duplicates() {
    let mut cards = vec![
        CardInfo {
            name: "打击".into(),
            cost: 1,
            card_type: "ATTACK".into(),
            upgraded: false,
            uuid: None,
        };
        4
    ];
    cards.extend(vec![
        CardInfo {
            name: "防御".into(),
            cost: 1,
            card_type: "SKILL".into(),
            upgraded: false,
            uuid: None,
        };
        3
    ]);
    let deck = DeckAnalysis::from_cards(&cards);
    assert_eq!(deck.duplicates.len(), 2);
    let dup_names: Vec<&str> = deck.duplicates.iter().map(|(n, _)| n.as_str()).collect();
    assert!(dup_names.contains(&"打击"));
    assert!(dup_names.contains(&"防御"));
}

#[test]
fn prompt_is_structured() {
    let i18n = load_i18n();
    let prompt = build_prompt(&test_state(), &i18n);
    assert!(prompt.contains("=== 当前状态 ==="));
    assert!(prompt.contains("推荐："));
    assert!(prompt.contains("理由："));
    assert!(prompt.contains("风险："));
    assert!(prompt.contains("吐槽："));
}

#[test]
fn rest_prompt_has_translated_options() {
    let i18n = load_i18n();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        character: Some("IRONCLAD".into()),
        current_hp: Some(25),
        max_hp: Some(75),
        floor: Some(5),
        rest_options: vec!["rest".into(), "smith".into()],
        danger: DangerFlags {
            hp_critical: true,
            level: DangerLevel::Danger,
            ..test_state().danger
        },
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("休息"));
    assert!(prompt.contains("锻造"));
}

#[test]
fn generic_prompt_shows_status_and_format() {
    let i18n = load_i18n();
    let state = NormalizedState {
        screen_type: None,
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("=== 当前状态 ==="));
    assert!(prompt.contains("角色：铁甲战士"));
    assert!(prompt.contains("推荐："));
}

#[test]
fn rest_prompt_advises_rest_when_hp_low() {
    let i18n = load_i18n();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        current_hp: Some(15),
        max_hp: Some(75),
        rest_options: vec!["rest".into()],
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("血量极低，强烈建议休息。"));
}

#[test]
fn rest_prompt_suggests_smith_when_hp_high() {
    let i18n = load_i18n();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        current_hp: Some(60),
        max_hp: Some(75),
        rest_options: vec!["smith".into()],
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("血量健康，可考虑锻造或挖遗物。"));
}

#[test]
fn danger_prefix_shows_wrath_warning() {
    let i18n = load_i18n();
    let state = NormalizedState {
        powers: vec![PowerInfo {
            name: "Wrath".into(),
            amount: 1,
        }],
        monsters: vec![MonsterInfo {
            name: "大颚虫".into(),
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
        danger: DangerFlags {
            wrath_stance: true,
            any_monster_attacking: true,
            level: DangerLevel::Danger,
            ..test_state().danger
        },
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("愤怒姿态下受到双倍伤害"));
}

#[test]
fn compact_pile_aggregates_duplicates() {
    let cards = vec![
        CardInfo {
            name: "打击".into(),
            cost: 1,
            card_type: "ATTACK".into(),
            upgraded: false,
            uuid: None,
        };
        3
    ];
    let output = super::compact_pile("=== 抽牌堆", &cards);
    assert!(output.contains("抽牌堆（3张）"));
    assert!(output.contains("打击×3"));
}

#[test]
fn monster_shows_multi_hit_damage() {
    let i18n = load_i18n();
    let state = NormalizedState {
        monsters: vec![MonsterInfo {
            name: "大颚虫".into(),
            index: 0,
            current_hp: Some(44),
            max_hp: Some(46),
            block: Some(0),
            intent: Some("ATTACK".into()),
            damage: Some(4),
            hits: Some(3),
            monster_powers: vec![],
            can_be_killed: false,
            is_scaling: false,
        }],
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("（×3）"));
}
