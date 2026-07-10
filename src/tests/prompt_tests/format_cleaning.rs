use super::*;

#[test]
fn clean_strips_markers() {
    let locale = test_locale();
    let out = clean_description("*Smite* into your hand.", &locale);
    assert!(!out.contains('*'));
    assert!(out.contains("Smite"));
}

#[test]
fn clean_replaces_energy_tokens() {
    let locale = test_locale();
    let out = clean_description("gain [E] .", &locale);
    assert_eq!(out, "gain 能量.");
}

#[test]
fn clean_replaces_all_energy_colors() {
    let locale = test_locale();
    let out = clean_description("[R] [G] [B] [W] [E]", &locale);
    assert_eq!(out, "5 能量");
}

#[test]
fn clean_compacts_multiple_energy_tokens() {
    let locale = test_locale();
    let out = clean_description("with [E] [E] [E] .", &locale);
    assert_eq!(out, "with 3 能量.");
}

#[test]
fn clean_preserves_game_text() {
    let locale = test_locale();
    let out = clean_description("Deal 6 damage.", &locale);
    assert_eq!(out, "Deal 6 damage.");
}

#[test]
fn clean_compacts_two_energy() {
    let locale = test_locale();
    let out = clean_description("获得 [R] [R] 。", &locale);
    assert_eq!(out, "获得 2 能量。");
}

#[test]
fn clean_three_energy_tokens() {
    let locale = test_locale();
    let out = clean_description("获得 [R] [R] [R] 。", &locale);
    assert_eq!(out, "获得 3 能量。");
}

#[test]
fn clean_double_spaces() {
    let locale = test_locale();
    let out = clean_description("Deal  6  damage.", &locale);
    assert_eq!(out, "Deal 6 damage.");
}

#[test]
fn clean_space_before_punctuation() {
    let locale = test_locale();
    let out = clean_description("Draw 1 card . Gain 3 block .", &locale);
    assert_eq!(out, "Draw 1 card. Gain 3 block.");
}

#[test]
fn clean_four_energy_tokens() {
    let locale = test_locale();
    let out = clean_description("获得 [R] [R] [R] [R] 。", &locale);
    assert_eq!(out, "获得 4 能量。");
}

#[test]
fn clean_five_energy_tokens() {
    let locale = test_locale();
    let out = clean_description("获得 [R] [R] [R] [R] [R] 。", &locale);
    assert_eq!(out, "获得 5 能量。");
}

#[test]
fn clean_six_energy_tokens() {
    let locale = test_locale();
    let out = clean_description("获得 [R] [R] [R] [R] [R] [R] 。", &locale);
    assert_eq!(out, "获得 6 能量。");
}

#[test]
fn clean_space_before_comma() {
    let locale = test_locale();
    let out = clean_description("Draw 1 card , then gain block.", &locale);
    assert_eq!(out, "Draw 1 card, then gain block.");
}

#[test]
fn clean_space_before_semicolon() {
    let locale = test_locale();
    let out = clean_description("Deal 6 ; gain 3.", &locale);
    assert_eq!(out, "Deal 6; gain 3.");
}

#[test]
fn clean_space_before_exclamation() {
    let locale = test_locale();
    let out = clean_description("Pow !", &locale);
    assert_eq!(out, "Pow!");
}

#[test]
fn clean_space_before_question() {
    let locale = test_locale();
    let out = clean_description("What ?", &locale);
    assert_eq!(out, "What?");
}
