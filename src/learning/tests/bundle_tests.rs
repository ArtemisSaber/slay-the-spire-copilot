use super::support::{decision_case, lesson_for};
use crate::learning::bundle::{BundleError, KnowledgeBundle, embedded_bundle};

#[test]
fn embedded_bundle_is_schema_valid_and_self_verifying() {
    let bundle = embedded_bundle().unwrap();

    assert_eq!(bundle.schema_version, 1);
    assert_eq!(bundle.descriptor_versions, vec![1]);
    assert!(bundle.bundle_id.starts_with("sha256:"));
    assert!(bundle.verify());
    assert!(bundle.cases.iter().all(|case| case.verify_id()));
}

#[test]
fn exported_bundle_round_trips_and_detects_tampering() {
    let case = decision_case("run-a", 1);
    let bundle = KnowledgeBundle::from_cases(vec![case]).unwrap();
    let encoded = serde_json::to_string_pretty(&bundle).unwrap();
    let decoded = KnowledgeBundle::from_json(&encoded).unwrap();

    assert_eq!(bundle, decoded);
    assert!(decoded.verify());

    let mut tampered = decoded;
    tampered.cases[0].run_id = "changed".into();
    assert!(!tampered.verify());
}

#[test]
fn bundle_schema_rejects_unknown_fields_instead_of_ignoring_them() {
    let bundle = KnowledgeBundle::from_cases(vec![decision_case("run-a", 1)]).unwrap();
    let mut value = serde_json::to_value(bundle).unwrap();
    value["unexpected"] = serde_json::json!(true);

    assert_eq!(
        KnowledgeBundle::from_json(&value.to_string()),
        Err(BundleError::Json)
    );
}

#[test]
fn bundle_order_is_canonical_and_duplicate_cases_are_removed() {
    let a = decision_case("run-a", 1);
    let b = decision_case("run-b", 2);
    let bundle = KnowledgeBundle::from_cases(vec![b.clone(), a.clone(), b]).unwrap();

    assert_eq!(bundle.cases.len(), 2);
    assert!(bundle.cases[0].case_id < bundle.cases[1].case_id);
}

#[test]
fn bundle_carries_verified_reusable_lessons_with_their_cases() {
    let case = decision_case("run-a", 1);
    let lesson = lesson_for(&case);
    let bundle = KnowledgeBundle::from_knowledge(vec![case], vec![lesson.clone()]).unwrap();

    assert_eq!(bundle.lessons, vec![lesson]);
    assert!(bundle.verify());
}

#[test]
fn bundle_rejects_a_lesson_without_its_attributable_source_case() {
    let case = decision_case("run-a", 1);
    let lesson = lesson_for(&case);

    assert_eq!(
        KnowledgeBundle::from_knowledge(vec![], vec![lesson]),
        Err(BundleError::MissingLessonSource)
    );
}
