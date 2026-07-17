use super::support::{decision_case, situation, strategic_lesson_for};
use crate::learning::config::MemoryConfig;
use crate::learning::context::build_experience_context;
use crate::learning::retrieval::{RetrievalQuery, retrieve};
use crate::learning::snapshot::KnowledgeSnapshot;

fn result(with_lesson: bool) -> crate::learning::retrieval::RetrievalResult {
    let case = decision_case("private-run-id", 1);
    let lessons = with_lesson
        .then(|| strategic_lesson_for(&case))
        .into_iter()
        .collect();
    let snapshot = KnowledgeSnapshot::build_with_lessons(vec![case], lessons, &[]).unwrap();
    retrieve(
        &snapshot,
        &RetrievalQuery {
            situation: situation(),
            ascension_level: Some(20),
            seed_hash: crate::learning::case::seed_hash(99, "mods"),
            compatibility_sha256: "mods".into(),
        },
        &MemoryConfig::default(),
    )
}

#[test]
fn context_is_bounded_and_omits_private_or_hindsight_fields() {
    let config = MemoryConfig::default();
    let context = build_experience_context(&result(true), "en", &config).unwrap();
    let encoded = serde_json::to_vec(&context).unwrap();
    let text = String::from_utf8(encoded.clone()).unwrap();

    assert!(encoded.len() <= config.max_context_bytes);
    assert!(!text.contains("private-run-id"));
    assert!(!text.contains("seed_hash"));
    assert!(!text.contains("run_victory"));
    assert!(!text.contains("final_floor"));
    assert!(text.contains("observational"));
}

#[test]
fn context_respects_a_tighter_byte_budget_by_dropping_items() {
    let config = MemoryConfig {
        max_context_bytes: 1_500,
        ..MemoryConfig::default()
    };
    let context = build_experience_context(&result(true), "en", &config).unwrap();
    assert!(serde_json::to_vec(&context).unwrap().len() <= 1_500);
}

#[test]
fn no_retrieval_produces_no_prompt_field() {
    let empty = KnowledgeSnapshot::build(vec![], &[]).unwrap();
    let result = retrieve(
        &empty,
        &RetrievalQuery {
            situation: situation(),
            ascension_level: Some(20),
            seed_hash: "sha256:none".into(),
            compatibility_sha256: "mods".into(),
        },
        &MemoryConfig::default(),
    );
    assert!(build_experience_context(&result, "en", &MemoryConfig::default()).is_none());
}
