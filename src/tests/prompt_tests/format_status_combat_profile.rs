use super::*;

#[test]
fn combat_profile_line_basic() {
    let locale = test_locale();
    let state = NormalizedState {
        character: Some("IRONCLAD".into()),
        floor: Some(5),
        max_hp: Some(75),
        gold: Some(150),
        relics: vec![],
        ..test_state()
    };
    let output = combat_profile_line(&state, &locale);
    assert!(output.contains("铁甲战士"));
    assert!(output.contains("5"));
    assert!(output.contains("75"));
    assert!(output.contains("150"));
}

#[test]
fn combat_profile_line_with_relics_and_counter() {
    let locale = test_locale();
    let state = NormalizedState {
        character: Some("THE_SILENT".into()),
        floor: Some(10),
        max_hp: Some(60),
        gold: Some(200),
        relics: vec![
            RelicInfo {
                id: "Incense Burner".into(),
                name: "香炉".into(),
                description: "".into(),
                counter: Some(5),
                price: None,
            },
            RelicInfo {
                id: "Burning Blood".into(),
                name: "燃烧之血".into(),
                description: "".into(),
                counter: None,
                price: None,
            },
        ],
        ..test_state()
    };
    let output = combat_profile_line(&state, &locale);
    assert!(output.contains("猎人"));
    assert!(output.contains("香炉 (5)"));
    assert!(output.contains("燃烧之血"));
}

#[test]
fn combat_profile_line_unknown_class() {
    let locale = test_locale();
    let state = NormalizedState {
        character: Some("CUSTOM_CLASS".into()),
        floor: Some(1),
        max_hp: Some(70),
        gold: Some(50),
        relics: vec![],
        ..test_state()
    };
    let output = combat_profile_line(&state, &locale);
    assert!(output.contains("CUSTOM_CLASS"));
}

#[test]
fn combat_profile_line_no_character() {
    let locale = test_locale();
    let state = NormalizedState {
        character: None,
        ..test_state()
    };
    let output = combat_profile_line(&state, &locale);
    assert!(!output.contains("角色"));
}
