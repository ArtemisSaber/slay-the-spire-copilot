use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandState {
    pub ready_for_command: bool,
    pub available_commands: Vec<String>,
    pub choice_list: Vec<String>,
}

impl CommandState {
    pub fn from_raw(raw: &Value) -> Self {
        let available_commands = raw
            .get("available_commands")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();

        let choice_list = raw
            .pointer("/game_state/choice_list")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();

        CommandState {
            ready_for_command: raw
                .get("ready_for_command")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            available_commands,
            choice_list,
        }
    }

    pub fn has_command(&self, command: &str) -> bool {
        self.available_commands.iter().any(|c| c == command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::load_fixture;
    use serde_json::json;

    #[test]
    fn extracts_command_state_from_fixture() {
        let raw = load_fixture("combat-state.json");
        let command_state = CommandState::from_raw(&raw);

        assert!(command_state.ready_for_command);
        assert!(command_state.has_command("play"));
        assert!(command_state.has_command("end"));
        assert!(!command_state.has_command("choose"));
    }

    #[test]
    fn extracts_choice_list_for_reward_like_screens() {
        let raw = json!({
            "available_commands": ["choose", "proceed", "wait"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "COMBAT_REWARD",
                "choice_list": ["gold", "card"]
            }
        });
        let command_state = CommandState::from_raw(&raw);

        assert_eq!(command_state.choice_list, vec!["gold", "card"]);
        assert!(command_state.has_command("choose"));
        assert!(command_state.has_command("proceed"));
    }

    #[test]
    fn missing_fields_fail_closed() {
        let command_state = CommandState::from_raw(&json!({}));

        assert!(!command_state.ready_for_command);
        assert!(command_state.available_commands.is_empty());
        assert!(command_state.choice_list.is_empty());
    }
}
