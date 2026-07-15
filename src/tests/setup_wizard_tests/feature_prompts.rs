use std::io::Cursor;

use super::*;

fn prompt(input: &str, existing: &[(&str, &str)]) -> (FeatureSetup, String) {
    let existing = existing
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect();
    let mut input = Cursor::new(input);
    let mut output = Vec::new();
    let setup = prompt_feature_setup(&mut input, &mut output, &existing).unwrap();
    (setup, String::from_utf8(output).unwrap())
}

#[test]
fn new_autoplay_setup_defaults_to_safe_collection() {
    let (setup, _) = prompt("y\n\n\n", &[]);

    assert_eq!(setup, FeatureSetup::new(true, LearningSetup::Collect));
}

#[test]
fn user_can_enable_retrieval_or_disable_learning() {
    let (on, _) = prompt("y\ny\ny\n", &[]);
    let (off, _) = prompt("y\nn\n", &[]);

    assert_eq!(on, FeatureSetup::new(true, LearningSetup::On));
    assert_eq!(off, FeatureSetup::new(true, LearningSetup::Off));
}

#[test]
fn disabling_autoplay_disables_learning_and_explains_why() {
    let (setup, output) = prompt("n\n", &[]);

    assert_eq!(setup, FeatureSetup::new(false, LearningSetup::Off));
    assert!(output.contains("Learning is off"));
}

#[test]
fn existing_shadow_mode_is_preserved_by_default() {
    let (setup, _) = prompt(
        "\n\n\n",
        &[("AUTO_PLAY", "true"), ("MEMORY_MODE", "shadow")],
    );

    assert_eq!(setup, FeatureSetup::new(true, LearningSetup::Shadow));
}

#[test]
fn summary_distinguishes_collection_from_active_use() {
    let mut collect = Vec::new();
    let mut on = Vec::new();
    write_feature_summary(
        &mut collect,
        FeatureSetup::new(true, LearningSetup::Collect),
    )
    .unwrap();
    write_feature_summary(&mut on, FeatureSetup::new(true, LearningSetup::On)).unwrap();

    assert!(
        String::from_utf8(collect)
            .unwrap()
            .contains("does not influence play")
    );
    assert!(
        String::from_utf8(on)
            .unwrap()
            .contains("supplies relevant past experience")
    );
}
