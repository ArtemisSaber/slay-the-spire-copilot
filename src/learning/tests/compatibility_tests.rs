use crate::learning::compatibility::runtime_compatibility_sha256;

#[test]
fn runtime_compatibility_is_automatic_stable_and_rules_scoped() {
    let first = runtime_compatibility_sha256("sha256:rules-a");
    let repeated = runtime_compatibility_sha256("sha256:rules-a");
    let changed_rules = runtime_compatibility_sha256("sha256:rules-b");

    assert_eq!(first, repeated);
    assert_ne!(first, changed_rules);
    assert!(first.starts_with("sha256:"));
}
