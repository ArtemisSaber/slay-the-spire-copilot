mod cards;
mod combat_prompt;
mod dispatch;
mod inventory;
mod map_helpers;
mod map_prompt;
mod monsters;
mod rewards;
mod selection;
mod shop_event;
mod status;

pub(crate) const MAP_CANDIDATE_LIMIT: usize = 5;

pub(crate) use cards::clean_description;
pub use dispatch::build_prompt;
pub(crate) use map_helpers::position_label;

#[cfg(test)]
#[allow(
    unused_imports,
    reason = "Re-exported only to preserve the existing prompt test namespace."
)]
pub(crate) use cards::{compact_pile, format_card, format_deck_section};
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "Re-exported only to preserve the existing prompt test namespace."
)]
pub(crate) use combat_prompt::build_combat;
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "Re-exported only to preserve the existing prompt test namespace."
)]
pub(crate) use inventory::build_relics_potions_section;
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "Re-exported only to preserve the existing prompt test namespace."
)]
pub(crate) use map_helpers::{
    LabeledPath, compact_route_chain, format_candidate_label, format_evaluation_line,
    from_left_label, ordinal, rank_labeled_paths, recommendation_features, recommendation_label,
};
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "Re-exported only to preserve the existing prompt test namespace."
)]
pub(crate) use map_prompt::{build_map_crossroad, build_map_suggestion};
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "Re-exported only to preserve the existing prompt test namespace."
)]
pub(crate) use monsters::{build_hand_section, build_monsters_section, format_monster};
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "Re-exported only to preserve the existing prompt test namespace."
)]
pub(crate) use rewards::{build_boss_relic, build_card_reward, build_rest};
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "Re-exported only to preserve the existing prompt test namespace."
)]
pub(crate) use selection::{build_grid_select, build_hand_select};
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "Re-exported only to preserve the existing prompt test namespace."
)]
pub(crate) use shop_event::{build_event_choice, build_generic, build_shop};
#[cfg(test)]
#[allow(
    unused_imports,
    reason = "Re-exported only to preserve the existing prompt test namespace."
)]
pub(crate) use status::{combat_profile_line, danger_prefix, status_line, turn_status_line};
