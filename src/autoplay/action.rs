use std::io::Write;

use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::{AutoPlayControl, AutoPlayMode, ControlLoad};
use crate::protocol;
use crate::state::NormalizedState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoPlayAction {
    Choose(usize),
    Skip,
    Proceed,
}

pub fn active_control(load: &ControlLoad) -> Option<&AutoPlayControl> {
    match load {
        ControlLoad::Updated(control) | ControlLoad::MissingDefault(control) => Some(control),
        ControlLoad::Stale | ControlLoad::Malformed(_) => None,
    }
}

pub fn resolve_action(
    control: &AutoPlayControl,
    command_state: &CommandState,
    state: &NormalizedState,
) -> Option<AutoPlayAction> {
    if control.mode != AutoPlayMode::Auto
        || control.require_confirmation
        || !command_state.ready_for_command
    {
        return None;
    }

    match state.screen_type.as_deref() {
        Some("COMBAT_REWARD") if control.allow_combat_rewards => {
            resolve_combat_reward_action(control, command_state)
        }
        Some("CARD_REWARD") if control.allow_card_rewards => {
            resolve_card_reward_action(command_state, state)
        }
        _ => None,
    }
}

pub fn execute_action_to(writer: &mut impl Write, action: &AutoPlayAction) {
    match action {
        AutoPlayAction::Choose(index) => protocol::send_choose_to(writer, *index),
        AutoPlayAction::Skip => protocol::send_skip_to(writer),
        AutoPlayAction::Proceed => protocol::send_proceed_to(writer),
    }
}

fn resolve_combat_reward_action(
    control: &AutoPlayControl,
    command_state: &CommandState,
) -> Option<AutoPlayAction> {
    if let Some(index) = first_collectible_reward_index(control, &command_state.choice_list) {
        if command_state.has_command("choose") {
            return Some(AutoPlayAction::Choose(index));
        }
        return None;
    }

    if command_state.has_command("proceed") {
        return Some(AutoPlayAction::Proceed);
    }

    None
}

fn first_collectible_reward_index(control: &AutoPlayControl, choices: &[String]) -> Option<usize> {
    choices.iter().position(|choice| {
        matches!(
            choice.as_str(),
            "gold" | "relic" | "potion" | "emerald_key" | "sapphire_key"
        ) || (choice == "card" && control.allow_card_rewards)
    })
}

fn resolve_card_reward_action(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Option<AutoPlayAction> {
    if state.skip_available && command_state.has_command("skip") {
        return Some(AutoPlayAction::Skip);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autoplay::control::{AutoPlayControl, AutoPlayMode, ControlLoad};
    use crate::locales::Locale;
    use serde_json::{Value, json};

    fn state(raw: Value) -> NormalizedState {
        NormalizedState::from_raw(&raw, &Locale::load("en"))
    }

    fn command_state(raw: &Value) -> CommandState {
        CommandState::from_raw(raw)
    }

    #[test]
    fn missing_default_control_is_active_for_testing() {
        let load = ControlLoad::MissingDefault(AutoPlayControl::default_enabled());

        assert_eq!(
            active_control(&load),
            Some(&AutoPlayControl::default_enabled())
        );
    }

    #[test]
    fn malformed_control_is_not_active() {
        let load = ControlLoad::Malformed("bad json".to_string());

        assert_eq!(active_control(&load), None);
    }

    #[test]
    fn paused_control_does_not_resolve_action() {
        let raw = json!({
            "available_commands": ["choose", "proceed"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "COMBAT_REWARD",
                "choice_list": ["gold"]
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.mode = AutoPlayMode::Paused;

        assert_eq!(
            resolve_action(&control, &command_state(&raw), &state(raw.clone())),
            None
        );
    }

    #[test]
    fn not_ready_does_not_resolve_action() {
        let raw = json!({
            "available_commands": ["choose", "proceed"],
            "ready_for_command": false,
            "game_state": {
                "screen_type": "COMBAT_REWARD",
                "choice_list": ["gold"]
            }
        });

        assert_eq!(
            resolve_action(
                &AutoPlayControl::default_enabled(),
                &command_state(&raw),
                &state(raw.clone())
            ),
            None
        );
    }

    #[test]
    fn combat_reward_collects_first_known_reward() {
        let raw = json!({
            "available_commands": ["choose", "proceed"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "COMBAT_REWARD",
                "choice_list": ["gold", "card"]
            }
        });

        assert_eq!(
            resolve_action(
                &AutoPlayControl::default_enabled(),
                &command_state(&raw),
                &state(raw.clone())
            ),
            Some(AutoPlayAction::Choose(0))
        );
    }

    #[test]
    fn combat_reward_opens_card_reward_when_allowed() {
        let raw = json!({
            "available_commands": ["choose", "proceed"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "COMBAT_REWARD",
                "choice_list": ["card"]
            }
        });

        assert_eq!(
            resolve_action(
                &AutoPlayControl::default_enabled(),
                &command_state(&raw),
                &state(raw.clone())
            ),
            Some(AutoPlayAction::Choose(0))
        );
    }

    #[test]
    fn combat_reward_proceeds_after_known_rewards_are_gone() {
        let raw = json!({
            "available_commands": ["proceed"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "COMBAT_REWARD",
                "choice_list": []
            }
        });

        assert_eq!(
            resolve_action(
                &AutoPlayControl::default_enabled(),
                &command_state(&raw),
                &state(raw.clone())
            ),
            Some(AutoPlayAction::Proceed)
        );
    }

    #[test]
    fn card_reward_skips_when_skip_is_available() {
        let raw = json!({
            "available_commands": ["choose", "skip"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "CARD_REWARD",
                "screen_state": {
                    "skip_available": true,
                    "cards": [{"id": "Anger", "name": "Anger"}]
                }
            }
        });

        assert_eq!(
            resolve_action(
                &AutoPlayControl::default_enabled(),
                &command_state(&raw),
                &state(raw.clone())
            ),
            Some(AutoPlayAction::Skip)
        );
    }

    #[test]
    fn execute_action_writes_protocol_command() {
        let mut buf = Vec::new();

        execute_action_to(&mut buf, &AutoPlayAction::Choose(2));
        execute_action_to(&mut buf, &AutoPlayAction::Skip);
        execute_action_to(&mut buf, &AutoPlayAction::Proceed);

        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(lines, vec!["choose 2", "skip", "proceed"]);
    }
}
