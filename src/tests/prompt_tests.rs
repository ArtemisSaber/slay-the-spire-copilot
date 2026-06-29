use super::builder::*;
use super::routing::*;
use crate::locales::Locale;
use crate::state::{
    CardInfo, DangerFlags, DangerLevel, MapCoord, MonsterInfo, NormalizedState, PotionInfo,
    PowerInfo, RelicInfo,
};
use crate::test_utils::card;

fn test_locale() -> Locale {
    Locale::load("zh")
}

fn test_state() -> NormalizedState {
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
        ..Default::default()
    }
}

#[test]
fn combat_prompt_shows_all_three_piles() {
    let locale = test_locale();
    let state = NormalizedState {
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
        hand_cards: vec![card("Strike_R", "打击", 1, "ATTACK")],
        draw_pile: vec![card("Defend_R", "防御", 1, "SKILL")],
        discard_pile: vec![card("Bash", "痛击", 2, "ATTACK")],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 手牌"));
    assert!(prompt.contains("=== 抽牌堆"));
    assert!(prompt.contains("=== 弃牌堆"));
    assert!(prompt.contains("打击"));
    assert!(prompt.contains("防御"));
    assert!(prompt.contains("痛击"));
}

#[test]
fn combat_prompt_marks_entry_plan_task() {
    let locale = test_locale();
    let state = NormalizedState {
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
    assert!(prompt.contains("=== 战斗类型 ==="));
    assert!(prompt.contains("=== 战斗概况 ==="));
    assert!(prompt.contains("=== 当前回合 ==="));
}

#[test]
fn combat_prompt_includes_potion_descriptions_but_omits_relic_descriptions() {
    let locale = test_locale();
    let state = NormalizedState {
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
        relics: vec![RelicInfo {
            id: "Burning Blood".into(),
            name: "燃烧之血".into(),
            description: "战斗结束时回复6点生命。".into(),
            counter: None,
            price: None,
        }],
        potions: vec![PotionInfo {
            slot: 0,
            name: "恐惧药水".into(),
            description: "给予3层易伤。".into(),
            price: None,
            can_use: true,
            can_discard: false,
            requires_target: false,
        }],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("燃烧之血"));
    assert!(prompt.contains("恐惧药水"));
    assert!(!prompt.contains("战斗结束时回复6点生命"));
    assert!(prompt.contains("给予3层易伤"));
}

#[test]
fn monster_section_shows_index_and_intent() {
    let locale = test_locale();
    let state = NormalizedState {
        monsters: vec![
            MonsterInfo {
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
            },
            MonsterInfo {
                name: "邪教徒".into(),
                monster_id: None,
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

    let prompt = build_prompt(&state, &locale, false);
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
    let locale = test_locale();
    let state = NormalizedState {
        monsters: vec![MonsterInfo {
            name: "大颚虫".into(),
            monster_id: None,
            index: 0,
            current_hp: Some(44),
            max_hp: Some(46),
            block: Some(0),
            intent: Some("ATTACK".into()),
            damage: Some(12),
            hits: None,
            monster_powers: vec![PowerInfo {
                id: "Strength".into(),
                name: "力量".into(),
                amount: 2,
            }],
            can_be_killed: false,
            is_scaling: true,
        }],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("成长中"));
    assert!(prompt.contains("力量(2)"));
}

#[test]
fn card_reward_shows_card_choices() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("CARD_REWARD".into()),
        character: Some("IRONCLAD".into()),
        floor: Some(3),
        current_hp: Some(62),
        max_hp: Some(75),
        card_reward_choices: vec![
            card("Uppercut", "上勾拳", 2, "ATTACK"),
            card("Anger", "愤怒", 1, "ATTACK"),
        ],
        master_cards: vec![card("Strike_R", "打击", 1, "ATTACK"); 4],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("A. 上勾拳"));
    assert!(prompt.contains("B. 愤怒"));
    assert!(prompt.contains("2费"));
    assert!(prompt.contains("1费"));
}

#[test]
fn card_reward_prompt_marks_pick_or_skip_task() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("CARD_REWARD".into()),
        floor: Some(14),
        card_reward_choices: vec![card("Uppercut", "上勾拳", 2, "ATTACK")],
        skip_available: true,
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.starts_with("[mode: card_reward]"));
    assert!(prompt.contains("上勾拳"));
}

#[test]
fn boss_card_reward_prompt_includes_full_heal_note() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("CARD_REWARD".into()),
        floor: Some(16),
        current_hp: Some(3),
        max_hp: Some(75),
        card_reward_choices: vec![card("Demon Form", "Demon Form", 3, "POWER")],
        skip_available: true,
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(
        prompt.contains(
            "注意：这是 Boss 战后的选牌。下一幕开始会回满血，不要把当前血量当成选牌依据。"
        )
    );
    assert!(prompt.contains("下一幕"));
    assert!(prompt.starts_with("[mode: boss_card_reward]"));
}

#[test]
fn ordinary_card_reward_prompt_omits_full_heal_note() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("CARD_REWARD".into()),
        floor: Some(14),
        current_hp: Some(3),
        max_hp: Some(75),
        card_reward_choices: vec![card("Uppercut", "上勾拳", 2, "ATTACK")],
        skip_available: true,
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(
        !prompt.contains(
            "注意：这是 Boss 战后的选牌。下一幕开始会回满血，不要把当前血量当成选牌依据。"
        )
    );
}

#[test]
fn card_reward_shows_skip_when_available() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("CARD_REWARD".into()),
        card_reward_choices: vec![card("Uppercut", "Uppercut", 2, "ATTACK")],
        master_cards: vec![card("Strike_R", "Strike", 1, "ATTACK")],
        skip_available: true,
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("跳过. 都不选"));
}

#[test]
fn card_reward_no_skip_when_unavailable() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("CARD_REWARD".into()),
        card_reward_choices: vec![card("Uppercut", "Uppercut", 2, "ATTACK")],
        master_cards: vec![card("Strike_R", "Strike", 1, "ATTACK")],
        skip_available: false,
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(!prompt.contains("跳过. 都不选"));
}

#[test]
fn deck_section_groups_single_type() {
    let locale = test_locale();
    let cards = vec![card("Strike_R", "打击", 1, "ATTACK"); 5];
    let output = format_deck_section(&cards, &locale);
    assert!(output.contains("攻击（5张）："));
    assert!(output.contains("打击(1费)（共5张）"));
    assert!(!output.contains("技能"));
    assert!(!output.contains("能力"));
}

#[test]
fn deck_section_shows_cards_by_type_with_counts() {
    let locale = test_locale();
    let mut cards = vec![card("Strike_R", "打击", 1, "ATTACK"); 4];
    cards.extend(vec![card("Defend_R", "防御", 1, "SKILL"); 2]);
    let output = format_deck_section(&cards, &locale);

    assert!(output.contains("=== 卡组 ==="));
    assert!(output.contains("攻击（4张）："));
    assert!(output.contains("技能（2张）："));
    assert!(output.contains("打击(1费)（共4张）"));
    assert!(output.contains("防御(1费)（共2张）"));
}

#[test]
fn prompt_is_structured() {
    let locale = test_locale();
    let prompt = build_prompt(&test_state(), &locale, false);
    assert!(prompt.contains("=== 当前状态 ==="));
    assert!(prompt.starts_with("[mode: generic]"));
}

#[test]
fn rest_prompt_has_translated_options() {
    let locale = test_locale();
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

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("休息"));
    assert!(prompt.contains("锻造"));
}

#[test]
fn rest_prompt_marks_campfire_decision_task() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        rest_options: vec!["rest".into(), "smith".into()],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.starts_with("[mode: rest]"));
    assert!(prompt.contains("休息"));
    assert!(prompt.contains("锻造"));
}

#[test]
fn rest_prompt_requires_smith_upgrade_target() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        rest_options: vec!["smith".into()],
        master_cards: vec![
            card("Bash", "痛击", 2, "ATTACK"),
            card("Armaments", "武装", 1, "SKILL"),
        ],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 可锻造升级目标 ==="));
    assert!(prompt.contains("痛击"));
    assert!(prompt.contains("武装"));
}

#[test]
fn boss_relic_prompt_lists_choices() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("BOSS_REWARD".into()),
        boss_relic_choices: vec![
            RelicInfo {
                id: "Snecko Eye".into(),
                name: "蛇眼".into(),
                description: "".into(),
                counter: None,
                price: None,
            },
            RelicInfo {
                id: "Runic Dome".into(),
                name: "符文圆顶".into(),
                description: "".into(),
                counter: None,
                price: None,
            },
            RelicInfo {
                id: "Cursed Key".into(),
                name: "诅咒钥匙".into(),
                description: "".into(),
                counter: None,
                price: None,
            },
        ],
        master_cards: vec![card("Bash", "Bash", 2, "ATTACK")],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== Boss 遗物 ==="));
    assert!(prompt.contains("A. 蛇眼"));
    assert!(prompt.contains("B. 符文圆顶"));
    assert!(prompt.contains("C. 诅咒钥匙"));
    assert!(prompt.starts_with("[mode: boss_relic]"));
}

#[test]
fn event_prompt_lists_event_text_and_choices() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("EVENT".into()),
        event_id: None,
        event_name: Some("金神像".into()),
        event_body: Some("一个金色神像闪闪发光。".into()),
        event_choices: vec!["拿走神像".into(), "离开".into()],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 事件 ==="));
    assert!(prompt.contains("金神像"));
    assert!(prompt.contains("一个金色神像闪闪发光。"));
    assert!(prompt.contains("A. 拿走神像"));
    assert!(prompt.contains("B. 离开"));
}

#[test]
fn event_prompt_does_not_emit_question_mark_garble() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("EVENT".into()),
        room_type: Some("NeowRoom".into()),
        event_id: None,
        event_name: None,
        event_body: None,
        event_choices: vec![
            "选项 1（事件文本不可读，请在游戏内核对按钮）".into(),
            "选项 2（事件文本不可读，请在游戏内核对按钮）".into(),
        ],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("事件文本不可读（房间：NeowRoom）"));
    assert!(prompt.contains("A. 选项 1（事件文本不可读，请在游戏内核对按钮）"));
    assert!(prompt.contains("B. 选项 2（事件文本不可读，请在游戏内核对按钮）"));
    assert!(!prompt.contains("???"));
}

#[test]
fn generic_prompt_shows_status_and_format() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: None,
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.starts_with("[mode: generic]"));
}

#[test]
fn rest_prompt_advises_rest_when_hp_low() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        current_hp: Some(15),
        max_hp: Some(75),
        rest_options: vec!["rest".into()],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("血量极低，强烈建议休息。"));
}

#[test]
fn rest_prompt_suggests_smith_when_hp_high() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        current_hp: Some(60),
        max_hp: Some(75),
        rest_options: vec!["smith".into()],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("血量健康，可考虑锻造或挖遗物。"));
}

#[test]
fn danger_prefix_shows_wrath_warning() {
    let locale = test_locale();
    let state = NormalizedState {
        powers: vec![PowerInfo {
            id: "".into(),
            name: "Wrath".into(),
            amount: 1,
        }],
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
        danger: DangerFlags {
            wrath_stance: true,
            any_monster_attacking: true,
            level: DangerLevel::Danger,
            ..test_state().danger
        },
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("愤怒姿态下受到双倍伤害"));
}

