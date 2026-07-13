use super::*;

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
