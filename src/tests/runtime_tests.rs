use crate::runtime::{
    has_monsters, is_error, is_game_over_state, run_end_reason, runtime_options_from,
    should_end_run, RuntimeOptions,
};
use serde_json::json;

#[path = "runtime_tests/monsters.rs"]
mod monsters;
#[path = "runtime_tests/options.rs"]
mod options;
#[path = "runtime_tests/status.rs"]
mod status;