#[test]
fn compact_pile_aggregates_duplicates() {
    let locale = test_locale();
    let cards = vec![card("Strike_R", "打击", 1, "ATTACK"); 3];
    let output = compact_pile("=== 抽牌堆", &cards, &locale);
    assert!(output.contains("抽牌堆（3张）"));
    assert!(output.contains("打击×3"));
}

#[test]
fn monster_shows_multi_hit_damage() {
    let locale = test_locale();
    let state = NormalizedState {
        monsters: vec![MonsterInfo {
            name: "大颚虫".into(),
            monster_id: None,
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

    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("（×3）"));
}

// --- clean_description tests ---

#[test]
fn clean_strips_markers() {
    let locale = test_locale();
    let out = clean_description("*Smite* into your hand.", &locale);
    assert!(!out.contains('*'));
    assert!(out.contains("Smite"));
}

#[test]
fn clean_replaces_energy_tokens() {
    let locale = test_locale();
    let out = clean_description("gain [E] .", &locale);
    assert_eq!(out, "gain 能量.");
}

#[test]
fn clean_replaces_all_energy_colors() {
    let locale = test_locale();
    let out = clean_description("[R] [G] [B] [W] [E]", &locale);
    assert_eq!(out, "5 能量");
}

#[test]
fn clean_compacts_multiple_energy_tokens() {
    let locale = test_locale();
    let out = clean_description("with [E] [E] [E] .", &locale);
    assert_eq!(out, "with 3 能量.");
}

#[test]
fn clean_preserves_game_text() {
    let locale = test_locale();
    let out = clean_description("Deal 6 damage.", &locale);
    assert_eq!(out, "Deal 6 damage.");
}

#[test]
fn clean_compacts_two_energy() {
    let locale = test_locale();
    let out = clean_description("获得 [R] [R] 。", &locale);
    assert_eq!(out, "获得 2 能量。");
}

#[test]
fn format_card_includes_name_cost_type_and_description() {
    let locale = test_locale();
    let c = CardInfo {
        name: "上勾拳".into(),
        description: "造成 13 点伤害。\n给予 1 层 虚弱 。".into(),
        ..card("Uppercut", "上勾拳", 2, "ATTACK")
    };
    let out = format_card(&c, &locale);
    assert!(out.contains("上勾拳"));
    assert!(out.contains("2费/攻击"));
    assert!(out.contains("13 点伤害"));
    assert!(out.contains("1 层 虚弱"));
}

#[test]
fn format_card_shows_plus_for_upgraded() {
    let locale = test_locale();
    let c = CardInfo {
        name: "防御".into(),
        description: "获得 8 点 格挡 。".into(),
        upgraded: true,
        ..card("Defend_R", "防御", 1, "SKILL")
    };
    let out = format_card(&c, &locale);
    assert!(out.starts_with("+"));
    assert!(out.contains("8 点 格挡"));
}

// --- build_prompt routing ---

#[test]
fn build_prompt_routes_card_reward() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("CARD_REWARD".into()),
        card_reward_choices: vec![card("Strike_R", "Strike", 1, "ATTACK")],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 选牌 ==="));
    assert!(!prompt.contains("=== 手牌"));
}

#[test]
fn build_prompt_routes_rest() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        rest_options: vec!["rest".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 选项 ==="));
    assert!(!prompt.contains("=== 手牌"));
}

#[test]
fn build_prompt_routes_boss_relic() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("BOSS_REWARD".into()),
        boss_relic_choices: vec![RelicInfo {
            id: "Snecko Eye".into(),
            name: "蛇眼".into(),
            description: String::new(),
            counter: None,
            price: None,
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("Boss 遗物"));
}

#[test]
fn build_prompt_routes_event_choice() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("EVENT".into()),
        event_choices: vec!["离开".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.starts_with("[mode: event_choice]"));
}

#[test]
fn build_prompt_routes_combat_when_monsters_present() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: None,
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
    assert!(prompt.contains("=== 怪物"));
    assert!(!prompt.contains("=== 选牌 ==="));
}

#[test]
fn build_prompt_routes_generic_when_no_monsters() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: None,
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 当前状态 ==="));
    assert!(!prompt.contains("=== 怪物"));
    assert!(!prompt.contains("=== 选牌 ==="));
    assert!(!prompt.contains("=== 选项 ==="));
}

// --- added coverage tests ---

#[test]
fn clean_three_energy_tokens() {
    let locale = test_locale();
    let out = clean_description("获得 [R] [R] [R] 。", &locale);
    assert_eq!(out, "获得 3 能量。");
}

#[test]
fn danger_prefix_empty_reasons() {
    let locale = test_locale();
    let state = NormalizedState {
        danger: DangerFlags {
            hp_critical: false,
            incoming_lethal: false,
            no_block_against_hit: false,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Danger,
        },
        ..test_state()
    };
    let output = danger_prefix(&state, &locale);
    assert_eq!(output, "危险！");
}

#[test]
fn status_line_shows_block_warning() {
    let locale = test_locale();
    let state = NormalizedState {
        block: Some(5),
        incoming_damage: 12,
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
    let output = status_line(&state, &locale);
    assert!(output.contains("需格挡！"));
}

#[test]
fn status_line_no_block_warning_when_block_sufficient() {
    let locale = test_locale();
    let state = NormalizedState {
        block: Some(20),
        incoming_damage: 12,
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
    let output = status_line(&state, &locale);
    assert!(!output.contains("需格挡！"));
}

#[test]
fn monster_with_block() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "大颚虫".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(44),
        max_hp: Some(46),
        block: Some(8),
        intent: Some("ATTACK".into()),
        damage: Some(12),
        hits: Some(1),
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("格挡：8"));
}

#[test]
fn rest_shows_upgradeable_cards() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        current_hp: Some(45),
        max_hp: Some(75),
        rest_options: vec!["smith".into()],
        master_cards: vec![
            card("Strike_R", "Strike", 1, "ATTACK"),
            CardInfo {
                upgraded: true,
                ..card("Defend_R", "Defend", 1, "SKILL")
            },
        ],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 可锻造升级目标 ==="));
}

#[test]
fn rest_no_upgradeable_when_all_upgraded() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        current_hp: Some(45),
        max_hp: Some(75),
        rest_options: vec!["smith".into()],
        master_cards: vec![CardInfo {
            upgraded: true,
            ..card("Strike_R", "Strike", 1, "ATTACK")
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(!prompt.contains("=== 可升级卡牌 ==="));
}

#[test]
fn compact_pile_empty() {
    let locale = test_locale();
    let cards: Vec<CardInfo> = vec![];
    let output = compact_pile("=== 抽牌堆", &cards, &locale);
    assert_eq!(output, "=== 抽牌堆（0张）\n");
}

#[test]
fn deck_section_empty() {
    let locale = test_locale();
    let cards: Vec<CardInfo> = vec![];
    let output = format_deck_section(&cards, &locale);
    assert_eq!(output, "");
}

#[test]
fn deck_section_single_cards() {
    let locale = test_locale();
    let cards = vec![card("Strike_R", "打击", 1, "ATTACK")];
    let output = format_deck_section(&cards, &locale);
    assert!(output.contains("打击(1费)"));
    assert!(!output.contains("（共"));
}

// --- describe_path tests ---

fn path_node(symbol: &str) -> MapCoord {
    MapCoord {
        symbol: symbol.to_string(),
        x: 0,
        y: 0,
        children: vec![],
    }
}

fn path_of(symbols: &[&str]) -> Vec<MapCoord> {
    symbols.iter().map(|s| path_node(s)).collect()
}

#[test]
fn describe_path_rest_before_elite_annotated() {
    let path = path_of(&["R", "E", "M", "?"]);
    let desc = describe_path(&path, 1);
    assert!(
        desc.annotations
            .iter()
            .any(|a| a.contains("Rest before first Elite"))
    );
}

#[test]
fn describe_path_no_rest_before_elite_not_annotated() {
    let path = path_of(&["M", "E", "M", "R"]);
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("Rest before")));
}

#[test]
fn describe_path_double_elite_annotated() {
    let path = path_of(&["R", "E", "M", "E", "R"]);
    let desc = describe_path(&path, 1);
    assert!(desc.annotations.iter().any(|a| a.contains("Double Elite")));
}

#[test]
fn describe_path_single_elite_no_double_warning() {
    let path = path_of(&["R", "E", "M", "R", "M", "R"]);
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("Double Elite")));
}

#[test]
fn describe_path_risk_gap_exceeds_threshold_act1() {
    let path = path_of(&["E", "M", "M", "M", "R"]); // E(10)+M(2)+M(2)+M(2)=16 > 15
    let desc = describe_path(&path, 1);
    assert!(desc.annotations.iter().any(|a| a.contains("E→R gap")));
    assert!(desc.annotations.iter().any(|a| a.contains("16")));
}

#[test]
fn describe_path_risk_gap_below_threshold_act1() {
    let path = path_of(&["E", "M", "M", "R"]); // E(10)+M(2)+M(2)=14 ≤ 15
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("E→R gap")));
}

#[test]
fn describe_path_risk_gap_direct_e_r() {
    let path = path_of(&["E", "R"]); // E(10)=10 ≤ 15
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("E→R gap")));
}

#[test]
fn describe_path_risk_varies_by_act() {
    let path = path_of(&["E", "M", "M", "R"]);
    // Act 1: E(10)+M(2)+M(2)=14 ≤ 15 (silent)
    let desc1 = describe_path(&path, 1);
    assert!(!desc1.annotations.iter().any(|a| a.contains("E→R gap")));
    // Act 2: E(10)+M(4)+M(4)=18 > 15 (warned)
    let desc2 = describe_path(&path, 17);
    assert!(desc2.annotations.iter().any(|a| a.contains("E→R gap")));
    assert!(desc2.annotations.iter().any(|a| a.contains("18")));
}

#[test]
fn describe_path_risk_resets_at_r() {
    // Segment 1: E→M→R gap=12 ≤ 15, Segment 2: E→M→R gap=12 ≤ 15
    let path = path_of(&["E", "M", "R", "E", "M", "R"]);
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("E→R gap")));
}

#[test]
fn describe_path_risk_from_start_to_r() {
    // No R before first E, segment is start→R: M→E→M→R gap from E=12 ≤ 15
    let path = path_of(&["M", "E", "M", "R"]);
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("E→R gap")));
}

#[test]
fn describe_path_no_shop() {
    let path = path_of(&["M", "R", "M", "R"]);
    let desc = describe_path(&path, 1);
    assert!(desc.annotations.iter().any(|a| a.contains("No shop")));
}

#[test]
fn describe_path_shop_early() {
    // $ at index 1, path length 9 → position 0.125 < 0.33 = early
    let path = path_of(&["M", "$", "M", "?", "R", "E", "M", "R", "M"]);
    let desc = describe_path(&path, 1);
    assert!(desc.annotations.iter().any(|a| a.contains("Shop (early)")));
}

#[test]
fn describe_path_shop_mid() {
    // $ at index 4, path length 9 → position 0.5 < 0.66 = mid
    let path = path_of(&["M", "M", "M", "R", "$", "E", "M", "R", "M"]);
    let desc = describe_path(&path, 1);
    assert!(desc.annotations.iter().any(|a| a.contains("Shop (mid)")));
}

