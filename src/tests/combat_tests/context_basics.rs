use super::*;

#[test]
fn context_builder_happy_path() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1"), strike("打击", "s2")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.cards.len(), 2);
    assert_eq!(ctx.energy, 3);
    assert_eq!(ctx.initial_stance, Stance::Neutral);
    assert_eq!(ctx.current_stance, Stance::Neutral);
    assert_eq!(ctx.strength_delta, 0);
    assert_eq!(ctx.x_cost_bonus, 0);
    assert_eq!(ctx.monsters.len(), 1);
    assert_eq!(ctx.remaining_card_plays, 2);
}

#[test]
fn context_builder_empty_hand_returns_none() {
    let mut s = state();
    s.energy = Some(3);
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    assert!(build_context(&s).is_none());
}

#[test]
fn context_builder_non_none_screen_returns_none() {
    let mut s = state();
    s.screen_type = Some(ScreenType::CardReward);
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);
    assert!(build_context(&s).is_none());
}

#[test]
fn context_builder_stance_detection_wrath() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);
    s.powers = vec![power("Wrath", "Wrath", 1)];

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.initial_stance, Stance::Wrath);
    assert_eq!(ctx.current_stance, Stance::Wrath);
}

#[test]
fn context_builder_stance_detection_calm() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);
    s.powers = vec![power("Calm", "Calm", 1)];

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.initial_stance, Stance::Calm);
}

#[test]
fn context_builder_stance_detection_divinity() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);
    s.powers = vec![power("Divinity", "Divinity", 1)];

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.initial_stance, Stance::Divinity);
}

#[test]
fn context_builder_stance_defaults_to_neutral() {
    let mut s = state();
    s.hand = vec![strike("打击", "s1")];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.initial_stance, Stance::Neutral);
}

#[test]
fn context_builder_cards_without_uuid_filtered() {
    let mut s = state();
    s.hand = vec![
        strike("打击", "s1"),
        CardInfo {
            id: "Strike_R".into(),
            name: "打击".into(),
            cost: 1,
            card_type: "ATTACK".into(),
            upgraded: false,
            uuid: None,
            description: "造成 6 点伤害。".into(),
            price: None,
            playable: true,
            has_target: true,
        },
    ];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.cards.len(), 1);
    assert_eq!(ctx.cards[0].uuid.as_deref(), Some("s1"));
}

#[test]
fn context_builder_remaining_card_plays_equals_hand_len() {
    let mut s = state();
    s.hand = vec![
        strike("打击", "s1"),
        strike("打击", "s2"),
        strike("打击", "s3"),
        defend("防御", "d1"),
        defend("防御", "d2"),
    ];
    s.monsters = vec![monster("Jaw Worm", 20, 0, vec![], 0)];
    s.energy = Some(3);

    let ctx = build_context(&s).unwrap();
    assert_eq!(ctx.remaining_card_plays, 5);
}
