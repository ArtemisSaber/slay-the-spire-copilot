use super::{Encounter, RunCapture};
use crate::learning::eligibility::{RunFacts, RunObjective, RunOutcome};
use crate::learning::session::SessionProvenance;
use crate::state::NormalizedState;

pub(in crate::learning::session) fn facts(
    capture: &RunCapture,
    provenance: &SessionProvenance,
    reason: &str,
) -> RunFacts {
    let state = capture.last_state.as_ref();
    let terminal_outcome = (reason == "game_over").then(|| {
        if state.and_then(|state| state.current_hp).unwrap_or(0) > 0 {
            RunOutcome::Victory
        } else {
            RunOutcome::Defeat
        }
    });
    RunFacts {
        ascension_level: state.and_then(|state| state.ascension_level),
        terminal_outcome,
        character: state.and_then(|state| state.character.clone()),
        seed: state.and_then(|state| state.seed),
        objective: Some(RunObjective::Act3Victory),
        app_version: Some(env!("CARGO_PKG_VERSION").into()),
        model_profile_sha256: Some(provenance.model_profile_sha256.clone()),
        prompt_schema_version: Some(1),
        locale: Some(provenance.locale.clone()),
        synthetic_input: provenance.synthetic_input,
        stdin_test: provenance.synthetic_input,
        telemetry_consistent: capture
            .pending
            .iter()
            .all(|case| case.outcome.command_succeeded && case.outcome.combat_completed),
        ..RunFacts::default()
    }
}

pub(super) fn is_active_combat(state: &NormalizedState) -> bool {
    state.screen_type.as_ref().map(|screen| screen.as_str()) == Some("NONE")
        && state
            .monsters
            .iter()
            .any(|monster| monster.current_hp.unwrap_or(0) > 0)
}

pub(super) fn new_encounter(state: &NormalizedState) -> Option<Encounter> {
    let mut ids = state
        .monsters
        .iter()
        .map(|monster| monster.monster_id.clone())
        .collect::<Option<Vec<_>>>()?;
    ids.sort();
    Some(Encounter {
        floor: state.floor,
        ids,
        start_hp: state.current_hp.unwrap_or(0),
        potions_used: vec![],
    })
}