#[test]
fn describe_path_shop_late() {
    // $ at index 7, path length 9 → position 0.875 > 0.66 = late
    let path = path_of(&["M", "M", "M", "R", "E", "M", "?", "$", "R"]);
    let desc = describe_path(&path, 1);
    assert!(desc.annotations.iter().any(|a| a.contains("Shop (late)")));
}

#[test]
fn describe_path_no_elites_no_warnings() {
    let path = path_of(&["M", "?", "R", "M", "R"]);
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("Elite")));
    assert!(!desc.annotations.iter().any(|a| a.contains("E→R")));
}

#[test]
fn describe_path_counts_includes_all_types() {
    let path = path_of(&["M", "E", "?", "$", "R", "T", "M"]);
    let desc = describe_path(&path, 1);
    assert!(desc.counts.contains("Monsters:2"));
    assert!(desc.counts.contains("Elites:1"));
    assert!(desc.counts.contains("Events:1"));
    assert!(desc.counts.contains("Shops:1"));
    assert!(desc.counts.contains("Rests:1"));
    assert!(desc.counts.contains("Treasures:1"));
}

#[test]
fn describe_path_route_chain_compact() {
    let path = path_of(&["M", "R", "E", "?", "R"]);
    let desc = describe_path(&path, 1);
    assert_eq!(desc.route_chain, "M→R→E→?→R");
}

#[test]
fn position_label_uses_readable_ordinals() {
    assert_eq!(position_label(2, 5, &Locale::load("en")), "3rd from left");
    assert_eq!(position_label(2, 5, &Locale::load("zh")), "左起第3个");
    assert_eq!(position_label(2, 5, &Locale::load("ja")), "左から3番目");
    assert_eq!(position_label(2, 5, &Locale::load("ko")), "왼쪽에서 3번째");
}

#[test]
fn compact_route_chain_shortens_long_routes() {
    assert_eq!(
        compact_route_chain("M→$→M→?→M→?→R→M→T→M→E→R→M→M→R"),
        "M→$→M→?→M→…→R→M→M→R"
    );
}

#[test]
fn describe_path_exposes_ranking_metrics() {
    let path = path_of(&["M", "$", "R", "E", "?", "R"]);
    let desc = describe_path(&path, 1);
    assert_eq!(desc.metrics.counts.monsters, 1);
    assert_eq!(desc.metrics.counts.shops, 1);
    assert_eq!(desc.metrics.counts.elites, 1);
    assert_eq!(desc.metrics.shop_timing, ShopTiming::Early);
    assert!(desc.metrics.rest_before_first_elite);
    assert!(!desc.metrics.double_elite_without_rest);
}

#[test]
fn evaluate_path_records_pros_and_cons() {
    let state = NormalizedState {
        floor: Some(1),
        current_hp: Some(62),
        max_hp: Some(75),
        gold: Some(180),
        ..test_state()
    };
    let path = path_of(&["M", "$", "R", "E", "M", "R"]);
    let eval = evaluate_path(&path, &state, false);
    assert!(eval.score > 80.0);
    assert!(eval.pros.iter().any(|p| p.contains("early shop")));
    assert!(
        eval.pros
            .iter()
            .any(|p| p.contains("rest before first elite"))
    );
    assert!(!eval.cons.iter().any(|c| c.contains("double elite")));
}

#[test]
fn evaluate_path_prefers_healthy_gold_route_with_early_shop_and_rest_elite() {
    let state = NormalizedState {
        floor: Some(1),
        current_hp: Some(68),
        max_hp: Some(75),
        gold: Some(180),
        ..test_state()
    };
    let shop_elite = path_of(&["M", "$", "M", "R", "E", "R"]);
    let safe_no_shop = path_of(&["M", "?", "?", "M", "R", "M", "R"]);
    let shop_elite_eval = evaluate_path(&shop_elite, &state, false);
    let safe_no_shop_eval = evaluate_path(&safe_no_shop, &state, false);
    assert!(shop_elite_eval.score > safe_no_shop_eval.score);
}

#[test]
fn evaluate_path_penalizes_double_elite_when_hp_is_low() {
    let state = NormalizedState {
        floor: Some(1),
        current_hp: Some(20),
        max_hp: Some(80),
        gold: Some(50),
        ..test_state()
    };
    let double_elite = path_of(&["E", "M", "E", "M", "R"]);
    let safe_route = path_of(&["M", "?", "R", "M", "R"]);
    let double_elite_eval = evaluate_path(&double_elite, &state, false);
    let safe_route_eval = evaluate_path(&safe_route, &state, false);
    assert!(safe_route_eval.score > double_elite_eval.score);
    assert!(
        double_elite_eval
            .cons
            .iter()
            .any(|c| c.contains("double elite"))
    );
}

// --- build_map_suggestion tests ---

#[test]
fn build_map_suggestion_includes_route_chains_and_counts() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 0, 0, vec![(0, 1)]),
            make_node("?", 0, 1, vec![(0, 2)]),
            make_node("R", 0, 2, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(0),
        map_current_y: Some(0),
        ..test_state()
    };
    let prompt = build_map_suggestion(&state, &locale);
    assert!(prompt.contains("Route 1 (唯一)"));
    assert!(prompt.contains("Candidate 1"));
    assert!(prompt.contains("Recommendation label: Route 1 (唯一) — M→?→R"));
    assert!(prompt.contains("M→?→R"));
    assert!(prompt.contains("Monsters:1"));
    assert!(prompt.starts_with("[mode: map_suggestion]"));
}

#[test]
fn build_map_suggestion_multiple_paths_labeled() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 0, 0, vec![(0, 1), (1, 1)]),
            make_node("R", 0, 1, vec![(0, 2)]),
            make_node("E", 1, 1, vec![(1, 2)]),
            make_node("?", 0, 2, vec![]),
            make_node("$", 1, 2, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(0),
        map_current_y: Some(0),
        ..test_state()
    };
    let prompt = build_map_suggestion(&state, &locale);
    assert!(prompt.contains("Route 1 (左侧)"));
    assert!(prompt.contains("Route 2 (右侧)"));
    assert!(!prompt.contains("Route 3"));
}

#[test]
fn build_map_suggestion_limits_current_route_candidates() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(5),
        map_nodes: vec![
            make_node(
                "M",
                0,
                0,
                vec![(0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1)],
            ),
            make_node("M", 0, 1, vec![]),
            make_node("?", 1, 1, vec![]),
            make_node("R", 2, 1, vec![]),
            make_node("$", 3, 1, vec![]),
            make_node("E", 4, 1, vec![]),
            make_node("T", 5, 1, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(0),
        map_current_y: Some(0),
        ..test_state()
    };
    let prompt = build_map_suggestion(&state, &locale);
    assert_eq!(prompt.matches("Candidate ").count(), 5);
    assert!(prompt.contains("Candidate 5"));
    assert!(!prompt.contains("Candidate 6"));
    assert!(prompt.contains("Recommendation label: Route"));
}

#[test]
fn build_map_suggestion_root_selection() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(1),
        map_nodes: vec![
            make_node("M", 0, 0, vec![(0, 1)]),
            make_node("?", 0, 1, vec![]),
            make_node("E", 1, 0, vec![(1, 1)]),
            make_node("R", 1, 1, vec![]),
        ],
        map_first_node_chosen: Some(false),
        ..test_state()
    };
    let prompt = build_map_suggestion(&state, &locale);
    assert!(prompt.contains("Root 1 (左侧)"));
    assert!(prompt.contains("Root 2 (右侧)"));
    assert!(prompt.contains("Recommendation label: Root"));
}

#[test]
fn build_map_suggestion_limits_root_candidates() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(1),
        map_nodes: vec![
            make_node("M", 0, 0, vec![(0, 1)]),
            make_node("?", 0, 1, vec![]),
            make_node("M", 1, 0, vec![(1, 1)]),
            make_node("R", 1, 1, vec![]),
            make_node("M", 2, 0, vec![(2, 1)]),
            make_node("$", 2, 1, vec![]),
            make_node("M", 3, 0, vec![(3, 1)]),
            make_node("E", 3, 1, vec![]),
            make_node("M", 4, 0, vec![(4, 1)]),
            make_node("T", 4, 1, vec![]),
            make_node("M", 5, 0, vec![(5, 1)]),
            make_node("M", 5, 1, vec![]),
        ],
        map_first_node_chosen: Some(false),
        ..test_state()
    };
    let prompt = build_map_suggestion(&state, &locale);
    assert_eq!(prompt.matches("Candidate ").count(), 5);
    assert!(prompt.contains("Candidate 5"));
    assert!(!prompt.contains("Candidate 6"));
}

#[test]
fn build_map_suggestion_includes_status_line() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        character: Some("IRONCLAD".into()),
        floor: Some(5),
        current_hp: Some(62),
        max_hp: Some(75),
        gold: Some(180),
        map_nodes: vec![
            make_node("M", 0, 0, vec![(0, 1)]),
            make_node("R", 0, 1, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(0),
        map_current_y: Some(0),
        ..test_state()
    };
    let prompt = build_map_suggestion(&state, &locale);
    assert!(prompt.contains("铁甲战士"));
    assert!(prompt.contains("62/75"));
    assert!(prompt.contains("180"));
}

#[test]
fn build_map_suggestion_empty_paths_graceful() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(5),
        map_nodes: vec![],
        map_first_node_chosen: Some(true),
        map_current_x: Some(99),
        map_current_y: Some(99),
        ..test_state()
    };
    let prompt = build_map_suggestion(&state, &locale);
    assert!(prompt.starts_with("[mode: map_suggestion]"));
}

// --- build_map_crossroad tests ---

#[test]
fn build_map_crossroad_shows_next_nodes() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 1, 2, vec![(1, 3), (2, 3)]),
            make_node("E", 1, 3, vec![(1, 4)]),
            make_node("?", 2, 3, vec![(2, 4)]),
            make_node("R", 1, 4, vec![]),
            make_node("R", 2, 4, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(1),
        map_current_y: Some(2),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert!(prompt.contains("(左侧)"));
    assert!(prompt.contains("(右侧)"));
    assert!(!prompt.contains("(中间)"));
}

#[test]
fn build_map_crossroad_no_full_coordinate_routes() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 1, 2, vec![(1, 3)]),
            make_node("E", 1, 3, vec![(1, 4)]),
            make_node("R", 1, 4, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(1),
        map_current_y: Some(2),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert!(!prompt.contains("(1,"));
    assert!(!prompt.contains("(1,2)"));
    assert!(!prompt.contains("(1,3)"));
}

#[test]
fn build_map_crossroad_includes_ahead_chain() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 1, 2, vec![(1, 3)]),
            make_node("E", 1, 3, vec![(1, 4)]),
            make_node("R", 1, 4, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(1),
        map_current_y: Some(2),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert!(prompt.contains("Ahead"));
}

#[test]
fn build_map_crossroad_includes_annotations() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 1, 2, vec![(1, 3)]),
            make_node("E", 1, 3, vec![(1, 4)]),
            make_node("M", 1, 4, vec![(1, 5)]),
            make_node("M", 1, 5, vec![(1, 6)]),
            make_node("M", 1, 6, vec![(1, 7)]),
            make_node("R", 1, 7, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(1),
        map_current_y: Some(2),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert!(prompt.contains("E→R gap"));
}

#[test]
fn build_map_crossroad_softens_no_shop_when_shop_visited() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 1, 2, vec![(1, 3)]),
            make_node("M", 1, 3, vec![(1, 4)]),
            make_node("R", 1, 4, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(1),
        map_current_y: Some(2),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, true);
    assert!(prompt.contains("No shop ahead"));
}

