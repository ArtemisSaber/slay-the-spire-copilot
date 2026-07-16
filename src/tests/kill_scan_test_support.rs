use std::collections::HashMap;
use std::time::Instant;

use crate::combat::{KillPlay, MonsterSnapshot, Stance};
use crate::state::CardInfo;

use super::{DfsContext, dfs};

const KNOWN_MONSTER_POWERS: &[&str] = &[
    "Artifact",
    "人工制品",
    "Curl Up",
    "Flight",
    "Intangible",
    "Invincible",
    "Malleable",
    "Minion",
    "爪牙",
    "Slow",
    "缓慢",
    "Vulnerable",
    "易伤",
    "Time Warp",
    "Strength",
    "力量",
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
    "Shifting",
    "变化",
    "Plated Armor",
    "多层护甲",
    "Barricade",
    "壁垒",
    "Anger",
    "Ritual",
    "Regeneration",
    "Metallicize",
];

fn has_dangerous_unknown_powers(monsters: &[MonsterSnapshot]) -> bool {
    for m in monsters {
        for p in &m.powers {
            if !KNOWN_MONSTER_POWERS.contains(&p.id.as_str()) {
                return true;
            }
        }
    }
    false
}

pub(crate) struct TestCard {
    pub uuid: &'static str,
    pub name: &'static str,
    pub cost: i16,
    pub card_type: &'static str,
    pub damage: Option<(
        i16,
        crate::combat::effects::HitCount,
        crate::combat::effects::TargetType,
    )>,
    pub vulnerable: Option<i16>,
    pub strength_gain: i16,
    pub energy_gain: i16,
    pub stance: crate::combat::effects::StanceEffect,
    pub mantra_gain: i16,
    pub execute_threshold: Option<i16>,
    pub x_cost: bool,
}

impl TestCard {
    fn to_effect(&self) -> crate::combat::effects::CardEffect {
        use crate::combat::effects::{CardEffect, DamageEffect, ExhaustKind};
        CardEffect {
            damage: self.damage.as_ref().map(|(amount, hits, tt)| DamageEffect {
                amount: *amount,
                hits: hits.clone(),
                target_type: tt.clone(),
            }),
            energy_gain: self.energy_gain,
            strength_gain: self.strength_gain,
            vulnerable: self.vulnerable,
            stance: self.stance.clone(),
            mantra_gain: self.mantra_gain,
            execute: self.execute_threshold,
            exhaust: ExhaustKind::None,
            x_cost: self.x_cost,
        }
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "test_scan takes all scan parameters explicitly"
)]
pub(crate) fn test_scan(
    hand_cards: &[TestCard],
    energy: i16,
    monsters: &[MonsterSnapshot],
    stance: Stance,
    strength_delta: i16,
    x_cost_bonus: i16,
    remaining_plays: usize,
    max_states: usize,
) -> Option<Vec<KillPlay>> {
    if has_dangerous_unknown_powers(monsters) {
        return None;
    }

    let effects: Vec<Option<crate::combat::effects::CardEffect>> =
        hand_cards.iter().map(|tc| Some(tc.to_effect())).collect();

    let cards: Vec<CardInfo> = hand_cards
        .iter()
        .map(|tc| CardInfo {
            id: String::new(),
            name: tc.name.into(),
            cost: tc.cost as i64,
            card_type: tc.card_type.into(),
            upgraded: false,
            uuid: Some(tc.uuid.into()),
            description: String::new(),
            price: None,
            playable: true,
            has_target: tc
                .damage
                .as_ref()
                .map(|(_, _, tt)| matches!(tt, crate::combat::effects::TargetType::Targeted))
                .unwrap_or(false)
                || tc.vulnerable.is_some()
                || tc.execute_threshold.is_some(),
        })
        .collect();

    let initial_mask: u32 = (0..hand_cards.len()).fold(0, |m, i| m | (1u32 << i));
    let max_depth = hand_cards.len().min(remaining_plays);

    let mut dfs_ctx = DfsContext {
        effects: &effects,
        cards: &cards,
        max_depth,
        max_states,
        memo: HashMap::new(),
        expanded: 0,
        deadline: Instant::now() + std::time::Duration::from_secs(30),
        x_cost_bonus,
        initial_stance: stance,
    };

    let result = dfs(
        initial_mask,
        energy,
        0,
        stance,
        strength_delta,
        0, // initial mantra
        monsters,
        &mut dfs_ctx,
    );

    result.map(|steps| {
        steps
            .into_iter()
            .map(|s| KillPlay {
                card: cards[s.card_index].uuid.clone().unwrap_or_default(),
                target: s.target,
            })
            .collect()
    })
}
