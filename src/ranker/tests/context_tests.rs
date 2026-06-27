use crate::ranker::context::{ActionContext, ActionType};
use crate::state::{CardInfo, MonsterInfo, NormalizedState, PotionInfo, PowerInfo, RelicInfo};

fn make_state(hand: Vec<CardInfo>, monsters: Vec<MonsterInfo>) -> NormalizedState {
    NormalizedState {
        hand,
        monsters,
        energy: Some(3),
        block: Some(5),
        current_hp: Some(60),
        max_hp: Some(75),
        incoming_damage: 6,
        screen_type: Some("NONE".to_string()),
        ..Default::default()
    }
}

fn strike() -> CardInfo {
    CardInfo {
        id: "Strike_R".into(),
        name: "Strike".into(),
        cost: 1,
        card_type: "ATTACK".into(),
        description: "造成 6 点伤害".into(),
        uuid: Some("uuid-strike".into()),
        has_target: true,
        playable: true,
        upgraded: false,
        price: None,
    }
}

fn defend() -> CardInfo {
    CardInfo {
        id: "Defend_R".into(),
        name: "Defend".into(),
        cost: 1,
        card_type: "SKILL".into(),
        description: "获得 5 点 格挡".into(),
        uuid: Some("uuid-defend".into()),
        has_target: false,
        playable: true,
        upgraded: false,
        price: None,
    }
}

fn jaw_worm() -> MonsterInfo {
    MonsterInfo {
        name: "Jaw Worm".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(44),
        max_hp: Some(46),
        block: Some(0),
        intent: Some("ATTACK".into()),
        damage: Some(12),
        hits: Some(1),
        is_scaling: false,
        can_be_killed: false,
        monster_powers: vec![],
    }
}

fn find_play_context<'a>(contexts: &'a [ActionContext]) -> &'a ActionContext {
    contexts
        .iter()
        .find(|c| matches!(c.action_type, ActionType::PlayCard { .. }))
        .unwrap()
}

// --- existing tests ---