#[test]
fn build_map_crossroad_keeps_no_shop_when_not_visited() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 1, 2, vec![(1, 3)]),
            make_node("M", 1, 3, vec![(1, 4)]),
            make_node("R", 1, 4, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(1),
        map_current_y: Some(2),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert!(prompt.contains("✗ No shop"));
    assert!(!prompt.contains("No shop ahead"));
}

// --- enumerate_paths tests ---

fn make_node(symbol: &str, x: i64, y: i64, children: Vec<(i64, i64)>) -> MapCoord {
    MapCoord {
        symbol: symbol.to_string(),
        x,
        y,
        children,
    }
}

#[test]
fn enumerate_paths_single_root_no_branches() {
    let nodes = vec![
        make_node("M", 0, 0, vec![(0, 1)]),
        make_node("R", 0, 1, vec![(0, 2)]),
        make_node("?", 0, 2, vec![]),
    ];
    let paths = enumerate_paths(0, 0, &nodes);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].len(), 3);
    assert_eq!(paths[0][0].symbol, "M");
    assert_eq!(paths[0][1].symbol, "R");
    assert_eq!(paths[0][2].symbol, "?");
}

#[test]
fn enumerate_paths_multiple_branches() {
    let nodes = vec![
        make_node("M", 0, 0, vec![(0, 1), (1, 1)]),
        make_node("R", 0, 1, vec![(0, 2)]),
        make_node("?", 1, 1, vec![(1, 2)]),
        make_node("$", 0, 2, vec![]),
        make_node("T", 1, 2, vec![]),
    ];
    let paths = enumerate_paths(0, 0, &nodes);
    assert_eq!(paths.len(), 2);
    // Path A: M → R → $
    assert!(
        paths
            .iter()
            .any(|p| p[1].symbol == "R" && p[2].symbol == "$")
    );
    // Path B: M → ? → T
    assert!(
        paths
            .iter()
            .any(|p| p[1].symbol == "?" && p[2].symbol == "T")
    );
}

#[test]
fn enumerate_paths_missing_child_terminates() {
    let nodes = vec![
        make_node("M", 0, 0, vec![(0, 1)]),
        make_node("R", 0, 1, vec![(0, 2)]),
        // (0,2) missing — child points to nonexistent node
    ];
    let paths = enumerate_paths(0, 0, &nodes);
    assert_eq!(paths.len(), 1);
    assert_eq!(paths[0].len(), 2); // terminates at R
    assert_eq!(paths[0][1].symbol, "R");
}

#[test]
fn enumerate_paths_start_not_found_returns_empty() {
    let nodes = vec![make_node("M", 0, 0, vec![])];
    let paths = enumerate_paths(99, 99, &nodes);
    assert!(paths.is_empty());
}

#[test]
fn enumerate_paths_from_roots_groups_by_root() {
    let nodes = vec![
        make_node("M", 0, 0, vec![(0, 1)]),
        make_node("R", 0, 1, vec![]),
        make_node("E", 1, 0, vec![(1, 1)]),
        make_node("$", 1, 1, vec![]),
    ];
    let result = enumerate_paths_from_roots(&nodes);
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].root.symbol, "M");
    assert_eq!(result[1].root.symbol, "E");
    assert_eq!(result[0].paths.len(), 1);
    assert_eq!(result[1].paths.len(), 1);
}

#[test]
fn enumerate_paths_from_roots_single_root() {
    let nodes = vec![
        make_node("M", 0, 0, vec![(0, 1)]),
        make_node("R", 0, 1, vec![]),
    ];
    let result = enumerate_paths_from_roots(&nodes);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].paths.len(), 1);
}

#[test]
fn summarize_path_all_types() {
    let path = vec![
        make_node("M", 0, 0, vec![]),
        make_node("E", 1, 0, vec![]),
        make_node("?", 2, 0, vec![]),
        make_node("$", 3, 0, vec![]),
        make_node("R", 4, 0, vec![]),
        make_node("T", 5, 0, vec![]),
        make_node("M", 6, 0, vec![]),
    ];
    let summary = summarize_path(&path);
    assert!(summary.contains("Monsters:2"));
    assert!(summary.contains("Elites:1"));
    assert!(summary.contains("Events:1"));
    assert!(summary.contains("Shops:1"));
    assert!(summary.contains("Rests:1"));
    assert!(summary.contains("Treasures:1"));
}

#[test]
fn summarize_path_empty() {
    let path: Vec<MapCoord> = vec![];
    let summary = summarize_path(&path);
    assert_eq!(summary, "");
}

#[test]
fn summarize_path_only_monsters() {
    let path = vec![
        make_node("M", 0, 0, vec![]),
        make_node("M", 1, 0, vec![]),
        make_node("M", 2, 0, vec![]),
    ];
    let summary = summarize_path(&path);
    assert!(summary.contains("Monsters:3"));
    assert!(!summary.contains("Elites"));
    assert!(!summary.contains("Events"));
}

#[test]
fn shop_prompt_contains_mode_tag() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("SHOP_SCREEN".into()),
        gold: Some(190),
        shop_cards: vec![CardInfo {
            name: "断魂斩".into(),
            cost: 2,
            price: Some(79),
            ..card("Sever Soul", "断魂斩", 2, "ATTACK")
        }],
        shop_relics: vec![RelicInfo {
            name: "铲子".into(),
            description: "现在你可以在休息处 挖掘 遗物。".into(),
            price: Some(286),
            ..RelicInfo {
                id: "Shovel".into(),
                name: "铲子".into(),
                description: "挖掘".into(),
                counter: None,
                price: None,
            }
        }],
        shop_potions: vec![PotionInfo {
            slot: 0,
            name: "再生药水".into(),
            description: "获得 5 层 再生 。".into(),
            price: Some(79),
            can_use: false,
            can_discard: false,
            requires_target: false,
        }],
        purge_available: true,
        purge_cost: Some(75),
        master_cards: vec![card("Strike_R", "打击", 1, "ATTACK")],
        deck_names: vec!["打击".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);

    assert!(prompt.contains("[mode: shop]"));
    assert!(prompt.contains("=== 商店 ==="));
    assert!(prompt.contains("=== 待购卡牌 ==="));
    assert!(prompt.contains("断魂斩(2费/攻击)"));
    assert!(prompt.contains("79 gold"));
    assert!(prompt.contains("=== 待购遗物 ==="));
    assert!(prompt.contains("A. 铲子"));
    assert!(prompt.contains("286 gold"));
    assert!(prompt.contains("=== 待购药水 ==="));
    assert!(prompt.contains("再生药水"));
    assert!(prompt.contains("79 gold"));
    assert!(prompt.contains("=== 删牌服务 ==="));
    assert!(prompt.contains("Remove a card for 75 gold"));
    assert!(prompt.contains("打击"));
}

#[test]
fn shop_state_parses_from_fixture() {
    let raw = crate::test_utils::load_fixture("shop-state.json");
    let locale = test_locale();
    let state = NormalizedState::from_raw(&raw, &locale);

    assert_eq!(state.screen_type.as_deref(), Some("SHOP_SCREEN"));
    assert_eq!(state.gold, Some(190));
    assert_eq!(state.current_hp, Some(33));
    assert_eq!(state.max_hp, Some(80));

    assert_eq!(state.shop_cards.len(), 7);
    assert!(
        state
            .shop_cards
            .iter()
            .any(|c| c.name == "断魂斩" && c.price == Some(79))
    );
    assert!(
        state
            .shop_cards
            .iter()
            .any(|c| c.name == "武装" && c.price == Some(51))
    );

    assert_eq!(state.shop_relics.len(), 3);
    assert!(
        state
            .shop_relics
            .iter()
            .any(|r| r.name == "铲子" && r.price == Some(286))
    );
    assert!(
        state
            .shop_relics
            .iter()
            .any(|r| r.name == "硫磺" && r.price == Some(151))
    );

    assert_eq!(state.shop_potions.len(), 3);
    assert!(
        state
            .shop_potions
            .iter()
            .any(|p| p.name == "鲜血药水" && p.price == Some(50))
    );

    assert!(state.purge_available);
    assert_eq!(state.purge_cost, Some(75));
}

#[test]
fn shop_prompt_from_fixture_includes_real_data() {
    let raw = crate::test_utils::load_fixture("shop-state.json");
    let locale = test_locale();
    let state = NormalizedState::from_raw(&raw, &locale);
    let prompt = build_prompt(&state, &locale, false);

    assert!(prompt.contains("[mode: shop]"));
    assert!(prompt.contains("=== 商店 ==="));
    assert!(prompt.contains("血量：33/80(41%)"));
    assert!(prompt.contains("金币：190"));
    assert!(prompt.contains("=== 待购卡牌 ==="));
    assert!(prompt.contains("A. 断魂斩(2费/攻击)"));
    assert!(prompt.contains("B. 金刚臂(2费/攻击)"));
    assert!(prompt.contains("C. 武装(1费/技能)"));
    assert!(prompt.contains("=== 待购遗物 ==="));
    assert!(prompt.contains("A. 铲子"));
    assert!(prompt.contains("=== 待购药水 ==="));
    assert!(prompt.contains("A. 再生药水"));
    assert!(prompt.contains("=== 删牌服务 ==="));
    assert!(prompt.contains("Remove a card for 75 gold"));
    assert!(prompt.contains("打击"));
    assert!(prompt.contains("痛击"));
}

#[test]
fn shop_prompt_without_purge_omits_removal_section() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("SHOP_SCREEN".into()),
        gold: Some(100),
        shop_cards: vec![],
        shop_relics: vec![],
        shop_potions: vec![],
        purge_available: false,
        purge_cost: None,
        master_cards: vec![],
        deck_names: vec![],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);

    assert!(prompt.contains("[mode: shop]"));
    assert!(!prompt.contains("=== 待购卡牌 ==="));
    assert!(!prompt.contains("=== 删牌服务 ==="));
    assert!(!prompt.contains("Remove a card"));
}

#[test]
fn build_hand_select_shows_purpose_with_card_in_play() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("HAND_SELECT".into()),
        hand: vec![card("Strike_R", "打击", 1, "ATTACK")],
        current_action: Some("ExhaustAction".into()),
        card_in_play: Some(card("Burning Pact", "燃烧契约", 1, "SKILL")),
        hand_select_max_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: hand_select]"));
    assert!(prompt.contains("Exhaust a card"));
    assert!(prompt.contains("燃烧契约"));
}

#[test]
fn build_hand_select_shows_selected_cards() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("HAND_SELECT".into()),
        hand: vec![card("Strike_R", "打击", 1, "ATTACK")],
        hand_select_selected: vec![card("Defend_R", "防御", 1, "SKILL")],
        hand_select_max_cards: Some(2),
        hand_select_can_pick_zero: true,
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("已选择"));
    assert!(prompt.contains("防御"));
    assert!(prompt.contains("Can skip: yes"));
}

#[test]
fn build_grid_select_upgrade_shows_purpose() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("GRID".into()),
        grid_cards: vec![
            card("Strike_R", "打击", 1, "ATTACK"),
            card("Defend_R", "防御", 1, "SKILL"),
        ],
        grid_for_upgrade: true,
        grid_num_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: grid_select]"));
    assert!(prompt.contains("升级"));
    assert!(prompt.contains("打击"));
}

