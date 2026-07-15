use super::SessionProvenance;
use crate::learning::case::{CaseDraft, CaseOutcome, CaseProvenance, DecisionCase};
use crate::learning::config::MemoryConfig;
use crate::learning::eligibility::{RunFacts, RunObjective, RunOutcome};
use crate::learning::telemetry::DecisionIdGenerator;
use crate::state::NormalizedState;

#[derive(Default)]
pub(super) struct RunCapture {
    last_state: Option<NormalizedState>,
    encounter: Option<Encounter>,
    pending: Vec<PendingCase>,
    generator: Option<(String, DecisionIdGenerator)>,
    debug_card_seen: bool,
}

struct Encounter {
    floor: Option<i64>,
    ids: Vec<String>,
    start_hp: i64,
    potions_used: Vec<String>,
}

struct PendingCase {
    draft: CaseDraft,
    floor: Option<i64>,
    turn: Option<i64>,
    start_hp: i64,
    encounter_start_hp: i64,
    observation_hash: String,
    outcome: CaseOutcome,
}

impl RunCapture {
    pub(super) fn observe(&mut self, state: &NormalizedState, debug_card_ids: &[String]) {
        let observation_hash = state.observation_hash();
        for pending in &mut self.pending {
            if pending.observation_hash != observation_hash {
                pending.outcome.command_succeeded = true;
            }
        }
        self.debug_card_seen |= [
            state.master_cards.as_slice(),
            state.hand.as_slice(),
            state.draw_pile.as_slice(),
            state.discard_pile.as_slice(),
            state.exhaust_cards.as_slice(),
        ]
        .into_iter()
        .flatten()
        .any(|card| debug_card_ids.contains(&card.id));
        let active = is_active_combat(state);
        if self.encounter.is_some() && !active {
            self.close_turns_and_combat(state);
            self.encounter = None;
        } else if active {
            self.close_completed_turns(state);
            if self
                .encounter
                .as_ref()
                .is_none_or(|encounter| encounter.floor != state.floor)
            {
                if self.encounter.is_some() {
                    self.close_turns_and_combat(state);
                }
                self.encounter = new_encounter(state);
            }
        }
        self.last_state = Some(state.clone());
    }

    pub(super) fn encounter_ids(&self) -> Option<&[String]> {
        self.encounter
            .as_ref()
            .filter(|encounter| !encounter.ids.is_empty())
            .map(|encounter| encounter.ids.as_slice())
    }

    pub(super) fn next_decision_id(
        &mut self,
        run_id: &str,
        floor: Option<i64>,
        turn: Option<i64>,
    ) -> String {
        if self.generator.as_ref().is_none_or(|(id, _)| id != run_id) {
            self.generator = Some((run_id.to_string(), DecisionIdGenerator::new(run_id)));
        }
        self.generator
            .as_mut()
            .expect("generator initialized above")
            .1
            .next(floor, turn)
    }

    pub(super) fn record(&mut self, draft: CaseDraft, state: &NormalizedState) {
        let Some(encounter) = self.encounter.as_mut() else {
            return;
        };
        if let crate::learning::action::SemanticAction::UsePotion { potion_id, .. } =
            &draft.selected_action
        {
            encounter.potions_used.push(potion_id.clone());
            encounter.potions_used.sort();
            encounter.potions_used.dedup();
            for pending in &mut self.pending {
                if pending.floor == encounter.floor && !pending.outcome.combat_completed {
                    pending.outcome.potions_used = encounter.potions_used.clone();
                }
            }
        }
        self.pending.push(PendingCase {
            draft,
            floor: state.floor,
            turn: state.turn_number,
            start_hp: state.current_hp.unwrap_or(0),
            encounter_start_hp: encounter.start_hp,
            observation_hash: state.observation_hash(),
            outcome: CaseOutcome {
                command_succeeded: false,
                turn_hp_lost: None,
                combat_completed: false,
                combat_won: None,
                combat_hp_lost: None,
                combat_turns: None,
                potions_used: encounter.potions_used.clone(),
                run_completed: false,
                run_victory: None,
                final_floor: None,
            },
        });
    }

    pub(super) fn discard_last_execution(&mut self) -> bool {
        self.pending.pop().is_some()
    }

    pub(super) fn finalize_cases(
        &mut self,
        terminal: Option<RunOutcome>,
        provenance: CaseProvenance,
        maximum: usize,
    ) -> anyhow::Result<Vec<DecisionCase>> {
        let final_floor = self.last_state.as_ref().and_then(|state| state.floor);
        let run_victory = terminal.map(|outcome| outcome == RunOutcome::Victory);
        self.pending
            .drain(..)
            .take(maximum)
            .map(|mut pending| {
                pending.outcome.run_completed = terminal.is_some();
                pending.outcome.run_victory = run_victory;
                pending.outcome.final_floor = final_floor;
                pending
                    .draft
                    .finalize(pending.outcome, provenance.clone())
                    .map_err(|error| anyhow::anyhow!("case finalize failed: {error:?}"))
            })
            .collect()
    }

    fn close_completed_turns(&mut self, state: &NormalizedState) {
        let current_turn = state.turn_number;
        let hp = state.current_hp.unwrap_or(0);
        for pending in &mut self.pending {
            if pending.outcome.turn_hp_lost.is_none()
                && pending.floor == state.floor
                && current_turn
                    .zip(pending.turn)
                    .is_some_and(|(now, then)| now > then)
            {
                pending.outcome.turn_hp_lost = Some((pending.start_hp - hp).max(0));
            }
        }
    }

    fn close_turns_and_combat(&mut self, state: &NormalizedState) {
        let hp = state.current_hp.unwrap_or(0);
        let combat_turns = self
            .last_state
            .as_ref()
            .filter(|last| last.floor == self.encounter.as_ref().and_then(|value| value.floor))
            .and_then(|last| last.turn_number)
            .or(state.turn_number);
        for pending in &mut self.pending {
            if pending.outcome.combat_completed
                || pending.floor != self.encounter.as_ref().and_then(|e| e.floor)
            {
                continue;
            }
            pending
                .outcome
                .turn_hp_lost
                .get_or_insert((pending.start_hp - hp).max(0));
            pending.outcome.combat_completed = true;
            pending.outcome.combat_won = Some(hp > 0);
            pending.outcome.combat_hp_lost = Some((pending.encounter_start_hp - hp).max(0));
            pending.outcome.combat_turns = combat_turns;
        }
    }
}

pub(super) fn facts(
    capture: &RunCapture,
    config: &MemoryConfig,
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
        mod_profile_sha256: config.mod_profile_sha256.clone(),
        mod_profile_approved: config.mod_profile_approved,
        debug_card_seen: capture.debug_card_seen,
        synthetic_input: provenance.synthetic_input,
        stdin_test: provenance.synthetic_input,
        telemetry_consistent: capture
            .pending
            .iter()
            .all(|case| case.outcome.command_succeeded && case.outcome.combat_completed),
        ..RunFacts::default()
    }
}

fn is_active_combat(state: &NormalizedState) -> bool {
    state.screen_type.as_ref().map(|screen| screen.as_str()) == Some("NONE")
        && state
            .monsters
            .iter()
            .any(|monster| monster.current_hp.unwrap_or(0) > 0)
}

fn new_encounter(state: &NormalizedState) -> Option<Encounter> {
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
