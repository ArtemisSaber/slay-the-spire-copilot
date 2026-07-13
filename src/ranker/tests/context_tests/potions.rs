use super::*;

#[test]
fn builds_potion_contexts() {
    let mut state = make_state(vec![], vec![jaw_worm()]);
    state.potions = vec![PotionInfo {
        slot: 0,
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
        slot: 0,
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
        slot: 0,
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
