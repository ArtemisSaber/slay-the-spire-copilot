use crate::config::Config;
use crate::locales::Locale;
use crate::state::NormalizedState;
use anyhow::Context;
use std::fs;
use std::io::Write;
use std::time::Instant;

fn log_prompt_with_system(system_prompt: &str, user_prompt: &str, response: &str) {
    log_prompt_into_dir(
        &crate::logging::project_root(),
        system_prompt,
        user_prompt,
        response,
    );
}

fn log_prompt_into_dir(
    base: &std::path::Path,
    system_prompt: &str,
    user_prompt: &str,
    response: &str,
) {
    let log_dir = base.join("logs");
    let _ = fs::create_dir_all(&log_dir);
    let path = log_dir.join("prompts.log");

    let entry = format!(
        "[system]\n{system_prompt}\n\n[user]\n{user_prompt}\n\n[assistant]\n{response}\n---\n",
    );

    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = file.write_all(entry.as_bytes());
    }
}

fn sanitize_err_body(body: &str, api_key: Option<&str>) -> String {
    let redacted = match api_key {
        Some(key) if !key.is_empty() => body.replace(key, "<REDACTED>"),
        _ => body.to_string(),
    };
    let truncated: String = redacted.chars().take(500).collect();
    if redacted.len() > truncated.len() {
        format!("{truncated}...(truncated)")
    } else {
        truncated
    }
}

fn validate_base_url(raw: &str) -> anyhow::Result<String> {
    let trimmed = raw.trim_end_matches('/');
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") {
        Ok(trimmed.to_string())
    } else if let Some(scheme) = lower.split("://").next() {
        if scheme.contains(':') || scheme.is_empty() {
            anyhow::bail!("LLM_BASE_URL must be a full http(s) URL, got '{raw}'");
        }
        anyhow::bail!("LLM_BASE_URL must use http or https scheme, got '{scheme}://'");
    } else {
        anyhow::bail!("LLM_BASE_URL must be a full http(s) URL, got '{raw}'");
    }
}

#[cfg(test)]
fn log_prompt_to(base: &std::path::Path, user_prompt: &str, response: &str) {
    let locale = crate::test_utils::test_locale();
    log_prompt_into_dir(base, &locale.system_prompts.generic, user_prompt, response);
}

#[derive(Clone, Copy)]
pub enum Effort {
    Fast,
    Medium,
    Heavy,
}

