use super::support::situation;
use crate::learning::action::SemanticAction;
use crate::learning::case::{CaseDraft, CaseOutcome, CaseProvenance, DecisionCase, seed_hash};
use crate::learning::config::{MemoryConfig, MemoryMode};
use crate::learning::descriptor::{
    AscensionBand, BlockThreatBucket, CardDescriptor, CountBucket, RatioBucket, TurnBucket,
};
use crate::learning::session::{LearningSession, SessionProvenance};
use crate::learning::snapshot::KnowledgeSnapshot;
use crate::learning::store::KnowledgeStore;
use crate::learning::telemetry::{DecisionSource, RecordedRankedAction};

pub(super) fn guardian_session(
    root: &std::path::Path,
) -> (LearningSession, DecisionCase, DecisionCase) {
    let incidental = guardian_incidental_case();
    let fatal = guardian_fatal_case();
    let cases = vec![incidental.clone(), fatal.clone()];
    let snapshot = KnowledgeSnapshot::build(cases.clone(), &[]).unwrap();
    let store = KnowledgeStore::new(root);
    store.append_cases(&cases).unwrap();
    let session = LearningSession::new(
        MemoryConfig {
            mode: MemoryMode::Collect,
            ..MemoryConfig::default()
        },
        store,
        snapshot,
        SessionProvenance {
            locale: "en".into(),
            model_profile_sha256: "model".into(),
            compatibility_sha256: "mods".into(),
            rules_sha256: "rules".into(),
            synthetic_input: false,
        },
    );
    (session, fatal, incidental)
}

fn guardian_fatal_case() -> DecisionCase {
    let mut guardian = situation();
    guardian.ascension_band = AscensionBand::A0;
    guardian.encounter_ids = vec!["TheGuardian".into()];
    guardian.alive_monsters[0].monster_id = "TheGuardian".into();
    guardian.alive_monsters[0].hp_ratio_bucket = RatioBucket::P01_20;
    guardian.alive_monsters[0].intent = "ATTACK".into();
    guardian.alive_monsters[0].power_ids =
        vec!["Sharp Hide".into(), "Vulnerable".into(), "Weakened".into()];
    guardian.turn_bucket = TurnBucket::Turn4Plus;
    guardian.hp_ratio_bucket = RatioBucket::P01_20;
    guardian.energy_bucket = CountBucket::Two;
    guardian.block_threat_bucket = BlockThreatBucket::Lethal;
    guardian.playable_cards = vec![
        card("Defend_R", "SKILL", CountBucket::One),
        card("Perfected Strike", "ATTACK", CountBucket::Two),
    ];
    guardian.ranker_tags = vec![
        "block".into(),
        "damage".into(),
        "killable".into(),
        "lethal".into(),
        "retaliation".into(),
    ];
    let defend = play("Defend_R", None);
    let fatal = play("Perfected Strike", Some("TheGuardian"));
    finalized_case(
        "guardian-run:16:12:99",
        guardian,
        fatal.clone(),
        vec![defend.clone(), fatal.clone(), SemanticAction::EndTurn],
        vec![
            ranked(defend, 2_740, &["block"]),
            ranked(
                fatal,
                -386_745,
                &["damage", "killable", "lethal", "retaliation"],
            ),
        ],
        action_outcome(2, 0, 2, true, 0),
    )
}

fn guardian_incidental_case() -> DecisionCase {
    let mut guardian = situation();
    guardian.ascension_band = AscensionBand::A0;
    guardian.encounter_ids = vec!["TheGuardian".into()];
    guardian.alive_monsters[0].monster_id = "TheGuardian".into();
    guardian.alive_monsters[0].intent = "DEFEND".into();
    guardian.alive_monsters[0].power_ids = vec!["Mode Shift".into()];
    guardian.turn_bucket = TurnBucket::Turn4Plus;
    guardian.hp_ratio_bucket = RatioBucket::P01_20;
    guardian.block_threat_bucket = BlockThreatBucket::NoIncoming;
    guardian.playable_cards = vec![card("Defend_R", "SKILL", CountBucket::One)];
    guardian.ranker_tags = vec!["excessive".into()];
    let defend = play("Defend_R", None);
    finalized_case(
        "guardian-run:16:10:91",
        guardian,
        defend.clone(),
        vec![defend.clone(), SemanticAction::EndTurn],
        vec![
            ranked(SemanticAction::EndTurn, 0, &[]),
            ranked(defend, -2_010, &["excessive"]),
        ],
        action_outcome(5, 5, 0, false, 1),
    )
}

fn action_outcome(
    hp_before: i64,
    hp_after: i64,
    action_hp_lost: i64,
    died: bool,
    monsters_after: usize,
) -> CaseOutcome {
    CaseOutcome {
        command_succeeded: true,
        player_hp_before_action: Some(hp_before),
        player_hp_after_action: Some(hp_after),
        action_hp_lost: Some(action_hp_lost),
        player_died_after_action: Some(died),
        alive_monsters_after_action: Some(monsters_after),
        turn_hp_lost: Some(action_hp_lost),
        combat_completed: true,
        combat_won: Some(false),
        combat_hp_lost: Some(55),
        combat_turns: Some(12),
        potions_used: vec![],
        run_completed: true,
        run_victory: Some(false),
        final_floor: Some(16),
    }
}

fn finalized_case(
    decision_id: &str,
    situation: crate::learning::descriptor::SituationDescriptor,
    selected_action: SemanticAction,
    available_semantic_actions: Vec<SemanticAction>,
    ranked_suggestions: Vec<RecordedRankedAction>,
    outcome: CaseOutcome,
) -> DecisionCase {
    CaseDraft {
        run_id: "guardian-run".into(),
        decision_id: decision_id.into(),
        seed_hash: seed_hash(42, "mods"),
        situation,
        selected_action,
        decision_source: DecisionSource::Llm,
        available_semantic_actions,
        ranked_suggestions,
        retrieved_memory_ids: vec![],
        memory_ids_used: vec![],
    }
    .finalize(outcome, provenance())
    .unwrap()
}

fn provenance() -> CaseProvenance {
    CaseProvenance {
        app_version: "0.2.0".into(),
        prompt_schema_version: 1,
        rules_sha256: "rules".into(),
        model_profile_sha256: "model".into(),
        compatibility_sha256: "mods".into(),
        knowledge_snapshot_id: None,
    }
}

fn play(card_id: &str, target: Option<&str>) -> SemanticAction {
    SemanticAction::PlayCard {
        card_id: card_id.into(),
        upgraded: false,
        target_monster_id: target.map(str::to_string),
    }
}

fn card(card_id: &str, card_type: &str, cost: CountBucket) -> CardDescriptor {
    CardDescriptor {
        card_id: card_id.into(),
        upgraded: false,
        effective_cost_bucket: cost,
        card_type: card_type.into(),
    }
}

fn ranked(action: SemanticAction, score: i64, tags: &[&str]) -> RecordedRankedAction {
    RecordedRankedAction {
        semantic_action: action,
        score,
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
    }
}
