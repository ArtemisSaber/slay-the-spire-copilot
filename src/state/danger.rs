use serde::{Deserialize, Serialize};

use super::{MonsterInfo, PowerInfo};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum DangerLevel {
    #[default]
    Safe,
    Caution,
    Danger,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DangerFlags {
    pub hp_critical: bool,
    pub incoming_lethal: bool,
    pub no_block_against_hit: bool,
    pub any_monster_attacking: bool,
    pub wrath_stance: bool,
    pub level: DangerLevel,
}

impl DangerFlags {
    pub fn compute(
        current_hp: Option<i64>,
        max_hp: Option<i64>,
        block: Option<i64>,
        incoming_damage: i64,
        monsters: &[MonsterInfo],
        powers: &[PowerInfo],
    ) -> Self {
        let hp = current_hp.unwrap_or(0);
        let max_hp_val = max_hp.unwrap_or(1);
        let block_val = block.unwrap_or(0);

        let hp_critical = max_hp_val > 0 && (hp as f64 / max_hp_val as f64) < 0.3;
        let incoming_lethal = incoming_damage > hp + block_val;
        let no_block_against_hit = block_val == 0 && incoming_damage > 0;
        let low_hp = max_hp_val > 0 && (hp as f64 / max_hp_val as f64) < 0.6;
        let any_monster_attacking = monsters
            .iter()
            .any(|monster| monster.intent.as_deref() != Some("NONE"));
        let wrath_stance = powers
            .iter()
            .any(|power| power.name == "Wrath" && power.amount > 0);

        let level = if hp_critical || incoming_lethal {
            DangerLevel::Danger
        } else if low_hp || no_block_against_hit || any_monster_attacking {
            DangerLevel::Caution
        } else {
            DangerLevel::Safe
        };

        DangerFlags {
            hp_critical,
            incoming_lethal,
            no_block_against_hit,
            any_monster_attacking,
            wrath_stance,
            level,
        }
    }
}
