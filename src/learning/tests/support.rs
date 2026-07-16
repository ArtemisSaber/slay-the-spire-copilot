use crate::learning::descriptor::{
    AscensionBand, BlockThreatBucket, CardDescriptor, CountBucket, MonsterDescriptor, RatioBucket,
    SituationDescriptor, TurnBucket,
};
use crate::learning::eligibility::RunObjective;
use crate::learning::{
    action::SemanticAction,
    case::{CaseDraft, CaseOutcome, CaseProvenance, DecisionCase, seed_hash},
    lesson::{
        ActionKind, ActionPattern, Guidance, GuidanceKind, Lesson, LessonBenchmark, LessonProposal,
        LessonScope, LessonTrigger, OutcomeCode, StrategicEvidence, StrategicHypothesis,
        StrategicLessonProposal,
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
    lesson_for_confidence(case, 700)
}

pub(super) fn lesson_for_confidence(case: &DecisionCase, confidence_millis: u16) -> Lesson {
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
                required_ranker_tags: vec![],
            },
            action_pattern: ActionPattern {
                kind: ActionKind::EndTurn,
                card_types: vec![],
                card_ids: vec![],
                potion_ids: vec![],
            },
            outcome_code: OutcomeCode::CombatWin,
            guidance: Guidance {
                kind: GuidanceKind::Prefer,
                text: "This is observational evidence; check the current threat.".into(),
            },
            rationale: "A cited case matched this action and outcome.".into(),
            source_case_ids: vec![case.case_id.clone()],
            critic_model_profile_sha256: "critic".into(),
            confidence_millis,
        },
        std::slice::from_ref(case),
    )
    .unwrap()
}

pub(super) fn strategic_lesson_for(case: &DecisionCase) -> Lesson {
    let benchmark = LessonBenchmark {
        origin_run_id: case.run_id.clone(),
        character: case.situation.character.clone(),
        ascension_level: match case.situation.ascension_band {
            AscensionBand::A0 => 0,
            AscensionBand::A20 => 20,
            _ => 10,
        },
        objective: case.situation.objective,
        final_floor: case.outcome.final_floor.unwrap(),
        victory: case.outcome.run_victory.unwrap(),
        compatibility_sha256: case.provenance.compatibility_sha256.clone(),
    };
    strategic_lesson_for_benchmark(case, benchmark)
}

pub(super) fn strategic_lesson_for_benchmark(
    case: &DecisionCase,
    benchmark: LessonBenchmark,
) -> Lesson {
    Lesson::propose_strategy(
        StrategicLessonProposal {
            language: "en".into(),
            scope: LessonScope {
                character: case.situation.character.clone(),
                objective: case.situation.objective,
                ascension_bands: vec![case.situation.ascension_band],
                encounter_ids: vec![],
            },
            strategy: StrategicHypothesis {
                text: "Prioritize the enemy whose continued presence creates the most pressure."
                    .into(),
                applies_when:
                    "Multiple targets are alive and target selection affects later turns.".into(),
                expected_effect: "This may reduce incoming pressure and preserve setup time."
                    .into(),
                evidence: vec![StrategicEvidence {
                    run_id: case.run_id.clone(),
                    decision_ids: vec![case.decision_id.clone()],
                    observed_chain: "The cited decision was followed by the recorded run outcome."
                        .into(),
                }],
                uncertainty: "Unchosen targets were not played, so the alternative is untested."
                    .into(),
            },
            source_case_ids: vec![case.case_id.clone()],
            critic_model_profile_sha256: "critic".into(),
            confidence_millis: 700,
            benchmark,
            parent_lesson_id: None,
            generation: 1,
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
    compatibility: &str,
) -> DecisionCase {
    decision_case_for_outcome(
        run_id,
        seed,
        situation,
        combat_won,
        retrieved_memory_ids,
        compatibility,
        (true, 51),
    )
}

pub(super) fn decision_case_for_outcome(
    run_id: &str,
    seed: i64,
    situation: SituationDescriptor,
    combat_won: bool,
    retrieved_memory_ids: &[String],
    compatibility: &str,
    run_result: (bool, i64),
) -> DecisionCase {
    let ascension_level = match situation.ascension_band {
        AscensionBand::A0 => Some(0),
        AscensionBand::A20 => Some(20),
        _ => Some(10),
    };
    CaseDraft {
        run_id: run_id.into(),
        decision_id: format!("{run_id}:8:2:1"),
        seed_hash: seed_hash(seed, compatibility),
        situation,
        ascension_level,
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
            player_hp_before_action: None,
            player_hp_after_action: None,
            action_hp_lost: None,
            player_died_after_action: None,
            alive_monsters_after_action: None,
            turn_hp_lost: Some(2),
            combat_completed: true,
            combat_won: Some(combat_won),
            combat_hp_lost: Some(5),
            combat_turns: Some(2),
            potions_used: vec![],
            run_completed: true,
            run_victory: Some(run_result.0),
            final_floor: Some(run_result.1),
        },
        CaseProvenance {
            app_version: "0.2.0".into(),
            prompt_schema_version: 1,
            rules_sha256: "rules".into(),
            model_profile_sha256: "model".into(),
            compatibility_sha256: compatibility.into(),
            knowledge_snapshot_id: None,
        },
    )
    .unwrap()
}
