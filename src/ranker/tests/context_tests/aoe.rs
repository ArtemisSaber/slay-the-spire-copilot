use super::*;

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
        ctx_vuln
            .target
            .as_ref()
            .is_some_and(|t| t.monster_powers.iter().any(|p| p.id == "Vulnerable")),
        "context for Vulnerable monster should carry its Vulnerable power"
    );
    assert!(
        ctx_norm
            .target
            .as_ref()
            .is_none_or(|t| !t.monster_powers.iter().any(|p| p.id == "Vulnerable")),
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
