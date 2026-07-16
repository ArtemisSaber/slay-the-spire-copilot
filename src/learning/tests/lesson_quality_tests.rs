use super::support::{decision_case, lesson_for};
use crate::learning::action::SemanticAction;
use crate::learning::case::DecisionCase;
use crate::learning::lesson::{Lesson, LessonError, LessonProposal, OutcomeCode};
use crate::learning::telemetry::RecordedRankedAction;

#[test]
fn lesson_rejects_unrelated_or_directionally_incoherent_outcomes() {
    let case = decision_case("run-a", 1);
    let mut potion_preserved = proposal(&case);
    potion_preserved.outcome_code = OutcomeCode::PotionPreserved;
    assert_eq!(
        Lesson::propose(potion_preserved, std::slice::from_ref(&case)),
        Err(LessonError::UnsupportedOutcome)
    );

    let mut prefer_death = proposal(&case);
    prefer_death.outcome_code = OutcomeCode::CombatDeath;
    assert_eq!(
        Lesson::propose(prefer_death, &[case]),
        Err(LessonError::IncoherentGuidance)
    );
}

#[test]
fn required_ranker_tags_must_belong_to_the_selected_action() {
    let mut case = decision_case("run-a", 1);
    case.ranked_suggestions = vec![RecordedRankedAction {
        semantic_action: SemanticAction::EndTurn,
        score: 0,
        tags: vec![],
    }];
    let mut tagged = proposal(&case);
    tagged.trigger.required_ranker_tags = vec!["damage".into()];

    assert_eq!(
        Lesson::propose(tagged, &[case]),
        Err(LessonError::SourceDoesNotMatch)
    );
}

#[test]
fn support_requires_ranker_tags_on_the_selected_action() {
    let mut source = decision_case("source", 1);
    source.ranked_suggestions = vec![RecordedRankedAction {
        semantic_action: SemanticAction::EndTurn,
        score: 0,
        tags: vec!["damage".into()],
    }];
    let mut merely_situation_tagged = decision_case("other", 2);
    merely_situation_tagged.ranked_suggestions = vec![RecordedRankedAction {
        semantic_action: SemanticAction::EndTurn,
        score: 0,
        tags: vec![],
    }];
    let mut tagged = proposal(&source);
    tagged.trigger.required_ranker_tags = vec!["damage".into()];

    let lesson = Lesson::propose(tagged, &[source, merely_situation_tagged]).unwrap();

    assert_eq!(lesson.support.independent_cases, 1);
}

fn proposal(case: &DecisionCase) -> LessonProposal {
    let lesson = lesson_for(case);
    LessonProposal {
        language: lesson.language,
        scope: lesson.scope,
        trigger: lesson.trigger,
        action_pattern: lesson.action_pattern,
        outcome_code: lesson.outcome_code,
        guidance: lesson.guidance,
        rationale: lesson.rationale,
        source_case_ids: vec![case.case_id.clone()],
        critic_model_profile_sha256: lesson.critic.model_profile_sha256,
        confidence_millis: lesson.critic.confidence_millis,
    }
}
