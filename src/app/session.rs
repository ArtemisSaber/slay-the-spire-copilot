mod advice;
mod autoplay;
mod control;
mod error;
mod input;
mod state;

use crate::advice::{AdviceCache, OverlayMetadata};
use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::{AutoPlayControl, AutoPlaySession};
use crate::autoplay::status::AutoPlayState;
use crate::gate::{CombatTurnGate, MapGate};
use crate::journal::Journal;
use crate::state::NormalizedState;
use std::path::{Path, PathBuf};
use tokio::io::AsyncBufReadExt;

pub(crate) struct GameRuntime {
    project_root: PathBuf,
    config: crate::config::Config,
    provider: crate::llm::LlmProvider,
    locale: crate::locales::Locale,
    cache: AdviceCache,
    journal: Journal,
    combat_turn_gate: CombatTurnGate,
    map_gate: MapGate,
    autoplay_last_revision: Option<u64>,
    current_autoplay_control: Option<AutoPlayControl>,
    autoplay_session: AutoPlaySession,
    autoplay_overlay_state: AutoPlayState,
    last_autoplay_state: Option<(serde_json::Value, NormalizedState, CommandState)>,
    consecutive_errors: usize,
    saw_game_state: bool,
    run_finalized: bool,
}

impl GameRuntime {
    pub(crate) fn new(
        project_root: PathBuf,
        config: crate::config::Config,
        provider: crate::llm::LlmProvider,
        locale: crate::locales::Locale,
    ) -> Self {
        let current_autoplay_control = if config.auto_play {
            Some(if config.auto_play_auto_start {
                AutoPlayControl::default_enabled()
            } else {
                AutoPlayControl::default_paused()
            })
        } else {
            None
        };
        if config.auto_play {
            let initial_overlay_path = crate::logging::advice_output_dir()
                .join("output")
                .join("overlay.json");
            let initial_mode = if config.auto_play_auto_start {
                "auto"
            } else {
                "paused"
            };
            crate::autoplay::status::write_overlay_autoplay(
                &initial_overlay_path,
                &OverlayMetadata {
                    screen_type: None,
                    scenario: String::new(),
                    in_combat: false,
                    state_hash: String::new(),
                    floor: None,
                    character: None,
                },
                &AutoPlayState {
                    mode: initial_mode.into(),
                    status: "startup".into(),
                },
            );
        }
        if config.auto_play_auto_start {
            let _ = std::fs::remove_file(
                crate::logging::advice_output_dir()
                    .join("output")
                    .join("autoplay-control.json"),
            );
        }

        Self {
            journal: Journal::new(project_root.join("runs")),
            project_root,
            config,
            provider,
            locale,
            cache: AdviceCache::new(),
            combat_turn_gate: CombatTurnGate::new(),
            map_gate: MapGate::new(),
            autoplay_last_revision: None,
            current_autoplay_control,
            autoplay_session: AutoPlaySession::default(),
            autoplay_overlay_state: AutoPlayState::default(),
            last_autoplay_state: None,
            consecutive_errors: 0,
            saw_game_state: false,
            run_finalized: false,
        }
    }

    pub(crate) async fn run(mut self) {
        let stdin = tokio::io::stdin();
        let mut lines = tokio::io::BufReader::new(stdin).lines();

        loop {
            let line = match lines.next_line().await {
                Ok(Some(line)) => line,
                Ok(None) => break,
                Err(error) => {
                    tracing::error!("failed to read stdin: {error}");
                    continue;
                }
            };
            let Some(raw) = input::parse_input_line(&line) else {
                continue;
            };

            let overlay_path = crate::logging::advice_output_dir()
                .join("output")
                .join("overlay.json");
            let autoplay_control_path = crate::logging::advice_output_dir()
                .join("output")
                .join("autoplay-control.json");

            if crate::runtime::is_error(&raw) {
                tracing::warn!("received error from CommunicationMod: {}", line.trim());
                self.handle_error(&overlay_path, &autoplay_control_path)
                    .await;
                continue;
            }

            if !crate::runtime::is_in_game(&raw) {
                if crate::runtime::should_end_run(&raw, self.saw_game_state) {
                    let reason = crate::runtime::run_end_reason(&raw, self.saw_game_state)
                        .unwrap_or("left_game");
                    super::finalization::finalize_run_once(
                        &self.journal,
                        &self.provider,
                        reason,
                        &mut self.run_finalized,
                        &self.locale,
                    )
                    .await;
                    if self.config.auto_play {
                        self.write_stopped_overlay(&overlay_path);
                    }
                    break;
                }
                tracing::debug!("skipping non-game state");
                continue;
            }

            self.process_game_state(raw, &overlay_path, &autoplay_control_path)
                .await;
        }

        super::finalization::finalize_run_once(
            &self.journal,
            &self.provider,
            "stdin_closed",
            &mut self.run_finalized,
            &self.locale,
        )
        .await;
        tracing::info!("stdin closed, exiting");
    }

    pub(crate) fn reset_after_game(&mut self) {
        self.journal = Journal::new(self.project_root.join("runs"));
        self.cache = AdviceCache::new();
        self.combat_turn_gate = CombatTurnGate::new();
        self.map_gate = MapGate::new();
        self.saw_game_state = false;
        self.run_finalized = false;
        self.last_autoplay_state = None;
        self.autoplay_overlay_state = AutoPlayState::default();
        self.consecutive_errors = 0;
    }

    fn write_stopped_overlay(&self, overlay_path: &Path) {
        crate::autoplay::status::write_overlay_autoplay(
            overlay_path,
            &OverlayMetadata {
                screen_type: None,
                scenario: String::new(),
                in_combat: false,
                state_hash: String::new(),
                floor: None,
                character: None,
            },
            &AutoPlayState {
                mode: "off".into(),
                status: "stopped".into(),
            },
        );
    }
}
