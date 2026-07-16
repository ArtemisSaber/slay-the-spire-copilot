use crate::learning::case::DecisionCase;
use std::collections::HashSet;

mod scoring;
pub(crate) use scoring::{decision_sequence, ranker_regret};
use scoring::{
    general_priority, harmful_ranker_disagreement, observed_damage, terminal_order, victory_order,
};

const MAX_HARMFUL_DISAGREEMENTS: usize = 3;
const MAX_TERMINAL_WINDOW: usize = 4;
const MAX_HIGH_DAMAGE_CASES: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditRole {
    DirectDeathTransition,
    DirectVictoryTransition,
    TerminalDecision,
    HarmfulRankerDisagreement,
    HighDamage,
    MemoryExposure,
    Context,
}

impl AuditRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DirectDeathTransition => "direct_death_transition",
            Self::DirectVictoryTransition => "direct_victory_transition",
            Self::TerminalDecision => "terminal_decision",
            Self::HarmfulRankerDisagreement => "harmful_ranker_disagreement",
            Self::HighDamage => "high_damage",
            Self::MemoryExposure => "memory_exposure",
            Self::Context => "context",
        }
    }

    pub fn is_causal_candidate(self) -> bool {
        matches!(
            self,
            Self::DirectDeathTransition
                | Self::DirectVictoryTransition
                | Self::TerminalDecision
                | Self::HarmfulRankerDisagreement
                | Self::HighDamage
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AuditCase<'a> {
    pub case: &'a DecisionCase,
    pub role: AuditRole,
}

pub fn select_audit_cases<'a>(
    cases: &'a [DecisionCase],
    run_id: &str,
    maximum: usize,
) -> Vec<AuditCase<'a>> {
    if maximum == 0 {
        return vec![];
    }
    let run_cases: Vec<_> = cases.iter().filter(|case| case.run_id == run_id).collect();
    if run_cases.is_empty() {
        return vec![];
    }

    let mut selected = Vec::new();
    let mut seen = HashSet::new();
    let failed: Vec<_> = run_cases
        .iter()
        .copied()
        .filter(|case| case.outcome.combat_won == Some(false))
        .collect();

    if !failed.is_empty() {
        let primary = failed
            .iter()
            .copied()
            .max_by(|left, right| terminal_order(left, right))
            .expect("failed cases are non-empty");
        let role = if primary.outcome.player_died_after_action == Some(true) {
            AuditRole::DirectDeathTransition
        } else {
            AuditRole::TerminalDecision
        };
        push_case(&mut selected, &mut seen, primary, role, maximum);

        let mut harmful: Vec<_> = run_cases
            .iter()
            .copied()
            .filter(|case| harmful_ranker_disagreement(case))
            .collect();
        harmful.sort_by(|left, right| {
            observed_damage(right)
                .cmp(&observed_damage(left))
                .then_with(|| ranker_regret(right).cmp(&ranker_regret(left)))
                .then_with(|| decision_sequence(right).cmp(&decision_sequence(left)))
                .then_with(|| left.case_id.cmp(&right.case_id))
        });
        for case in harmful.into_iter().take(MAX_HARMFUL_DISAGREEMENTS) {
            push_case(
                &mut selected,
                &mut seen,
                case,
                AuditRole::HarmfulRankerDisagreement,
                maximum,
            );
        }

        let mut terminal = failed;
        terminal.sort_by(|left, right| {
            decision_sequence(right)
                .cmp(&decision_sequence(left))
                .then_with(|| left.case_id.cmp(&right.case_id))
        });
        for case in terminal.into_iter().take(MAX_TERMINAL_WINDOW) {
            push_case(
                &mut selected,
                &mut seen,
                case,
                AuditRole::TerminalDecision,
                maximum,
            );
        }
    } else {
        let primary = run_cases
            .iter()
            .copied()
            .max_by(|left, right| victory_order(left, right))
            .expect("run cases are non-empty");
        let role = if primary.outcome.alive_monsters_after_action == Some(0)
            && primary.outcome.player_died_after_action != Some(true)
        {
            AuditRole::DirectVictoryTransition
        } else {
            AuditRole::TerminalDecision
        };
        push_case(&mut selected, &mut seen, primary, role, maximum);
    }

    let mut damaging: Vec<_> = run_cases
        .iter()
        .copied()
        .filter(|case| observed_damage(case) > 0)
        .collect();
    damaging.sort_by(|left, right| {
        observed_damage(right)
            .cmp(&observed_damage(left))
            .then_with(|| decision_sequence(right).cmp(&decision_sequence(left)))
            .then_with(|| left.case_id.cmp(&right.case_id))
    });
    for case in damaging.into_iter().take(MAX_HIGH_DAMAGE_CASES) {
        push_case(
            &mut selected,
            &mut seen,
            case,
            AuditRole::HighDamage,
            maximum,
        );
    }

    let mut remainder = run_cases;
    remainder.sort_by(|left, right| {
        general_priority(right)
            .cmp(&general_priority(left))
            .then_with(|| decision_sequence(right).cmp(&decision_sequence(left)))
            .then_with(|| left.case_id.cmp(&right.case_id))
    });
    for case in remainder {
        let role = if !case.retrieved_memory_ids.is_empty() {
            AuditRole::MemoryExposure
        } else {
            AuditRole::Context
        };
        push_case(&mut selected, &mut seen, case, role, maximum);
    }
    selected
}

fn push_case<'a>(
    selected: &mut Vec<AuditCase<'a>>,
    seen: &mut HashSet<&'a str>,
    case: &'a DecisionCase,
    role: AuditRole,
    maximum: usize,
) {
    if selected.len() < maximum && seen.insert(case.case_id.as_str()) {
        selected.push(AuditCase { case, role });
    }
}
