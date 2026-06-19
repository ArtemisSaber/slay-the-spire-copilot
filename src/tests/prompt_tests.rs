use super::*;
use crate::i18n::I18n;
use crate::state::{DangerFlags, DangerLevel, MonsterInfo, PowerInfo};

fn load_i18n() -> I18n {
    I18n::load()
}

fn card(id: &str, name: &str, cost: i64, card_type: &str) -> CardInfo {
    CardInfo {
        id: id.into(),
        name: name.into(),
        cost,
        card_type: card_type.into(),
        upgraded: false,
        uuid: None,
    }
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
        skip_available: false,
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
        hand_cards: vec![card("Strike_R", "Strike", 1, "ATTACK")],
        draw_pile: vec![card("Defend_R", "Defend", 1, "SKILL")],
        discard_pile: vec![card("Bash", "Bash", 2, "ATTACK")],
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
fn combat_prompt_marks_entry_plan_task() {
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
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("=== 任务 ==="));
    assert!(prompt.contains("进入战斗"));
    assert!(prompt.contains("整体打法"));
    assert!(prompt.contains("不要逐回合假设后续抽牌"));
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
            card("Uppercut", "Uppercut", 2, "ATTACK"),
            card("Anger", "Anger", 1, "ATTACK"),
        ],
        master_cards: vec![card("Strike_R", "Strike", 1, "ATTACK"); 4],
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("A. 上勾拳"));
    assert!(prompt.contains("B. 愤怒"));
    assert!(prompt.contains("2费"));
    assert!(prompt.contains("1费"));
}

#[test]
fn card_reward_prompt_marks_pick_or_skip_task() {
    let i18n = load_i18n();
    let state = NormalizedState {
        screen_type: Some("CARD_REWARD".into()),
        floor: Some(14),
        card_reward_choices: vec![card("Uppercut", "Uppercut", 2, "ATTACK")],
        skip_available: true,
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("=== 任务 ==="));
    assert!(prompt.contains("选择一张牌"));
    assert!(prompt.contains("推荐跳过"));
}

