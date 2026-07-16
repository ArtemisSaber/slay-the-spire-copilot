use std::collections::HashSet;

use super::LearningSession;
use crate::learning::bundle::embedded_bundle;
use crate::learning::case::DecisionCase;
use crate::learning::lesson::{LessonBenchmark, LessonEvent, LessonEventKind, TrialDisposition};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct LifecycleSummary {
    pub evaluated: usize,
    pub retired: usize,
}

impl LearningSession {
    pub(super) fn evaluate_lesson_lifecycle(
        &mut self,
        cases: &[DecisionCase],
        benchmark: &LessonBenchmark,
    ) -> anyhow::Result<LifecycleSummary> {
        let active: HashSet<_> = self
            .snapshot
            .lessons
            .iter()
            .filter(|lesson| lesson.is_strategic())
            .map(|lesson| lesson.lesson_id.as_str())
            .collect();
        let used: HashSet<_> = cases
            .iter()
            .flat_map(|case| case.memory_ids_used.iter().map(String::as_str))
            .filter(|id| active.contains(id))
            .collect();
        if used.len() != 1 {
            return Ok(LifecycleSummary::default());
        }
        let lesson_id = *used.iter().next().expect("one used lesson exists");
        let Some(mut lesson) = self
            .snapshot
            .lessons
            .iter()
            .find(|lesson| lesson.lesson_id == lesson_id)
            .cloned()
        else {
            return Ok(LifecycleSummary::default());
        };
        let disposition = lesson
            .record_trial(benchmark)
            .map_err(|error| anyhow::anyhow!("lesson trial failed: {error:?}"))?;
        let (event_kind, reason) = match disposition {
            TrialDisposition::Ignored => return Ok(LifecycleSummary::default()),
            TrialDisposition::Retained => (LessonEventKind::Evaluated, None),
            TrialDisposition::Retired => (
                LessonEventKind::Retired,
                Some(
                    "Automatically retired after two consecutive comparable runs finished below the lesson benchmark."
                        .to_string(),
                ),
            ),
        };
        let event = LessonEvent::new(event_kind, lesson, reason, crate::journal::timestamp_ms())
            .map_err(|error| anyhow::anyhow!("lesson lifecycle event failed: {error:?}"))?;
        self.store.append_lesson_events(&[event])?;
        let bundle = embedded_bundle()
            .map_err(|error| anyhow::anyhow!("embedded bundle invalid: {error:?}"))?;
        self.snapshot = self.store.rebuild(&[bundle])?;
        Ok(LifecycleSummary {
            evaluated: 1,
            retired: usize::from(disposition == TrialDisposition::Retired),
        })
    }

    pub(super) fn resolve_regeneration_without_replacement(
        &mut self,
        lesson_id: &str,
    ) -> anyhow::Result<()> {
        let Some(mut lesson) = self
            .snapshot
            .lessons
            .iter()
            .find(|lesson| lesson.lesson_id == lesson_id)
            .cloned()
        else {
            return Ok(());
        };
        let Some(lifecycle) = lesson.lifecycle.as_mut() else {
            return Ok(());
        };
        lifecycle.regeneration_resolved = true;
        let event = LessonEvent::new(
            LessonEventKind::Evaluated,
            lesson,
            None,
            crate::journal::timestamp_ms(),
        )
        .map_err(|error| anyhow::anyhow!("lesson regeneration event failed: {error:?}"))?;
        self.store.append_lesson_events(&[event])?;
        let bundle = embedded_bundle()
            .map_err(|error| anyhow::anyhow!("embedded bundle invalid: {error:?}"))?;
        self.snapshot = self.store.rebuild(&[bundle])?;
        self.store.write_status(
            &self.snapshot,
            self.config.mode,
            self.last_eligibility.as_ref(),
        )
    }
}
