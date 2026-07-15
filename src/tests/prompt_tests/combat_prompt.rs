use super::*;

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
            id: None,
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
