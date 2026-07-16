#[cfg(test)]
use super::{Critic, LessonError, LessonProposal, SupportStats, identity, validation};
use super::{Lesson, LessonStatus, matching};
use crate::learning::case::DecisionCase;

impl Lesson {
    #[cfg(test)]
    pub fn propose(
        mut proposal: LessonProposal,
        cases: &[DecisionCase],
    ) -> Result<Self, LessonError> {
        validation::validate_and_canonicalize(&mut proposal)?;
        let cited = proposal
            .source_case_ids
            .iter()
            .map(|id| {
                cases
                    .iter()
                    .find(|case| &case.case_id == id)
                    .ok_or(LessonError::UnknownSourceCase)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if cited
            .iter()
            .any(|case| !matching::matches_proposal(&proposal, case))
        {
            return Err(LessonError::SourceDoesNotMatch);
        }
        if !cited.iter().any(|case| {
            matching::matches_proposal(&proposal, case)
                && matching::outcome_matches(proposal.outcome_code, case) == Some(true)
        }) {
            return Err(LessonError::OutcomeNotObserved);
        }
        let mut lesson = Self {
            schema_version: 1,
            lesson_id: String::new(),
            family_key: String::new(),
            status: LessonStatus::Proposed,
            language: proposal.language,
            outcome_predicate_version: 1,
            scope: proposal.scope,
            trigger: proposal.trigger,
            action_pattern: proposal.action_pattern,
            outcome_code: proposal.outcome_code,
            guidance: proposal.guidance,
            rationale: proposal.rationale,
            source_case_ids: proposal.source_case_ids,
            support: SupportStats::default(),
            critic: Critic {
                model_profile_sha256: proposal.critic_model_profile_sha256,
                confidence_millis: proposal.confidence_millis,
            },
            strategy: None,
            lifecycle: None,
        };
        lesson.family_key = identity::family_key(&lesson)?;
        lesson.lesson_id = identity::lesson_id(&lesson)?;
        lesson.recalculate_support(cases);
        Ok(lesson)
    }

    pub fn recalculate_support(&mut self, cases: &[DecisionCase]) {
        if self.strategy.is_some() {
            return;
        }
        self.support = matching::support(self, cases);
        if matches!(
            self.status,
            LessonStatus::Validated | LessonStatus::Contested | LessonStatus::Retired
        ) {
            return;
        }
        let total = self.support.independent_cases
            + self.support.dependent_cases
            + self.support.contradicting_cases;
        if self.support.contradicting_cases >= 3
            && self.support.contradicting_cases * 100 >= total.max(1) * 40
        {
            self.status = LessonStatus::Contested;
        } else if self.support.independent_cases >= 5
            && self.support.distinct_independent_seeds >= 5
            && self.support.contradicting_cases <= self.support.independent_cases / 3
        {
            self.status = LessonStatus::Supported;
        } else {
            self.status = LessonStatus::Proposed;
        }
    }
}
