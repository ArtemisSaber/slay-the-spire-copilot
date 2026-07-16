use super::{ActionKind, Lesson, LessonProposal, OutcomeCode, SupportStats};
use crate::learning::action::SemanticAction;
use crate::learning::case::DecisionCase;
use std::collections::HashSet;

pub(super) fn matches_proposal(proposal: &LessonProposal, case: &DecisionCase) -> bool {
    scope_trigger(&proposal.scope, &proposal.trigger, case)
        && action_matches(&proposal.action_pattern, case)
        && selected_action_has_required_tags(&proposal.trigger.required_ranker_tags, case)
}

pub(super) fn support(lesson: &Lesson, cases: &[DecisionCase]) -> SupportStats {
    let mut stats = SupportStats::default();
    let mut independent_seeds = HashSet::new();
    for case in cases {
        if !lesson_matches_case(lesson, case) {
            continue;
        }
        match outcome_matches(lesson.outcome_code, case) {
            Some(true) => {
                if case
                    .retrieved_memory_ids
                    .iter()
                    .any(|id| id == &lesson.lesson_id)
                {
                    stats.dependent_cases += 1;
                } else {
                    stats.independent_cases += 1;
                    independent_seeds.insert(case.seed_hash.clone());
                }
            }
            Some(false) => stats.contradicting_cases += 1,
            None => {}
        }
    }
    stats.distinct_independent_seeds = independent_seeds.len();
    stats
}

fn scope_trigger(
    scope: &super::LessonScope,
    trigger: &super::LessonTrigger,
    case: &DecisionCase,
) -> bool {
    lesson_scope_trigger(scope, trigger, &case.situation)
}

fn lesson_scope_trigger(
    scope: &super::LessonScope,
    trigger: &super::LessonTrigger,
    situation: &crate::learning::descriptor::SituationDescriptor,
) -> bool {
    scope.character == situation.character
        && scope.objective == situation.objective
        && scope.ascension_bands.contains(&situation.ascension_band)
        && scope.encounter_ids == situation.encounter_ids
        && member_or_any(&trigger.turn_buckets, &situation.turn_bucket)
        && member_or_any(
            &trigger.block_threat_buckets,
            &situation.block_threat_bucket,
        )
        && subset(
            &trigger.required_card_ids,
            situation.playable_cards.iter().map(|card| &card.card_id),
        )
        && subset(
            &trigger.required_enemy_power_ids,
            situation
                .alive_monsters
                .iter()
                .flat_map(|monster| monster.power_ids.iter()),
        )
        && subset(&trigger.required_ranker_tags, situation.ranker_tags.iter())
}

pub(super) fn lesson_matches_situation(
    lesson: &Lesson,
    situation: &crate::learning::descriptor::SituationDescriptor,
) -> bool {
    lesson_scope_trigger(&lesson.scope, &lesson.trigger, situation)
}

pub(super) fn lesson_matches_case(lesson: &Lesson, case: &DecisionCase) -> bool {
    scope_trigger(&lesson.scope, &lesson.trigger, case)
        && action_matches(&lesson.action_pattern, case)
        && selected_action_has_required_tags(&lesson.trigger.required_ranker_tags, case)
}

fn selected_action_has_required_tags(required: &[String], case: &DecisionCase) -> bool {
    required.is_empty()
        || case
            .ranked_suggestions
            .iter()
            .find(|ranked| ranked.semantic_action == case.selected_action)
            .is_some_and(|ranked| subset(required, ranked.tags.iter()))
}

fn member_or_any<T: PartialEq>(allowed: &[T], actual: &T) -> bool {
    allowed.is_empty() || allowed.contains(actual)
}

fn subset<'a>(required: &[String], actual: impl Iterator<Item = &'a String>) -> bool {
    let actual: HashSet<_> = actual.map(String::as_str).collect();
    required.iter().all(|value| actual.contains(value.as_str()))
}

fn action_matches(pattern: &super::ActionPattern, case: &DecisionCase) -> bool {
    match (pattern.kind, &case.selected_action) {
        (
            ActionKind::PlayCard,
            SemanticAction::PlayCard {
                card_id,
                upgraded,
                target_monster_id: _,
            },
        ) => {
            let id_matches = pattern.card_ids.is_empty() || pattern.card_ids.contains(card_id);
            let type_matches = pattern.card_types.is_empty()
                || case.situation.playable_cards.iter().any(|card| {
                    card.card_id == *card_id
                        && card.upgraded == *upgraded
                        && pattern.card_types.contains(&card.card_type)
                });
            id_matches && type_matches
        }
        (ActionKind::UsePotion, SemanticAction::UsePotion { potion_id, .. }) => {
            pattern.potion_ids.is_empty() || pattern.potion_ids.contains(potion_id)
        }
        (ActionKind::EndTurn, SemanticAction::EndTurn) => true,
        _ => false,
    }
}

pub(super) fn outcome_matches(code: OutcomeCode, case: &DecisionCase) -> Option<bool> {
    let outcome = &case.outcome;
    match code {
        OutcomeCode::CombatDeath => outcome.combat_won.map(|won| !won),
        OutcomeCode::CombatWin => outcome.combat_won,
        OutcomeCode::HighCombatHpLoss => outcome.combat_hp_lost.map(|loss| loss >= 15),
        OutcomeCode::LowCombatHpLoss => outcome.combat_hp_lost.map(|loss| loss <= 5),
        OutcomeCode::PotionSpent => Some(!outcome.potions_used.is_empty()),
        OutcomeCode::PotionPreserved => outcome
            .combat_completed
            .then_some(outcome.potions_used.is_empty()),
        OutcomeCode::TurnDamageTaken => outcome.turn_hp_lost.map(|loss| loss > 0),
        OutcomeCode::CombatCompletedQuickly => outcome
            .combat_turns
            .map(|turns| outcome.combat_completed && turns <= 3),
    }
}
