use super::*;

#[test]
fn screen_type_deserializes_known_variants() {
    let cases: &[(&str, ScreenType)] = &[
        ("\"BOSS_REWARD\"", ScreenType::BossReward),
        ("\"CARD_REWARD\"", ScreenType::CardReward),
        ("\"CHEST\"", ScreenType::Chest),
        ("\"COMBAT_REWARD\"", ScreenType::CombatReward),
        ("\"COMPLETE\"", ScreenType::Complete),
        ("\"EVENT\"", ScreenType::Event),
        ("\"GAME_OVER\"", ScreenType::GameOver),
        ("\"GRID\"", ScreenType::Grid),
        ("\"HAND_SELECT\"", ScreenType::HandSelect),
        ("\"MAP\"", ScreenType::Map),
        ("\"NONE\"", ScreenType::None),
        ("\"REST\"", ScreenType::Rest),
        ("\"SHOP_ROOM\"", ScreenType::ShopRoom),
        ("\"SHOP_SCREEN\"", ScreenType::ShopScreen),
        ("\"UNKNOWN\"", ScreenType::Unknown),
    ];
    for (json, expected) in cases {
        let parsed: ScreenType =
            serde_json::from_str(json).unwrap_or_else(|e| panic!("failed to parse {json}: {e}"));
        assert_eq!(parsed, *expected, "mismatch for {json}");
    }
}

#[test]
fn screen_type_rejects_unknown_strings() {
    let result: Result<ScreenType, _> = serde_json::from_str("\"UNKNOWN_WEIRD_SCREEN\"");
    assert!(
        result.is_err(),
        "unknown screen type should fail to deserialize"
    );
}

#[test]
fn screen_type_serializes_to_screaming_snake_case() {
    assert_eq!(
        serde_json::to_string(&ScreenType::CardReward).unwrap(),
        "\"CARD_REWARD\""
    );
    assert_eq!(
        serde_json::to_string(&ScreenType::BossReward).unwrap(),
        "\"BOSS_REWARD\""
    );
    assert_eq!(
        serde_json::to_string(&ScreenType::ShopScreen).unwrap(),
        "\"SHOP_SCREEN\""
    );
    assert_eq!(
        serde_json::to_string(&ScreenType::None).unwrap(),
        "\"NONE\""
    );
}

#[test]
fn screen_type_display_matches_serialized_form() {
    for variant in [
        ScreenType::BossReward,
        ScreenType::CardReward,
        ScreenType::Chest,
        ScreenType::CombatReward,
        ScreenType::Complete,
        ScreenType::Event,
        ScreenType::GameOver,
        ScreenType::Grid,
        ScreenType::HandSelect,
        ScreenType::Map,
        ScreenType::None,
        ScreenType::Rest,
        ScreenType::ShopRoom,
        ScreenType::ShopScreen,
        ScreenType::Unknown,
    ] {
        let serialized = serde_json::to_string(&variant).unwrap();
        let stripped = serialized.trim_matches('"');
        assert_eq!(
            variant.to_string(),
            stripped,
            "Display mismatch for {variant:?}"
        );
    }
}

#[test]
fn screen_type_round_trips_through_serde() {
    let original = ScreenType::HandSelect;
    let json = serde_json::to_string(&original).unwrap();
    let back: ScreenType = serde_json::from_str(&json).unwrap();
    assert_eq!(original, back);
}

#[test]
fn room_type_treasure_room_round_trips_through_serde() {
    let original = RoomType::TreasureRoom;
    let json = serde_json::to_string(&original).unwrap();
    assert_eq!(json, "\"TreasureRoom\"");

    let back: RoomType = serde_json::from_str(&json).unwrap();
    assert_eq!(back, original);
}

#[test]
fn normalize_treasure_room_preserves_room_type() {
    let raw = json!({
        "in_game": true,
        "game_state": {
            "screen_type": "CHEST",
            "room_type": "TreasureRoom"
        }
    });

    let state = NormalizedState::from_raw(&raw, test_locale());
    assert_eq!(state.room_type, Some(RoomType::TreasureRoom));
}