#[test]
fn boss_card_reward_prompt_includes_full_heal_note() {
    let i18n = load_i18n();
    let state = NormalizedState {
        screen_type: Some("CARD_REWARD".into()),
        floor: Some(17),
        current_hp: Some(3),
        max_hp: Some(75),
        card_reward_choices: vec![card("Demon Form", "Demon Form", 3, "POWER")],
        skip_available: true,
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
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
    let i18n = load_i18n();
    let state = NormalizedState {
        screen_type: Some("CARD_REWARD".into()),
        floor: Some(14),
        current_hp: Some(3),
        max_hp: Some(75),
        card_reward_choices: vec![card("Uppercut", "Uppercut", 2, "ATTACK")],
        skip_available: true,
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(
        !prompt.contains(
            "注意：这是 Boss 战后的选牌。下一幕开始会回满血，不要把当前血量当成选牌依据。"
        )
    );
}

#[test]
fn card_reward_shows_skip_when_available() {
    let i18n = load_i18n();
    let state = NormalizedState {
        screen_type: Some("CARD_REWARD".into()),
        card_reward_choices: vec![card("Uppercut", "Uppercut", 2, "ATTACK")],
        master_cards: vec![card("Strike_R", "Strike", 1, "ATTACK")],
        skip_available: true,
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("跳过. 都不选"));
}

#[test]
fn card_reward_no_skip_when_unavailable() {
    let i18n = load_i18n();
    let state = NormalizedState {
        screen_type: Some("CARD_REWARD".into()),
        card_reward_choices: vec![card("Uppercut", "Uppercut", 2, "ATTACK")],
        master_cards: vec![card("Strike_R", "Strike", 1, "ATTACK")],
        skip_available: false,
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(!prompt.contains("跳过. 都不选"));
}

#[test]
fn deck_section_groups_single_type() {
    let i18n = load_i18n();
    let cards = vec![card("Strike_R", "Strike", 1, "ATTACK"); 5];
    let output = format_deck_section(&cards, &i18n);
    assert!(output.contains("攻击（5张）："));
    assert!(output.contains("打击(1费)（共5张）"));
    assert!(!output.contains("技能"));
    assert!(!output.contains("能力"));
}

#[test]
fn deck_section_shows_cards_by_type_with_counts() {
    let i18n = load_i18n();
    let mut cards = vec![card("Strike_R", "Strike", 1, "ATTACK"); 4];
    cards.extend(vec![card("Defend_R", "Defend", 1, "SKILL"); 2]);
    let output = format_deck_section(&cards, &i18n);

    assert!(output.contains("=== 卡组 ==="));
    assert!(output.contains("攻击（4张）："));
    assert!(output.contains("技能（2张）："));
    assert!(output.contains("打击(1费)（共4张）"));
    assert!(output.contains("防御(1费)（共2张）"));
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
fn rest_prompt_marks_campfire_decision_task() {
    let i18n = load_i18n();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        rest_options: vec!["rest".into(), "smith".into()],
        ..test_state()
    };

    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("=== 任务 ==="));
    assert!(prompt.contains("篝火选项"));
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
    let i18n = load_i18n();
    let cards = vec![card("Strike_R", "Strike", 1, "ATTACK"); 3];
    let output = super::compact_pile("=== 抽牌堆", &cards, &i18n);
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

// --- resolve_description tests ---

#[test]
fn resolve_substitutes_d() {
    let i18n = load_i18n();
    let out = resolve_description("Strike_R", false, "造成 !D! 点伤害。", &i18n);
    assert_eq!(out, "造成 6 点伤害。");
}

#[test]
fn resolve_substitutes_b() {
    let i18n = load_i18n();
    let out = resolve_description("Defend_R", false, "获得 !B! 点 格挡 。", &i18n);
    assert_eq!(out, "获得 5 点 格挡 。");
}

#[test]
fn resolve_substitutes_m() {
    let i18n = load_i18n();
    let out = resolve_description("PathToVictory", false, "给予 !M! 层 *印记 。", &i18n);
    assert_eq!(out, "给予 3 层 印记 。");
}

#[test]
fn resolve_strips_keyword_markers() {
    let i18n = load_i18n();
    let out = resolve_description(
        "PathToVictory",
        false,
        "给予 !M! 层 *印记* 。 NL 所有拥有 *印记* 的敌人，失去与层数相等的生命。",
        &i18n,
    );
    assert!(!out.contains('*'));
    assert!(out.contains("印记"));
}

#[test]
fn resolve_replaces_nl_with_newline() {
    let i18n = load_i18n();
    let out = resolve_description(
        "IronWave",
        false,
        "获得 !B! 点 格挡 。 NL 造成 !D! 点伤害。",
        &i18n,
    );
    assert_eq!(out, "获得 5 点 格挡 。\n造成 5 点伤害。");
}

#[test]
fn resolve_uses_upgraded_values() {
    let i18n = load_i18n();
    let out = resolve_description(
        "Hemokinesis",
        true,
        "失去 !M! 点生命。 NL 造成 !D! 点伤害。",
        &i18n,
    );
    assert_eq!(out, "失去 2 点生命。\n造成 20 点伤害。");
}

#[test]
fn resolve_uses_base_values_when_not_upgraded() {
    let i18n = load_i18n();
    let out = resolve_description(
        "Hemokinesis",
        false,
        "失去 !M! 点生命。 NL 造成 !D! 点伤害。",
        &i18n,
    );
    assert_eq!(out, "失去 2 点生命。\n造成 15 点伤害。");
}

#[test]
fn resolve_skips_zero_values() {
    let i18n = load_i18n();
    let out = resolve_description("Strike_R", false, "!D! !B! !M!", &i18n);
    assert_eq!(out, "6 !B! !M!");
}

#[test]
fn resolve_replaces_energy_tokens() {
    let i18n = load_i18n();
    let out = resolve_description(
        "Bloodletting",
        false,
        "获得 [R] [R] 。 NL 失去 3 点生命。",
        &i18n,
    );
    assert_eq!(out, "获得 能量2 。\n失去 3 点生命。");
}

#[test]
fn resolve_compacts_multiple_energy_tokens() {
    let i18n = load_i18n();
    let out = resolve_description(
        "Offering",
        false,
        "失去 6 点生命。 NL 获得 [R] [R] 。 NL 抽 !M! 张牌。 NL 消耗 。",
        &i18n,
    );
    assert_eq!(out, "失去 6 点生命。\n获得 能量2 。\n抽 3 张牌。\n消耗 。");
}

#[test]
fn resolve_unknown_card_id_passes_through() {
    let i18n = load_i18n();
    let out = resolve_description("NoSuchCard", false, "造成 !D! 点伤害。", &i18n);
    assert_eq!(out, "造成 !D! 点伤害。");
}

#[test]
fn format_card_includes_name_cost_type_and_description() {
    let i18n = load_i18n();
    let c = card("Uppercut", "Uppercut", 2, "ATTACK");
    let out = format_card(&c, &i18n);
    assert!(out.contains("上勾拳"));
    assert!(out.contains("2费/攻击"));
    assert!(out.contains("13 点伤害"));
    assert!(out.contains("1 层 虚弱"));
}

#[test]
fn format_card_shows_plus_for_upgraded() {
    let i18n = load_i18n();
    let c = CardInfo {
        upgraded: true,
        ..card("Defend_R", "Defend", 1, "SKILL")
    };
    let out = format_card(&c, &i18n);
    assert!(out.starts_with("+"));
    assert!(out.contains("8 点 格挡"));
}

// --- build_prompt routing ---

#[test]
fn build_prompt_routes_card_reward() {
    let i18n = load_i18n();
    let state = NormalizedState {
        screen_type: Some("CARD_REWARD".into()),
        card_reward_choices: vec![card("Strike_R", "Strike", 1, "ATTACK")],
        ..test_state()
    };
    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("=== 选牌 ==="));
    assert!(!prompt.contains("=== 手牌"));
}

#[test]
fn build_prompt_routes_rest() {
    let i18n = load_i18n();
    let state = NormalizedState {
        screen_type: Some("REST".into()),
        rest_options: vec!["rest".into()],
        ..test_state()
    };
    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("=== 选项 ==="));
    assert!(!prompt.contains("=== 手牌"));
}

#[test]
fn build_prompt_routes_combat_when_monsters_present() {
    let i18n = load_i18n();
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
    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("=== 怪物"));
    assert!(!prompt.contains("=== 选牌 ==="));
}

