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
        let Some(control) = self.current_autoplay_control.as_mut() else {
            return false;
        };

        self.autoplay_overlay_state.mode = format!("{:?}", control.mode).to_lowercase();
        self.autoplay_overlay_state.status = "planning".into();
        crate::autoplay::status::write_overlay_autoplay(
            overlay_path,
            state.metadata,
            &self.autoplay_overlay_state,
        );

        match crate::autoplay::planner::plan_action(
            &self.provider,
            control,
            &mut self.autoplay_session,
            state.command_state,
            state.normalized,
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
                    self.autoplay_overlay_state.status = "executing".into();
                    crate::autoplay::status::write_overlay_autoplay(
                        overlay_path,
                        state.metadata,
                        &self.autoplay_overlay_state,
                    );
                    tracing::info!(
                        "autoplay executing {:?} screen={} hash={}",
                        action,
                        state.screen_type,
                        &state.hash[..16],
                    );
                    self.last_autoplay_state = Some((
                        state.raw.clone(),
                        state.normalized.clone(),
                        state.command_state.clone(),
                    ));
                    let mut stdout = io::stdout().lock();
                    crate::autoplay::action::execute_action_to(&mut stdout, &action);
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
