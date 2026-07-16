use super::support::{
    decision_case, decision_case_for, lesson_for, lesson_for_confidence, situation,
    strategic_lesson_for,
};
use crate::learning::config::MemoryConfig;
use crate::learning::descriptor::{CountBucket, RatioBucket};
use crate::learning::lesson::{
    Lesson, LessonStatus, OutcomeCode, StrategicEvidence, StrategicLessonProposal,
};
use crate::learning::retrieval::{RetrievalItem, RetrievalQuery, retrieve, situation_similarity};
use crate::learning::snapshot::KnowledgeSnapshot;

#[test]
fn similarity_is_exact_and_weighted_with_adjacent_buckets() {
    let baseline = situation();
    assert_eq!(situation_similarity(&baseline, &baseline), 1_000);

    let mut adjacent_energy = baseline.clone();
    adjacent_energy.energy_bucket = CountBucket::Three;
    assert_eq!(situation_similarity(&baseline, &adjacent_energy), 975);

    let mut adjacent_hp = baseline.clone();
    adjacent_hp.hp_ratio_bucket = RatioBucket::P61_80;
    assert_eq!(situation_similarity(&baseline, &adjacent_hp), 963);
}

#[test]
fn hard_filters_exclude_same_seed_and_different_compatibility_profiles() {
    let same_seed = decision_case("same", 1);
    let usable = decision_case("usable", 2);
    let wrong_mod = decision_case_for("mod", 3, situation(), true, &[], "other-mods");
    let snapshot =
        KnowledgeSnapshot::build(vec![same_seed, usable.clone(), wrong_mod], &[]).unwrap();
    let result = retrieve(
        &snapshot,
        &RetrievalQuery {
            situation: situation(),
            ascension_level: Some(20),
            seed_hash: crate::learning::case::seed_hash(1, "mods"),
            compatibility_sha256: "mods".into(),
            language: "en".into(),
        },
        &MemoryConfig::default(),
    );

    assert_eq!(result.items.len(), 1);
    assert_eq!(result.items[0].id(), usable.case_id);
}

#[test]
fn strategic_lesson_requires_language_and_exact_ascension_match() {
    let case = decision_case("source", 1);
    let lesson = strategic_lesson_for(&case);
    let snapshot =
        KnowledgeSnapshot::build_with_lessons(vec![case], vec![lesson.clone()], &[]).unwrap();
    let query = |language: &str| RetrievalQuery {
        situation: situation(),
        ascension_level: Some(20),
        seed_hash: crate::learning::case::seed_hash(99, "mods"),
        compatibility_sha256: "mods".into(),
        language: language.into(),
    };

    let matching = retrieve(&snapshot, &query("en"), &MemoryConfig::default());
    assert!(
        matching
            .items
            .iter()
            .any(|item| item.id() == lesson.lesson_id)
    );
    let wrong_language = retrieve(&snapshot, &query("zh"), &MemoryConfig::default());
    assert!(
        !wrong_language
            .items
            .iter()
            .any(|item| item.id() == lesson.lesson_id)
    );
    let mut wrong_ascension = query("en");
    wrong_ascension.ascension_level = Some(19);
    assert!(
        !retrieve(&snapshot, &wrong_ascension, &MemoryConfig::default())
            .items
            .iter()
            .any(|item| item.id() == lesson.lesson_id)
    );
}

#[test]
fn unvalidated_legacy_lesson_is_not_retrieved_even_with_high_confidence() {
    let case = decision_case("source", 1);
    let lesson = lesson_for_confidence(&case, 1_000);
    let snapshot =
        KnowledgeSnapshot::build_with_lessons(vec![case], vec![lesson.clone()], &[]).unwrap();
    let result = retrieve(
        &snapshot,
        &RetrievalQuery {
            situation: situation(),
            ascension_level: Some(20),
            seed_hash: crate::learning::case::seed_hash(99, "mods"),
            compatibility_sha256: "mods".into(),
            language: "en".into(),
        },
        &MemoryConfig::default(),
    );

    assert!(
        !result
            .items
            .iter()
            .any(|item| item.id() == lesson.lesson_id)
    );
}