#[test]
fn build_prompt_routes_generic_when_no_monsters() {
    let i18n = load_i18n();
    let state = NormalizedState {
        screen_type: None,
        ..test_state()
    };
    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("=== 当前状态 ==="));
    assert!(!prompt.contains("=== 怪物"));
    assert!(!prompt.contains("=== 选牌 ==="));
    assert!(!prompt.contains("=== 选项 ==="));
}

// --- added coverage tests ---

#[test]
fn resolve_three_energy_tokens() {
    let i18n = load_i18n();
    let out = resolve_description("Offering", false, "获得 [R] [R] [R] 。", &i18n);
    assert_eq!(out, "获得 能量3 。");
}

#[test]
fn danger_prefix_empty_reasons() {
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
    let output = danger_prefix(&state);
    assert_eq!(output, "危险！");
}

#[test]
fn status_line_shows_block_warning() {
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
    let output = status_line(&state);
    assert!(output.contains("需格挡！"));
}

#[test]
fn status_line_no_block_warning_when_block_sufficient() {
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
    let output = status_line(&state);
    assert!(!output.contains("需格挡！"));
}

#[test]
fn monster_with_block() {
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
    let output = format_monster(&m);
    assert!(output.contains("格挡：8"));
}

#[test]
fn rest_shows_upgradeable_cards() {
    let i18n = load_i18n();
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
    let prompt = build_prompt(&state, &i18n);
    assert!(prompt.contains("=== 可升级卡牌 ==="));
}

#[test]
fn rest_no_upgradeable_when_all_upgraded() {
    let i18n = load_i18n();
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
    let prompt = build_prompt(&state, &i18n);
    assert!(!prompt.contains("=== 可升级卡牌 ==="));
}

#[test]
fn compact_pile_empty() {
    let i18n = load_i18n();
    let cards: Vec<CardInfo> = vec![];
    let output = compact_pile("=== 抽牌堆", &cards, &i18n);
    assert_eq!(output, "=== 抽牌堆（0张）\n");
}

#[test]
fn deck_section_empty() {
    let i18n = load_i18n();
    let cards: Vec<CardInfo> = vec![];
    let output = format_deck_section(&cards, &i18n);
    assert_eq!(output, "");
}

#[test]
fn deck_section_single_cards() {
    let i18n = load_i18n();
    let cards = vec![card("Strike_R", "Strike", 1, "ATTACK")];
    let output = format_deck_section(&cards, &i18n);
    assert!(output.contains("打击(1费)"));
    assert!(!output.contains("（共"));
}
