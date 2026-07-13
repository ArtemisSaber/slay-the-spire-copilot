use super::*;

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
fn target_vars_include_sharp_hide_retaliation() {
    let mut monster = jaw_worm();
    monster.monster_powers.push(PowerInfo {
        id: "Sharp Hide".into(),
        name: "Sharp Hide".into(),
        amount: 3,
    });
    let state = make_state(vec![strike()], vec![monster]);

    let contexts = ActionContext::build_all(&state);
    let vars = &find_play_context(&contexts).vars;

    assert_eq!(vars.get("retaliatory_damage").copied(), Some(3.0));
}

#[test]
fn aoe_attack_totals_retaliation_before_block() {
    let cleave = CardInfo {
        id: "Cleave".into(),
        name: "Cleave".into(),
        cost: 1,
        card_type: "ATTACK".into(),
        description: "Deal 8 damage to all enemies.".into(),
        uuid: Some("uuid-cleave".into()),
        has_target: false,
        playable: true,
        upgraded: false,
        price: None,
    };
    let mut sharp_hide = jaw_worm();
    sharp_hide.monster_powers.push(PowerInfo {
        id: "Sharp Hide".into(),
        name: "Sharp Hide".into(),
        amount: 3,
    });
    let mut thorns = MonsterInfo {
        index: 1,
        ..jaw_worm()
    };
    thorns.monster_powers.push(PowerInfo {
        id: "Thorns".into(),
        name: "Thorns".into(),
        amount: 2,
    });
    let state = make_state(vec![cleave], vec![sharp_hide, thorns]);

    let retaliation: Vec<_> = ActionContext::build_all(&state)
        .iter()
        .filter(|ctx| matches!(ctx.action_type, ActionType::PlayCard { .. }))
        .map(|ctx| ctx.vars.get("retaliatory_damage").copied())
        .collect();

    assert_eq!(retaliation, vec![Some(5.0), Some(5.0)]);
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