#[test]
fn validated_lesson_ranks_above_a_raw_case_but_both_remain_observational() {
    let case = decision_case("source", 1);
    let mut lesson = lesson_for(&case);
    lesson.set_human_status(LessonStatus::Validated).unwrap();
    let snapshot =
        KnowledgeSnapshot::build_with_lessons(vec![case.clone()], vec![lesson.clone()], &[])
            .unwrap();
    let result = retrieve(
        &snapshot,
        &RetrievalQuery {
            situation: situation(),
            ascension_level: Some(20),
            seed_hash: crate::learning::case::seed_hash(99, "mods"),
            compatibility_sha256: "mods".into(),
            language: "en".into(),
        },
        &MemoryConfig::default(),
    );

    assert!(matches!(result.items[0], RetrievalItem::Lesson { .. }));
    assert_eq!(result.items[0].id(), lesson.lesson_id);
    assert!(result.items.iter().any(|item| item.id() == case.case_id));
}

#[test]
fn contested_and_retired_lessons_are_not_retrieved() {
    for status in [LessonStatus::Contested, LessonStatus::Retired] {
        let case = decision_case("source", 1);
        let mut lesson = lesson_for(&case);
        lesson.set_human_status(status).unwrap();
        let snapshot =
            KnowledgeSnapshot::build_with_lessons(vec![case], vec![lesson.clone()], &[]).unwrap();
        let result = retrieve(
            &snapshot,
            &RetrievalQuery {
                situation: situation(),
                ascension_level: Some(20),
                seed_hash: crate::learning::case::seed_hash(99, "mods"),
                compatibility_sha256: "mods".into(),
                language: "en".into(),
            },
            &MemoryConfig::default(),
        );
        assert!(
            !result
                .items
                .iter()
                .any(|item| item.id() == lesson.lesson_id)
        );
    }
}

#[test]
fn lesson_cannot_reveal_itself_to_its_only_source_seed() {
    let case = decision_case("source", 1);
    let lesson = lesson_for(&case);
    let snapshot = KnowledgeSnapshot::build_with_lessons(vec![case], vec![lesson], &[]).unwrap();
    let result = retrieve(
        &snapshot,
        &RetrievalQuery {
            situation: situation(),
            ascension_level: Some(20),
            seed_hash: crate::learning::case::seed_hash(1, "mods"),
            compatibility_sha256: "mods".into(),
            language: "en".into(),
        },
        &MemoryConfig::default(),
    );

    assert!(result.items.is_empty());
}

#[test]
fn multi_source_lesson_is_hidden_when_any_source_uses_the_current_seed() {
    let first = decision_case("origin", 1);
    let second = decision_case("related", 2);
    let template = strategic_lesson_for(&first);
    let mut strategy = template.strategy.clone().unwrap();
    strategy.evidence.push(StrategicEvidence {
        run_id: second.run_id.clone(),
        decision_ids: vec![second.decision_id.clone()],
        observed_chain: "A related run supplied another observed trajectory.".into(),
    });
    let lesson = Lesson::propose_strategy(
        StrategicLessonProposal {
            language: template.language.clone(),
            scope: template.scope.clone(),
            strategy,
            source_case_ids: vec![first.case_id.clone(), second.case_id.clone()],
            critic_model_profile_sha256: template.critic.model_profile_sha256.clone(),
            confidence_millis: template.critic.confidence_millis,
            benchmark: template.lifecycle.unwrap().benchmark,
            parent_lesson_id: None,
            generation: 1,
        },
        &[first.clone(), second.clone()],
    )
    .unwrap();
    let snapshot =
        KnowledgeSnapshot::build_with_lessons(vec![first, second], vec![lesson.clone()], &[])
            .unwrap();
    let result = retrieve(
        &snapshot,
        &RetrievalQuery {
            situation: situation(),
            ascension_level: Some(20),
            seed_hash: crate::learning::case::seed_hash(1, "mods"),
            compatibility_sha256: "mods".into(),
            language: "en".into(),
        },
        &MemoryConfig::default(),
    );

    assert!(
        !result
            .items
            .iter()
            .any(|item| item.id() == lesson.lesson_id)
    );
}

#[test]
fn lesson_outcome_code_remains_an_association_not_a_reward() {
    let case = decision_case("source", 1);
    let lesson = lesson_for(&case);
    assert_eq!(lesson.outcome_code, OutcomeCode::CombatWin);
    assert!(!serde_json::to_string(&lesson).unwrap().contains("reward"));
}