impl Effort {
    pub fn from_screen_type(st: &str, in_combat: bool) -> Self {
        match st {
            "CARD_REWARD" if in_combat => Effort::Fast,
            "CARD_REWARD" => Effort::Heavy,
            "BOSS_REWARD" => Effort::Heavy,
            "MAP" => Effort::Heavy,
            "NONE" => Effort::Fast,
            "HAND_SELECT" => Effort::Fast,
            "GRID" if in_combat => Effort::Fast,
            "GRID" => Effort::Medium,
            _ if in_combat => Effort::Fast,
            _ => Effort::Medium,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Effort::Fast => "fast",
            Effort::Medium => "medium",
            Effort::Heavy => "heavy",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdviceScenario {
    CardReward,
    BossCardReward,
    BossRelic,
    Rest,
    EventChoice,
    Shop,
    CombatEntry,
    MapSuggestion,
    MapCrossroad,
    Generic,
    Postmortem,
}

impl AdviceScenario {
    pub fn from_state(state: &NormalizedState) -> Self {
        match state.screen_type.as_deref() {
            Some("CARD_REWARD") if state.is_boss_card_reward() => AdviceScenario::BossCardReward,
            Some("CARD_REWARD") => AdviceScenario::CardReward,
            Some("BOSS_REWARD") => AdviceScenario::BossRelic,
            Some("REST") => AdviceScenario::Rest,
            Some("EVENT") => AdviceScenario::EventChoice,
            Some("SHOP_SCREEN") => AdviceScenario::Shop,
            Some("MAP") if state.map_first_node_chosen == Some(true) => {
                AdviceScenario::MapCrossroad
            }
            Some("MAP") => AdviceScenario::MapSuggestion,
            _ if state.has_active_monsters() => AdviceScenario::CombatEntry,
            _ => AdviceScenario::Generic,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            AdviceScenario::CardReward => "card_reward",
            AdviceScenario::BossCardReward => "boss_card_reward",
            AdviceScenario::BossRelic => "boss_relic",
            AdviceScenario::Rest => "rest",
            AdviceScenario::EventChoice => "event_choice",
            AdviceScenario::Shop => "shop",
            AdviceScenario::CombatEntry => "combat_entry",
            AdviceScenario::MapSuggestion => "map_suggestion",
            AdviceScenario::MapCrossroad => "map_crossroad",
            AdviceScenario::Generic => "generic",
            AdviceScenario::Postmortem => "postmortem",
        }
    }

    pub fn system_prompt(self, locale: &Locale) -> String {
        if matches!(self, AdviceScenario::Postmortem) {
            return format!(
                "{}\n\n{}",
                &locale.system_prompts.postmortem,
                self.few_shot_example(locale)
            );
        }
        unified_system_prompt(locale)
    }

    fn few_shot_example(self, locale: &Locale) -> &str {
        match self {
            AdviceScenario::CardReward => &locale.few_shot_examples.card_reward,
            AdviceScenario::BossCardReward => &locale.few_shot_examples.boss_card_reward,
            AdviceScenario::BossRelic => &locale.few_shot_examples.boss_relic,
            AdviceScenario::Rest => &locale.few_shot_examples.rest,
            AdviceScenario::EventChoice => &locale.few_shot_examples.event_choice,
            AdviceScenario::Shop => &locale.few_shot_examples.shop,
            AdviceScenario::CombatEntry => &locale.few_shot_examples.combat_entry,
            AdviceScenario::MapSuggestion => &locale.few_shot_examples.map_suggestion,
            AdviceScenario::MapCrossroad => &locale.few_shot_examples.map_crossroad,
            AdviceScenario::Generic => &locale.few_shot_examples.generic,
            AdviceScenario::Postmortem => &locale.few_shot_examples.postmortem,
        }
    }
}

fn unified_system_prompt(locale: &Locale) -> String {
    let mut out = vec![locale.unified_preamble.clone()];
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
    for (mode, sp, fs) in modes {
        out.push(format!("\n---\n\n[mode: {mode}]\n{sp}\n\n{fs}"));
    }
    out.concat()
}

const AUTOPLAY_ACTION_SYSTEM_PROMPT: &str = r#"AUTO_PLAY_ACTION_PLANNER
You are the Slay the Spire auto-play action planner.
Return strict JSON only, with this shape:
{"schema_version":1,"actions":[{"kind":"choose|skip|proceed|play|end|leave","action_id":"...","target_index":0,"label":"...","reason":"...","risk":"..."}]}
Choose exactly one action_id from the provided available_actions.
Never invent an action_id. Never output prose or Markdown.
For targeted combat cards, include target_index. If no available action is safe, choose an available non-destructive exit such as end/leave/proceed when present."#;

fn autoplay_action_system_prompt(locale: &Locale) -> String {
    format!(
        "{}\n\n{}",
        locale.unified_preamble, AUTOPLAY_ACTION_SYSTEM_PROMPT
    )
}

#[derive(Debug)]
pub(crate) struct OpenAiConfig {
    model: String,
    max_tokens: u32,
    disable_thinking: bool,
}

fn chat_completion_body(
    cfg: &OpenAiConfig,
    system_prompt: &str,
    prompt: &str,
    temperature: f64,
) -> serde_json::Value {
    let mut body = serde_json::json!({
        "model": cfg.model,
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": prompt}
        ],
        "max_tokens": cfg.max_tokens,
        "temperature": temperature
    });
    if cfg.disable_thinking {
        body["thinking"] = serde_json::json!({ "type": "disabled" });
    }
    body
}

fn anthropic_messages_body(
    cfg: &OpenAiConfig,
    system_prompt: &str,
    prompt: &str,
) -> serde_json::Value {
    serde_json::json!({
        "model": cfg.model,
        "system": system_prompt,
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "max_tokens": cfg.max_tokens
    })
}

fn anthropic_response_text(json: &serde_json::Value) -> anyhow::Result<String> {
    let content = json["content"]
        .as_array()
        .context("missing content in Anthropic response")?;
    let text = content
        .iter()
        .filter_map(|block| {
            (block["type"].as_str() == Some("text"))
                .then(|| block["text"].as_str())
                .flatten()
        })
        .collect::<Vec<_>>()
        .join("");
    if text.is_empty() {
        anyhow::bail!("missing text content in Anthropic response");
    }
    Ok(text)
}

fn chat_response_text(json: &serde_json::Value) -> anyhow::Result<String> {
    json["choices"][0]["message"]["content"]
        .as_str()
        .context("missing content in LLM response")
        .map(ToOwned::to_owned)
}

fn mock_autoplay_action_response(prompt: &str) -> String {
    let prompt_json: serde_json::Value = serde_json::from_str(prompt).unwrap_or_default();
    let localized_status_context = prompt_json
        .get("localized_status_context")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let has_rejections = prompt_json
        .get("rejected_attempts")
        .and_then(|v| v.as_array())
        .is_some_and(|arr| !arr.is_empty());

    if localized_status_context.contains("fallback_test_marker") {
        return r#"{"schema_version":1,"actions":[{"kind":"choose","action_id":"event:99","label":"Invalid","reason":"","risk":""}]}"#.to_string();
    }

    if localized_status_context.contains("retry_test_marker") && !has_rejections {
        return r#"{"schema_version":1,"actions":[{"kind":"choose","action_id":"event:99","label":"Invalid","reason":"","risk":""}]}"#.to_string();
    }

    let actions = prompt_json
        .get("available_actions")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let preferred = actions
        .iter()
        .find(|action| action.get("action_id").and_then(|v| v.as_str()) == Some("card_reward:skip"))
        .or_else(|| {
            actions
                .iter()
                .find(|action| action.get("kind").and_then(|v| v.as_str()) == Some("play"))
        })
        .or_else(|| actions.first());

    let Some(action) = preferred else {
        return r#"{"schema_version":1,"actions":[]}"#.to_string();
    };

    let kind = action
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("choose");
    let action_id = action
        .get("action_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let target_index = if action
        .get("target_required")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        serde_json::json!(0)
    } else {
        serde_json::Value::Null
    };
    let label = action
        .get("label")
        .and_then(|v| v.as_str())
        .unwrap_or("Mock action");

    serde_json::json!({
        "schema_version": 1,
        "actions": [{
            "kind": kind,
            "action_id": action_id,
            "target_index": target_index,
            "label": label,
            "reason": "Mock auto-play planner selected the first preferred available action.",
            "risk": ""
        }]
    })
    .to_string()
}

pub enum LlmProvider {
    Mock,
    OpenAiCompatible {
        base_url: String,
        api_key: String,
        temperature: f64,
        client: reqwest::Client,
        fast: OpenAiConfig,
        medium: OpenAiConfig,
        heavy: OpenAiConfig,
    },
    PollinationsFree {
        base_url: String,
        temperature: f64,
        client: reqwest::Client,
        fast: OpenAiConfig,
        medium: OpenAiConfig,
        heavy: OpenAiConfig,
    },
    Anthropic {
        base_url: String,
        api_key: String,
        client: reqwest::Client,
        fast: OpenAiConfig,
        medium: OpenAiConfig,
        heavy: OpenAiConfig,
    },
}

impl std::fmt::Debug for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mock => f.debug_struct("Mock").finish(),
            Self::OpenAiCompatible {
                base_url,
                api_key: _,
                temperature,
                client,
                fast,
                medium,
                heavy,
            } => f
                .debug_struct("OpenAiCompatible")
                .field("base_url", base_url)
                .field("api_key", &"<REDACTED>")
                .field("temperature", temperature)
                .field("client", client)
                .field("fast", fast)
                .field("medium", medium)
                .field("heavy", heavy)
                .finish(),
            Self::PollinationsFree {
                base_url,
                temperature,
                client,
                fast,
                medium,
                heavy,
            } => f
                .debug_struct("PollinationsFree")
                .field("base_url", base_url)
                .field("temperature", temperature)
                .field("client", client)
                .field("fast", fast)
                .field("medium", medium)
                .field("heavy", heavy)
                .finish(),
            Self::Anthropic {
                base_url,
                api_key: _,
                client,
                fast,
                medium,
                heavy,
            } => f
                .debug_struct("Anthropic")
                .field("base_url", base_url)
                .field("api_key", &"<REDACTED>")
                .field("client", client)
                .field("fast", fast)
                .field("medium", medium)
                .field("heavy", heavy)
                .finish(),
        }
    }
}

