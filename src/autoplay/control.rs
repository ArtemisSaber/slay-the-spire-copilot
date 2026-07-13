use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoPlayMode {
    Off,
    Advise,
    Auto,
    Paused,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AutoPlayControl {
    pub schema_version: u32,
    pub revision: u64,
    pub mode: AutoPlayMode,
    #[serde(default)]
    pub updated_at_ms: u128,
    #[serde(default)]
    pub require_confirmation: bool,
    #[serde(default = "default_true")]
    pub allow_card_rewards: bool,
    #[serde(default = "default_true")]
    pub allow_combat_rewards: bool,
    #[serde(default = "default_true")]
    pub allow_boss_rewards: bool,
    #[serde(default = "default_true")]
    pub allow_rest: bool,
    #[serde(default = "default_true")]
    pub allow_events: bool,
    #[serde(default = "default_true")]
    pub allow_map: bool,
    #[serde(default = "default_true")]
    pub allow_shop: bool,
    #[serde(default = "default_true")]
    pub allow_combat: bool,
    #[serde(default = "default_true")]
    pub allow_selection_screens: bool,
    pub min_hp_percent: Option<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct AutoPlaySession {
    pub last_shop_room_floor: Option<i64>,
    pub last_combat_reward_floor: Option<i64>,
    pub skipped_combat_reward_potion: bool,
    pub skipped_combat_reward_card: bool,
    pub last_seen_relic_ids: Vec<String>,
    pub pending_boss_relic_grid: Option<String>,
}

impl AutoPlayControl {
    pub fn default_enabled() -> Self {
        AutoPlayControl {
            schema_version: 1,
            revision: 0,
            mode: AutoPlayMode::Auto,
            updated_at_ms: 0,
            require_confirmation: false,
            allow_card_rewards: true,
            allow_combat_rewards: true,
            allow_boss_rewards: true,
            allow_rest: true,
            allow_events: true,
            allow_map: true,
            allow_shop: true,
            allow_combat: true,
            allow_selection_screens: true,
            min_hp_percent: None,
        }
    }

    pub fn default_paused() -> Self {
        AutoPlayControl {
            mode: AutoPlayMode::Paused,
            ..Self::default_enabled()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlLoad {
    Updated(AutoPlayControl),
    MissingDefault(AutoPlayControl),
    Stale,
    Malformed(String),
}

pub fn load_control(path: &Path, last_seen_revision: Option<u64>) -> ControlLoad {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return ControlLoad::MissingDefault(AutoPlayControl::default_enabled());
        }
        Err(e) => return ControlLoad::Malformed(e.to_string()),
    };

    let control: AutoPlayControl = match serde_json::from_str(&content) {
        Ok(control) => control,
        Err(e) => return ControlLoad::Malformed(e.to_string()),
    };

    if control.schema_version != 1 {
        return ControlLoad::Malformed(format!(
            "unsupported schema_version {}",
            control.schema_version
        ));
    }

    if last_seen_revision.is_some_and(|seen| control.revision == seen) {
        return ControlLoad::Stale;
    }

    ControlLoad::Updated(control)
}

#[cfg(test)]
#[path = "tests/control_tests.rs"]
mod tests;
