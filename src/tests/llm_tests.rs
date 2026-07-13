use super::*;
use crate::locales::Locale;
use crate::state::{
    DangerFlags, DangerLevel, MonsterInfo, NormalizedState, RelicInfo, RoomType, ScreenType,
};
use crate::test_utils::test_locale;
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

const PROMPT_LOCK_CHILD_BASE: &str = "STS_COPILOT_PROMPT_LOCK_CHILD_BASE";
const PROMPT_LOCK_CHILD_READY: &str = "STS_COPILOT_PROMPT_LOCK_CHILD_READY";

fn wait_for_ready(path: &std::path::Path) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("child did not signal readiness: {}", path.display());
}

fn assert_child_waits_while_locked(child: &mut Child) {
    thread::sleep(Duration::from_millis(150));
    assert!(
        child.try_wait().unwrap().is_none(),
        "child append completed while parent lock was held"
    );
}

fn test_state() -> NormalizedState {
    NormalizedState {
        screen_type: Some(ScreenType::None),
        room_type: Some(RoomType::MonsterRoom),
        character: Some("IRONCLAD".into()),
        seed: None,
        ascension_level: None,
        floor: Some(1),
        current_hp: Some(68),
        max_hp: Some(75),
        gold: Some(99),
        energy: Some(3),
        block: Some(0),
        powers: vec![],
        hand: vec![],
        monsters: vec![],
        card_reward_choices: vec![],
        boss_relic_choices: vec![],
        event_id: None,
        event_name: None,
        event_body: None,
        event_choices: vec![],
        relics: vec![],
        potions: vec![],
        deck_names: vec![],
        incoming_damage: 0,
        rest_options: vec![],
        danger: DangerFlags {
            hp_critical: false,
            incoming_lethal: false,
            no_block_against_hit: false,
            any_monster_attacking: false,
            wrath_stance: false,
            level: DangerLevel::Safe,
        },
        skip_available: false,
        shop_cards: vec![],
        shop_relics: vec![],
        shop_potions: vec![],
        purge_available: false,
        purge_cost: None,
        hand_cards: vec![],
        draw_pile: vec![],
        discard_pile: vec![],
        exhaust_cards: vec![],
        master_cards: vec![],
        map_nodes: vec![],
        map_first_node_chosen: None,
        map_current_x: None,
        map_current_y: None,
        hand_select_max_cards: None,
        hand_select_can_pick_zero: false,
        hand_select_selected: vec![],
        current_action: None,
        card_in_play: None,
        grid_cards: vec![],
        grid_selected_cards: vec![],
        grid_for_upgrade: false,
        grid_for_transform: false,
        grid_for_purge: false,
        grid_num_cards: None,
        empty_potion_slots: 0,
        ..Default::default()
    }
}

fn monster() -> MonsterInfo {
    MonsterInfo {
        name: "大颚虫".into(),
        monster_id: None,
        index: 0,
        current_hp: Some(40),
        max_hp: Some(40),
        block: Some(0),
        intent: Some("ATTACK".into()),
        damage: Some(12),
        hits: Some(1),
        monster_powers: vec![],
        can_be_killed: false,
        is_scaling: false,
    }
}

#[test]
fn prompt_log_append_waits_for_cross_process_lock() {
    if let Ok(base) = std::env::var(PROMPT_LOCK_CHILD_BASE) {
        let ready = std::env::var(PROMPT_LOCK_CHILD_READY).unwrap();
        std::fs::write(ready, "ready").unwrap();
        log_prompt_into_dir(
            std::path::Path::new(&base),
            "locked-system",
            "locked-user",
            "locked-assistant",
        );
        return;
    }

    let dir = tempfile::tempdir().unwrap();
    let log_dir = dir.path().join("logs");
    std::fs::create_dir_all(&log_dir).unwrap();
    let log_path = log_dir.join("prompts.log");
    let ready_path = dir.path().join("child-ready");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .unwrap();
    fs4::FileExt::lock(&file).unwrap();

    let mut child = Command::new(std::env::current_exe().unwrap())
        .arg("llm::tests::prompt_log_append_waits_for_cross_process_lock")
        .arg("--exact")
        .env(PROMPT_LOCK_CHILD_BASE, dir.path())
        .env(PROMPT_LOCK_CHILD_READY, &ready_path)
        .spawn()
        .unwrap();

    wait_for_ready(&ready_path);
    assert_child_waits_while_locked(&mut child);
    fs4::FileExt::unlock(&file).unwrap();
    let status = child.wait().unwrap();
    assert!(status.success(), "child test failed: {status}");

    let content = std::fs::read_to_string(&log_path).unwrap();
    assert!(content.contains("[system]\nlocked-system"));
    assert!(content.contains("[user]\nlocked-user"));
    assert!(content.contains("[assistant]\nlocked-assistant"));
    assert!(content.contains("\n---\n"));
}

#[path = "llm_tests/config.rs"]
mod config;
#[path = "llm_tests/debug.rs"]
mod debug;
#[path = "llm_tests/effort.rs"]
mod effort;
#[path = "llm_tests/logging.rs"]
mod logging;
#[path = "llm_tests/metadata.rs"]
mod metadata;
#[path = "llm_tests/mock_autoplay.rs"]
mod mock_autoplay;
#[path = "llm_tests/prompts.rs"]
mod prompts;
#[path = "llm_tests/provider_requests.rs"]
mod provider_requests;
#[path = "llm_tests/sanitize_url.rs"]
mod sanitize_url;
#[path = "llm_tests/scenario.rs"]
mod scenario;
