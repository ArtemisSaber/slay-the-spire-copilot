use super::*;

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
