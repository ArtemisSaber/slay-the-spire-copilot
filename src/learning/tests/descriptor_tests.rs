use crate::learning::descriptor::{
    AscensionBand, BlockThreatBucket, CountBucket, DescriptorError, RatioBucket,
    SituationDescriptor, TurnBucket, block_threat_bucket, count_bucket, ratio_bucket,
};
use crate::learning::eligibility::RunObjective;
use crate::state::{CardInfo, MonsterInfo, NormalizedState, PowerInfo, RelicInfo};

fn card(id: &str, uuid: &str) -> CardInfo {
    CardInfo {
        id: id.into(),
        name: format!("localized {id}"),
        cost: 2,
        card_type: "ATTACK".into(),
        upgraded: false,
        uuid: Some(uuid.into()),
        description: "hidden prose".into(),
        price: None,
        playable: true,
        has_target: true,
    }
}

fn monster(id: Option<&str>) -> MonsterInfo {
    MonsterInfo {
        name: "localized monster".into(),
        monster_id: id.map(str::to_string),
        index: 0,
        current_hp: Some(60),
        max_hp: Some(100),
        block: Some(0),
        intent: Some("ATTACK".into()),
        damage: Some(12),
        hits: Some(1),
        monster_powers: vec![PowerInfo {
            id: "Enrage".into(),
            name: "localized power".into(),
            amount: 2,
        }],
        can_be_killed: false,
        is_scaling: true,
    }
}

fn combat_state() -> NormalizedState {
    NormalizedState {
        character: Some("IRONCLAD".into()),
        seed: Some(1),
        ascension_level: Some(20),
        floor: Some(8),
        current_hp: Some(40),
        max_hp: Some(80),
        energy: Some(2),
        block: Some(3),
        hand: vec![card("Bash", "uuid-a")],
        monsters: vec![monster(Some("GremlinNob"))],
        powers: vec![],
        relics: vec![RelicInfo {
            id: "Burning Blood".into(),
            name: "localized relic".into(),
            description: "hidden".into(),
            counter: None,
            price: None,
        }],
        incoming_damage: 12,
        turn_number: Some(2),
        draw_pile: vec![card("Strike_R", "future-uuid")],
        ..NormalizedState::default()
    }
}

#[test]
fn buckets_have_explicit_boundary_behavior() {
    assert_eq!(AscensionBand::from_level(0), Some(AscensionBand::A0));
    assert_eq!(AscensionBand::from_level(9), Some(AscensionBand::A1_9));
    assert_eq!(AscensionBand::from_level(16), Some(AscensionBand::A10_16));
    assert_eq!(AscensionBand::from_level(19), Some(AscensionBand::A17_19));
    assert_eq!(AscensionBand::from_level(20), Some(AscensionBand::A20));
    assert_eq!(AscensionBand::from_level(-15), None);
    assert_eq!(ratio_bucket(0, 80), RatioBucket::Zero);
    assert_eq!(ratio_bucket(16, 80), RatioBucket::P01_20);
    assert_eq!(ratio_bucket(17, 80), RatioBucket::P21_40);
    assert_eq!(ratio_bucket(81, 80), RatioBucket::Over100);
    assert_eq!(count_bucket(0), CountBucket::Zero);
    assert_eq!(count_bucket(4), CountBucket::FourPlus);
}

#[test]
fn threat_order_handles_covered_lethal_chip_and_danger() {
    assert_eq!(
        block_threat_bucket(0, 0, 10, 80),
        BlockThreatBucket::NoIncoming
    );
    assert_eq!(
        block_threat_bucket(9, 9, 10, 80),
        BlockThreatBucket::FullyCovered
    );
    assert_eq!(block_threat_bucket(12, 3, 9, 80), BlockThreatBucket::Lethal);
    assert_eq!(block_threat_bucket(10, 3, 80, 80), BlockThreatBucket::Chip);
    assert_eq!(
        block_threat_bucket(20, 3, 80, 80),
        BlockThreatBucket::Danger
    );
}

#[test]
fn descriptor_excludes_seed_names_uuids_and_ordered_piles() {
    let first = combat_state();
    let mut second = first.clone();
    second.seed = Some(999);
    second.hand[0].name = "different locale".into();
    second.hand[0].uuid = Some("another-uuid".into());
    second.draw_pile.clear();
    second.monsters[0].name = "another locale".into();

    let build = |state| {
        SituationDescriptor::from_state(
            state,
            &["GremlinNob".into()],
            RunObjective::Act3Victory,
            &["block".into(), "damage".into(), "block".into()],
        )
        .unwrap()
    };
    let first = build(&first);
    let second = build(&second);

    assert_eq!(first, second);
    assert_eq!(
        first.situation_hash().unwrap(),
        second.situation_hash().unwrap()
    );
    assert_eq!(first.turn_bucket, TurnBucket::Turn2);
    assert_eq!(first.ranker_tags, vec!["block", "damage"]);
}

#[test]
fn missing_stable_monster_identity_fails_closed() {
    let mut state = combat_state();
    state.monsters = vec![monster(None)];

    assert_eq!(
        SituationDescriptor::from_state(
            &state,
            &["GremlinNob".into()],
            RunObjective::Act3Victory,
            &[],
        ),
        Err(DescriptorError::MissingMonsterId)
    );
}