#[test]
fn build_grid_select_purge_shows_purpose() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("GRID".into()),
        grid_cards: vec![card("Strike_R", "打击", 1, "ATTACK")],
        grid_for_purge: true,
        grid_num_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: grid_select]"));
    assert!(prompt.contains("移除"));
}

// --- build_grid_select: transform and default ---

#[test]
fn build_grid_select_transform_shows_purpose() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("GRID".into()),
        grid_cards: vec![
            card("Strike_R", "打击", 1, "ATTACK"),
            card("Defend_R", "防御", 1, "SKILL"),
        ],
        grid_for_transform: true,
        grid_num_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: grid_select]"));
    assert!(prompt.contains("变化"));
}

#[test]
fn build_grid_select_default_shows_generic_purpose() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("GRID".into()),
        grid_cards: vec![card("Strike_R", "打击", 1, "ATTACK")],
        grid_for_upgrade: false,
        grid_for_transform: false,
        grid_for_purge: false,
        grid_num_cards: Some(2),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: grid_select]"));
    assert!(prompt.contains("选择一张卡牌"));
}

#[test]
fn build_grid_select_falls_back_to_hand_when_grid_cards_empty() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("GRID".into()),
        grid_cards: vec![],
        hand: vec![card("Bash", "痛击", 2, "ATTACK")],
        grid_for_upgrade: true,
        grid_num_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("痛击"));
    assert!(prompt.contains("升级"));
}

#[test]
fn build_grid_select_no_num_cards() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("GRID".into()),
        grid_cards: vec![card("Strike_R", "打击", 1, "ATTACK")],
        grid_for_purge: true,
        grid_num_cards: None,
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: grid_select]"));
    assert!(prompt.contains("移除"));
    assert!(!prompt.contains("Select"));
}

// --- build_hand_select: without card_in_play and different actions ---

#[test]
fn build_hand_select_without_card_in_play() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("HAND_SELECT".into()),
        hand: vec![card("Strike_R", "打击", 1, "ATTACK")],
        current_action: Some("DiscardAction".into()),
        card_in_play: None,
        hand_select_max_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: hand_select]"));
    assert!(prompt.contains("Discard a card"));
    assert!(!prompt.contains("Card playing"));
}

#[test]
fn build_hand_select_put_on_deck_action() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("HAND_SELECT".into()),
        hand: vec![card("Defend_R", "防御", 1, "SKILL")],
        current_action: Some("PutOnDeckAction".into()),
        card_in_play: None,
        hand_select_max_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("Put a card on top of your draw pile"));
}

#[test]
fn build_hand_select_unknown_action() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("HAND_SELECT".into()),
        hand: vec![card("Strike_R", "打击", 1, "ATTACK")],
        current_action: Some("SomeUnknownAction".into()),
        card_in_play: None,
        hand_select_max_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("SomeUnknownAction"));
}

#[test]
fn build_hand_select_cannot_skip() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("HAND_SELECT".into()),
        hand: vec![card("Strike_R", "打击", 1, "ATTACK")],
        current_action: Some("ExhaustAction".into()),
        card_in_play: Some(card("Burning Pact", "燃烧契约", 1, "SKILL")),
        hand_select_max_cards: Some(1),
        hand_select_can_pick_zero: false,
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("Can skip: no"));
}

// --- danger_prefix: multiple reasons combined, Caution, Safe ---

#[test]
fn danger_prefix_multiple_reasons() {
    let locale = test_locale();
    let state = NormalizedState {
        danger: DangerFlags {
            incoming_lethal: true,
            hp_critical: true,
            no_block_against_hit: true,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Danger,
        },
        ..test_state()
    };
    let output = danger_prefix(&state, &locale);
    assert!(output.contains("危险！"));
    assert!(output.contains("致命伤害"));
    assert!(output.contains("血量危急"));
    assert!(output.contains("无格挡"));
}

#[test]
fn danger_prefix_caution_level() {
    let locale = test_locale();
    let state = NormalizedState {
        danger: DangerFlags {
            hp_critical: false,
            incoming_lethal: false,
            no_block_against_hit: false,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Caution,
        },
        ..test_state()
    };
    let output = danger_prefix(&state, &locale);
    assert_eq!(output, "小心行事。");
}

#[test]
fn danger_prefix_safe_level() {
    let locale = test_locale();
    let state = NormalizedState {
        danger: DangerFlags {
            hp_critical: false,
            incoming_lethal: false,
            no_block_against_hit: false,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Safe,
        },
        ..test_state()
    };
    let output = danger_prefix(&state, &locale);
    assert_eq!(output, "形势不错。");
}

#[test]
fn danger_prefix_incoming_lethal_alone() {
    let locale = test_locale();
    let state = NormalizedState {
        danger: DangerFlags {
            incoming_lethal: true,
            hp_critical: false,
            no_block_against_hit: false,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Danger,
        },
        ..test_state()
    };
    let output = danger_prefix(&state, &locale);
    assert!(output.contains("危险！"));
    assert!(output.contains("致命伤害"));
}

#[test]
fn danger_prefix_hp_critical_alone() {
    let locale = test_locale();
    let state = NormalizedState {
        danger: DangerFlags {
            hp_critical: true,
            incoming_lethal: false,
            no_block_against_hit: false,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Danger,
        },
        ..test_state()
    };
    let output = danger_prefix(&state, &locale);
    assert!(output.contains("危险！"));
    assert!(output.contains("血量危急"));
}

#[test]
fn danger_prefix_no_block_alone() {
    let locale = test_locale();
    let state = NormalizedState {
        danger: DangerFlags {
            hp_critical: false,
            incoming_lethal: false,
            no_block_against_hit: true,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Danger,
        },
        ..test_state()
    };
    let output = danger_prefix(&state, &locale);
    assert!(output.contains("危险！"));
    assert!(output.contains("无格挡"));
}

// --- Monster section: intent variants ---

#[test]
fn monster_intent_attack_buff() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "拜蛇术士".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(30),
        max_hp: Some(30),
        block: Some(0),
        intent: Some("ATTACK_BUFF".into()),
        damage: Some(10),
        hits: Some(1),
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("攻击+增益"));
}

#[test]
fn monster_intent_defend() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "哨兵".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(30),
        max_hp: Some(30),
        block: Some(10),
        intent: Some("DEFEND".into()),
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("防御"));
    assert!(output.contains("伤害：无"));
}

#[test]
fn monster_intent_sleep() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "地精大法师".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(20),
        max_hp: Some(20),
        block: Some(0),
        intent: Some("SLEEP".into()),
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("睡眠"));
}

#[test]
fn monster_intent_stun() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "哨兵".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(30),
        max_hp: Some(30),
        block: Some(0),
        intent: Some("STUN".into()),
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("眩晕"));
}

#[test]
fn monster_intent_unknown() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "神秘生物".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(10),
        max_hp: Some(10),
        block: Some(0),
        intent: Some("UNKNOWN".into()),
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("未知"));
}

#[test]
fn monster_intent_escape() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "盗贼".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(5),
        max_hp: Some(30),
        block: Some(0),
        intent: Some("ESCAPE".into()),
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("逃跑"));
}

#[test]
fn monster_intent_magic() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "六火亡魂".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(60),
        max_hp: Some(60),
        block: Some(0),
        intent: Some("MAGIC".into()),
        damage: Some(8),
        hits: Some(1),
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("法术"));
}

#[test]
fn monster_intent_raw_string_fallback() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "怪异".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(10),
        max_hp: Some(10),
        block: Some(0),
        intent: Some("CUSTOM_INTENT".into()),
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("CUSTOM_INTENT"));
}

#[test]
fn monster_no_intent() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "测试怪物".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(10),
        max_hp: Some(10),
        block: Some(0),
        intent: None,
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(!output.contains("意图"));
}

#[test]
fn monster_is_scaling_with_powers() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "邪教徒".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(18),
        max_hp: Some(40),
        block: Some(0),
        intent: Some("BUFF".into()),
        damage: None,
        hits: None,
        monster_powers: vec![PowerInfo {
            id: "Strength".into(),
            name: "力量".into(),
            amount: 1,
        }],
        can_be_killed: false,
        is_scaling: true,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("成长中"));
    assert!(output.contains("力量(1)"));
}

#[test]
fn build_monsters_section_empty() {
    let locale = test_locale();
    let state = NormalizedState {
        monsters: vec![],
        ..test_state()
    };
    let output = build_monsters_section(&state, &locale);
    assert!(output.is_empty());
}

#[test]
fn build_hand_section_when_empty() {
    let locale = test_locale();
    let state = NormalizedState {
        hand_cards: vec![],
        ..test_state()
    };
    let output = build_hand_section(&state, &locale);
    assert!(output.is_empty());
}

#[test]
fn build_hand_section_with_cards() {
    let locale = test_locale();
    let state = NormalizedState {
        hand_cards: vec![
            card("Strike_R", "打击", 1, "ATTACK"),
            card("Defend_R", "防御", 1, "SKILL"),
        ],
        ..test_state()
    };
    let output = build_hand_section(&state, &locale);
    assert!(output.contains("手牌（2张"));
    assert!(output.contains("打击"));
    assert!(output.contains("防御"));
}

// --- turn_status_line tests ---

#[test]
fn turn_status_line_with_powers_and_potions() {
    let locale = test_locale();
    let state = NormalizedState {
        current_hp: Some(50),
        max_hp: Some(75),
        block: Some(10),
        energy: Some(3),
        powers: vec![PowerInfo {
            id: "Strength".into(),
            name: "力量".into(),
            amount: 3,
        }],
        potions: vec![PotionInfo {
            slot: 0,
            name: "再生药水".into(),
            description: "获得 5 层 再生 。".into(),
            price: None,
            can_use: true,
            can_discard: false,
            requires_target: false,
        }],
        incoming_damage: 0,
        danger: danger_safe(),
        ..test_state()
    };
    let output = turn_status_line(&state, &locale);
    assert!(output.contains("力量(3)"));
    assert!(output.contains("再生药水"));
    assert!(output.contains("50/75"));
}

#[test]
fn turn_status_line_with_incoming_damage_and_block() {
    let locale = test_locale();
    let state = NormalizedState {
        current_hp: Some(50),
        max_hp: Some(75),
        block: Some(5),
        energy: Some(3),
        incoming_damage: 12,
        danger: danger_safe(),
        ..test_state()
    };
    let output = turn_status_line(&state, &locale);
    assert!(output.contains("12"));
    assert!(output.contains("需格挡"));
}

#[test]
fn turn_status_line_with_incoming_damage_no_block_warning() {
    let locale = test_locale();
    let state = NormalizedState {
        current_hp: Some(50),
        max_hp: Some(75),
        block: Some(20),
        energy: Some(3),
        incoming_damage: 12,
        danger: danger_safe(),
        ..test_state()
    };
    let output = turn_status_line(&state, &locale);
    assert!(output.contains("12"));
    assert!(!output.contains("需格挡"));
}

#[test]
fn turn_status_line_with_incoming_damage_no_block_at_all() {
    let locale = test_locale();
    let state = NormalizedState {
        current_hp: Some(50),
        max_hp: Some(75),
        block: None,
        energy: Some(3),
        incoming_damage: 8,
        danger: danger_safe(),
        ..test_state()
    };
    let output = turn_status_line(&state, &locale);
    assert!(output.contains("8"));
    assert!(!output.contains("需格挡"));
}