impl LlmProvider {
    pub fn from_config(config: &Config) -> anyhow::Result<Self> {
        match config.provider.as_str() {
            "mock" => Ok(LlmProvider::Mock),
            "openai-compatible" => {
                let raw = config
                    .base_url
                    .as_ref()
                    .context("LLM_BASE_URL is required for openai-compatible provider")?;
                let base_url = validate_base_url(raw)?;
                let api_key = config
                    .api_key
                    .as_ref()
                    .context("LLM_API_KEY is required for openai-compatible provider")?
                    .clone();
                Ok(LlmProvider::OpenAiCompatible {
                    base_url,
                    api_key,
                    temperature: config.temperature,
                    client: reqwest::Client::new(),
                    fast: OpenAiConfig {
                        model: config.model_fast.clone(),
                        max_tokens: config.max_tokens_fast,
                        disable_thinking: config.disable_fast_thinking,
                    },
                    medium: OpenAiConfig {
                        model: config.model_medium.clone(),
                        max_tokens: config.max_tokens_medium,
                        disable_thinking: false,
                    },
                    heavy: OpenAiConfig {
                        model: config.model_heavy.clone(),
                        max_tokens: config.max_tokens_heavy,
                        disable_thinking: false,
                    },
                })
            }
            "pollinations-free" => {
                let base_url = validate_base_url(
                    config
                        .base_url
                        .as_deref()
                        .unwrap_or("https://text.pollinations.ai/openai"),
                )?;
                Ok(LlmProvider::PollinationsFree {
                    base_url,
                    temperature: config.temperature,
                    client: reqwest::Client::new(),
                    fast: OpenAiConfig {
                        model: config.model_fast.clone(),
                        max_tokens: config.max_tokens_fast,
                        disable_thinking: false,
                    },
                    medium: OpenAiConfig {
                        model: config.model_medium.clone(),
                        max_tokens: config.max_tokens_medium,
                        disable_thinking: false,
                    },
                    heavy: OpenAiConfig {
                        model: config.model_heavy.clone(),
                        max_tokens: config.max_tokens_heavy,
                        disable_thinking: false,
                    },
                })
            }
            "anthropic" => {
                let base_url = validate_base_url(
                    config
                        .base_url
                        .as_deref()
                        .unwrap_or("https://api.anthropic.com"),
                )?;
                let api_key = config
                    .api_key
                    .as_ref()
                    .context("LLM_API_KEY is required for anthropic provider")?
                    .clone();
                Ok(LlmProvider::Anthropic {
                    base_url,
                    api_key,
                    client: reqwest::Client::new(),
                    fast: OpenAiConfig {
                        model: config.model_fast.clone(),
                        max_tokens: config.max_tokens_fast,
                        disable_thinking: false,
                    },
                    medium: OpenAiConfig {
                        model: config.model_medium.clone(),
                        max_tokens: config.max_tokens_medium,
                        disable_thinking: false,
                    },
                    heavy: OpenAiConfig {
                        model: config.model_heavy.clone(),
                        max_tokens: config.max_tokens_heavy,
                        disable_thinking: false,
                    },
                })
            }
            other => anyhow::bail!("unknown LLM_PROVIDER: {other}"),
        }
    }

