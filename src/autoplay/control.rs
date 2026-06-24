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

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AutoPlayControl {
    pub schema_version: u32,
    pub revision: u64,
    pub mode: AutoPlayMode,
    pub updated_at_ms: u128,
    pub require_confirmation: bool,
    pub allow_card_rewards: bool,
    pub allow_combat_rewards: bool,
    pub allow_boss_rewards: bool,
    pub allow_rest: bool,
    pub allow_events: bool,
    pub allow_map: bool,
    pub allow_shop: bool,
    pub allow_combat: bool,
    pub allow_selection_screens: bool,
    pub min_hp_percent: Option<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct AutoPlaySession {
    pub last_shop_room_floor: Option<i64>,
    pub last_combat_reward_floor: Option<i64>,
    pub skipped_combat_reward_potion: bool,
    pub skipped_combat_reward_card: bool,
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

    if last_seen_revision.is_some_and(|seen| control.revision <= seen) {
        return ControlLoad::Stale;
    }

    ControlLoad::Updated(control)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn missing_control_defaults_to_auto_for_testing() {
        let dir = tempfile::tempdir().unwrap();
        let loaded = load_control(&dir.path().join("autoplay-control.json"), None);

        assert_eq!(
            loaded,
            ControlLoad::MissingDefault(AutoPlayControl::default_enabled())
        );
    }

    #[test]
    fn valid_control_loads_when_revision_is_new() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("autoplay-control.json");
        fs::write(
            &path,
            r#"{
                "schema_version": 1,
                "revision": 3,
                "mode": "paused",
                "updated_at_ms": 1000,
                "require_confirmation": true,
                "allow_card_rewards": true,
                "allow_combat_rewards": true,
                "allow_boss_rewards": true,
                "allow_rest": true,
                "allow_events": false,
                "allow_map": false,
                "allow_shop": false,
                "allow_combat": false,
                "allow_selection_screens": false,
                "min_hp_percent": 20
            }"#,
        )
        .unwrap();

        let loaded = load_control(&path, Some(2));
        let ControlLoad::Updated(control) = loaded else {
            panic!("expected updated control");
        };

        assert_eq!(control.revision, 3);
        assert_eq!(control.mode, AutoPlayMode::Paused);
        assert!(control.require_confirmation);
        assert!(!control.allow_events);
        assert_eq!(control.min_hp_percent, Some(20));
    }

    #[test]
    fn stale_control_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("autoplay-control.json");
        fs::write(
            &path,
            r#"{
                "schema_version": 1,
                "revision": 3,
                "mode": "auto",
                "updated_at_ms": 1000,
                "require_confirmation": false,
                "allow_card_rewards": true,
                "allow_combat_rewards": true,
                "allow_boss_rewards": true,
                "allow_rest": true,
                "allow_events": true,
                "allow_map": false,
                "allow_shop": false,
                "allow_combat": false,
                "allow_selection_screens": false,
                "min_hp_percent": null
            }"#,
        )
        .unwrap();

        assert_eq!(load_control(&path, Some(3)), ControlLoad::Stale);
    }

    #[test]
    fn malformed_control_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("autoplay-control.json");
        fs::write(&path, "{not-json").unwrap();

        assert!(matches!(
            load_control(&path, None),
            ControlLoad::Malformed(_)
        ));
    }
}
