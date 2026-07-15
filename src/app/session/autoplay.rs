use super::{GameRuntime, control, state::ProcessedState};
use std::io;
use std::path::Path;

impl GameRuntime {
    pub(super) async fn handle_autoplay(
        &mut self,
        state: &ProcessedState<'_>,
        overlay_path: &Path,
        autoplay_control_path: &Path,
    ) -> bool {
        if self.current_autoplay_control.is_none() {
            return false;
        }
        let prepared = self.prepare_learning(state.normalized);
        let control = self
            .current_autoplay_control
            .as_mut()
            .expect("presence checked above");

        self.autoplay_overlay_state.mode = format!("{:?}", control.mode).to_lowercase();
        self.autoplay_overlay_state.status = "planning".into();
        crate::autoplay::status::write_overlay_autoplay(
            overlay_path,
            state.metadata,
            &self.autoplay_overlay_state,
        );

        let candidates = crate::autoplay::action::available_action_candidates(
            control,
            &self.autoplay_session,
            state.command_state,
            state.normalized,
        );
        let available_semantic_actions =
            crate::learning::action::available_semantic_actions(&candidates, state.normalized);
        let ranked_suggestions =
            crate::autoplay::combat_adviser::recorded_ranked_actions(state.normalized);

        match crate::autoplay::planner::plan_action_with_memory(
            &self.provider,
            control,
            &mut self.autoplay_session,
            state.command_state,
            state.normalized,
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
                    self.autoplay_overlay_state.status = "executing".into();
                    crate::autoplay::status::write_overlay_autoplay(
                        overlay_path,
                        state.metadata,
                        &self.autoplay_overlay_state,
                    );
                    tracing::info!(
                        "autoplay executing {:?} screen={} hash={}",
                        planned.action,
                        state.screen_type,
                        &state.hash[..16],
                    );
                    self.last_autoplay_state = Some((
                        state.raw.clone(),
                        state.normalized.clone(),
                        state.command_state.clone(),
                    ));
                    self.record_learning_execution(
                        state.normalized,
                        &planned,
                        prepared.as_ref(),
                        available_semantic_actions,
                        ranked_suggestions,
                    );
                    let mut stdout = io::stdout().lock();
                    crate::autoplay::action::execute_action_to(&mut stdout, &planned.action);
                    true
                } else {
                    self.autoplay_overlay_state.mode = format!(
                        "{:?}",
                        self.current_autoplay_control
                            .as_ref()
                            .map(|control| control.mode)
                            .unwrap_or(crate::autoplay::control::AutoPlayMode::Off)
                    )
                    .to_lowercase();
                    self.autoplay_overlay_state.status = "idle".into();
                    crate::autoplay::status::write_overlay_autoplay(
                        overlay_path,
                        state.metadata,
                        &self.autoplay_overlay_state,
                    );
                    tracing::info!(
                        "autoplay blocked before execution: control={} load={}",
                        control::autoplay_mode_name(self.current_autoplay_control.as_ref()),
                        control::autoplay_load_status(&control_load),
                    );
                    false
                }
            }
            Ok(None) => {
                self.autoplay_overlay_state.status = "idle".into();
                crate::autoplay::status::write_overlay_autoplay(
                    overlay_path,
                    state.metadata,
                    &self.autoplay_overlay_state,
                );
                false
            }
            Err(error) => {
                self.autoplay_overlay_state.status = "error".into();
                crate::autoplay::status::write_overlay_autoplay(
                    overlay_path,
                    state.metadata,
                    &self.autoplay_overlay_state,
                );
                tracing::warn!("autoplay planner did not produce an executable action: {error}");
                false
            }
        }
    }
}
