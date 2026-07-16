use crate::locales::Locale;

pub(crate) fn unified_system_prompt(locale: &Locale) -> String {
    let mut output = vec![locale.unified_preamble.clone()];
    let modes: &[(&str, &str, &str)] = &[
        (
            "combat",
            &locale.system_prompts.combat_entry,
            &locale.few_shot_examples.combat_entry,
        ),
        (
            "card_reward",
            &locale.system_prompts.card_reward,
            &locale.few_shot_examples.card_reward,
        ),
        (
            "boss_card_reward",
            &locale.system_prompts.boss_card_reward,
            &locale.few_shot_examples.boss_card_reward,
        ),
        (
            "rest",
            &locale.system_prompts.rest,
            &locale.few_shot_examples.rest,
        ),
        (
            "boss_relic",
            &locale.system_prompts.boss_relic,
            &locale.few_shot_examples.boss_relic,
        ),
        (
            "event_choice",
            &locale.system_prompts.event_choice,
            &locale.few_shot_examples.event_choice,
        ),
        (
            "shop",
            &locale.system_prompts.shop,
            &locale.few_shot_examples.shop,
        ),
        (
            "map_suggestion",
            &locale.system_prompts.map_suggestion,
            &locale.few_shot_examples.map_suggestion,
        ),
        (
            "map_crossroad",
            &locale.system_prompts.map_crossroad,
            &locale.few_shot_examples.map_crossroad,
        ),
        (
            "generic",
            &locale.system_prompts.generic,
            &locale.few_shot_examples.generic,
        ),
    ];
    for (mode, system_prompt, few_shot_example) in modes {
        output.push(format!(
            "\n---\n\n[mode: {mode}]\n{system_prompt}\n\n{few_shot_example}"
        ));
    }
    output.concat()
}

const AUTOPLAY_ACTION_SYSTEM_PROMPT: &str = r#"AUTO_PLAY_ACTION_PLANNER
You are the Slay the Spire auto-play action planner.
Return strict JSON only, with this shape:
{"schema_version":1,"actions":[{"kind":"choose|skip|proceed|play|end|leave","action_id":"...","target_index":0,"label":"...","reason":"...","risk":"..."}]}
Choose exactly one action_id from the provided available_actions.
Never invent an action_id. Never output prose or Markdown.
For targeted combat cards, include target_index. If no available action is safe, choose an available non-destructive exit such as end/leave/proceed when present."#;

const LEARNING_POSTMORTEM_SYSTEM_PROMPT: &str = r##"LEARNING_CRITIC_ENVELOPE_V2
You are a Slay the Spire loss analyst, postmortem writer, and conservative experience critic.
Return exactly one strict JSON object and no other text or code fence.
The top-level keys must be schema_version, report_markdown, run_analysis, and lesson_proposals.
Required envelope: {"schema_version":2,"report_markdown":"# localized report","run_analysis":{"outcome":"defeat|victory","primary_case_id":"supplied case_id","contributing_case_ids":[],"explanation":"factual explanation","confidence_millis":0},"lesson_proposals":[]}.
Write the localized human-readable Markdown report inside report_markdown as a valid JSON string.
Never output Markdown outside the JSON object.
Treat every supplied case as an observation, never as proof of causality or optimality.
For a defeat, analyze the supplied deterministic primary case before any contributing case.
Lesson proposals must cite only supplied case_id values and match the cited cases exactly.
Never substitute an incidental observation for the terminal failure mechanism.
Follow the output contract in the user prompt. If the primary case cannot justify a narrow lesson, use an empty lesson_proposals array."##;

pub(crate) fn autoplay_action_system_prompt(locale: &Locale) -> String {
    format!(
        "{}\n\n{}",
        locale.unified_preamble, AUTOPLAY_ACTION_SYSTEM_PROMPT
    )
}

pub(crate) fn learning_postmortem_system_prompt(locale: &Locale) -> String {
    format!(
        "{}\n\n{}",
        locale.unified_preamble, LEARNING_POSTMORTEM_SYSTEM_PROMPT
    )
}
