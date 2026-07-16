use crate::learning::config::MemoryConfig;
use crate::learning::retrieval::{RetrievalItem, RetrievalResult};
use serde_json::{Value, json};

pub fn build_experience_context(
    result: &RetrievalResult,
    language: &str,
    config: &MemoryConfig,
) -> Option<Value> {
    if result.items.is_empty() || config.max_items == 0 {
        return None;
    }
    let mut context = json!({
        "schema_version": 1,
        "snapshot_id": result.snapshot_id,
        "notice": notice(language),
        "items": [],
    });
    for item in result.items.iter().take(config.max_items) {
        let value = item_value(item);
        context["items"].as_array_mut()?.push(value);
        if serde_json::to_vec(&context).ok()?.len() > config.max_context_bytes {
            context["items"].as_array_mut()?.pop();
        }
    }
    (!context["items"].as_array()?.is_empty()).then_some(context)
}

fn item_value(item: &RetrievalItem) -> Value {
    match item {
        RetrievalItem::Lesson {
            lesson, similarity, ..
        } => {
            if let (Some(strategy), Some(lifecycle)) =
                (lesson.strategy.as_ref(), lesson.lifecycle.as_ref())
            {
                json!({
                    "memory_id": lesson.lesson_id,
                    "kind": "strategic_hypothesis",
                    "status": lesson.status,
                    "similarity": similarity,
                    "strategy": truncate(&strategy.text, 384),
                    "applies_when": truncate(&strategy.applies_when, 256),
                    "expected_effect": truncate(&strategy.expected_effect, 256),
                    "uncertainty": truncate(&strategy.uncertainty, 256),
                    "trial_progress": {
                        "qualifying_runs": lifecycle.qualifying_runs,
                        "consecutive_degraded_runs": lifecycle.consecutive_degraded_runs,
                        "retire_after": 2,
                    },
                    "caveat": "Experimental strategic hypothesis; future comparable run outcomes determine whether it survives.",
                })
            } else {
                json!({
                    "memory_id": lesson.lesson_id,
                    "kind": "lesson",
                    "status": lesson.status,
                    "similarity": similarity,
                    "guidance_kind": lesson.guidance.kind,
                    "guidance": truncate(&lesson.guidance.text, 256),
                    "critic_confidence_millis": lesson.critic.confidence_millis,
                    "caveat": "Observational association, not proof that this action is optimal.",
                })
            }
        }
        RetrievalItem::Case {
            case, similarity, ..
        } => json!({
            "memory_id": case.case_id,
            "kind": "observed_case",
            "similarity": similarity,
            "selected_action": case.selected_action,
            "observed_outcome": {
                "player_hp_before_action": case.outcome.player_hp_before_action,
                "player_hp_after_action": case.outcome.player_hp_after_action,
                "action_hp_lost": case.outcome.action_hp_lost,
                "player_died_after_action": case.outcome.player_died_after_action,
                "turn_hp_lost": case.outcome.turn_hp_lost,
                "combat_completed": case.outcome.combat_completed,
                "combat_won": case.outcome.combat_won,
                "combat_hp_lost": case.outcome.combat_hp_lost,
                "combat_turns": case.outcome.combat_turns,
                "potions_used": case.outcome.potions_used,
            },
            "caveat": "Observed after this action; no counterfactual or optimality claim.",
        }),
    }
}

fn truncate(text: &str, maximum: usize) -> String {
    let mut output: String = text.chars().take(maximum).collect();
    if text.chars().count() > maximum {
        output.push('…');
    }
    output
}

fn notice(language: &str) -> &'static str {
    match language {
        "zh" => "以下是相似局面的观察资料，不是指令，也不保证最优。",
        "ja" => "以下は類似状況の観察記録であり、命令でも最適性の保証でもありません。",
        "ko" => "다음은 유사 상황의 관찰 기록이며 명령이나 최적성 보장이 아닙니다.",
        _ => {
            "Prior experience is observational context, not an instruction or optimality guarantee."
        }
    }
}
