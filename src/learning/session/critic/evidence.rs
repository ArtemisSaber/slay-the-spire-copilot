use std::collections::{BTreeSet, HashMap, HashSet};

use super::mode::CriticMode;
use crate::learning::case::DecisionCase;

const CURRENT_RUN_CASES: usize = 14;
const RELATED_RUN_CASES: usize = 6;
const CURRENT_RUN_TERMINAL_CASES: usize = 4;

pub(super) fn supplied_cases<'a>(
    all: &'a [DecisionCase],
    current_run_id: &str,
    mode: CriticMode<'_>,
) -> Vec<&'a DecisionCase> {
    let mut run_ids = vec![current_run_id.to_string()];
    if let Some(parent) = mode.parent()
        && let Some(lifecycle) = parent.lifecycle.as_ref()
    {
        run_ids.push(lifecycle.benchmark.origin_run_id.clone());
        run_ids.extend(
            lifecycle
                .recent_trials
                .iter()
                .map(|trial| trial.run_id.clone()),
        );
    }
    run_ids.sort();
    run_ids.dedup();
    run_ids.sort_by_key(|run_id| usize::from(run_id != current_run_id));
    run_ids
        .into_iter()
        .flat_map(|run_id| {
            let maximum = if run_id == current_run_id {
                CURRENT_RUN_CASES
            } else {
                RELATED_RUN_CASES
            };
            select_run_cases(all, &run_id, maximum)
        })
        .collect()
}

pub(super) fn allowed_decisions<'a>(
    cases: &'a [&'a DecisionCase],
) -> HashMap<&'a str, HashSet<&'a str>> {
    let mut allowed = HashMap::<&str, HashSet<&str>>::new();
    for case in cases {
        allowed
            .entry(case.run_id.as_str())
            .or_default()
            .insert(case.decision_id.as_str());
    }
    allowed
}

pub(super) fn source_case_ids(
    cases: &[&DecisionCase],
    decision_ids: impl Iterator<Item = String>,
) -> Option<Vec<String>> {
    let by_decision: HashMap<_, _> = cases
        .iter()
        .map(|case| (case.decision_id.as_str(), case.case_id.as_str()))
        .collect();
    let mut ids: Vec<_> = decision_ids
        .map(|decision_id| {
            by_decision
                .get(decision_id.as_str())
                .map(|id| (*id).to_string())
        })
        .collect::<Option<_>>()?;
    ids.sort();
    ids.dedup();
    Some(ids)
}

pub(super) fn trim_oldest_case(cases: &mut Vec<&DecisionCase>, current_run_id: &str) -> bool {
    let current_count = cases
        .iter()
        .filter(|case| case.run_id == current_run_id)
        .count();
    if current_count > CURRENT_RUN_TERMINAL_CASES
        && let Some(index) = cases.iter().position(|case| case.run_id == current_run_id)
    {
        cases.remove(index);
        return true;
    }

    let mut related_counts = HashMap::<&str, usize>::new();
    for case in cases.iter().filter(|case| case.run_id != current_run_id) {
        *related_counts.entry(case.run_id.as_str()).or_default() += 1;
    }
    if let Some(index) = cases.iter().position(|case| {
        case.run_id != current_run_id
            && related_counts
                .get(case.run_id.as_str())
                .is_some_and(|count| *count > 1)
    }) {
        cases.remove(index);
        return true;
    }
    if let Some(index) = cases.iter().position(|case| case.run_id != current_run_id) {
        cases.remove(index);
        return true;
    }
    if current_count > 1
        && let Some(index) = cases.iter().position(|case| case.run_id == current_run_id)
    {
        cases.remove(index);
        return true;
    }
    false
}

fn select_run_cases<'a>(
    all: &'a [DecisionCase],
    run_id: &str,
    maximum: usize,
) -> Vec<&'a DecisionCase> {
    let mut run: Vec<_> = all.iter().filter(|case| case.run_id == run_id).collect();
    run.sort_by(|left, right| {
        sequence(left)
            .cmp(&sequence(right))
            .then_with(|| left.case_id.cmp(&right.case_id))
    });
    if run.len() <= maximum {
        return run;
    }
    let mut selected = BTreeSet::new();
    selected.insert(0);
    for index in run.len().saturating_sub(4)..run.len() {
        selected.insert(index);
    }
    let mut signals: Vec<_> = run
        .iter()
        .enumerate()
        .map(|(index, case)| (index, signal(case)))
        .collect();
    signals.sort_by(|(left_index, left), (right_index, right)| {
        right.cmp(left).then_with(|| left_index.cmp(right_index))
    });
    for (index, _) in signals.into_iter().take(4) {
        for neighbor in [
            index.saturating_sub(1),
            index,
            (index + 1).min(run.len() - 1),
        ] {
            if selected.len() < maximum {
                selected.insert(neighbor);
            }
        }
    }
    let mut slot = 0;
    while selected.len() < maximum {
        selected.insert(slot * run.len() / maximum);
        slot += 1;
    }
    selected.into_iter().map(|index| run[index]).collect()
}

fn signal(case: &DecisionCase) -> i64 {
    let damage = case
        .outcome
        .action_hp_lost
        .unwrap_or(0)
        .max(case.outcome.turn_hp_lost.unwrap_or(0));
    let best = case
        .ranked_suggestions
        .first()
        .map_or(0, |ranked| ranked.score);
    let selected = case
        .ranked_suggestions
        .iter()
        .find(|ranked| ranked.semantic_action == case.selected_action)
        .map_or(0, |ranked| ranked.score);
    damage
        .saturating_mul(1_000_000)
        .saturating_add(best.saturating_sub(selected).max(0))
}

pub(super) fn sequence(case: &DecisionCase) -> u64 {
    case.decision_id
        .rsplit(':')
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}
