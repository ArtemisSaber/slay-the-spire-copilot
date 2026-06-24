use std::io::Write;

use crate::autoplay::command_state::CommandState;
use crate::autoplay::control::{AutoPlayControl, AutoPlayMode, ControlLoad};
use crate::protocol;
use crate::state::NormalizedState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoPlayAction {
    Choose(usize),
    Play {
        hand_index: usize,
        target_index: Option<usize>,
    },
    End,
    Skip,
    Proceed,
    Leave,
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
        Some("BOSS_REWARD") if control.allow_boss_rewards => {
            resolve_boss_reward_action(command_state, state)
        }
        Some("REST") if control.allow_rest => resolve_rest_action(command_state, state),
        Some("EVENT") if control.allow_events => resolve_event_action(command_state, state),
        Some("SHOP_SCREEN") if control.allow_shop => resolve_shop_action(command_state),
        Some("MAP") if control.allow_map => resolve_map_action(command_state),
        Some("NONE") if control.allow_combat => resolve_combat_action(command_state, state),
        _ => None,
    }
}

pub fn execute_action_to(writer: &mut impl Write, action: &AutoPlayAction) {
    match action {
        AutoPlayAction::Choose(index) => protocol::send_choose_to(writer, *index),
        AutoPlayAction::Play {
            hand_index,
            target_index,
        } => protocol::send_play_to(writer, *hand_index, *target_index),
        AutoPlayAction::End => protocol::send_end_to(writer),
        AutoPlayAction::Skip => protocol::send_skip_to(writer),
        AutoPlayAction::Proceed => protocol::send_proceed_to(writer),
        AutoPlayAction::Leave => protocol::send_leave_to(writer),
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

fn resolve_boss_reward_action(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Option<AutoPlayAction> {
    if !state.boss_relic_choices.is_empty() && command_state.has_command("choose") {
        return Some(AutoPlayAction::Choose(0));
    }

    None
}

fn resolve_rest_action(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Option<AutoPlayAction> {
    if !command_state.has_command("choose") {
        return None;
    }

    let rest_index = state
        .rest_options
        .iter()
        .position(|option| option == "rest");
    let smith_index = state
        .rest_options
        .iter()
        .position(|option| option == "smith");

    let hp_is_low = match (state.current_hp, state.max_hp) {
        (Some(current), Some(max)) if max > 0 => current * 2 < max,
        _ => false,
    };

    if hp_is_low {
        rest_index.or(smith_index).map(AutoPlayAction::Choose)
    } else {
        smith_index.or(rest_index).map(AutoPlayAction::Choose)
    }
}

fn resolve_event_action(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Option<AutoPlayAction> {
    if command_state.has_command("choose") && state.event_choices.len() == 1 {
        return Some(AutoPlayAction::Choose(0));
    }

    None
}

fn resolve_shop_action(command_state: &CommandState) -> Option<AutoPlayAction> {
    if command_state.has_command("leave") {
        return Some(AutoPlayAction::Leave);
    }

    None
}

fn resolve_map_action(command_state: &CommandState) -> Option<AutoPlayAction> {
    if command_state.has_command("choose") && command_state.choice_list.len() == 1 {
        return Some(AutoPlayAction::Choose(0));
    }

    None
}

fn resolve_combat_action(
    command_state: &CommandState,
    state: &NormalizedState,
) -> Option<AutoPlayAction> {
    let energy = state.energy.unwrap_or(0);

    if command_state.has_command("play")
        && let Some((hand_index, target_index)) = first_playable_combat_card(state, energy)
    {
        return Some(AutoPlayAction::Play {
            hand_index,
            target_index,
        });
    }

    if command_state.has_command("end") {
        return Some(AutoPlayAction::End);
    }

    None
}

fn first_playable_combat_card(
    state: &NormalizedState,
    energy: i64,
) -> Option<(usize, Option<usize>)> {
    let first_target = state.monsters.first().map(|monster| monster.index);

    state
        .hand
        .iter()
        .enumerate()
        .find_map(|(hand_index, card)| {
            if card.cost > energy || card.card_type == "STATUS" || card.card_type == "CURSE" {
                return None;
            }

            let target_index = if card.card_type == "ATTACK" {
                first_target
            } else {
                None
            };

            if card.card_type == "ATTACK" && target_index.is_none() {
                return None;
            }

            Some((hand_index, target_index))
        })
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
    fn boss_reward_chooses_first_relic() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "BOSS_REWARD",
                "screen_state": {
                    "relics": [
                        {"id": "Coffee Dripper", "name": "Coffee Dripper"},
                        {"id": "Astrolabe", "name": "Astrolabe"}
                    ]
                }
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
    fn rest_chooses_rest_when_hp_is_low() {
        let raw = json!({
            "available_commands": ["choose", "return"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "REST",
                "current_hp": 25,
                "max_hp": 75,
                "screen_state": {
                    "rest_options": ["rest", "smith", "toke"]
                }
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
    fn rest_chooses_smith_when_hp_is_not_low() {
        let raw = json!({
            "available_commands": ["choose", "return"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "REST",
                "current_hp": 60,
                "max_hp": 75,
                "screen_state": {
                    "rest_options": ["rest", "smith", "toke"]
                }
            }
        });

        assert_eq!(
            resolve_action(
                &AutoPlayControl::default_enabled(),
                &command_state(&raw),
                &state(raw.clone())
            ),
            Some(AutoPlayAction::Choose(1))
        );
    }

    #[test]
    fn single_choice_event_chooses_the_only_option() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "EVENT",
                "choice_list": ["Continue"]
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
    fn multiple_choice_event_blocks_for_advice() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "EVENT",
                "choice_list": ["Fight", "Leave"]
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
    fn shop_leaves_without_buying_by_default() {
        let raw = json!({
            "available_commands": ["choose", "leave"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "SHOP_SCREEN",
                "choice_list": ["purge", "Strike"]
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.allow_shop = true;

        assert_eq!(
            resolve_action(&control, &command_state(&raw), &state(raw.clone())),
            Some(AutoPlayAction::Leave)
        );
    }

    #[test]
    fn map_chooses_single_available_choice_when_allowed() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "MAP",
                "choice_list": ["M"]
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.allow_map = true;

        assert_eq!(
            resolve_action(&control, &command_state(&raw), &state(raw.clone())),
            Some(AutoPlayAction::Choose(0))
        );
    }

    #[test]
    fn map_blocks_when_more_than_one_choice_is_available() {
        let raw = json!({
            "available_commands": ["choose"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "MAP",
                "choice_list": ["M", "?"]
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.allow_map = true;

        assert_eq!(
            resolve_action(&control, &command_state(&raw), &state(raw.clone())),
            None
        );
    }

    #[test]
    fn combat_plays_first_affordable_attack_at_first_monster_when_allowed() {
        let raw = json!({
            "available_commands": ["play", "end"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "NONE",
                "combat_state": {
                    "player": {"energy": 3, "block": 0, "powers": []},
                    "hand": [
                        {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK", "uuid": "strike-1"},
                        {"id": "Defend_R", "name": "Defend", "cost": 1, "type": "SKILL", "uuid": "defend-1"}
                    ],
                    "monsters": [
                        {"name": "Jaw Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                    ]
                }
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.allow_combat = true;

        assert_eq!(
            resolve_action(&control, &command_state(&raw), &state(raw.clone())),
            Some(AutoPlayAction::Play {
                hand_index: 0,
                target_index: Some(0),
            })
        );
    }

    #[test]
    fn combat_skips_unaffordable_cards() {
        let raw = json!({
            "available_commands": ["play", "end"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "NONE",
                "combat_state": {
                    "player": {"energy": 1, "block": 0, "powers": []},
                    "hand": [
                        {"id": "Bash", "name": "Bash", "cost": 2, "type": "ATTACK", "uuid": "bash-1"},
                        {"id": "Defend_R", "name": "Defend", "cost": 1, "type": "SKILL", "uuid": "defend-1"}
                    ],
                    "monsters": [
                        {"name": "Jaw Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                    ]
                }
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.allow_combat = true;

        assert_eq!(
            resolve_action(&control, &command_state(&raw), &state(raw.clone())),
            Some(AutoPlayAction::Play {
                hand_index: 1,
                target_index: None,
            })
        );
    }

    #[test]
    fn combat_ends_when_no_card_is_playable() {
        let raw = json!({
            "available_commands": ["play", "end"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "NONE",
                "combat_state": {
                    "player": {"energy": 0, "block": 0, "powers": []},
                    "hand": [
                        {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK", "uuid": "strike-1"}
                    ],
                    "monsters": [
                        {"name": "Jaw Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                    ]
                }
            }
        });
        let mut control = AutoPlayControl::default_enabled();
        control.allow_combat = true;

        assert_eq!(
            resolve_action(&control, &command_state(&raw), &state(raw.clone())),
            Some(AutoPlayAction::End)
        );
    }

    #[test]
    fn combat_is_disabled_by_default_until_explicitly_allowed() {
        let raw = json!({
            "available_commands": ["play", "end"],
            "ready_for_command": true,
            "game_state": {
                "screen_type": "NONE",
                "combat_state": {
                    "player": {"energy": 3, "block": 0, "powers": []},
                    "hand": [
                        {"id": "Strike_R", "name": "Strike", "cost": 1, "type": "ATTACK", "uuid": "strike-1"}
                    ],
                    "monsters": [
                        {"name": "Jaw Worm", "current_hp": 44, "max_hp": 46, "block": 0, "intent": "ATTACK", "is_gone": false}
                    ]
                }
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
    fn execute_action_writes_protocol_command() {
        let mut buf = Vec::new();

        execute_action_to(&mut buf, &AutoPlayAction::Choose(2));
        execute_action_to(
            &mut buf,
            &AutoPlayAction::Play {
                hand_index: 1,
                target_index: Some(0),
            },
        );
        execute_action_to(&mut buf, &AutoPlayAction::End);
        execute_action_to(&mut buf, &AutoPlayAction::Skip);
        execute_action_to(&mut buf, &AutoPlayAction::Proceed);
        execute_action_to(&mut buf, &AutoPlayAction::Leave);

        let output = String::from_utf8(buf).unwrap();
        let lines: Vec<&str> = output.lines().collect();
        assert_eq!(
            lines,
            vec!["choose 2", "play 1 0", "end", "skip", "proceed", "leave"]
        );
    }
}
