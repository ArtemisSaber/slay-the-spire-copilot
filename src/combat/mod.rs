pub mod context;
pub(crate) mod damage;
pub(crate) mod effects;
pub(crate) mod kill_scan;

use crate::state::NormalizedState;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stance {
    Neutral,
    Calm,
    Wrath,
    Divinity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerState {
    pub id: String,
    pub amount: i16,
    pub triggered: bool,
}

#[derive(Debug, Clone)]
pub struct MonsterSnapshot {
    pub command_index: usize,
    pub hp: i16,
    pub block: i16,
    pub powers: Vec<PowerState>,
    pub is_minion: bool,
}

#[derive(Debug, Clone)]
pub struct CombatScanContext {
    pub cards: Vec<crate::state::CardInfo>,
    pub energy: i16,
    pub initial_stance: Stance,
    pub current_stance: Stance,
    pub strength_delta: i16,
    pub x_cost_bonus: i16,
    pub monsters: Vec<MonsterSnapshot>,
    pub remaining_card_plays: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KillPlay {
    pub card: String,
    pub target: Option<usize>,
}

pub type KillSequence = Vec<KillPlay>;

#[derive(Debug, Clone)]
pub struct KillScanOptions {
    pub deadline: Duration,
    #[allow(dead_code, reason = "planned: parallel kill-scan workers")]
    pub worker_threads: usize,
    pub max_expanded_states: usize,
    #[allow(dead_code, reason = "planned: memo-entry cap per worker")]
    pub max_memo_entries: usize,
}

impl Default for KillScanOptions {
    fn default() -> Self {
        KillScanOptions {
            deadline: Duration::from_secs(2),
            worker_threads: std::thread::available_parallelism()
                .map(|n| n.get().min(8))
                .unwrap_or(1),
            max_expanded_states: 2_000_000,
            max_memo_entries: 1_000_000,
        }
    }
}

pub fn find_kill_sequence(state: &NormalizedState) -> Option<KillSequence> {
    kill_scan::find_kill_sequence_inner(state, &KillScanOptions::default())
}

#[cfg(test)]
pub fn can_end_fight(state: &NormalizedState) -> bool {
    find_kill_sequence(state).is_some()
}
