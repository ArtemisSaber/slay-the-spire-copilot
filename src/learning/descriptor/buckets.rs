use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AscensionBand {
    A0,
    A1_9,
    A10_16,
    A17_19,
    A20,
}

impl AscensionBand {
    pub fn from_level(level: i64) -> Option<Self> {
        match level {
            0 => Some(Self::A0),
            1..=9 => Some(Self::A1_9),
            10..=16 => Some(Self::A10_16),
            17..=19 => Some(Self::A17_19),
            20 => Some(Self::A20),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnBucket {
    Turn1,
    Turn2,
    Turn3,
    Turn4Plus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RatioBucket {
    Zero,
    P01_20,
    P21_40,
    P41_60,
    P61_80,
    P81_100,
    Over100,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CountBucket {
    Zero,
    One,
    Two,
    Three,
    FourPlus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockThreatBucket {
    NoIncoming,
    FullyCovered,
    Chip,
    Danger,
    Lethal,
}

pub fn ratio_bucket(value: i64, maximum: i64) -> RatioBucket {
    if value <= 0 {
        return RatioBucket::Zero;
    }
    let ratio = i128::from(value) * 100;
    let maximum = i128::from(maximum.max(1));
    match ratio {
        ratio if ratio <= maximum * 20 => RatioBucket::P01_20,
        ratio if ratio <= maximum * 40 => RatioBucket::P21_40,
        ratio if ratio <= maximum * 60 => RatioBucket::P41_60,
        ratio if ratio <= maximum * 80 => RatioBucket::P61_80,
        ratio if ratio <= maximum * 100 => RatioBucket::P81_100,
        _ => RatioBucket::Over100,
    }
}

pub fn count_bucket(value: i64) -> CountBucket {
    match value {
        i64::MIN..=0 => CountBucket::Zero,
        1 => CountBucket::One,
        2 => CountBucket::Two,
        3 => CountBucket::Three,
        _ => CountBucket::FourPlus,
    }
}

pub fn block_threat_bucket(incoming: i64, block: i64, hp: i64, max_hp: i64) -> BlockThreatBucket {
    if incoming <= 0 {
        return BlockThreatBucket::NoIncoming;
    }
    if block >= incoming {
        return BlockThreatBucket::FullyCovered;
    }
    let uncovered = incoming - block.max(0);
    if uncovered >= hp {
        return BlockThreatBucket::Lethal;
    }
    let chip_limit = 3.max((max_hp.max(0) + 9) / 10);
    if uncovered < chip_limit {
        BlockThreatBucket::Chip
    } else {
        BlockThreatBucket::Danger
    }
}

pub fn turn_bucket(turn: i64) -> TurnBucket {
    match turn {
        i64::MIN..=1 => TurnBucket::Turn1,
        2 => TurnBucket::Turn2,
        3 => TurnBucket::Turn3,
        _ => TurnBucket::Turn4Plus,
    }
}

pub fn act_for_floor(floor: i64) -> u8 {
    match floor {
        i64::MIN..=17 => 1,
        18..=34 => 2,
        35..=51 => 3,
        _ => 4,
    }
}