// --- combat_profile_line tests ---

#[test]
fn combat_profile_line_basic() {
    let locale = test_locale();
    let state = NormalizedState {
        character: Some("IRONCLAD".into()),
        floor: Some(5),
        max_hp: Some(75),
        gold: Some(150),
        relics: vec![],
        ..test_state()
    };
    let output = combat_profile_line(&state, &locale);
    assert!(output.contains("铁甲战士"));
    assert!(output.contains("5"));
    assert!(output.contains("75"));
    assert!(output.contains("150"));
}

#[test]
fn combat_profile_line_with_relics_and_counter() {
    let locale = test_locale();
    let state = NormalizedState {
        character: Some("THE_SILENT".into()),
        floor: Some(10),
        max_hp: Some(60),
        gold: Some(200),
        relics: vec![
            RelicInfo {
                id: "Incense Burner".into(),
                name: "香炉".into(),
                description: "".into(),
                counter: Some(5),
                price: None,
            },
            RelicInfo {
                id: "Burning Blood".into(),
                name: "燃烧之血".into(),
                description: "".into(),
                counter: None,
                price: None,
            },
        ],
        ..test_state()
    };
    let output = combat_profile_line(&state, &locale);
    assert!(output.contains("猎人"));
    assert!(output.contains("香炉 (5)"));
    assert!(output.contains("燃烧之血"));
}

#[test]
fn combat_profile_line_unknown_class() {
    let locale = test_locale();
    let state = NormalizedState {
        character: Some("CUSTOM_CLASS".into()),
        floor: Some(1),
        max_hp: Some(70),
        gold: Some(50),
        relics: vec![],
        ..test_state()
    };
    let output = combat_profile_line(&state, &locale);
    assert!(output.contains("CUSTOM_CLASS"));
}

#[test]
fn combat_profile_line_no_character() {
    let locale = test_locale();
    let state = NormalizedState {
        character: None,
        ..test_state()
    };
    let output = combat_profile_line(&state, &locale);
    assert!(!output.contains("角色"));
}

// --- status_line: comprehensive combinations ---

#[test]
fn status_line_full_kitchen_sink() {
    let locale = test_locale();
    let state = NormalizedState {
        character: Some("DEFECT".into()),
        floor: Some(20),
        current_hp: Some(40),
        max_hp: Some(60),
        block: Some(12),
        energy: Some(4),
        gold: Some(250),
        powers: vec![PowerInfo {
            id: "Focus".into(),
            name: "集中".into(),
            amount: 2,
        }],
        relics: vec![RelicInfo {
            id: "Snecko Eye".into(),
            name: "蛇眼".into(),
            description: "".into(),
            counter: None,
            price: None,
        }],
        potions: vec![PotionInfo {
            slot: 0,
            name: "恐惧药水".into(),
            description: "".into(),
            price: None,
            can_use: true,
            can_discard: false,
            requires_target: false,
        }],
        incoming_damage: 15,
        danger: danger_safe(),
        ..test_state()
    };
    let output = status_line(&state, &locale);
    assert!(output.contains("机器人"));
    assert!(output.contains("40/60"));
    assert!(output.contains("格挡：12"));
    assert!(output.contains("能量：4"));
    assert!(output.contains("250"));
    assert!(output.contains("集中(2)"));
    assert!(output.contains("蛇眼"));
    assert!(output.contains("恐惧药水"));
    assert!(output.contains("15"));
    assert!(output.contains("需格挡"));
}

#[test]
fn status_line_no_energy_or_block() {
    let locale = test_locale();
    let state = NormalizedState {
        character: Some("WATCHER".into()),
        floor: Some(1),
        current_hp: Some(63),
        max_hp: Some(70),
        block: None,
        energy: None,
        gold: Some(50),
        danger: danger_safe(),
        ..test_state()
    };
    let output = status_line(&state, &locale);
    assert!(output.contains("观者"));
    assert!(output.contains("63/70"));
    assert!(!output.contains("格挡："));
    assert!(!output.contains("能量："));
}

#[test]
fn status_line_zero_max_hp() {
    let locale = test_locale();
    let state = NormalizedState {
        current_hp: Some(0),
        max_hp: Some(0),
        block: None,
        energy: None,
        danger: danger_safe(),
        ..test_state()
    };
    let output = status_line(&state, &locale);
    assert!(output.contains("0%"));
}

// --- position_label edge cases ---

#[test]
fn position_label_only_one() {
    let locale = test_locale();
    assert_eq!(position_label(0, 1, &locale), "唯一");
}

#[test]
fn position_label_left_of_two() {
    let locale = test_locale();
    assert_eq!(position_label(0, 2, &locale), "左侧");
}

#[test]
fn position_label_right_of_two() {
    let locale = test_locale();
    assert_eq!(position_label(1, 2, &locale), "右侧");
}

#[test]
fn position_label_left_of_three() {
    let locale = test_locale();
    assert_eq!(position_label(0, 3, &locale), "左侧");
}

#[test]
fn position_label_middle_of_three() {
    let locale = test_locale();
    assert_eq!(position_label(1, 3, &locale), "中间");
}

#[test]
fn position_label_right_of_three() {
    let locale = test_locale();
    assert_eq!(position_label(2, 3, &locale), "右侧");
}

#[test]
fn position_label_leftmost_of_many() {
    let locale = test_locale();
    assert_eq!(position_label(0, 6, &locale), "最左侧");
}

#[test]
fn position_label_rightmost_of_many() {
    let locale = test_locale();
    assert_eq!(position_label(5, 6, &locale), "最右侧");
}

#[test]
fn position_label_middle_of_many_falls_back_to_from_left() {
    let locale = test_locale();
    let label = position_label(2, 6, &locale);
    assert!(label.contains("第3"));
}

// --- format_card: CURSE and STATUS types ---

#[test]
fn format_card_curse_type() {
    let locale = test_locale();
    let c = CardInfo {
        card_type: "CURSE".into(),
        ..card("Shame", "羞耻", -2, "CURSE")
    };
    let out = format_card(&c, &locale);
    assert!(out.contains("诅咒"));
}

#[test]
fn format_card_status_type() {
    let locale = test_locale();
    let c = CardInfo {
        card_type: "STATUS".into(),
        ..card("Slimed", "黏液", 1, "STATUS")
    };
    let out = format_card(&c, &locale);
    assert!(out.contains("状态"));
}

#[test]
fn format_card_power_type() {
    let locale = test_locale();
    let c = CardInfo {
        card_type: "POWER".into(),
        ..card("Demon Form", "恶魔形态", 3, "POWER")
    };
    let out = format_card(&c, &locale);
    assert!(out.contains("能力"));
}

#[test]
fn format_card_unknown_type_fallback() {
    let locale = test_locale();
    let c = CardInfo {
        card_type: "STRANGE".into(),
        ..card("Weird", "奇怪", 0, "STRANGE")
    };
    let out = format_card(&c, &locale);
    assert!(out.contains("STRANGE"));
}

#[test]
fn format_card_without_description() {
    let locale = test_locale();
    let c = CardInfo {
        name: "打击".into(),
        description: String::new(),
        ..card("Strike_R", "打击", 1, "ATTACK")
    };
    let out = format_card(&c, &locale);
    assert!(out.contains("打击(1费/攻击)"));
    assert!(!out.contains('—'));
}

// --- deck section edge cases ---

#[test]
fn deck_section_includes_curse_and_status() {
    let locale = test_locale();
    let mut cards = vec![card("Strike_R", "打击", 1, "ATTACK")];
    cards.push(CardInfo {
        card_type: "CURSE".into(),
        ..card("Shame", "羞耻", -2, "CURSE")
    });
    cards.push(CardInfo {
        card_type: "STATUS".into(),
        ..card("Slimed", "黏液", 1, "STATUS")
    });
    let output = format_deck_section(&cards, &locale);
    assert!(output.contains("攻击（1张）"));
    assert!(output.contains("诅咒（1张）"));
    assert!(output.contains("状态（1张）"));
    assert!(output.contains("羞耻"));
    assert!(output.contains("黏液"));
}

#[test]
fn deck_section_upgraded_with_count() {
    let locale = test_locale();
    let cards = vec![
        CardInfo {
            upgraded: true,
            name: "打击+".into(),
            ..card("Strike_R", "打击+", 1, "ATTACK")
        },
        CardInfo {
            upgraded: true,
            name: "打击+".into(),
            ..card("Strike_R", "打击+", 1, "ATTACK")
        },
    ];
    let output = format_deck_section(&cards, &locale);
    assert!(output.contains("+打击+(1费)（共2张）"));
}

#[test]
fn deck_section_card_with_description() {
    let locale = test_locale();
    let cards = vec![CardInfo {
        name: "痛击".into(),
        description: "造成 8 点伤害。\n给予 2 层 易伤 。".into(),
        ..card("Bash", "痛击", 2, "ATTACK")
    }];
    let output = format_deck_section(&cards, &locale);
    assert!(output.contains("痛击(2费)"));
    assert!(output.contains("8 点伤害"));
    assert!(output.contains("2 层 易伤"));
}

#[test]
fn deck_section_power_with_count() {
    let locale = test_locale();
    let cards = vec![
        CardInfo {
            card_type: "POWER".into(),
            ..card("Demon Form", "恶魔形态", 3, "POWER")
        },
        CardInfo {
            card_type: "POWER".into(),
            ..card("Demon Form", "恶魔形态", 3, "POWER")
        },
    ];
    let output = format_deck_section(&cards, &locale);
    assert!(output.contains("能力（2张）"));
    assert!(output.contains("恶魔形态(3费)（共2张）"));
}

// --- compact_pile: single card ---

#[test]
fn compact_pile_single_card() {
    let locale = test_locale();
    let cards = vec![card("Bash", "痛击", 2, "ATTACK")];
    let output = compact_pile("=== 弃牌堆", &cards, &locale);
    assert!(output.contains("弃牌堆（1张）"));
    assert!(output.contains("痛击"));
    assert!(!output.contains("×"));
}

// --- build_relics_potions_section ---

#[test]
fn build_relics_potions_section_with_both() {
    let locale = test_locale();
    let state = NormalizedState {
        relics: vec![RelicInfo {
            id: "Burning Blood".into(),
            name: "燃烧之血".into(),
            description: "战斗结束时回复6点生命。".into(),
            counter: None,
            price: None,
        }],
        potions: vec![PotionInfo {
            slot: 0,
            name: "恐惧药水".into(),
            description: "给予3层易伤。".into(),
            price: None,
            can_use: true,
            can_discard: false,
            requires_target: false,
        }],
        ..test_state()
    };
    let output = build_relics_potions_section(&state, &locale);
    assert!(output.contains("=== 遗物 ==="));
    assert!(output.contains("燃烧之血"));
    assert!(output.contains("=== 药水 ==="));
    assert!(output.contains("恐惧药水"));
}

#[test]
fn build_relics_potions_section_empty() {
    let locale = test_locale();
    let state: NormalizedState = test_state();
    let output = build_relics_potions_section(&state, &locale);
    assert!(output.is_empty());
}

// --- compact_pile: exhaust pile with cards ---

#[test]
fn compact_pile_exhaust() {
    let locale = test_locale();
    let cards = vec![
        card("Slimed", "黏液", 1, "STATUS"),
        card("Slimed", "黏液", 1, "STATUS"),
        card("Dazed", "晕眩", 1, "STATUS"),
    ];
    let output = compact_pile("=== 已消耗", &cards, &locale);
    assert!(output.contains("已消耗（3张）"));
    assert!(output.contains("黏液×2"));
    assert!(output.contains("晕眩"));
}

