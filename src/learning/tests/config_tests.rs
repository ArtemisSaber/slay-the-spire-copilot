use crate::learning::config::{MemoryConfig, MemoryMode};
use std::collections::HashMap;

fn config(values: &[(&str, &str)]) -> MemoryConfig {
    let values: HashMap<_, _> = values.iter().copied().collect();
    MemoryConfig::from_lookup(|key| values.get(key).map(|value| value.to_string()))
}

#[test]
fn defaults_are_conservative_and_bounded() {
    let config = config(&[]);

    assert_eq!(config.mode, MemoryMode::Off);
    assert_eq!(config.max_items, 3);
    assert_eq!(config.max_context_bytes, 2_048);
    assert_eq!(config.case_min_similarity, 800);
    assert_eq!(config.lesson_min_similarity, 700);
    assert_eq!(config.proposed_lesson_min_similarity, 850);
    assert_eq!(config.max_cases_per_run, 500);
    assert!(!config.captures());
    assert!(!config.retrieves());
    assert!(!config.injects());
}

#[test]
fn legacy_operator_identity_settings_are_ignored() {
    let with_legacy_settings = config(&[
        ("MEMORY_MOD_PROFILE_SHA256", "sha256:approved"),
        ("MEMORY_MOD_PROFILE_APPROVED", "true"),
        ("MEMORY_DEBUG_CARD_IDS", "DebugWin, TestCard,DebugWin"),
    ]);

    assert_eq!(with_legacy_settings, config(&[]));
}

#[test]
fn modes_enable_only_their_documented_capabilities() {
    let collect = config(&[("MEMORY_MODE", "collect")]);
    let shadow = config(&[("MEMORY_MODE", "shadow")]);
    let on = config(&[("MEMORY_MODE", "ON")]);

    assert!(collect.captures());
    assert!(!collect.retrieves());
    assert!(!collect.injects());
    assert!(shadow.captures());
    assert!(shadow.retrieves());
    assert!(!shadow.injects());
    assert!(on.captures());
    assert!(on.retrieves());
    assert!(on.injects());
}

#[test]
fn malformed_and_out_of_range_values_fail_to_safe_defaults() {
    let config = config(&[
        ("MEMORY_MODE", "unexpected"),
        ("MEMORY_MAX_ITEMS", "99"),
        ("MEMORY_MAX_CONTEXT_BYTES", "0"),
        ("MEMORY_CASE_MIN_SIMILARITY", "1001"),
        ("MEMORY_MAX_CASES_PER_RUN", "not-a-number"),
    ]);

    assert_eq!(config.mode, MemoryMode::Off);
    assert_eq!(config.max_items, 3);
    assert_eq!(config.max_context_bytes, 2_048);
    assert_eq!(config.case_min_similarity, 800);
    assert_eq!(config.max_cases_per_run, 500);
}
