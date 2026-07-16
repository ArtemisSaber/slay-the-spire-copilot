#[cfg(test)]
use super::LessonProposal;
use super::{ActionKind, GuidanceKind, Lesson, LessonError, OutcomeCode};

#[cfg(test)]
pub(super) fn validate_and_canonicalize(proposal: &mut LessonProposal) -> Result<(), LessonError> {
    if proposal.confidence_millis > 1_000 {
        return Err(LessonError::InvalidConfidence);
    }
    let pattern = &proposal.action_pattern;
    let invalid_pattern = match pattern.kind {
        ActionKind::PlayCard => !pattern.potion_ids.is_empty(),
        ActionKind::UsePotion => !pattern.card_types.is_empty() || !pattern.card_ids.is_empty(),
        ActionKind::EndTurn => {
            !pattern.card_types.is_empty()
                || !pattern.card_ids.is_empty()
                || !pattern.potion_ids.is_empty()
        }
        ActionKind::StrategicPolicy => true,
    };
    if invalid_pattern {
        return Err(LessonError::InvalidActionPattern);
    }
    validate_guidance_coherence(proposal.outcome_code, pattern.kind, proposal.guidance.kind)?;
    let identifiers = [
        &proposal.language,
        &proposal.scope.character,
        &proposal.critic_model_profile_sha256,
    ]
    .into_iter()
    .chain(proposal.scope.encounter_ids.iter())
    .chain(proposal.source_case_ids.iter());
    if identifiers.into_iter().any(|id| !valid_identifier(id)) {
        return Err(LessonError::InvalidIdentifier);
    }
    if !safe_text(&proposal.guidance.text, 512) || !safe_text(&proposal.rationale, 1_024) {
        return Err(LessonError::UnsafeText);
    }
    proposal.scope.ascension_bands.sort();
    sort_dedup(&mut proposal.scope.encounter_ids);
    proposal.trigger.turn_buckets.sort();
    proposal.trigger.block_threat_buckets.sort();
    sort_dedup(&mut proposal.trigger.required_card_ids);
    sort_dedup(&mut proposal.trigger.required_enemy_power_ids);
    sort_dedup(&mut proposal.trigger.required_ranker_tags);
    sort_dedup(&mut proposal.action_pattern.card_types);
    sort_dedup(&mut proposal.action_pattern.card_ids);
    sort_dedup(&mut proposal.action_pattern.potion_ids);
    sort_dedup(&mut proposal.source_case_ids);
    Ok(())
}

pub(super) fn validate_guidance_coherence(
    outcome_code: OutcomeCode,
    action_kind: ActionKind,
    guidance_kind: GuidanceKind,
) -> Result<(), LessonError> {
    let coherent = match outcome_code {
        OutcomeCode::CombatDeath | OutcomeCode::HighCombatHpLoss | OutcomeCode::TurnDamageTaken => {
            matches!(guidance_kind, GuidanceKind::Avoid | GuidanceKind::Caution)
        }
        OutcomeCode::CombatWin
        | OutcomeCode::LowCombatHpLoss
        | OutcomeCode::CombatCompletedQuickly => {
            matches!(guidance_kind, GuidanceKind::Prefer | GuidanceKind::Consider)
        }
        OutcomeCode::PotionSpent => {
            action_kind == ActionKind::UsePotion
                && matches!(guidance_kind, GuidanceKind::Prefer | GuidanceKind::Consider)
        }
        OutcomeCode::PotionPreserved => return Err(LessonError::UnsupportedOutcome),
        OutcomeCode::RunProgression => {
            action_kind == ActionKind::StrategicPolicy
                && guidance_kind == GuidanceKind::Experimental
        }
    };
    if !coherent {
        return Err(LessonError::IncoherentGuidance);
    }
    Ok(())
}

pub(crate) fn lesson_guidance_is_coherent(lesson: &Lesson) -> bool {
    validate_guidance_coherence(
        lesson.outcome_code,
        lesson.action_pattern.kind,
        lesson.guidance.kind,
    )
    .is_ok()
}

#[cfg(test)]
fn sort_dedup(values: &mut Vec<String>) {
    values.sort();
    values.dedup();
}

#[cfg(test)]
fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.trim() == value
        && value.len() <= 256
        && !value.chars().any(char::is_control)
}

#[cfg(test)]
fn safe_text(value: &str, max: usize) -> bool {
    let lower = value.to_ascii_lowercase();
    !value.trim().is_empty()
        && value.len() <= max
        && !value
            .chars()
            .any(|character| character.is_control() && character != '\n')
        && !["```", "<script", "http://", "https://"]
            .iter()
            .any(|needle| lower.contains(needle))
}
