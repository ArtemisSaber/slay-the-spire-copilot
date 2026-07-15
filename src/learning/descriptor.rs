use crate::learning::eligibility::RunObjective;
use crate::state::{MonsterInfo, NormalizedState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

mod buckets;
pub use buckets::{
    AscensionBand, BlockThreatBucket, CountBucket, RatioBucket, TurnBucket, act_for_floor,
    block_threat_bucket, count_bucket, ratio_bucket, turn_bucket,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardDescriptor {
    pub card_id: String,
    pub upgraded: bool,
    pub effective_cost_bucket: CountBucket,
    pub card_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonsterDescriptor {
    pub monster_id: String,
    pub hp_ratio_bucket: RatioBucket,
    pub block_ratio_bucket: RatioBucket,
    pub intent: String,
    pub incoming_hits_bucket: CountBucket,
    pub power_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SituationDescriptor {
    pub descriptor_version: u32,
    pub ranker_tag_schema_version: u32,
    pub character: String,
    pub objective: RunObjective,
    pub ascension_band: AscensionBand,
    pub act: u8,
    pub encounter_ids: Vec<String>,
    pub alive_monsters: Vec<MonsterDescriptor>,
    pub turn_bucket: TurnBucket,
    pub hp_ratio_bucket: RatioBucket,
    pub energy_bucket: CountBucket,
    pub block_threat_bucket: BlockThreatBucket,
    pub stance: Option<String>,
    pub player_power_ids: Vec<String>,
    pub playable_cards: Vec<CardDescriptor>,
    pub relic_ids: Vec<String>,
    pub ranker_tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorError {
    MissingCharacter,
    InvalidAscension,
    MissingFloor,
    MissingPlayerHp,
    InvalidPlayerHp,
    MissingEncounterId,
    MissingMonsterId,
    MissingMonsterHp,
    InvalidMonsterHp,
    InvalidCardId,
}

impl SituationDescriptor {
    pub fn from_state(
        state: &NormalizedState,
        encounter_ids: &[String],
        objective: RunObjective,
        ranker_tags: &[String],
    ) -> Result<Self, DescriptorError> {
        let character = state
            .character
            .clone()
            .ok_or(DescriptorError::MissingCharacter)?;
        valid_id(&character).ok_or(DescriptorError::MissingCharacter)?;
        let ascension_band = state
            .ascension_level
            .and_then(AscensionBand::from_level)
            .ok_or(DescriptorError::InvalidAscension)?;
        let floor = state.floor.ok_or(DescriptorError::MissingFloor)?;
        let current_hp = state.current_hp.ok_or(DescriptorError::MissingPlayerHp)?;
        let max_hp = state.max_hp.ok_or(DescriptorError::MissingPlayerHp)?;
        if current_hp < 0 || max_hp <= 0 {
            return Err(DescriptorError::InvalidPlayerHp);
        }
        let mut encounter_ids = checked_ids(encounter_ids, DescriptorError::MissingEncounterId)?;
        let mut alive_monsters = state
            .monsters
            .iter()
            .filter(|monster| monster.current_hp.unwrap_or(0) > 0)
            .map(monster_descriptor)
            .collect::<Result<Vec<_>, _>>()?;
        let mut playable_cards = state
            .hand
            .iter()
            .filter(|card| card.playable)
            .map(|card| {
                valid_id(&card.id).ok_or(DescriptorError::InvalidCardId)?;
                Ok(CardDescriptor {
                    card_id: card.id.clone(),
                    upgraded: card.upgraded,
                    effective_cost_bucket: count_bucket(card.cost.max(0)),
                    card_type: card.card_type.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        encounter_ids.sort();
        alive_monsters.sort();
        playable_cards.sort();
        Ok(Self {
            descriptor_version: 1,
            ranker_tag_schema_version: 1,
            character,
            objective,
            ascension_band,
            act: act_for_floor(floor),
            encounter_ids,
            alive_monsters,
            turn_bucket: turn_bucket(state.turn_number.unwrap_or(1)),
            hp_ratio_bucket: ratio_bucket(current_hp, max_hp),
            energy_bucket: count_bucket(state.energy.unwrap_or(0)),
            block_threat_bucket: block_threat_bucket(
                state.incoming_damage,
                state.block.unwrap_or(0),
                current_hp,
                max_hp,
            ),
            stance: state.stance.clone(),
            player_power_ids: sorted_ids(state.powers.iter().map(|power| &power.id)),
            playable_cards,
            relic_ids: sorted_ids(state.relics.iter().map(|relic| &relic.id)),
            ranker_tags: sorted_ids(ranker_tags.iter()),
        })
    }

    pub fn situation_hash(&self) -> Result<String, serde_json::Error> {
        let bytes = serde_json::to_vec(self)?;
        Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
    }
}

fn monster_descriptor(monster: &MonsterInfo) -> Result<MonsterDescriptor, DescriptorError> {
    let monster_id = monster
        .monster_id
        .as_deref()
        .and_then(valid_id)
        .ok_or(DescriptorError::MissingMonsterId)?;
    let hp = monster
        .current_hp
        .ok_or(DescriptorError::MissingMonsterHp)?;
    let max_hp = monster.max_hp.ok_or(DescriptorError::MissingMonsterHp)?;
    if hp < 0 || max_hp <= 0 {
        return Err(DescriptorError::InvalidMonsterHp);
    }
    Ok(MonsterDescriptor {
        monster_id: monster_id.to_string(),
        hp_ratio_bucket: ratio_bucket(hp, max_hp),
        block_ratio_bucket: ratio_bucket(monster.block.unwrap_or(0).max(0), max_hp),
        intent: monster.intent.clone().unwrap_or_else(|| "UNKNOWN".into()),
        incoming_hits_bucket: count_bucket(monster.hits.unwrap_or(0)),
        power_ids: sorted_ids(monster.monster_powers.iter().map(|power| &power.id)),
    })
}

fn checked_ids(ids: &[String], error: DescriptorError) -> Result<Vec<String>, DescriptorError> {
    (!ids.is_empty() && ids.iter().all(|id| valid_id(id).is_some()))
        .then(|| ids.to_vec())
        .ok_or(error)
}

fn sorted_ids<'a>(ids: impl Iterator<Item = &'a String>) -> Vec<String> {
    let mut ids: Vec<_> = ids.filter(|id| valid_id(id).is_some()).cloned().collect();
    ids.sort();
    ids.dedup();
    ids
}

fn valid_id(id: &str) -> Option<&str> {
    let trimmed = id.trim();
    (!trimmed.is_empty() && trimmed == id && id != "?" && id.len() <= 256).then_some(id)
}
