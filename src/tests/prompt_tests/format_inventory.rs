use super::*;

#[test]
fn build_relics_potions_section_with_both() {
    let locale = test_locale();
    let state = NormalizedState {
        relics: vec![RelicInfo {
            id: "Burning Blood".into(),
            name: "燃烧之血".into(),
            description: "战斗结束时回复6点生命。".into(),
            counter: None,
            price: None,
        }],
        potions: vec![PotionInfo {
            id: None,
            slot: 0,
            name: "恐惧药水".into(),
            description: "给予3层易伤。".into(),
            price: None,
            can_use: true,
            can_discard: false,
            requires_target: false,
        }],
        ..test_state()
    };
    let output = build_relics_potions_section(&state, &locale);
    assert!(output.contains("=== 遗物 ==="));
    assert!(output.contains("燃烧之血"));
    assert!(output.contains("=== 药水 ==="));
    assert!(output.contains("恐惧药水"));
}

#[test]
fn build_relics_potions_section_empty() {
    let locale = test_locale();
    let state: NormalizedState = test_state();
    let output = build_relics_potions_section(&state, &locale);
    assert!(output.is_empty());
}