// --- format_evaluation_line ---

fn make_path_evaluation(score: f64, pros: Vec<&str>, cons: Vec<&str>) -> PathEvaluation {
    PathEvaluation {
        score,
        pros: pros.into_iter().map(|s| s.to_string()).collect(),
        cons: cons.into_iter().map(|s| s.to_string()).collect(),
        description: PathDescription {
            route_chain: String::new(),
            counts: String::new(),
            annotations: vec![],
            metrics: PathMetrics {
                counts: PathCounts {
                    monsters: 0,
                    elites: 0,
                    events: 0,
                    shops: 0,
                    rests: 0,
                    treasures: 0,
                },
                shop_timing: ShopTiming::None,
                rest_before_first_elite: false,
                double_elite_without_rest: false,
                max_elite_to_rest_risk: 0.0,
            },
        },
    }
}

#[test]
fn format_evaluation_line_with_pros_and_cons() {
    let eval = make_path_evaluation(
        85.0,
        vec!["early shop", "rest before elite"],
        vec!["double elite"],
    );
    let line = format_evaluation_line(&eval);
    assert!(line.contains("Score:85"));
    assert!(line.contains("+early shop"));
    assert!(line.contains("-double elite"));
}

#[test]
fn format_evaluation_line_no_pros_no_cons() {
    let eval = make_path_evaluation(50.0, vec![], vec![]);
    let line = format_evaluation_line(&eval);
    assert_eq!(line, "Score:50");
}

// --- format_candidate_label ---

#[test]
fn candidate_label_format() {
    assert_eq!(format_candidate_label(0), "Candidate 1");
    assert_eq!(format_candidate_label(4), "Candidate 5");
}

// --- recommendation_label ---

#[test]
fn recommendation_label_basic() {
    let eval = PathEvaluation {
        score: 90.0,
        pros: vec![],
        cons: vec![],
        description: PathDescription {
            route_chain: "M→$→R→E→R".into(),
            counts: String::new(),
            annotations: vec![],
            metrics: PathMetrics {
                counts: PathCounts {
                    monsters: 1,
                    elites: 1,
                    events: 0,
                    shops: 1,
                    rests: 2,
                    treasures: 0,
                },
                shop_timing: ShopTiming::Early,
                rest_before_first_elite: true,
                double_elite_without_rest: false,
                max_elite_to_rest_risk: 0.0,
            },
        },
    };
    let label = recommendation_label("Route 1 (左侧)", &eval);
    assert!(label.starts_with("Route 1 (左侧)"));
    assert!(label.contains("M→$→R→E→R"));
    assert!(label.contains("early shop"));
    assert!(label.contains("rest before elite"));
}

// --- rank_labeled_paths ---

#[test]
fn rank_labeled_paths_sorts_by_score_descending() {
    let state = NormalizedState {
        floor: Some(1),
        current_hp: Some(68),
        max_hp: Some(75),
        gold: Some(99),
        ..test_state()
    };
    let paths = vec![
        ("Route 1".to_string(), path_of(&["M", "R", "M", "R"])),
        ("Route 2".to_string(), path_of(&["M", "$", "R", "E", "R"])),
    ];
    let ranked = rank_labeled_paths(paths, &state, false);
    assert_eq!(ranked.len(), 2);
}

// --- shop prompt: purge not available with deck ---

#[test]
fn shop_prompt_purge_not_available_omits_removal_but_shows_deck() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("SHOP_SCREEN".into()),
        gold: Some(100),
        shop_cards: vec![],
        shop_relics: vec![],
        shop_potions: vec![],
        purge_available: false,
        purge_cost: None,
        master_cards: vec![card("Strike_R", "打击", 1, "ATTACK")],
        deck_names: vec!["打击".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 卡组 ==="));
    assert!(!prompt.contains("=== 删牌服务 ==="));
}

// --- describe_path: route_chain annotation edge cases ---

#[test]
fn describe_path_triple_elite_warning() {
    let path = path_of(&["R", "E", "?", "E", "$", "E", "R"]);
    let desc = describe_path(&path, 1);
    assert!(desc.annotations.iter().any(|a| a.contains("Double Elite")));
}

#[test]
fn describe_path_rest_before_elite_seen() {
    let path = path_of(&["R", "E", "M"]);
    let desc = describe_path(&path, 1);
    assert!(
        desc.annotations
            .iter()
            .any(|a| a.contains("Rest before first Elite"))
    );
}

#[test]
fn describe_path_event_types() {
    // '?' events are counted
    let path = path_of(&["M", "?", "R"]);
    let desc = describe_path(&path, 1);
    assert!(desc.counts.contains("Events:1"));
}

// --- clean_description: edge cases ---

#[test]
fn clean_double_spaces() {
    let locale = test_locale();
    let out = clean_description("Deal  6  damage.", &locale);
    assert_eq!(out, "Deal 6 damage.");
}

#[test]
fn clean_space_before_punctuation() {
    let locale = test_locale();
    let out = clean_description("Draw 1 card . Gain 3 block .", &locale);
    assert_eq!(out, "Draw 1 card. Gain 3 block.");
}

// --- combat: exhuast pile shown ---

#[test]
fn combat_prompt_shows_exhaust_pile() {
    let locale = test_locale();
    let state = NormalizedState {
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
        exhaust_cards: vec![card("Slimed", "黏液", 1, "STATUS")],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 已消耗"));
    assert!(prompt.contains("黏液"));
}

// --- build_map_crossroad: single child ---

#[test]
fn build_map_crossroad_single_child_direct() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 1, 2, vec![(1, 3)]),
            make_node("R", 1, 3, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(1),
        map_current_y: Some(2),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert!(prompt.contains("(唯一)"));
}

// --- build_map_crossroad: unknown node symbol ---

#[test]
fn build_map_crossroad_unknown_symbol() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(5),
        map_nodes: vec![
            make_node("M", 1, 2, vec![(1, 3)]),
            make_node("X", 1, 3, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(1),
        map_current_y: Some(2),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert!(prompt.contains("X(X)"));
}

// --- compact_route_chain: short chain unchanged ---

#[test]
fn compact_route_chain_short() {
    assert_eq!(compact_route_chain("M→R→E→R"), "M→R→E→R");
}

#[test]
fn compact_route_chain_exactly_ten_parts() {
    let chain = "M→R→M→E→?→$→M→R→M→T";
    assert_eq!(compact_route_chain(chain), chain);
}

// --- route description: describe_path with raw chain having no rest before elite ---

#[test]
fn describe_path_elite_first_node_no_rest_before() {
    let path = path_of(&["E", "M", "R", "M", "R"]);
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("Rest before")));
}

// --- ordnial tests ---

#[test]
fn ordinal_basic() {
    assert_eq!(ordinal(1), "1st");
    assert_eq!(ordinal(2), "2nd");
    assert_eq!(ordinal(3), "3rd");
    assert_eq!(ordinal(4), "4th");
    assert_eq!(ordinal(11), "11th");
    assert_eq!(ordinal(12), "12th");
    assert_eq!(ordinal(13), "13th");
    assert_eq!(ordinal(21), "21st");
    assert_eq!(ordinal(22), "22nd");
    assert_eq!(ordinal(23), "23rd");
    assert_eq!(ordinal(101), "101st");
}

// --- from_left_label ---

#[test]
fn from_left_label_basic() {
    let locale = test_locale();
    assert_eq!(from_left_label(1, &locale), "左起第1个");
    assert_eq!(from_left_label(7, &locale), "左起第7个");
}

// --- Helper ---

fn danger_safe() -> DangerFlags {
    DangerFlags {
        hp_critical: false,
        incoming_lethal: false,
        no_block_against_hit: false,
        any_monster_attacking: false,
        wrath_stance: false,
        level: DangerLevel::Safe,
    }
}

// --- clean_description: more energy tokens ---

#[test]
fn clean_four_energy_tokens() {
    let locale = test_locale();
    let out = clean_description("获得 [R] [R] [R] [R] 。", &locale);
    assert_eq!(out, "获得 4 能量。");
}

#[test]
fn clean_five_energy_tokens() {
    let locale = test_locale();
    let out = clean_description("获得 [R] [R] [R] [R] [R] 。", &locale);
    assert_eq!(out, "获得 5 能量。");
}

#[test]
fn clean_six_energy_tokens() {
    let locale = test_locale();
    let out = clean_description("获得 [R] [R] [R] [R] [R] [R] 。", &locale);
    assert_eq!(out, "获得 6 能量。");
}

// --- clean_description: punctuation ---

#[test]
fn clean_space_before_comma() {
    let locale = test_locale();
    let out = clean_description("Draw 1 card , then gain block.", &locale);
    assert_eq!(out, "Draw 1 card, then gain block.");
}

#[test]
fn clean_space_before_semicolon() {
    let locale = test_locale();
    let out = clean_description("Deal 6 ; gain 3.", &locale);
    assert_eq!(out, "Deal 6; gain 3.");
}

#[test]
fn clean_space_before_exclamation() {
    let locale = test_locale();
    let out = clean_description("Pow !", &locale);
    assert_eq!(out, "Pow!");
}

#[test]
fn clean_space_before_question() {
    let locale = test_locale();
    let out = clean_description("What ?", &locale);
    assert_eq!(out, "What?");
}

// --- format_card: SKILL type ---

#[test]
fn format_card_skill_type() {
    let locale = test_locale();
    let c = CardInfo {
        card_type: "SKILL".into(),
        description: "获得 8 点 格挡 。".into(),
        ..card("Defend_R", "防御", 1, "SKILL")
    };
    let out = format_card(&c, &locale);
    assert!(out.contains("技能"));
    assert!(out.contains("防御"));
}

// --- build_combat: room types ---

#[test]
fn build_combat_elite_room() {
    let locale = test_locale();
    let state = NormalizedState {
        room_type: Some("MonsterRoomElite".into()),
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
        room_type: Some("MonsterRoomBoss".into()),
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
        room_type: Some("MonsterRoom".into()),
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

// --- build_rest: option types and edge cases ---

#[test]
fn build_rest_all_option_types() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        current_hp: Some(45),
        max_hp: Some(75),
        rest_options: vec![
            "rest".into(),
            "smith".into(),
            "toke".into(),
            "dig".into(),
            "lift".into(),
            "recall".into(),
            "girya".into(),
        ],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("回忆"));
    assert!(prompt.contains("挖遗物"));
    assert!(prompt.contains("举重"));
    assert!(prompt.contains("回忆钥匙"));
    assert!(prompt.contains("深蹲"));
}

#[test]
fn build_rest_mid_hp_no_advice() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        current_hp: Some(45),
        max_hp: Some(75),
        rest_options: vec!["rest".into(), "smith".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(!prompt.contains("强烈建议休息"));
    assert!(!prompt.contains("可考虑锻造或挖遗物"));
}

#[test]
fn build_rest_filters_curse_status_from_upgrade() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        current_hp: Some(45),
        max_hp: Some(75),
        rest_options: vec!["smith".into()],
        master_cards: vec![
            card("Strike_R", "打击", 1, "ATTACK"),
            CardInfo {
                card_type: "CURSE".into(),
                ..card("Shame", "羞耻", -2, "CURSE")
            },
            CardInfo {
                card_type: "STATUS".into(),
                ..card("Slimed", "黏液", 1, "STATUS")
            },
        ],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("=== 可锻造升级目标 ==="));
    assert!(prompt.contains("打击"));
    assert!(!prompt.contains("羞耻"));
    assert!(!prompt.contains("黏液"));
}

