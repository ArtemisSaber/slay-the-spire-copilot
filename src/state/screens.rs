use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScreenType {
    BossReward,
    CardReward,
    Chest,
    CombatReward,
    Complete,
    Event,
    GameOver,
    Grid,
    HandSelect,
    Map,
    None,
    Rest,
    ShopRoom,
    ShopScreen,
    Unknown,
}

impl std::fmt::Display for ScreenType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl ScreenType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BossReward => "BOSS_REWARD",
            Self::CardReward => "CARD_REWARD",
            Self::Chest => "CHEST",
            Self::CombatReward => "COMBAT_REWARD",
            Self::Complete => "COMPLETE",
            Self::Event => "EVENT",
            Self::GameOver => "GAME_OVER",
            Self::Grid => "GRID",
            Self::HandSelect => "HAND_SELECT",
            Self::Map => "MAP",
            Self::None => "NONE",
            Self::Rest => "REST",
            Self::ShopRoom => "SHOP_ROOM",
            Self::ShopScreen => "SHOP_SCREEN",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum RoomType {
    EventRoom,
    MonsterRoom,
    MonsterRoomBoss,
    MonsterRoomElite,
    NeowRoom,
    RestRoom,
    ShopRoom,
    TreasureRoom,
    VictoryRoom,
}

impl std::fmt::Display for RoomType {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl RoomType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EventRoom => "EventRoom",
            Self::MonsterRoom => "MonsterRoom",
            Self::MonsterRoomBoss => "MonsterRoomBoss",
            Self::MonsterRoomElite => "MonsterRoomElite",
            Self::NeowRoom => "NeowRoom",
            Self::RestRoom => "RestRoom",
            Self::ShopRoom => "ShopRoom",
            Self::TreasureRoom => "TreasureRoom",
            Self::VictoryRoom => "VictoryRoom",
        }
    }
}
