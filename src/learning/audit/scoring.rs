use crate::learning::case::DecisionCase;

pub(super) fn terminal_order(left: &DecisionCase, right: &DecisionCase) -> std::cmp::Ordering {
    (left.outcome.player_died_after_action == Some(true))
        .cmp(&(right.outcome.player_died_after_action == Some(true)))
        .then_with(|| decision_sequence(left).cmp(&decision_sequence(right)))
        .then_with(|| right.case_id.cmp(&left.case_id))
}

pub(super) fn victory_order(left: &DecisionCase, right: &DecisionCase) -> std::cmp::Ordering {
    let direct = |case: &DecisionCase| {
        case.outcome.alive_monsters_after_action == Some(0)
            && case.outcome.player_died_after_action != Some(true)
    };
    direct(left)
        .cmp(&direct(right))
        .then_with(|| decision_sequence(left).cmp(&decision_sequence(right)))
        .then_with(|| right.case_id.cmp(&left.case_id))
}

pub(super) fn harmful_ranker_disagreement(case: &DecisionCase) -> bool {
    observed_damage(case) > 0
        && case
            .ranked_suggestions
            .first()
            .is_some_and(|best| best.semantic_action != case.selected_action)
}

pub(super) fn observed_damage(case: &DecisionCase) -> i64 {
    case.outcome
        .action_hp_lost
        .unwrap_or(0)
        .max(case.outcome.turn_hp_lost.unwrap_or(0))
}

pub(crate) fn ranker_regret(case: &DecisionCase) -> i64 {
    let Some(best) = case.ranked_suggestions.first() else {
        return 0;
    };
    let selected_score = case
        .ranked_suggestions
        .iter()
        .find(|ranked| ranked.semantic_action == case.selected_action)
        .map_or(0, |ranked| ranked.score);
    best.score.saturating_sub(selected_score).max(0)
}

pub(crate) fn decision_sequence(case: &DecisionCase) -> u64 {
    case.decision_id
        .rsplit(':')
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

pub(super) fn general_priority(case: &DecisionCase) -> u8 {
    if case.outcome.combat_won == Some(false) {
        return 5;
    }
    if case.outcome.combat_hp_lost.is_some_and(|loss| loss >= 15) {
        return 4;
    }
    if !case.retrieved_memory_ids.is_empty() {
        return 3;
    }
    if case
        .ranked_suggestions
        .first()
        .is_some_and(|ranked| ranked.semantic_action != case.selected_action)
    {
        return 2;
    }
    u8::from(observed_damage(case) > 0)
}
