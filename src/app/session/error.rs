use super::{GameRuntime, control};
use crate::advice::OverlayMetadata;
use crate::llm::AdviceScenario;
use std::io;
use std::path::Path;

impl GameRuntime {
    pub(super) async fn handle_error(&mut self, overlay_path: &Path, autoplay_control_path: &Path) {
        let Some((saved_raw, saved_normalized, saved_command_state)) =
            self.last_autoplay_state.as_ref()
        else {
            return;
        };

        self.consecutive_errors += 1;
        if self.consecutive_errors > 5 {
            tracing::error!(
                "autoplay reached {} consecutive errors, blocking",
                self.consecutive_errors
            );
            self.current_autoplay_control = None;
            return;
        }

        if let Some(control) = self.current_autoplay_control.as_mut() {
            tracing::info!("autoplay retrying after error #{}", self.consecutive_errors);
            match crate::autoplay::planner::plan_action(
                &self.provider,
                control,
                &mut self.autoplay_session,
                saved_command_state,
                saved_normalized,
                &self.locale,
                self.map_gate.shop_visited,
            )
            .await
            {
                Ok(Some(action)) => {
                    let control_load = control::refresh_autoplay_control(
                        autoplay_control_path,
                        &mut self.autoplay_last_revision,
                        &mut self.current_autoplay_control,
                    );
                    if control::autoplay_allows_execution(self.current_autoplay_control.as_ref()) {
                        tracing::info!(
                            "autoplay retry executing {:?} screen={}",
                            action,
                            saved_normalized
                                .screen_type
                                .as_ref()
                                .map(|screen_type| screen_type.as_str())
                                .unwrap_or("?"),
                        );
                        let mut stdout = io::stdout().lock();
                        crate::autoplay::action::execute_action_to(&mut stdout, &action);
                    } else {
                        let retry_scenario = AdviceScenario::from_state(saved_normalized);
                        crate::autoplay::status::write_overlay_autoplay(
                            overlay_path,
                            &OverlayMetadata {
                                screen_type: saved_normalized.screen_type.clone(),
                                scenario: retry_scenario.as_str().to_string(),
                                in_combat: crate::runtime::has_monsters(saved_raw),
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
