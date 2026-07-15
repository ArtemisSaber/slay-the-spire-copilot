use crate::learning::descriptor::{
    BlockThreatBucket, CountBucket, MonsterDescriptor, RatioBucket, SituationDescriptor, TurnBucket,
};
use std::collections::BTreeMap;

pub fn situation_similarity(left: &SituationDescriptor, right: &SituationDescriptor) -> u16 {
    let features = [
        (
            150,
            monster_similarity(&left.alive_monsters, &right.alive_monsters),
        ),
        (
            100,
            jaccard(
                left.alive_monsters.iter().map(|m| m.intent.clone()),
                right.alive_monsters.iter().map(|m| m.intent.clone()),
            ),
        ),
        (75, jaccard(monster_powers(left), monster_powers(right))),
        (200, jaccard(card_tokens(left), card_tokens(right))),
        (
            50,
            adjacent(
                count_index(left.energy_bucket),
                count_index(right.energy_bucket),
            ),
        ),
        (
            50,
            adjacent(turn_index(left.turn_bucket), turn_index(right.turn_bucket)),
        ),
        (
            75,
            adjacent(
                ratio_index(left.hp_ratio_bucket),
                ratio_index(right.hp_ratio_bucket),
            ),
        ),
        (
            75,
            adjacent(
                threat_index(left.block_threat_bucket),
                threat_index(right.block_threat_bucket),
            ),
        ),
        (50, equality(&left.stance, &right.stance)),
        (
            75,
            jaccard(
                left.player_power_ids.clone(),
                right.player_power_ids.clone(),
            ),
        ),
        (50, jaccard(left.relic_ids.clone(), right.relic_ids.clone())),
        (
            50,
            jaccard(left.ranker_tags.clone(), right.ranker_tags.clone()),
        ),
    ];
    features
        .into_iter()
        .map(|(weight, similarity)| (weight * similarity + 500) / 1_000)
        .sum::<u32>() as u16
}

fn monster_similarity(left: &[MonsterDescriptor], right: &[MonsterDescriptor]) -> u32 {
    let left = indexed_monsters(left);
    let right = indexed_monsters(right);
    let keys: std::collections::BTreeSet<_> = left.keys().chain(right.keys()).cloned().collect();
    if keys.is_empty() {
        return 1_000;
    }
    let total: u32 = keys
        .iter()
        .map(|key| match (left.get(key), right.get(key)) {
            (Some(left), Some(right)) => {
                (adjacent(
                    ratio_index(left.hp_ratio_bucket),
                    ratio_index(right.hp_ratio_bucket),
                ) + adjacent(
                    ratio_index(left.block_ratio_bucket),
                    ratio_index(right.block_ratio_bucket),
                ) + adjacent(
                    count_index(left.incoming_hits_bucket),
                    count_index(right.incoming_hits_bucket),
                )) / 3
            }
            _ => 0,
        })
        .sum();
    (total + keys.len() as u32 / 2) / keys.len() as u32
}

fn indexed_monsters(
    monsters: &[MonsterDescriptor],
) -> BTreeMap<(String, usize), &MonsterDescriptor> {
    let mut counts = BTreeMap::new();
    let mut indexed = BTreeMap::new();
    for monster in monsters {
        let ordinal = counts.entry(monster.monster_id.clone()).or_insert(0);
        indexed.insert((monster.monster_id.clone(), *ordinal), monster);
        *ordinal += 1;
    }
    indexed
}

fn monster_powers(situation: &SituationDescriptor) -> Vec<String> {
    situation
        .alive_monsters
        .iter()
        .flat_map(|monster| monster.power_ids.iter().cloned())
        .collect()
}

fn card_tokens(situation: &SituationDescriptor) -> Vec<String> {
    situation
        .playable_cards
        .iter()
        .map(|card| {
            format!(
                "{}|{}|{}|{}",
                card.card_id,
                card.upgraded,
                count_index(card.effective_cost_bucket),
                card.card_type
            )
        })
        .collect()
}

fn jaccard(left: impl IntoIterator<Item = String>, right: impl IntoIterator<Item = String>) -> u32 {
    let counts = |values: Vec<String>| {
        let mut counts = BTreeMap::new();
        for value in values {
            *counts.entry(value).or_insert(0_u32) += 1;
        }
        counts
    };
    let left = counts(left.into_iter().collect());
    let right = counts(right.into_iter().collect());
    let keys: std::collections::BTreeSet<_> = left.keys().chain(right.keys()).collect();
    let intersection: u32 = keys
        .iter()
        .map(|key| {
            left.get(*key)
                .unwrap_or(&0)
                .min(right.get(*key).unwrap_or(&0))
        })
        .sum();
    let union: u32 = keys
        .iter()
        .map(|key| {
            left.get(*key)
                .unwrap_or(&0)
                .max(right.get(*key).unwrap_or(&0))
        })
        .sum();
    (intersection * 1_000 + union / 2)
        .checked_div(union)
        .unwrap_or(1_000)
}

fn equality<T: PartialEq>(left: &T, right: &T) -> u32 {
    if left == right { 1_000 } else { 0 }
}

fn adjacent(left: u8, right: u8) -> u32 {
    match left.abs_diff(right) {
        0 => 1_000,
        1 => 500,
        _ => 0,
    }
}

fn count_index(value: CountBucket) -> u8 {
    match value {
        CountBucket::Zero => 0,
        CountBucket::One => 1,
        CountBucket::Two => 2,
        CountBucket::Three => 3,
        CountBucket::FourPlus => 4,
    }
}

fn turn_index(value: TurnBucket) -> u8 {
    match value {
        TurnBucket::Turn1 => 0,
        TurnBucket::Turn2 => 1,
        TurnBucket::Turn3 => 2,
        TurnBucket::Turn4Plus => 3,
    }
}

fn ratio_index(value: RatioBucket) -> u8 {
    match value {
        RatioBucket::Zero => 0,
        RatioBucket::P01_20 => 1,
        RatioBucket::P21_40 => 2,
        RatioBucket::P41_60 => 3,
        RatioBucket::P61_80 => 4,
        RatioBucket::P81_100 => 5,
        RatioBucket::Over100 => 6,
    }
}

fn threat_index(value: BlockThreatBucket) -> u8 {
    match value {
        BlockThreatBucket::NoIncoming => 0,
        BlockThreatBucket::FullyCovered => 1,
        BlockThreatBucket::Chip => 2,
        BlockThreatBucket::Danger => 3,
        BlockThreatBucket::Lethal => 4,
    }
}
