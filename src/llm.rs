use crate::config::Config;
use crate::locales::Locale;
use crate::state::NormalizedState;
use anyhow::Context;
use std::fs;
use std::io::Write;

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
    pub fn from_screen_type(st: &str) -> Self {
        match st {
            "CARD_REWARD" => Effort::Heavy,
            "BOSS_REWARD" => Effort::Heavy,
            "NONE" => Effort::Fast,
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
            AdviceScenario::CombatEntry => "combat_entry",
            AdviceScenario::MapSuggestion => "map_suggestion",
            AdviceScenario::MapCrossroad => "map_crossroad",
            AdviceScenario::Generic => "generic",
            AdviceScenario::Postmortem => "postmortem",
        }
    }

    pub fn system_prompt(self, locale: &Locale) -> &str {
        match self {
            AdviceScenario::CardReward => &locale.system_prompts.card_reward,
            AdviceScenario::BossCardReward => &locale.system_prompts.boss_card_reward,
            AdviceScenario::BossRelic => &locale.system_prompts.boss_relic,
            AdviceScenario::Rest => &locale.system_prompts.rest,
            AdviceScenario::EventChoice => &locale.system_prompts.event_choice,
            AdviceScenario::CombatEntry => &locale.system_prompts.combat_entry,
            AdviceScenario::MapSuggestion => &locale.system_prompts.map_suggestion,
            AdviceScenario::MapCrossroad => &locale.system_prompts.map_crossroad,
            AdviceScenario::Generic => &locale.system_prompts.generic,
            AdviceScenario::Postmortem => &locale.system_prompts.postmortem,
        }
    }
}

#[derive(Debug)]
pub(crate) struct OpenAiConfig {
    model: String,
    max_tokens: u32,
}

#[derive(Debug)]
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
}

impl LlmProvider {
    pub fn from_config(config: &Config) -> anyhow::Result<Self> {
        match config.provider.as_str() {
            "mock" => Ok(LlmProvider::Mock),
            "openai-compatible" => {
                let base_url = config
                    .base_url
                    .as_ref()
                    .context("LLM_BASE_URL is required for openai-compatible provider")?
                    .clone();
                let api_key = config
                    .api_key
                    .as_ref()
                    .context("LLM_API_KEY is required for openai-compatible provider")?
                    .clone();
                Ok(LlmProvider::OpenAiCompatible {
                    base_url: base_url.trim_end_matches('/').to_string(),
                    api_key,
                    temperature: config.temperature,
                    client: reqwest::Client::new(),
                    fast: OpenAiConfig {
                        model: config.model_fast.clone(),
                        max_tokens: config.max_tokens_fast,
                    },
                    medium: OpenAiConfig {
                        model: config.model_medium.clone(),
                        max_tokens: config.max_tokens_medium,
                    },
                    heavy: OpenAiConfig {
                        model: config.model_heavy.clone(),
                        max_tokens: config.max_tokens_heavy,
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
        self.query_with_system_prompt(scenario.system_prompt(locale), prompt, effort)
            .await
    }

    pub async fn query_postmortem(&self, prompt: &str, locale: &Locale) -> anyhow::Result<String> {
        self.query_with_system_prompt(
            AdviceScenario::Postmortem.system_prompt(locale),
            prompt,
            Effort::Heavy,
        )
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

                let body = serde_json::json!({
                    "model": cfg.model,
                    "messages": [
                        {"role": "system", "content": system_prompt},
                        {"role": "user", "content": prompt}
                    ],
                    "max_tokens": cfg.max_tokens,
                    "temperature": temperature
                });

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
                    anyhow::bail!("LLM API error {status}: {err_body}");
                }

                let json: serde_json::Value = response
                    .json()
                    .await
                    .context("failed to parse LLM response")?;

                json["choices"][0]["message"]["content"]
                    .as_str()
                    .context("missing content in LLM response")?
                    .to_string()
            }
        };
        log_prompt_with_system(system_prompt, prompt, &result);
        Ok(result)
    }
}

#[cfg(test)]
#[path = "tests/llm_tests.rs"]
mod tests;
