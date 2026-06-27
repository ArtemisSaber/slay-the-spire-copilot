#![allow(
    dead_code,
    reason = "WIP: context builder functions will be used in subsequent phases"
)]

use crate::state::{CardInfo, MonsterInfo, NormalizedState, PowerInfo};

use super::{CombatScanContext, MonsterSnapshot, PowerState, Stance};

fn try_i16(value: i64) -> Option<i16> {
    i16::try_from(value).ok()
}

fn detect_stance(powers: &[PowerInfo]) -> Option<Stance> {
    for p in powers {
        match p.name.as_str() {
            "Wrath" | "愤怒" if p.amount > 0 => return Some(Stance::Wrath),
            "Calm" | "宁静" if p.amount > 0 => return Some(Stance::Calm),
            "Divinity" | "神格" if p.amount > 0 => return Some(Stance::Divinity),
            _ => {}
        }
    }
    None
}

const DANGEROUS_MONSTER_POWERS: &[&str] = &["Shifting", "变化"];

const SAFE_IGNORE_MONSTER_POWERS: &[&str] = &[
    "Fading",
    "消逝",
    "Life Link",
    "生命链接",
    "Shackled",
    "镣铐",
    "Weakened",
    "虚弱",
    "Generic Strength Up Power",
    "强化",
    "Anger",
    "Ritual",
    "Regeneration",
    "Metallicize",
];

const ONE_SHOT_PLAYER_POWERS: &[&str] = &["Vigor", "活力", "Wreath of Flame", "火焰纹"];

fn has_dangerous_monster_power(monsters: &[MonsterInfo]) -> bool {
    for m in monsters {
        for p in &m.monster_powers {
            if DANGEROUS_MONSTER_POWERS.contains(&p.name.as_str())
                || DANGEROUS_MONSTER_POWERS.contains(&p.id.as_str())
            {
                return true;
            }
        }
    }
    false
}

fn has_one_shot_player_power(powers: &[PowerInfo]) -> bool {
    powers.iter().any(|p| {
        p.amount > 0
            && (ONE_SHOT_PLAYER_POWERS.contains(&p.name.as_str())
                || ONE_SHOT_PLAYER_POWERS.contains(&p.id.as_str()))
    })
}

fn has_play_limiter_and_unknown_count(state: &NormalizedState) -> bool {
    let has_time_warp = state
        .monsters
        .iter()
        .flat_map(|m| &m.monster_powers)
        .any(|p| p.id == "Time Warp");
    let has_normality = state.hand.iter().any(|c| c.id == "Normality");
    let has_velvet_choker = state.relics.iter().any(|r| r.id == "Velvet Choker");

    has_time_warp || has_normality || has_velvet_choker
}

fn build_monster_snapshots(monsters: &[MonsterInfo]) -> Option<Vec<MonsterSnapshot>> {
    monsters
        .iter()
        .filter(|m| m.current_hp.map(|hp| hp > 0).unwrap_or(false))
        .map(|m| {
            let hp = try_i16(m.current_hp?)?;
            let block = try_i16(m.block.unwrap_or(0))?;
            let powers: Option<Vec<PowerState>> = m
                .monster_powers
                .iter()
                .map(|p| {
                    let amount = try_i16(p.amount)?;
                    Some(PowerState {
                        id: p.id.clone(),
                        amount,
                        triggered: false,
                    })
                })
                .collect();
            let is_minion = m
                .monster_powers
                .iter()
                .any(|p| p.id == "Minion" || p.id == "爪牙");
            Some(MonsterSnapshot {
                command_index: m.index,
                hp,
                block,
                powers: powers?,
                is_minion,
            })
        })
        .collect()
}

pub fn build_context(state: &NormalizedState) -> Option<CombatScanContext> {
    if state.hand.is_empty() {
        tracing::debug!("kill_scan: skipping — empty hand");
        return None;
    }
    if state.screen_type.as_deref() != Some("NONE") {
        tracing::debug!("kill_scan: skipping — screen_type={:?}", state.screen_type);
        return None;
    }

    if has_one_shot_player_power(&state.powers) {
        tracing::debug!("kill_scan: skipping — one-shot player power present");
        return None;
    }

    if has_dangerous_monster_power(&state.monsters) {
        tracing::debug!("kill_scan: skipping — dangerous monster power (Shifting/变化)");
        return None;
    }

    if has_play_limiter_and_unknown_count(state) {
        tracing::debug!("kill_scan: skipping — play limiter present");
        return None;
    }

    for m in &state.monsters {
        for p in &m.monster_powers {
            let known = DANGEROUS_MONSTER_POWERS.contains(&p.name.as_str())
                || DANGEROUS_MONSTER_POWERS.contains(&p.id.as_str())
                || SAFE_IGNORE_MONSTER_POWERS.contains(&p.name.as_str())
                || SAFE_IGNORE_MONSTER_POWERS.contains(&p.id.as_str())
                || p.id == "Artifact"
                || p.id == "人工制品"
                || p.id == "Curl Up"
                || p.id == "Flight"
                || p.id == "Intangible"
                || p.id == "Invincible"
                || p.id == "Malleable"
                || p.id == "Minion"
                || p.id == "爪牙"
                || p.id == "Slow"
                || p.id == "缓慢"
                || p.id == "Vulnerable"
                || p.id == "易伤"
                || p.id == "Time Warp"
                || p.id == "Strength"
                || p.id == "力量";
            if !known {
                tracing::debug!("kill_scan: skipping — unknown monster power \"{}\"", p.id);
                return None;
            }
        }
    }

    let energy = try_i16(state.energy.unwrap_or(0))?;
    let hp = try_i16(state.current_hp.unwrap_or(0))?;
    if hp <= 0 {
        return None;
    }

    let stance = detect_stance(&state.powers).unwrap_or(Stance::Neutral);

    let remaining_card_plays = state.hand.len();

    if remaining_card_plays == 0 {
        return None;
    }

    let monsters = build_monster_snapshots(&state.monsters)?;

    let cards: Vec<CardInfo> = state
        .hand
        .iter()
        .filter(|c| c.uuid.is_some() && c.playable)
        .cloned()
        .collect();
    if cards.is_empty() {
        tracing::debug!("kill_scan: skipping — no UUID cards after filter");
        return None;
    }

    tracing::debug!(
        "kill_scan: built context cards={} energy={} mons={} stance={:?}",
        cards.len(),
        energy,
        monsters.len(),
        stance,
    );
    Some(CombatScanContext {
        cards,
        energy,
        initial_stance: stance,
        current_stance: stance,
        strength_delta: 0,
        x_cost_bonus: 0,
        monsters,
        remaining_card_plays,
    })
}

#[cfg(test)]
#[path = "../tests/combat_tests.rs"]
mod tests;
