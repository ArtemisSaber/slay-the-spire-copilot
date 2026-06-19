use super::*;
use crate::state::DangerFlags;
use serde_json::Value;

fn load_i18n() -> crate::i18n::I18n {
    crate::i18n::I18n::load()
}

fn load_fixture(name: &str) -> Value {
    let path = format!("tests/fixtures/{name}");
    let content = std::fs::read_to_string(&path).unwrap();
    serde_json::from_str(&content).unwrap()
}

#[test]
fn normalize_combat_state() {
    let i18n = load_i18n();
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, &i18n);

    assert_eq!(state.screen_type.as_deref(), Some("NONE"));
    assert_eq!(state.character.as_deref(), Some("IRONCLAD"));
    assert_eq!(state.floor, Some(1));
    assert_eq!(state.current_hp, Some(68));
    assert_eq!(state.max_hp, Some(75));
    assert_eq!(state.gold, Some(99));
    assert_eq!(state.energy, Some(3));
    assert_eq!(state.block, Some(6));

    assert_eq!(state.hand.len(), 3);
    assert_eq!(state.monsters.len(), 1);
    assert_eq!(state.relics.len(), 2);
    assert_eq!(state.potions.len(), 1);
    assert!(state.card_reward_choices.is_empty());

    assert_eq!(state.incoming_damage, 12);
    assert_eq!(state.danger.level, DangerLevel::Caution);
    assert!(state.danger.any_monster_attacking);

    let jaw_worm = &state.monsters[0];
    assert_eq!(jaw_worm.name, "大颚虫");
    assert_eq!(jaw_worm.index, 0);
    assert_eq!(jaw_worm.intent.as_deref(), Some("ATTACK"));
    assert!(jaw_worm.is_scaling);

    assert_eq!(state.hand_cards.len(), 3);
    assert_eq!(state.draw_pile.len(), 2);
    assert_eq!(state.discard_pile.len(), 0);
    assert_eq!(state.exhaust_cards.len(), 0);
    assert_eq!(state.master_cards.len(), 1);
}

#[test]
fn normalize_card_reward_state() {
    let i18n = load_i18n();
    let raw = load_fixture("card-reward-state.json");
    let state = NormalizedState::from_raw(&raw, &i18n);

    assert_eq!(state.screen_type.as_deref(), Some("CARD_REWARD"));
    assert_eq!(state.card_reward_choices.len(), 3);
    assert!(state.card_reward_choices.iter().any(|c| c.name == "上勾拳"));
    assert!(state.card_reward_choices.iter().any(|c| c.name == "愤怒"));
    assert!(state.card_reward_choices.iter().any(|c| c.name == "头槌"));

    assert!(state.hand.is_empty());
    assert!(state.monsters.is_empty());
    assert_eq!(state.incoming_damage, 0);
    assert_eq!(state.danger.level, DangerLevel::Safe);

    assert_eq!(state.master_cards.len(), 1);
}

#[test]
fn normalize_rest_state() {
    let i18n = load_i18n();
    let raw = load_fixture("rest-state.json");
    let state = NormalizedState::from_raw(&raw, &i18n);

    assert_eq!(state.screen_type.as_deref(), Some("REST"));
    assert_eq!(
        state.rest_options,
        vec!["rest".to_string(), "smith".to_string(), "toke".to_string()]
    );
    assert_eq!(state.current_hp, Some(25));
    assert_eq!(state.max_hp, Some(75));
    assert_eq!(state.floor, Some(5));
    assert_eq!(state.gold, Some(120));

    assert_eq!(state.master_cards.len(), 9);
    assert!(state.potions.is_empty());
    assert!(state.monsters.is_empty());
    assert!(state.hand_cards.is_empty());
    assert_eq!(state.danger.level, DangerLevel::Caution);
}

