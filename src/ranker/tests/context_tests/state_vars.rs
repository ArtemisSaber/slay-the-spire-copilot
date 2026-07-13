use super::*;

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
