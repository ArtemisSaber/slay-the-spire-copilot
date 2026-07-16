use super::support::{decision_case, decision_case_for_outcome, situation, strategic_lesson_for};
use crate::learning::eligibility::RunObjective;
use crate::learning::lesson::{LessonBenchmark, LessonStatus, TrialDisposition};

fn benchmark(run_id: &str, floor: i64, victory: bool, ascension: i64) -> LessonBenchmark {
    LessonBenchmark {
        origin_run_id: run_id.into(),
        character: "IRONCLAD".into(),
        ascension_level: ascension,
        objective: RunObjective::Act3Victory,
        final_floor: floor,
        victory,
        compatibility_sha256: "mods".into(),
    }
}

#[test]
fn two_consecutive_comparable_losses_retire_a_victory_benchmarked_lesson() {
    let case = decision_case("origin", 1);
    let mut lesson = strategic_lesson_for(&case);
    let lesson_id = lesson.lesson_id.clone();

    assert_eq!(
        lesson.record_trial(&benchmark("trial-1", 40, false, 20)),
        Ok(TrialDisposition::Retained)
    );
    assert_eq!(
        lesson.record_trial(&benchmark("trial-2", 39, false, 20)),
        Ok(TrialDisposition::Retired)
    );

    let lifecycle = lesson.lifecycle.as_ref().unwrap();
    assert_eq!(lesson.status, LessonStatus::Retired);
    assert_eq!(lifecycle.qualifying_runs, 2);
    assert_eq!(lifecycle.consecutive_degraded_runs, 2);
    assert_eq!(lifecycle.recent_trials.len(), 2);
    assert!(lifecycle.recent_trials.iter().all(|trial| trial.degraded));
    assert_eq!(lesson.lesson_id, lesson_id);
    assert!(lesson.verify_identity());
}

#[test]
fn a_non_degraded_run_resets_the_consecutive_failure_streak() {
    let case = decision_case("origin", 1);
    let mut lesson = strategic_lesson_for(&case);

    lesson
        .record_trial(&benchmark("trial-1", 40, false, 20))
        .unwrap();
    lesson
        .record_trial(&benchmark("trial-2", 51, true, 20))
        .unwrap();
    lesson
        .record_trial(&benchmark("trial-3", 30, false, 20))
        .unwrap();

    let lifecycle = lesson.lifecycle.as_ref().unwrap();
    assert_eq!(lesson.status, LessonStatus::Proposed);
    assert_eq!(lifecycle.qualifying_runs, 3);
    assert_eq!(lifecycle.consecutive_degraded_runs, 1);
    assert_eq!(lifecycle.recent_trials.len(), 2);
    assert_eq!(lifecycle.recent_trials[0].run_id, "trial-2");
}

#[test]
fn incompatible_and_duplicate_trials_do_not_change_the_lifecycle() {
    let case = decision_case("origin", 1);
    let mut lesson = strategic_lesson_for(&case);

    assert_eq!(
        lesson.record_trial(&benchmark("wrong-a", 1, false, 19)),
        Ok(TrialDisposition::Ignored)
    );
    let first = benchmark("trial-1", 40, false, 20);
    assert_eq!(lesson.record_trial(&first), Ok(TrialDisposition::Retained));
    assert_eq!(lesson.record_trial(&first), Ok(TrialDisposition::Ignored));

    let lifecycle = lesson.lifecycle.as_ref().unwrap();
    assert_eq!(lifecycle.qualifying_runs, 1);
    assert_eq!(lifecycle.consecutive_degraded_runs, 1);
}

#[test]
fn validated_strategic_lessons_still_retire_after_two_degraded_trials() {
    let case = decision_case("origin", 1);
    let mut lesson = strategic_lesson_for(&case);
    lesson.set_human_status(LessonStatus::Validated).unwrap();

    lesson
        .record_trial(&benchmark("trial-1", 40, false, 20))
        .unwrap();
    assert_eq!(
        lesson.record_trial(&benchmark("trial-2", 39, false, 20)),
        Ok(TrialDisposition::Retired)
    );
    assert_eq!(lesson.status, LessonStatus::Retired);
}

#[test]
fn lifecycle_counters_are_verified_even_though_the_lesson_id_is_stable() {
    let case = decision_case("origin", 1);
    let mut lesson = strategic_lesson_for(&case);
    lesson
        .record_trial(&benchmark("trial-1", 40, false, 20))
        .unwrap();
    assert!(lesson.verify_identity());

    lesson.lifecycle.as_mut().unwrap().consecutive_degraded_runs = 0;
    assert!(!lesson.verify_identity());
}

#[test]
fn a_defeat_benchmark_degrades_only_on_a_strictly_lower_floor_loss() {
    let case = decision_case_for_outcome("origin", 1, situation(), false, &[], "mods", (false, 30));
    let mut lesson = strategic_lesson_for(&case);

    lesson
        .record_trial(&benchmark("same-floor", 30, false, 20))
        .unwrap();
    assert_eq!(
        lesson.lifecycle.as_ref().unwrap().consecutive_degraded_runs,
        0
    );
    lesson
        .record_trial(&benchmark("lower-floor", 29, false, 20))
        .unwrap();
    assert_eq!(
        lesson.lifecycle.as_ref().unwrap().consecutive_degraded_runs,
        1
    );
}
