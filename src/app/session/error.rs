use super::{GameRuntime, control};
use crate::advice::OverlayMetadata;
use crate::llm::AdviceScenario;
use std::io;
use std::path::Path;

impl GameRuntime {
    pub(super) async fn handle_error(&mut self, overlay_path: &Path, autoplay_control_path: &Path) {
        let Some((saved_raw, saved_normalized, saved_command_state)) =
            self.last_autoplay_state.clone()
        else {
            return;
        };

        let _ = self.learning.discard_last_execution();

        self.consecutive_errors += 1;
        if self.consecutive_errors > 5 {
            tracing::error!(
                "autoplay reached {} consecutive errors, blocking",
                self.consecutive_errors
            );
            self.current_autoplay_control = None;
            return;
        }

        let prepared = self.prepare_learning(&saved_normalized);
        if let Some(control) = self.current_autoplay_control.as_mut() {
            tracing::info!("autoplay retrying after error #{}", self.consecutive_errors);
            let candidates = crate::autoplay::action::available_action_candidates(
                control,
                &self.autoplay_session,
                &saved_command_state,
                &saved_normalized,
            );
            let available_semantic_actions =
                crate::learning::action::available_semantic_actions(&candidates, &saved_normalized);
            let ranked_suggestions =
                crate::autoplay::combat_adviser::recorded_ranked_actions(&saved_normalized);
            match crate::autoplay::planner::plan_action_with_memory(
                &self.provider,
                control,
                &mut self.autoplay_session,
                &saved_command_state,
                &saved_normalized,
                &self.locale,
                self.map_gate.shop_visited,
                prepared.as_ref().and_then(|memory| memory.context.as_ref()),
                prepared
                    .as_ref()
                    .map(|memory| memory.exposed_memory_ids.as_slice())
                    .unwrap_or(&[]),
            )
            .await
            {
                Ok(Some(planned)) => {
                    let control_load = control::refresh_autoplay_control(
                        autoplay_control_path,
                        &mut self.autoplay_last_revision,
                        &mut self.current_autoplay_control,
                    );
                    if control::autoplay_allows_execution(self.current_autoplay_control.as_ref()) {
                        tracing::info!(
                            "autoplay retry executing {:?} screen={}",
                            planned.action,
                            saved_normalized
                                .screen_type
                                .as_ref()
                                .map(|screen_type| screen_type.as_str())
                                .unwrap_or("?"),
                        );
                        self.record_learning_execution(
                            &saved_normalized,
                            &planned,
                            prepared.as_ref(),
                            available_semantic_actions,
                            ranked_suggestions,
                        );
                        let mut stdout = io::stdout().lock();
                        crate::autoplay::action::execute_action_to(&mut stdout, &planned.action);
                        crate::autoplay::action::record_executed_action(
                            &mut self.autoplay_session,
                            &saved_normalized,
                            &planned.action,
                        );
                    } else {
                        let retry_scenario = AdviceScenario::from_state(&saved_normalized);
                        crate::autoplay::status::write_overlay_autoplay(
                            overlay_path,
                            &OverlayMetadata {
                                screen_type: saved_normalized.screen_type.clone(),
                                scenario: retry_scenario.as_str().to_string(),
                                in_combat: crate::runtime::has_monsters(&saved_raw),
                                state_hash: saved_normalized.stable_hash(),
                                floor: saved_normalized.floor,
                                character: saved_normalized.character.clone(),
                            },
                            &crate::autoplay::status::AutoPlayState {
                                mode: format!(
                                    "{:?}",
                                    self.current_autoplay_control
                                        .as_ref()
                                        .map(|control| control.mode)
                                        .unwrap_or(crate::autoplay::control::AutoPlayMode::Off)
                                )
                                .to_lowercase(),
                                status: "idle".into(),
                            },
                        );
                        tracing::info!(
                            "autoplay retry blocked before execution: control={} load={}",
                            control::autoplay_mode_name(self.current_autoplay_control.as_ref()),
                            control::autoplay_load_status(&control_load),
                        );
                    }
                }
                Ok(None) => {
                    tracing::warn!("autoplay retry produced no action");
                }
                Err(error) => {
                    tracing::warn!("autoplay retry planner failed: {error}");
                }
            }
        }
    }
}