#[test]
fn danger_detects_low_hp() {
    let d = compute_danger(10, 80, 0, 0, false, &[]);
    assert!(d.hp_critical);
    assert!(!d.incoming_lethal);
    assert_eq!(d.level, DangerLevel::Danger);
}

#[test]
fn danger_detects_incoming_lethal() {
    let d = compute_danger(20, 80, 5, 30, false, &[]);
    assert!(d.incoming_lethal);
    assert_eq!(d.level, DangerLevel::Danger);
}

#[test]
fn danger_no_block_against_hit_is_caution() {
    let d = compute_danger(60, 80, 0, 10, false, &["ATTACK"]);
    assert!(d.no_block_against_hit);
    assert!(!d.hp_critical);
    assert_eq!(d.level, DangerLevel::Caution);
}

#[test]
fn danger_monster_attacking_is_caution() {
    let d = compute_danger(60, 80, 10, 0, false, &["ATTACK"]);
    assert!(d.any_monster_attacking);
    assert!(!d.hp_critical);
    assert_eq!(d.level, DangerLevel::Caution);
}

#[test]
fn danger_low_hp_is_caution() {
    let d = compute_danger(35, 80, 10, 0, false, &[]);
    assert!(!d.hp_critical);
    assert!(!d.any_monster_attacking);
    assert_eq!(d.level, DangerLevel::Caution);
}

#[test]
fn danger_wrath_stance_detected() {
    let d = compute_danger(60, 80, 10, 0, true, &["ATTACK"]);
    assert!(d.wrath_stance);
    assert!(d.any_monster_attacking);
    assert_eq!(d.level, DangerLevel::Caution);
}

#[test]
fn danger_safe_when_healthy_and_not_threatened() {
    let d = compute_danger(70, 80, 20, 0, false, &[]);
    assert!(!d.hp_critical);
    assert!(!d.incoming_lethal);
    assert!(!d.no_block_against_hit);
    assert!(!d.any_monster_attacking);
    assert_eq!(d.level, DangerLevel::Safe);
}

fn compute_danger(
    hp: i64,
    max_hp: i64,
    block: i64,
    incoming: i64,
    wrath: bool,
    intents: &[&str],
) -> DangerFlags {
    let monsters: Vec<MonsterInfo> = intents
        .iter()
        .map(|intent| MonsterInfo {
            name: "Test".into(),
            index: 0,
            current_hp: None,
            max_hp: None,
            block: None,
            intent: if *intent == "NONE" {
                None
            } else {
                Some(intent.to_string())
            },
            damage: None,
            hits: None,
            monster_powers: vec![],
            can_be_killed: false,
            is_scaling: false,
        })
        .collect();

    let powers: Vec<PowerInfo> = if wrath {
        vec![PowerInfo {
            name: "Wrath".into(),
            amount: 1,
        }]
    } else {
        vec![]
    };

    DangerFlags::compute(
        Some(hp),
        Some(max_hp),
        Some(block),
        incoming,
        &monsters,
        &powers,
    )
}

#[test]
fn stable_hash_same_state_same_hash() {
    let i18n = load_i18n();
    let raw = load_fixture("combat-state.json");
    let state1 = NormalizedState::from_raw(&raw, &i18n);
    let state2 = NormalizedState::from_raw(&raw, &i18n);

    assert_eq!(state1.stable_hash(), state2.stable_hash());
}

#[test]
fn stable_hash_different_state_different_hash() {
    let i18n = load_i18n();
    let combat = load_fixture("combat-state.json");
    let reward = load_fixture("card-reward-state.json");

    let state1 = NormalizedState::from_raw(&combat, &i18n);
    let state2 = NormalizedState::from_raw(&reward, &i18n);

    assert_ne!(state1.stable_hash(), state2.stable_hash());
}

#[test]
fn stable_hash_produces_hex() {
    let i18n = load_i18n();
    let raw = load_fixture("combat-state.json");
    let state = NormalizedState::from_raw(&raw, &i18n);
    let hash = state.stable_hash();

    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}