#[test]
fn build_rest_dedup_same_card_id() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        current_hp: Some(45),
        max_hp: Some(75),
        rest_options: vec!["smith".into()],
        master_cards: vec![
            card("Strike_R", "打击", 1, "ATTACK"),
            card("Strike_R", "打击", 1, "ATTACK"),
            card("Defend_R", "防御", 1, "SKILL"),
        ],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert_eq!(prompt.matches("打击(1费/攻击)").count(), 1);
    assert!(prompt.contains("防御"));
}

// --- build_boss_relic: hp notes at act ends ---

#[test]
fn build_boss_relic_floor_17_hp_note() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("BOSS_REWARD".into()),
        floor: Some(17),
        boss_relic_choices: vec![RelicInfo {
            id: "Snecko Eye".into(),
            name: "蛇眼".into(),
            description: "".into(),
            counter: None,
            price: None,
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("下一幕开始会回满血"));
}

#[test]
fn build_boss_relic_floor_34_hp_note() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("BOSS_REWARD".into()),
        floor: Some(34),
        boss_relic_choices: vec![RelicInfo {
            id: "Snecko Eye".into(),
            name: "蛇眼".into(),
            description: "".into(),
            counter: None,
            price: None,
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("下一幕开始会回满血"));
}

// --- build_event_choice ---

#[test]
fn build_event_choice_with_id_and_name() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("EVENT".into()),
        event_id: Some("Big Fish".into()),
        event_name: Some("大鲸".into()),
        event_body: Some("一个巨大的鲸鱼挡住了去路。".into()),
        event_choices: vec!["香蕉".into(), "甜甜圈".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("大鲸"));
    assert!(prompt.contains("Big Fish"));
    assert!(prompt.contains("一个巨大的鲸鱼挡住了去路"));
    assert!(prompt.contains("A. 香蕉"));
    assert!(prompt.contains("B. 甜甜圈"));
}

#[test]
fn build_event_choice_body_only_no_event_info() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("EVENT".into()),
        event_id: None,
        event_name: None,
        room_type: None,
        event_body: Some("一段描述文本。".into()),
        event_choices: vec!["选项A".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("一段描述文本"));
    assert!(prompt.contains("A. 选项A"));
    assert!(!prompt.contains("???"));
}

// --- build_shop: price and purge edge cases ---

#[test]
fn build_shop_items_price_none() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("SHOP_SCREEN".into()),
        gold: Some(200),
        shop_cards: vec![CardInfo {
            name: "打击".into(),
            cost: 1,
            price: None,
            ..card("Strike_R", "打击", 1, "ATTACK")
        }],
        shop_relics: vec![RelicInfo {
            id: "Shovel".into(),
            name: "铲子".into(),
            description: "挖掘".into(),
            counter: None,
            price: None,
        }],
        shop_potions: vec![PotionInfo {
            slot: 0,
            name: "再生药水".into(),
            description: "获得 5 层 再生 。".into(),
            price: None,
            can_use: false,
            can_discard: false,
            requires_target: false,
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("? gold"));
}

#[test]
fn build_shop_purge_cost_none() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("SHOP_SCREEN".into()),
        gold: Some(200),
        purge_available: true,
        purge_cost: None,
        master_cards: vec![card("Strike_R", "打击", 1, "ATTACK")],
        deck_names: vec!["打击".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("Remove a card for unknown gold"));
}

#[test]
fn build_shop_all_empty_no_section_header() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("SHOP_SCREEN".into()),
        gold: Some(100),
        shop_cards: vec![],
        shop_relics: vec![],
        shop_potions: vec![],
        purge_available: false,
        purge_cost: None,
        master_cards: vec![],
        deck_names: vec![],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(!prompt.contains("=== 商店 ==="));
    assert!(prompt.contains("[mode: shop]"));
}

// --- build_map_crossroad ---

#[test]
fn build_map_crossroad_current_position_not_found() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(5),
        map_nodes: vec![make_node("M", 0, 0, vec![])],
        map_first_node_chosen: Some(true),
        map_current_x: Some(99),
        map_current_y: Some(99),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert!(prompt.contains("[mode: map_crossroad]"));
    assert!(!prompt.contains("Ahead"));
}

#[test]
fn build_map_crossroad_many_candidates_limited() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(5),
        map_nodes: vec![
            make_node(
                "M",
                0,
                0,
                vec![(0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (5, 1)],
            ),
            make_node("M", 0, 1, vec![]),
            make_node("?", 1, 1, vec![]),
            make_node("R", 2, 1, vec![]),
            make_node("$", 3, 1, vec![]),
            make_node("E", 4, 1, vec![]),
            make_node("T", 5, 1, vec![]),
        ],
        map_first_node_chosen: Some(true),
        map_current_x: Some(0),
        map_current_y: Some(0),
        ..test_state()
    };
    let prompt = build_map_crossroad(&state, &locale, false);
    assert_eq!(prompt.matches("Candidate ").count(), 5);
    assert!(prompt.contains("Candidate 5"));
    assert!(!prompt.contains("Candidate 6"));
}

// --- format_monster ---

#[test]
fn format_monster_powers_without_scaling() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "邪教徒".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(18),
        max_hp: Some(40),
        block: Some(0),
        intent: Some("BUFF".into()),
        damage: None,
        hits: None,
        monster_powers: vec![PowerInfo {
            id: "Strength".into(),
            name: "力量".into(),
            amount: 1,
        }],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("力量(1)"));
    assert!(!output.contains("成长中"));
}

#[test]
fn format_monster_attack_debuff_intent() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "震荡波".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(30),
        max_hp: Some(30),
        block: Some(0),
        intent: Some("ATTACK_DEBUFF".into()),
        damage: Some(8),
        hits: Some(1),
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("攻击+减益"));
}

#[test]
fn format_monster_defend_buff_intent() {
    let locale = test_locale();
    let m = MonsterInfo {
        name: "哨兵".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(30),
        max_hp: Some(30),
        block: Some(10),
        intent: Some("DEFEND_BUFF".into()),
        damage: None,
        hits: None,
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    };
    let output = format_monster(&m, &locale);
    assert!(output.contains("防御+增益"));
}

// --- build_hand_select ---

#[test]
fn build_hand_select_no_current_action() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("HAND_SELECT".into()),
        hand: vec![card("Strike_R", "打击", 1, "ATTACK")],
        current_action: None,
        card_in_play: None,
        hand_select_max_cards: Some(1),
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: hand_select]"));
    assert!(!prompt.contains("Purpose:"));
    assert!(prompt.contains("打击"));
}

#[test]
fn build_hand_select_no_max_cards() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("HAND_SELECT".into()),
        hand: vec![card("Strike_R", "打击", 1, "ATTACK")],
        current_action: Some("DiscardAction".into()),
        card_in_play: None,
        hand_select_max_cards: None,
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("[mode: hand_select]"));
    assert!(prompt.contains("Discard a card"));
    assert!(!prompt.contains("Max:"));
}

// --- recommendation_features ---

#[test]
fn recommendation_features_four_events() {
    let eval = PathEvaluation {
        score: 75.0,
        pros: vec![],
        cons: vec![],
        description: PathDescription {
            route_chain: "M→?→?→?→R".into(),
            counts: String::new(),
            annotations: vec![],
            metrics: PathMetrics {
                counts: PathCounts {
                    monsters: 1,
                    elites: 0,
                    events: 4,
                    shops: 0,
                    rests: 1,
                    treasures: 0,
                },
                shop_timing: ShopTiming::None,
                rest_before_first_elite: false,
                double_elite_without_rest: false,
                max_elite_to_rest_risk: 0.0,
            },
        },
    };
    let features = recommendation_features(&eval);
    assert!(features.contains("4 events"));
}

// --- turn_status_line ---

#[test]
fn turn_status_line_block_zero_incoming_no_warning() {
    let locale = test_locale();
    let state = NormalizedState {
        current_hp: Some(50),
        max_hp: Some(75),
        block: Some(0),
        energy: Some(3),
        incoming_damage: 12,
        danger: danger_safe(),
        ..test_state()
    };
    let output = turn_status_line(&state, &locale);
    assert!(output.contains("12"));
    assert!(!output.contains("需格挡"));
}

// --- status_line ---

#[test]
fn status_line_block_none_incoming_no_warning() {
    let locale = test_locale();
    let state = NormalizedState {
        current_hp: Some(50),
        max_hp: Some(75),
        block: None,
        energy: Some(3),
        incoming_damage: 15,
        danger: danger_safe(),
        ..test_state()
    };
    let output = status_line(&state, &locale);
    assert!(output.contains("15"));
    assert!(!output.contains("需格挡"));
}

// --- compact_route_chain ---

#[test]
fn compact_route_chain_eleven_parts() {
    let chain = "M→R→M→E→?→$→M→R→M→T→R";
    let result = compact_route_chain(chain);
    assert_eq!(result, "M→R→M→E→?→…→R→M→T→R");
}

// --- build_map_suggestion ---

#[test]
fn build_map_suggestion_root_with_branches() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("MAP".into()),
        floor: Some(1),
        map_nodes: vec![
            make_node("M", 0, 0, vec![(0, 1), (1, 1)]),
            make_node("R", 0, 1, vec![(0, 2)]),
            make_node("?", 1, 1, vec![(1, 2)]),
            make_node("$", 0, 2, vec![]),
            make_node("T", 1, 2, vec![]),
        ],
        map_first_node_chosen: Some(false),
        ..test_state()
    };
    let prompt = build_map_suggestion(&state, &locale);
    assert!(prompt.contains("Root 1 (唯一)"));
    assert_eq!(prompt.matches("Candidate ").count(), 2);
}

// --- build_hand_select: unknown rest option ---

#[test]
fn build_rest_unknown_option_fallback() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        current_hp: Some(45),
        max_hp: Some(75),
        rest_options: vec!["custom_action".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale, false);
    assert!(prompt.contains("custom_action"));
}

// --- build_combat: tradeoff advice for elite and boss ---

#[test]
fn build_combat_elite_tradeoff() {
    let locale = test_locale();
    let state = NormalizedState {
        room_type: Some("MonsterRoomElite".into()),
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
        room_type: Some("MonsterRoomBoss".into()),
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

// --- describe_path: compacts triple elite warning ---

#[test]
fn describe_path_triple_elite_with_rest_after_first_hides_double() {
    let path = path_of(&["M", "R", "E", "R", "E", "R", "E", "R"]);
    let desc = describe_path(&path, 1);
    assert!(!desc.annotations.iter().any(|a| a.contains("Double Elite")));
}

// --- compact_pile: mixed duplicates ---

#[test]
fn compact_pile_mixed_duplicates_sorted() {
    let locale = test_locale();
    let cards = vec![
        card("Defend_R", "防御", 1, "SKILL"),
        card("Strike_R", "打击", 1, "ATTACK"),
        card("Defend_R", "防御", 1, "SKILL"),
    ];
    let output = compact_pile("=== 抽牌堆", &cards, &locale);
    assert!(output.contains("防御×2"));
    assert!(output.contains("打击"));
}
