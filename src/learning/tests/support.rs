use crate::learning::descriptor::{
    AscensionBand, BlockThreatBucket, CardDescriptor, CountBucket, MonsterDescriptor, RatioBucket,
    SituationDescriptor, TurnBucket,
};
use crate::learning::eligibility::RunObjective;
use crate::learning::{
    action::SemanticAction,
    case::{CaseDraft, CaseOutcome, CaseProvenance, DecisionCase, seed_hash},
    lesson::{
        ActionKind, ActionPattern, Guidance, GuidanceKind, Lesson, LessonProposal, LessonScope,
        LessonTrigger, OutcomeCode,
    },
    telemetry::DecisionSource,
};

pub(super) fn situation() -> SituationDescriptor {
    SituationDescriptor {
        descriptor_version: 1,
        ranker_tag_schema_version: 1,
        character: "IRONCLAD".into(),
        objective: RunObjective::Act3Victory,
        ascension_band: AscensionBand::A20,
        act: 1,
        encounter_ids: vec!["GremlinNob".into()],
        alive_monsters: vec![MonsterDescriptor {
            monster_id: "GremlinNob".into(),
            hp_ratio_bucket: RatioBucket::P61_80,
            block_ratio_bucket: RatioBucket::Zero,
            intent: "ATTACK".into(),
            incoming_hits_bucket: CountBucket::One,
            power_ids: vec!["Enrage".into()],
        }],
        turn_bucket: TurnBucket::Turn2,
        hp_ratio_bucket: RatioBucket::P41_60,
        energy_bucket: CountBucket::Two,
        block_threat_bucket: BlockThreatBucket::Danger,
        stance: None,
        player_power_ids: vec![],
        playable_cards: vec![CardDescriptor {
            card_id: "Bash".into(),
            upgraded: false,
            effective_cost_bucket: CountBucket::Two,
            card_type: "ATTACK".into(),
        }],
        relic_ids: vec!["Burning Blood".into()],
        ranker_tags: vec!["damage".into()],
    }
}

pub(super) fn lesson_for(case: &DecisionCase) -> Lesson {
    Lesson::propose(
        LessonProposal {
            language: "en".into(),
            scope: LessonScope {
                character: case.situation.character.clone(),
                objective: case.situation.objective,
                ascension_bands: vec![case.situation.ascension_band],
                encounter_ids: case.situation.encounter_ids.clone(),
            },
            trigger: LessonTrigger {
                turn_buckets: vec![case.situation.turn_bucket],
                block_threat_buckets: vec![case.situation.block_threat_bucket],
                required_card_ids: vec![],
                required_enemy_power_ids: vec!["Enrage".into()],
                required_ranker_tags: vec!["damage".into()],
            },
            action_pattern: ActionPattern {
                kind: ActionKind::EndTurn,
                card_types: vec![],
                card_ids: vec![],
                potion_ids: vec![],
            },
            outcome_code: OutcomeCode::CombatWin,
            guidance: Guidance {
                kind: GuidanceKind::Caution,
                text: "This is observational evidence; check the current threat.".into(),
            },
            rationale: "A cited case matched this action and outcome.".into(),
            source_case_ids: vec![case.case_id.clone()],
            critic_model_profile_sha256: "critic".into(),
            confidence_millis: 700,
        },
        std::slice::from_ref(case),
    )
    .unwrap()
}

pub(super) fn decision_case(run_id: &str, seed: i64) -> DecisionCase {
    decision_case_with(run_id, seed, true, &[])
}

pub(super) fn decision_case_with(
    run_id: &str,
    seed: i64,
    combat_won: bool,
    retrieved_memory_ids: &[String],
) -> DecisionCase {
    decision_case_for(
        run_id,
        seed,
        situation(),
        combat_won,
        retrieved_memory_ids,
        "mods",
    )
}

pub(super) fn decision_case_for(
    run_id: &str,
    seed: i64,
    situation: SituationDescriptor,
    combat_won: bool,
    retrieved_memory_ids: &[String],
    mod_profile: &str,
) -> DecisionCase {
    CaseDraft {
        run_id: run_id.into(),
        decision_id: format!("{run_id}:8:2:1"),
        seed_hash: seed_hash(seed, "mods"),
        situation,
        selected_action: SemanticAction::EndTurn,
        decision_source: DecisionSource::Llm,
        available_semantic_actions: vec![SemanticAction::EndTurn],
        ranked_suggestions: vec![],
        retrieved_memory_ids: retrieved_memory_ids.to_vec(),
        memory_ids_used: vec![],
    }
    .finalize(
        CaseOutcome {
            command_succeeded: true,
            turn_hp_lost: Some(2),
            combat_completed: true,
            combat_won: Some(combat_won),
            combat_hp_lost: Some(5),
            combat_turns: Some(2),
            potions_used: vec![],
            run_completed: true,
            run_victory: Some(true),
            final_floor: Some(51),
        },
        CaseProvenance {
            app_version: "0.2.0".into(),
            prompt_schema_version: 1,
            rules_sha256: "rules".into(),
            model_profile_sha256: "model".into(),
            mod_profile_sha256: mod_profile.into(),
            knowledge_snapshot_id: None,
        },
    )
    .unwrap()
}
