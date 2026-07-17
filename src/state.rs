mod combat;
mod danger;
mod event;
mod model;
mod normalization;
mod normalized;
mod parse;
mod screen_data;
mod screens;
mod stable;

pub use danger::{DangerFlags, DangerLevel};
pub use model::{CardInfo, MapCoord, MonsterInfo, OrbInfo, PotionInfo, PowerInfo, RelicInfo};
pub use normalized::NormalizedState;
pub use screens::{RoomType, ScreenType};

#[cfg(test)]
use stable::hash_bytes;

#[cfg(test)]
#[path = "tests/state_tests/mod.rs"]
mod tests;
