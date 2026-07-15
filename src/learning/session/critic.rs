use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashSet;

use super::LearningSession;
use crate::learning::audit::select_audit_cases;
use crate::learning::bundle::embedded_bundle;
use crate::learning::lesson::{
    ActionPattern, Guidance, Lesson, LessonEvent, LessonEventKind, LessonProposal, LessonScope,
    LessonStatus, LessonTrigger, OutcomeCode,
};

const MAX_PROMPT_BYTES: usize = 20_000;
const MAX_REPORT_BYTES: usize = 8_000;
const MAX_AUDIT_CASES: usize = 10;
const MAX_PROPOSALS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriticIngest {
    pub report_markdown: String,
    pub accepted_lessons: usize,
    pub rejected_lessons: usize,
    pub snapshot_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CriticEnvelope {
    schema_version: u32,
    report_markdown: String,
    #[serde(default)]
    lesson_proposals: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CandidateLesson {
    scope: LessonScope,
    trigger: LessonTrigger,
    action_pattern: ActionPattern,
    outcome_code: OutcomeCode,
    guidance: Guidance,
    rationale: String,
    source_case_ids: Vec<String>,
    confidence_millis: u16,
}

impl LearningSession {
    pub fn build_critic_prompt(&self, base_report_prompt: &str, run_id: &str) -> Option<String> {
        let cases: Vec<_> = select_audit_cases(&self.snapshot.cases, run_id, MAX_AUDIT_CASES)
            .into_iter()
            .map(case_summary)
            .collect();
        if cases.is_empty() {
            return None;
        }
        let base = truncate_utf8(base_report_prompt, MAX_REPORT_BYTES);
        let mut kept = cases;
        loop {
            let appendix = critic_appendix(&kept);
            let prompt = format!("{base}\n\nLEARNING_CRITIC_ENVELOPE_V1\n{appendix}");
            if prompt.len() <= MAX_PROMPT_BYTES {
                return Some(prompt);
            }
            if kept.len() == 1 {
                return None;
            }
            kept.pop();
        }
    }

    pub fn ingest_critic_response(
        &mut self,
        response: &str,
        run_id: &str,
    ) -> anyhow::Result<CriticIngest> {
        let Ok(envelope) = serde_json::from_str::<CriticEnvelope>(response.trim()) else {
            return Ok(self.empty_ingest(response));
        };
        if envelope.schema_version != 1 {
            return Ok(self.empty_ingest(response));
        }
        let allowed: HashSet<_> = select_audit_cases(&self.snapshot.cases, run_id, MAX_AUDIT_CASES)
            .into_iter()
            .map(|case| case.case_id.clone())
            .collect();
        let total = envelope.lesson_proposals.len().min(MAX_PROPOSALS);
        let mut events = Vec::new();
        for (index, raw_candidate) in envelope
            .lesson_proposals
            .into_iter()
            .take(MAX_PROPOSALS)
            .enumerate()
        {
            let Ok(candidate) = serde_json::from_value::<CandidateLesson>(raw_candidate) else {
                continue;
            };
            if candidate.source_case_ids.is_empty()
                || candidate
                    .source_case_ids
                    .iter()
                    .any(|id| !allowed.contains(id))
            {
                continue;
            }
            let proposal = candidate.into_proposal(
                self.provenance.locale.clone(),
                self.provenance.model_profile_sha256.clone(),
            );
            let Ok(lesson) = Lesson::propose(proposal, &self.snapshot.cases) else {
                continue;
            };
            let kind = if lesson.status == LessonStatus::Proposed {
                LessonEventKind::Proposed
            } else {
                LessonEventKind::Recalculated
            };
            if let Ok(event) = LessonEvent::new(
                kind,
                lesson,
                None,
                crate::journal::timestamp_ms() + index as u128,
            ) {
                events.push(event);
            }
        }
        let accepted = events.len();
        if !events.is_empty() {
            self.store.append_lesson_events(&events)?;
            let bundle = embedded_bundle()
                .map_err(|error| anyhow::anyhow!("embedded bundle invalid: {error:?}"))?;
            self.snapshot = self.store.rebuild(&[bundle])?;
            self.store.write_status(
                &self.snapshot,
                self.config.mode,
                self.last_eligibility.as_ref(),
            )?;
        }
        Ok(CriticIngest {
            report_markdown: envelope.report_markdown,
            accepted_lessons: accepted,
            rejected_lessons: total.saturating_sub(accepted),
            snapshot_id: self.snapshot.snapshot_id.clone(),
        })
    }

    fn empty_ingest(&self, report: &str) -> CriticIngest {
        CriticIngest {
            report_markdown: report.to_string(),
            accepted_lessons: 0,
            rejected_lessons: 0,
            snapshot_id: self.snapshot.snapshot_id.clone(),
        }
    }
}

impl CandidateLesson {
    fn into_proposal(self, language: String, critic_profile: String) -> LessonProposal {
        LessonProposal {
            language,
            scope: self.scope,
            trigger: self.trigger,
            action_pattern: self.action_pattern,
            outcome_code: self.outcome_code,
            guidance: self.guidance,
            rationale: self.rationale,
            source_case_ids: self.source_case_ids,
            critic_model_profile_sha256: critic_profile,
            confidence_millis: self.confidence_millis,
        }
    }
}

fn case_summary(case: &crate::learning::case::DecisionCase) -> Value {
    json!({
        "case_id": case.case_id,
        "situation": case.situation,
        "selected_action": case.selected_action,
        "decision_source": case.decision_source,
        "retrieved_memory_ids": case.retrieved_memory_ids,
        "memory_ids_used": case.memory_ids_used,
        "outcome": {
            "command_succeeded": case.outcome.command_succeeded,
            "turn_hp_lost": case.outcome.turn_hp_lost,
            "combat_completed": case.outcome.combat_completed,
            "combat_won": case.outcome.combat_won,
            "combat_hp_lost": case.outcome.combat_hp_lost,
            "combat_turns": case.outcome.combat_turns,
            "potions_used": case.outcome.potions_used,
        }
    })
}

fn critic_appendix(cases: &[Value]) -> String {
    serde_json::to_string_pretty(&json!({
        "instruction": "Return the required JSON envelope. Put the localized human-readable Markdown only in report_markdown. Treat cases as observations, not proof of causality or optimality. Cite only supplied case_id values. Propose at most five narrowly scoped lessons, or an empty array.",
        "output_contract": {
            "schema_version": 1,
            "report_markdown": "localized Markdown string with the complete human-readable postmortem",
            "lesson_proposals": [{
                "scope": {
                    "character": "copy the exact cited situation character",
                    "objective": "act3_victory|act4_victory",
                    "ascension_bands": ["a0|a1_9|a10_16|a17_19|a20"],
                    "encounter_ids": ["copy the exact cited situation encounter_ids"]
                },
                "trigger": {
                    "turn_buckets": ["turn1|turn2|turn3|turn4_plus"],
                    "block_threat_buckets": ["no_incoming|fully_covered|chip|danger|lethal"],
                    "required_card_ids": ["optional exact card_id from the cited situation"],
                    "required_enemy_power_ids": ["optional exact enemy power ID from the cited situation"],
                    "required_ranker_tags": ["optional exact ranker tag from the cited situation"]
                },
                "action_pattern": {
                    "kind": "play_card|use_potion|end_turn",
                    "card_types": ["empty unless kind is play_card"],
                    "card_ids": ["empty unless kind is play_card"],
                    "potion_ids": ["empty unless kind is use_potion"]
                },
                "outcome_code": "combat_death|combat_win|high_combat_hp_loss|low_combat_hp_loss|potion_spent|potion_preserved|turn_damage_taken|combat_completed_quickly",
                "guidance": {"kind": "caution|consider|avoid|prefer", "text": "observational guidance"},
                "rationale": "bounded factual rationale",
                "source_case_ids": ["supplied case_id"],
                "confidence_millis": "integer 0..1000"
            }]
        },
        "eligible_cases": cases,
    }))
    .expect("critic appendix contains serializable values")
}

fn truncate_utf8(value: &str, maximum: usize) -> &str {
    if value.len() <= maximum {
        return value;
    }
    let mut boundary = maximum;
    while !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    &value[..boundary]
}
