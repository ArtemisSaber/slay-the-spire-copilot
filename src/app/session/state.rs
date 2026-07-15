use super::{GameRuntime, control};
use crate::advice::OverlayMetadata;
use crate::llm::AdviceScenario;
use std::path::Path;

pub(crate) struct ProcessedState<'a> {
    pub(crate) raw: &'a serde_json::Value,
    pub(crate) normalized: &'a crate::state::NormalizedState,
    pub(crate) command_state: &'a crate::autoplay::command_state::CommandState,
    pub(crate) metadata: &'a OverlayMetadata,
    pub(crate) screen_type: &'a str,
    pub(crate) room_type: &'a str,
    pub(crate) hash: &'a str,
    pub(crate) scenario: AdviceScenario,
}

impl GameRuntime {
    pub(super) async fn process_game_state(
        &mut self,
        raw: serde_json::Value,
        overlay_path: &Path,
        autoplay_control_path: &Path,
    ) {
        self.saw_game_state = true;
        self.consecutive_errors = 0;

        let screen_type = raw
            .pointer("/game_state/screen_type")
            .and_then(|value| value.as_str())
            .unwrap_or("?");
        let room_type = raw
            .pointer("/game_state/room_type")
            .and_then(|value| value.as_str())
            .unwrap_or("?");

        let normalized = crate::state::NormalizedState::from_raw(&raw, &self.locale);
        self.learning.observe(&normalized);
        let hash = normalized.stable_hash();
        let command_state = crate::autoplay::command_state::CommandState::from_raw(&raw);
        let scenario = AdviceScenario::from_state(&normalized);
        let metadata = OverlayMetadata {
            screen_type: normalized.screen_type.clone(),
            scenario: scenario.as_str().to_string(),
            in_combat: crate::runtime::has_monsters(&raw),
            state_hash: hash.clone(),
            floor: normalized.floor,
            character: normalized.character.clone(),
        };

        if self.config.auto_play {
            let autoplay_control_load = control::refresh_autoplay_control(
                autoplay_control_path,
                &mut self.autoplay_last_revision,
                &mut self.current_autoplay_control,
            );
            let autoplay_load_status = control::autoplay_load_status(&autoplay_control_load);
            let autoplay_mode = control::autoplay_mode_name(self.current_autoplay_control.as_ref());
            tracing::debug!(
                "autoplay control={autoplay_mode} load={autoplay_load_status} ready={} commands={} choose_available={}",
                command_state.ready_for_command,
                command_state.available_commands.len(),
                command_state.has_command("choose"),
            );
        }

        let confirming_run = !self.journal.is_confirmed();
        if confirming_run
            && let (Some(seed), Some(character)) = (normalized.seed, normalized.character.as_ref())
        {
            self.journal.confirm(
                seed,
                character,
                normalized.ascension_level.unwrap_or(0),
                &self.config,
                &self.locale,
            );
        }
        if confirming_run
            && self.journal.is_continued_run()
            && let Some(path) = self.journal.path()
        {
            match self.learning.pin_snapshot_from_journal(path) {
                Ok(true) => tracing::info!("restored continued run knowledge snapshot"),
                Ok(false) => {}
                Err(error) => tracing::warn!(
                    "continued run snapshot unavailable; memory disabled for this run: {error:#}"
                ),
            }
        }
        if !self.journal.is_confirmed() {
            return;
        }

        let processed = ProcessedState {
            raw: &raw,
            normalized: &normalized,
            command_state: &command_state,
            metadata: &metadata,
            screen_type,
            room_type,
            hash: &hash,
            scenario,
        };

        self.journal
            .log_state_change(processed.hash, processed.normalized);
        if self
            .handle_autoplay(&processed, overlay_path, autoplay_control_path)
            .await
        {
            return;
        }

        if processed.screen_type == "MAP" {
            super::super::map::log_map_paths(processed.raw, &processed.normalized.map_nodes);
        }

        if crate::runtime::is_game_over_state(processed.raw) {
            super::super::finalization::finalize_run_once_with_learning(
                &self.journal,
                &self.provider,
                "game_over",
                &mut self.run_finalized,
                &self.locale,
                &mut self.learning,
            )
            .await;
            self.reset_after_game();
            tracing::info!("run ended, waiting for next run...");
            return;
        }

        self.generate_advice(&processed).await;
    }
}
