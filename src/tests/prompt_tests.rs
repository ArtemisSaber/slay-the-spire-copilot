use super::*;
use crate::locales::Locale;
use crate::state::{DangerFlags, DangerLevel, MapCoord, MonsterInfo, PowerInfo, RelicInfo};
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
    }
}

#[test]
fn combat_prompt_shows_all_three_piles() {
    let locale = test_locale();
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
        hand_cards: vec![card("Strike_R", "打击", 1, "ATTACK")],
        draw_pile: vec![card("Defend_R", "防御", 1, "SKILL")],
        discard_pile: vec![card("Bash", "痛击", 2, "ATTACK")],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale);
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

    let prompt = build_prompt(&state, &locale);
    assert!(prompt.contains("=== 任务 ==="));
    assert!(prompt.contains("本回合"));
    assert!(prompt.contains("最佳出牌"));
    assert!(prompt.contains("不要写分析过程"));
}

#[test]
fn monster_section_shows_index_and_intent() {
    let locale = test_locale();
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

    let prompt = build_prompt(&state, &locale);
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

    let prompt = build_prompt(&state, &locale);
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

    let prompt = build_prompt(&state, &locale);
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

    let prompt = build_prompt(&state, &locale);
    assert!(prompt.contains("=== 任务 ==="));
    assert!(prompt.contains("选择一张牌"));
    assert!(prompt.contains("推荐跳过"));
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

    let prompt = build_prompt(&state, &locale);
    assert!(
        prompt.contains(
            "注意：这是 Boss 战后的选牌。下一幕开始会回满血，不要把当前血量当成选牌依据。"
        )
    );
    assert!(prompt.contains("下一幕"));
    assert!(prompt.contains("卡组方向"));
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

    let prompt = build_prompt(&state, &locale);
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

    let prompt = build_prompt(&state, &locale);
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

    let prompt = build_prompt(&state, &locale);
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
    let prompt = build_prompt(&test_state(), &locale);
    assert!(prompt.contains("=== 当前状态 ==="));
    assert!(prompt.contains("推荐："));
    assert!(prompt.contains("理由："));
    assert!(prompt.contains("风险："));
    assert!(prompt.contains("吐槽："));
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

    let prompt = build_prompt(&state, &locale);
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

    let prompt = build_prompt(&state, &locale);
    assert!(prompt.contains("=== 任务 ==="));
    assert!(prompt.contains("篝火选项"));
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

    let prompt = build_prompt(&state, &locale);
    assert!(prompt.contains("必须写出要升级哪张牌"));
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
                name: "蛇眼".into(),
                description: "".into(),
            },
            RelicInfo {
                name: "符文圆顶".into(),
                description: "".into(),
            },
            RelicInfo {
                name: "诅咒钥匙".into(),
                description: "".into(),
            },
        ],
        master_cards: vec![card("Bash", "Bash", 2, "ATTACK")],
        ..test_state()
    };

    let prompt = build_prompt(&state, &locale);
    assert!(prompt.contains("=== Boss 遗物 ==="));
    assert!(prompt.contains("A. 蛇眼"));
    assert!(prompt.contains("B. 符文圆顶"));
    assert!(prompt.contains("C. 诅咒钥匙"));
    assert!(prompt.contains("副作用"));
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

    let prompt = build_prompt(&state, &locale);
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

    let prompt = build_prompt(&state, &locale);
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

    let prompt = build_prompt(&state, &locale);
    assert!(prompt.contains("=== 当前状态 ==="));
    assert!(prompt.contains("角色：铁甲战士"));
    assert!(prompt.contains("推荐："));
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

    let prompt = build_prompt(&state, &locale);
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

    let prompt = build_prompt(&state, &locale);
    assert!(prompt.contains("血量健康，可考虑锻造或挖遗物。"));
}

#[test]
fn danger_prefix_shows_wrath_warning() {
    let locale = test_locale();
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

    let prompt = build_prompt(&state, &locale);
    assert!(prompt.contains("愤怒姿态下受到双倍伤害"));
}

#[test]
fn compact_pile_aggregates_duplicates() {
    let locale = test_locale();
    let cards = vec![card("Strike_R", "打击", 1, "ATTACK"); 3];
    let output = super::compact_pile("=== 抽牌堆", &cards, &locale);
    assert!(output.contains("抽牌堆（3张）"));
    assert!(output.contains("打击×3"));
}

#[test]
fn monster_shows_multi_hit_damage() {
    let locale = test_locale();
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

    let prompt = build_prompt(&state, &locale);
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
    let prompt = build_prompt(&state, &locale);
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
    let prompt = build_prompt(&state, &locale);
    assert!(prompt.contains("=== 选项 ==="));
    assert!(!prompt.contains("=== 手牌"));
}

#[test]
fn build_prompt_routes_boss_relic() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: Some("BOSS_REWARD".into()),
        boss_relic_choices: vec![RelicInfo {
            name: "蛇眼".into(),
            description: String::new(),
        }],
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale);
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
    let prompt = build_prompt(&state, &locale);
    assert!(prompt.contains("事件选项"));
}

#[test]
fn build_prompt_routes_combat_when_monsters_present() {
    let locale = test_locale();
    let state = NormalizedState {
        screen_type: None,
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
        ..test_state()
    };
    let prompt = build_prompt(&state, &locale);
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
    let prompt = build_prompt(&state, &locale);
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
    let prompt = build_prompt(&state, &locale);
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
    let prompt = build_prompt(&state, &locale);
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
    let prompt = build_prompt(&state, &locale);
    assert!(prompt.contains("M→?→R"));
    assert!(prompt.contains("Monsters:1"));
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
    let prompt = build_prompt(&state, &locale);
    assert!(prompt.contains("A."));
    assert!(prompt.contains("B."));
    assert!(!prompt.contains("C."));
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
    let prompt = build_prompt(&state, &locale);
    assert!(prompt.contains("Root"));
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
    let prompt = build_prompt(&state, &locale);
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
    let prompt = build_prompt(&state, &locale);
    assert!(prompt.contains("=== 任务 ==="));
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
