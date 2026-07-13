use super::*;

#[test]
fn effort_from_screen_type_card_reward_is_heavy() {
    assert!(matches!(
        Effort::from_screen_type("CARD_REWARD", false),
        Effort::Heavy
    ));
}

#[test]
fn effort_from_screen_type_map_is_heavy() {
    assert!(matches!(
        Effort::from_screen_type("MAP", false),
        Effort::Heavy
    ));
}

#[test]
fn effort_from_screen_type_none_is_fast() {
    assert!(matches!(
        Effort::from_screen_type("NONE", false),
        Effort::Fast
    ));
}

#[test]
fn effort_from_screen_type_hand_select_is_fast() {
    assert!(matches!(
        Effort::from_screen_type("HAND_SELECT", false),
        Effort::Fast
    ));
}

#[test]
fn effort_from_screen_type_grid_is_medium() {
    assert!(matches!(
        Effort::from_screen_type("GRID", false),
        Effort::Medium
    ));
}

#[test]
fn effort_from_screen_type_other_is_medium() {
    assert!(matches!(
        Effort::from_screen_type("REST", false),
        Effort::Medium
    ));
    assert!(matches!(
        Effort::from_screen_type("UNKNOWN", false),
        Effort::Medium
    ));
}

#[test]
fn effort_card_reward_fast_when_in_combat() {
    assert!(matches!(
        Effort::from_screen_type("CARD_REWARD", true),
        Effort::Fast
    ));
}

#[test]
fn effort_grid_fast_when_in_combat() {
    assert!(matches!(
        Effort::from_screen_type("GRID", true),
        Effort::Fast
    ));
}

#[test]
fn effort_unknown_fast_when_in_combat() {
    assert!(matches!(
        Effort::from_screen_type("SHOP_SCREEN", true),
        Effort::Fast
    ));
}