#[test]
fn builds_context_per_card_per_target() {
    let state = make_state(vec![strike()], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let play_ctx: Vec<_> = contexts
        .iter()
        .filter(|c| matches!(c.action_type, ActionType::PlayCard { .. }))
        .collect();
    assert_eq!(play_ctx.len(), 1);
    let ctx = play_ctx[0];
    assert_eq!(ctx.target_index, Some(0));
    assert_eq!(ctx.parsed.damage, Some(6));
    assert_eq!(ctx.parsed.hits, 1);
    assert_eq!(ctx.monsters.len(), 1);
}

#[test]
fn non_targeted_card_one_context() {
    let state = make_state(vec![defend()], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let play_ctx: Vec<_> = contexts
        .iter()
        .filter(|c| matches!(c.action_type, ActionType::PlayCard { .. }))
        .collect();
    assert_eq!(play_ctx.len(), 1);
    assert!(play_ctx[0].target_index.is_none());
    assert_eq!(play_ctx[0].parsed.block, Some(5));
}

#[test]
fn vars_contain_state_values() {
    let state = make_state(vec![strike()], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("cost").copied(), Some(1.0));
    assert_eq!(vars.get("current_energy").copied(), Some(3.0));
    assert_eq!(vars.get("current_block").copied(), Some(5.0));
    assert_eq!(vars.get("current_hp").copied(), Some(60.0));
    assert_eq!(vars.get("incoming_damage").copied(), Some(6.0));
}

#[test]
fn vars_contain_parsed_values() {
    let state = make_state(vec![defend()], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("block").copied(), Some(5.0));
    assert_eq!(vars.get("hits").copied(), Some(1.0));
}

#[test]
fn vars_contain_computed_values() {
    let state = make_state(vec![strike()], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("remaining_energy").copied(), Some(2.0));
    assert_eq!(vars.get("monster_count").copied(), Some(1.0));
    assert_eq!(
        vars.get("monsters_total_hp_plus_block").copied(),
        Some(44.0)
    );
    assert_eq!(vars.get("useful_cards_in_hand").copied(), Some(1.0));
}

#[test]
fn incoming_lethal_false_when_safe() {
    let state = make_state(vec![strike()], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("incoming_lethal").copied(), Some(0.0));
}

#[test]
fn incoming_lethal_true_when_would_die() {
    let mut state = make_state(vec![strike()], vec![jaw_worm()]);
    state.current_hp = Some(4);
    state.block = Some(0);
    state.incoming_damage = 12;
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("incoming_lethal").copied(), Some(1.0));
}

#[test]
fn multiple_targets_multiple_contexts() {
    let m1 = jaw_worm();
    let m2 = MonsterInfo {
        index: 1,
        ..jaw_worm()
    };
    let state = make_state(vec![strike()], vec![m1, m2]);
    let play_ctx: Vec<_> = ActionContext::build_all(&state)
        .into_iter()
        .filter(|c| matches!(c.action_type, ActionType::PlayCard { .. }))
        .collect();
    assert_eq!(play_ctx.len(), 2);
    assert_eq!(play_ctx[0].target_index, Some(0));
    assert_eq!(play_ctx[1].target_index, Some(1));
}

#[test]
fn builds_end_turn_context() {
    let state = make_state(vec![strike(), defend()], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let end_turn = contexts
        .iter()
        .find(|c| c.action_type == ActionType::EndTurn);
    assert!(end_turn.is_some());
    let et = end_turn.unwrap();
    assert_eq!(et.parsed.damage, None);
    assert_eq!(et.parsed.block, None);
}

// --- new parsed vars ---

fn card_with_desc(desc: &str) -> CardInfo {
    CardInfo {
        id: "Test".into(),
        name: "Test".into(),
        cost: 1,
        card_type: "SKILL".into(),
        description: desc.into(),
        uuid: Some("uuid-test".into()),
        has_target: false,
        playable: true,
        upgraded: false,
        price: None,
    }
}

#[test]
fn vars_contain_heal() {
    let state = make_state(vec![card_with_desc("回复 5 点生命")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("heal").copied(), Some(5.0));
}

#[test]
fn vars_contain_draw() {
    let state = make_state(vec![card_with_desc("抽 2 张牌")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("draw").copied(), Some(2.0));
}

#[test]
fn vars_contain_str_gain() {
    let state = make_state(vec![card_with_desc("获得 3 点 力量")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("str_gain").copied(), Some(3.0));
}

#[test]
fn vars_contain_dex_gain() {
    let state = make_state(vec![card_with_desc("获得 2 点 敏捷")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("dex_gain").copied(), Some(2.0));
}

#[test]
fn vars_contain_poison() {
    let state = make_state(vec![card_with_desc("给予 5 层 中毒")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("poison").copied(), Some(5.0));
}

#[test]
fn vars_contain_vulnerable() {
    let state = make_state(vec![card_with_desc("给予 2 层 易伤")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("vulnerable").copied(), Some(2.0));
}

#[test]
fn vars_contain_weak() {
    let state = make_state(vec![card_with_desc("给予 3 层 虚弱")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("weak").copied(), Some(3.0));
}

#[test]
fn vars_contain_energy_gain() {
    let state = make_state(
        vec![card_with_desc("[E] [E] 造成 8 点伤害")],
        vec![jaw_worm()],
    );
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("energy_gain").copied(), Some(2.0));
}

#[test]
fn vars_contain_str_loss() {
    let state = make_state(vec![card_with_desc("敌人失去 4 点力量")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("str_loss").copied(), Some(4.0));
    assert_eq!(vars.get("str_loss_temp").copied(), Some(0.0));
}

#[test]
fn vars_contain_str_loss_temp() {
    let state = make_state(
        vec![card_with_desc("敌人失去 2 点力量。一回合")],
        vec![jaw_worm()],
    );
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("str_loss").copied(), Some(2.0));
    assert_eq!(vars.get("str_loss_temp").copied(), Some(1.0));
}

#[test]
fn vars_contain_mantra() {
    let state = make_state(vec![card_with_desc("获得 3 层 真言")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("mantra").copied(), Some(3.0));
}

#[test]
fn vars_contain_focus_gain() {
    let state = make_state(vec![card_with_desc("获得 2 点 集中")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("focus_gain").copied(), Some(2.0));
}

#[test]
fn vars_contain_channel_orb() {
    let state = make_state(vec![card_with_desc("充能 1 个 闪电 球")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("channel_orb_Lightning").copied(), Some(1.0));
}

#[test]
fn vars_contain_evoke_orb() {
    let state = make_state(
        vec![card_with_desc("激发 你的 所有 闪电 球")],
        vec![jaw_worm()],
    );
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("evoke_orb_Lightning").copied(), Some(1.0));
}

#[test]
fn vars_contain_orb_slot_expand() {
    let state = make_state(
        vec![card_with_desc("获得 1 个 充能球栏位")],
        vec![jaw_worm()],
    );
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("expand_count").copied(), Some(1.0));
}

#[test]
fn vars_contain_exits_stance() {
    let state = make_state(
        vec![card_with_desc("退出 当前 姿态。造成 8 点伤害")],
        vec![jaw_worm()],
    );
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("exits_stance").copied(), Some(1.0));
}

#[test]
fn vars_contain_enters_wrath() {
    let state = make_state(vec![card_with_desc("进入 愤怒 姿态")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("enters_wrath").copied(), Some(1.0));
}

#[test]
fn vars_contain_enters_calm() {
    let state = make_state(vec![card_with_desc("进入 宁静")], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("enters_calm").copied(), Some(1.0));
}

#[test]
fn vars_contain_ethereal() {
    let state = make_state(
        vec![card_with_desc("造成 6 点伤害。 虚无")],
        vec![jaw_worm()],
    );
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("ethereal").copied(), Some(1.0));
}

// --- state-derived vars ---

#[test]
fn vars_contain_turn() {
    let mut state = make_state(vec![strike()], vec![jaw_worm()]);
    state.turn_number = Some(3);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("turn").copied(), Some(3.0));
}

#[test]
fn vars_contain_player_stance() {
    let mut state = make_state(vec![strike()], vec![jaw_worm()]);
    state.powers = vec![PowerInfo {
        id: "Wrath".into(),
        name: "Wrath".into(),
        amount: 1,
    }];
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("player_stance_Wrath").copied(), Some(1.0));
}

#[test]
fn vars_contain_focus() {
    let mut state = make_state(vec![strike()], vec![jaw_worm()]);
    state.powers = vec![PowerInfo {
        id: "Focus".into(),
        name: "Focus".into(),
        amount: 3,
    }];
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("focus").copied(), Some(3.0));
}

#[test]
fn vars_contain_player_str_amount() {
    let mut state = make_state(vec![strike()], vec![jaw_worm()]);
    state.powers = vec![PowerInfo {
        id: "Strength".into(),
        name: "Strength".into(),
        amount: 4,
    }];
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("player_str_amount").copied(), Some(4.0));
}

#[test]
fn vars_contain_chemical_x_bonus() {
    let mut state = make_state(vec![strike()], vec![jaw_worm()]);
    state.relics = vec![RelicInfo {
        id: "Chemical X".into(),
        name: "Chemical X".into(),
        description: String::new(),
        counter: None,
        price: None,
    }];
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("chemical_x_bonus").copied(), Some(2.0));
}

#[test]
fn vars_contain_free_count_and_pile_size() {
    let mut state = make_state(vec![strike()], vec![jaw_worm()]);
    state.draw_pile = vec![
        CardInfo {
            id: "Defend_R".into(),
            name: "Defend".into(),
            cost: 0,
            card_type: "SKILL".into(),
            description: String::new(),
            uuid: Some("uuid-d1".into()),
            has_target: false,
            playable: true,
            upgraded: false,
            price: None,
        },
        CardInfo {
            id: "Slash".into(),
            name: "Slash".into(),
            cost: 2,
            card_type: "ATTACK".into(),
            description: String::new(),
            uuid: Some("uuid-d2".into()),
            has_target: true,
            playable: true,
            upgraded: false,
            price: None,
        },
        CardInfo {
            id: "Burn".into(),
            name: "Burn".into(),
            cost: 0,
            card_type: "STATUS".into(),
            description: String::new(),
            uuid: Some("uuid-d3".into()),
            has_target: false,
            playable: false,
            upgraded: false,
            price: None,
        },
    ];
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("free_count").copied(), Some(1.0));
    assert_eq!(vars.get("pile_size").copied(), Some(3.0));
}

#[test]
fn vars_contain_card_base_score() {
    let state = make_state(vec![strike()], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert!(vars.contains_key("card_base_score"));
}

// --- X-cost resolution ---

#[test]
fn x_cost_attack_sets_hits_to_energy() {
    let mut x_card = strike();
    x_card.cost = -1;
    x_card.description = "造成 4 点伤害 X 次".into();
    let state = make_state(vec![x_card], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("cost").copied(), Some(3.0));
    assert_eq!(vars.get("hits").copied(), Some(3.0));
    assert_eq!(vars.get("remaining_energy").copied(), Some(0.0));
}

#[test]
fn x_cost_with_chemical_x_adds_bonus() {
    let mut x_card = strike();
    x_card.cost = -1;
    x_card.description = "造成 4 点伤害 X 次".into();
    let mut state = make_state(vec![x_card], vec![jaw_worm()]);
    state.relics = vec![RelicInfo {
        id: "Chemical X".into(),
        name: "Chemical X".into(),
        description: String::new(),
        counter: None,
        price: None,
    }];
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("hits").copied(), Some(5.0));
}

#[test]
fn non_attack_x_cost_passes_energy_as_effect() {
    let x_card = CardInfo {
        id: "Malaise".into(),
        name: "Malaise".into(),
        cost: -1,
        card_type: "SKILL".into(),
        description: "敌人失去 X 点力量".into(),
        uuid: Some("uuid-x".into()),
        has_target: true,
        playable: true,
        upgraded: false,
        price: None,
    };
    let state = make_state(vec![x_card], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("cost").copied(), Some(3.0));
}

// --- AoE and random-target detection ---

#[test]
fn detects_aoe_card() {
    let card = card_with_desc("造成 4 点伤害 给 所有 敌人");
    let state = make_state(vec![card], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("aoe").copied(), Some(1.0));
}

#[test]
fn detects_aoe_en() {
    let card = card_with_desc("Deal 4 damage to ALL enemies");
    let state = make_state(vec![card], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("aoe").copied(), Some(1.0));
}

#[test]
fn detects_random_target() {
    let mut card = strike();
    card.has_target = false;
    card.description = "造成 6 点伤害".into();
    let state = make_state(vec![card], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("random_target").copied(), Some(1.0));
}

#[test]
fn non_aoe_card_has_no_aoe_flag() {
    let state = make_state(vec![strike()], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;
    assert_eq!(vars.get("aoe").copied(), None);
}

// --- AoE creates per-monster contexts (for per-monster state resolution) ---

fn aoe_card() -> CardInfo {
    CardInfo {
        id: "Whirlwind".into(),
        name: "Whirlwind".into(),
        cost: -1, // X-cost
        card_type: "ATTACK".into(),
        description: "造成 5 点伤害 给 所有 敌人 X 次".into(),
        uuid: Some("uuid-whirlwind".into()),
        has_target: false,
        playable: true,
        upgraded: false,
        price: None,
    }
}

fn monster_with(name: &str, short: &str, index: usize, hp: i64) -> MonsterInfo {
    MonsterInfo {
        name: format!("{name} {short}"),
        monster_id: None,
        index,
        current_hp: Some(hp),
        max_hp: Some(hp),
        block: Some(0),
        intent: Some("ATTACK".into()),
        damage: Some(8),
        hits: Some(1),
        is_scaling: false,
        can_be_killed: false,
        monster_powers: vec![],
    }
}

#[test]
fn aoe_creates_one_context_per_monster() {
    let monsters = vec![
        monster_with("Slime", "S", 0, 20),
        monster_with("Slime", "M", 1, 25),
        monster_with("Slime", "L", 2, 30),
    ];
    let state = make_state(vec![aoe_card()], monsters);
    let contexts = ActionContext::build_all(&state);
    let play_ctx: Vec<_> = contexts
        .iter()
        .filter(|c| matches!(c.action_type, ActionType::PlayCard { .. }))
        .collect();
    assert_eq!(
        play_ctx.len(),
        3,
        "AoE should create one context per monster"
    );
    assert_eq!(play_ctx[0].target_index, Some(0));
    assert_eq!(play_ctx[1].target_index, Some(1));
    assert_eq!(play_ctx[2].target_index, Some(2));
}

#[test]
fn aoe_contexts_have_per_monster_target_vars() {
    let monsters = vec![
        monster_with("Slime", "S", 0, 20),
        monster_with("Slime", "M", 1, 25),
        monster_with("Slime", "L", 2, 30),
    ];
    let state = make_state(vec![aoe_card()], monsters);
    let contexts = ActionContext::build_all(&state);
    let play_ctx: Vec<_> = contexts
        .iter()
        .filter(|c| matches!(c.action_type, ActionType::PlayCard { .. }))
        .collect();
    assert_eq!(play_ctx.len(), 3);
    for (i, ctx) in play_ctx.iter().enumerate() {
        assert_eq!(ctx.target_index, Some(i));
        assert!(
            ctx.target.is_some(),
            "AoE context should have target monster"
        );
        let target = ctx.target.as_ref().unwrap();
        assert_eq!(target.index, i);
        assert!(ctx.vars.contains_key("target_damage"));
    }
}

#[test]
fn aoe_resolves_per_monster_vulnerable() {
    let mut vuln = monster_with("Slime", "V", 0, 15);
    vuln.monster_powers = vec![PowerInfo {
        id: "Vulnerable".into(),
        name: "Vulnerable".into(),
        amount: 2,
    }];
    let normal = monster_with("Slime", "N", 1, 15);
    let state = make_state(vec![aoe_card()], vec![vuln.clone(), normal.clone()]);
    let contexts = ActionContext::build_all(&state);
    let play_ctx: Vec<_> = contexts
        .iter()
        .filter(|c| matches!(c.action_type, ActionType::PlayCard { .. }))
        .collect();
    assert_eq!(play_ctx.len(), 2);
    let ctx_vuln = play_ctx.iter().find(|c| c.target_index == Some(0)).unwrap();
    let ctx_norm = play_ctx.iter().find(|c| c.target_index == Some(1)).unwrap();
    assert!(
        ctx_vuln.target.as_ref().map_or(false, |t| t
            .monster_powers
            .iter()
            .any(|p| p.id == "Vulnerable")),
        "context for Vulnerable monster should carry its Vulnerable power"
    );
    assert!(
        ctx_norm.target.as_ref().map_or(true, |t| !t
            .monster_powers
            .iter()
            .any(|p| p.id == "Vulnerable")),
        "context for normal monster should NOT carry Vulnerable"
    );
}

#[test]
fn aoe_contexts_still_see_all_monsters_for_conditions() {
    let monsters = vec![
        monster_with("Slime", "A", 0, 20),
        monster_with("Slime", "B", 1, 25),
    ];
    let state = make_state(vec![aoe_card()], monsters);
    let contexts = ActionContext::build_all(&state);
    let play_ctx: Vec<_> = contexts
        .iter()
        .filter(|c| matches!(c.action_type, ActionType::PlayCard { .. }))
        .collect();
    assert_eq!(play_ctx.len(), 2);
    for ctx in &play_ctx {
        assert_eq!(
            ctx.monsters.len(),
            2,
            "each AoE context should see all monsters"
        );
    }
}

#[test]
fn targeted_attack_still_creates_per_monster_contexts() {
    let monsters = vec![
        monster_with("Cultist", "A", 0, 20),
        monster_with("Cultist", "B", 1, 25),
        monster_with("Cultist", "C", 2, 30),
    ];
    let state = make_state(vec![strike()], monsters);
    let contexts = ActionContext::build_all(&state);
    let play_ctx: Vec<_> = contexts
        .iter()
        .filter(|c| matches!(c.action_type, ActionType::PlayCard { .. }))
        .collect();
    assert_eq!(
        play_ctx.len(),
        3,
        "targeted attack should have one context per monster"
    );
    assert_eq!(play_ctx[0].target_index, Some(0));
    assert_eq!(play_ctx[1].target_index, Some(1));
    assert_eq!(play_ctx[2].target_index, Some(2));
}

#[test]
fn non_aoe_skill_remains_one_context() {
    let monsters = vec![
        monster_with("Cultist", "A", 0, 20),
        monster_with("Cultist", "B", 1, 25),
    ];
    let state = make_state(vec![defend()], monsters);
    let contexts = ActionContext::build_all(&state);
    let play_ctx: Vec<_> = contexts
        .iter()
        .filter(|c| matches!(c.action_type, ActionType::PlayCard { .. }))
        .collect();
    assert_eq!(play_ctx.len(), 1, "non-AoE skill should have one context");
    assert!(play_ctx[0].target_index.is_none());
}

// --- potions ---

#[test]
fn builds_potion_contexts() {
    let mut state = make_state(vec![], vec![jaw_worm()]);
    state.potions = vec![PotionInfo {
        name: "Fire Potion".into(),
        description: "Deal 20 damage".into(),
        price: None,
        can_use: true,
        can_discard: false,
        requires_target: true,
    }];
    let contexts = ActionContext::build_all(&state);
    let potion_ctx: Vec<_> = contexts
        .iter()
        .filter(|c| matches!(c.action_type, ActionType::UsePotion { .. }))
        .collect();
    assert_eq!(potion_ctx.len(), 1);
    let pc = potion_ctx[0];
    assert_eq!(pc.target_index, Some(0));
    assert_eq!(pc.parsed.damage, Some(20));
    assert_eq!(pc.parsed.hits, 1);
}

#[test]
fn skips_unusable_potions() {
    let mut state = make_state(vec![], vec![jaw_worm()]);
    state.potions = vec![PotionInfo {
        name: "Block Potion".into(),
        description: "Gain 12 Block".into(),
        price: None,
        can_use: false,
        can_discard: false,
        requires_target: false,
    }];
    let contexts = ActionContext::build_all(&state);
    let potion_ctx: Vec<_> = contexts
        .iter()
        .filter(|c| matches!(c.action_type, ActionType::UsePotion { .. }))
        .collect();
    assert!(potion_ctx.is_empty());
}

#[test]
fn untargeted_potion_one_context() {
    let mut state = make_state(vec![], vec![jaw_worm()]);
    state.potions = vec![PotionInfo {
        name: "Block Potion".into(),
        description: "Gain 12 Block".into(),
        price: None,
        can_use: true,
        can_discard: false,
        requires_target: false,
    }];
    let contexts = ActionContext::build_all(&state);
    let potion_ctx: Vec<_> = contexts
        .iter()
        .filter(|c| matches!(c.action_type, ActionType::UsePotion { .. }))
        .collect();
    assert_eq!(potion_ctx.len(), 1);
    assert!(potion_ctx[0].target_index.is_none());
}

// --- end turn @vars ---

#[test]
fn end_turn_has_no_parsed_effects() {
    let state = make_state(vec![], vec![jaw_worm()]);
    let contexts = ActionContext::build_all(&state);
    let end_turn = contexts
        .iter()
        .find(|c| c.action_type == ActionType::EndTurn)
        .unwrap();
    assert_eq!(end_turn.parsed.damage, None);
    assert_eq!(end_turn.parsed.block, None);
    assert_eq!(end_turn.parsed.heal, None);
}