    pub async fn query_advice(
        &self,
        prompt: &str,
        effort: Effort,
        scenario: AdviceScenario,
        locale: &Locale,
    ) -> anyhow::Result<String> {
        let system_prompt = scenario.system_prompt(locale);
        self.query_with_system_prompt(&system_prompt, prompt, effort)
            .await
    }

    pub async fn query_postmortem(&self, prompt: &str, locale: &Locale) -> anyhow::Result<String> {
        let system_prompt = AdviceScenario::Postmortem.system_prompt(locale);
        self.query_with_system_prompt(&system_prompt, prompt, Effort::Heavy)
            .await
    }

    pub async fn query_autoplay_action(
        &self,
        prompt: &str,
        effort: Effort,
        locale: &Locale,
    ) -> anyhow::Result<String> {
        let system_prompt = autoplay_action_system_prompt(locale);
        self.query_with_system_prompt(&system_prompt, prompt, effort)
            .await
    }

    async fn query_with_system_prompt(
        &self,
        system_prompt: &str,
        prompt: &str,
        effort: Effort,
    ) -> anyhow::Result<String> {
        let result: String = match self {
            LlmProvider::Mock if prompt == "TRIGGER_LLM_ERROR" => anyhow::bail!("mock error"),
            LlmProvider::Mock if system_prompt.contains("AUTO_PLAY_ACTION_PLANNER") => {
                mock_autoplay_action_response(prompt)
            }
            LlmProvider::Mock if system_prompt.contains("## 总览") || system_prompt.contains("## Overview") => {
                "# 本局复盘\n## 总览\n这是 mock 复盘。\n## 关键决策\n回看选牌、篝火和战斗入口建议。\n## 风险与转折\n关注血量变化和卡组膨胀。\n## 下次改进\n优先保证生存，再贪长期收益。"
                    .to_string()
            }
            LlmProvider::Mock => "推荐：出防御牌，注意格挡。\n\
                   理由：怪物意图攻击且你HP较低。\n\
                   风险：如果不出防御牌可能被斩杀。\n\
                   吐槽：这手牌是真的烂。"
                .to_string(),
            LlmProvider::OpenAiCompatible {
                base_url,
                api_key,
                temperature,
                client,
                fast,
                medium,
                heavy,
            } => {
                let cfg = match effort {
                    Effort::Fast => fast,
                    Effort::Medium => medium,
                    Effort::Heavy => heavy,
                };

                let url = format!("{base_url}/chat/completions");
                let started = Instant::now();
                tracing::info!(
                    "LLM request effort={} model={} prompt_chars={} system_chars={} max_tokens={} disable_thinking={}",
                    effort.as_str(),
                    cfg.model,
                    prompt.len(),
                    system_prompt.len(),
                    cfg.max_tokens,
                    cfg.disable_thinking,
                );

                let body = chat_completion_body(cfg, system_prompt, prompt, *temperature);

                let response = client
                    .post(&url)
                    .header("Authorization", format!("Bearer {api_key}"))
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await
                    .context("failed to send LLM request")?;

                let status = response.status();
                if !status.is_success() {
                    let err_body = response.text().await.unwrap_or_default();
                    let err_body = sanitize_err_body(&err_body, Some(api_key));
                    anyhow::bail!("LLM API error {status}: {err_body}");
                }

                let json: serde_json::Value = response
                    .json()
                    .await
                    .context("failed to parse LLM response")?;

                let content = chat_response_text(&json)?;
                tracing::info!(
                    "LLM response effort={} model={} duration_ms={} response_chars={}",
                    effort.as_str(),
                    cfg.model,
                    started.elapsed().as_millis(),
                    content.len(),
                );
                content
            }
            LlmProvider::PollinationsFree {
                base_url,
                temperature,
                client,
                fast,
                medium,
                heavy,
            } => {
                let cfg = match effort {
                    Effort::Fast => fast,
                    Effort::Medium => medium,
                    Effort::Heavy => heavy,
                };

                let started = Instant::now();
                tracing::info!(
                    "Pollinations free request effort={} model={} prompt_chars={} system_chars={} max_tokens={}",
                    effort.as_str(),
                    cfg.model,
                    prompt.len(),
                    system_prompt.len(),
                    cfg.max_tokens,
                );

                let body = chat_completion_body(cfg, system_prompt, prompt, *temperature);
                let response = client
                    .post(base_url.as_str())
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await
                    .context("failed to send Pollinations free request")?;

                let status = response.status();
                if !status.is_success() {
                    let err_body = response.text().await.unwrap_or_default();
                    let err_body = sanitize_err_body(&err_body, None);
                    anyhow::bail!("Pollinations free API error {status}: {err_body}");
                }

                let json: serde_json::Value = response
                    .json()
                    .await
                    .context("failed to parse Pollinations free response")?;
                let content = chat_response_text(&json)?;
                tracing::info!(
                    "Pollinations free response effort={} model={} duration_ms={} response_chars={}",
                    effort.as_str(),
                    cfg.model,
                    started.elapsed().as_millis(),
                    content.len(),
                );
                content
            }
            LlmProvider::Anthropic {
                base_url,
                api_key,
                client,
                fast,
                medium,
                heavy,
            } => {
                let cfg = match effort {
                    Effort::Fast => fast,
                    Effort::Medium => medium,
                    Effort::Heavy => heavy,
                };

                let url = format!("{base_url}/v1/messages");
                let started = Instant::now();
                tracing::info!(
                    "Anthropic request effort={} model={} prompt_chars={} system_chars={} max_tokens={}",
                    effort.as_str(),
                    cfg.model,
                    prompt.len(),
                    system_prompt.len(),
                    cfg.max_tokens,
                );

                let body = anthropic_messages_body(cfg, system_prompt, prompt);
                let response = client
                    .post(&url)
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .header("Content-Type", "application/json")
                    .json(&body)
                    .send()
                    .await
                    .context("failed to send Anthropic request")?;

                let status = response.status();
                if !status.is_success() {
                    let err_body = response.text().await.unwrap_or_default();
                    let err_body = sanitize_err_body(&err_body, Some(api_key));
                    anyhow::bail!("Anthropic API error {status}: {err_body}");
                }

                let json: serde_json::Value = response
                    .json()
                    .await
                    .context("failed to parse Anthropic response")?;
                let content = anthropic_response_text(&json)?;
                tracing::info!(
                    "Anthropic response effort={} model={} duration_ms={} response_chars={}",
                    effort.as_str(),
                    cfg.model,
                    started.elapsed().as_millis(),
                    content.len(),
                );
                content
            }
        };
        log_prompt_with_system(system_prompt, prompt, &result);
        Ok(result)
    }
}

#[cfg(test)]
#[path = "tests/llm_tests.rs"]
mod tests;
