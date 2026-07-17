mod configuration;
mod debug;
#[cfg(test)]
mod testing;
mod transport;

use super::types::{AdviceScenario, Effort};
use crate::config::Config;
use crate::locales::Locale;
#[cfg(test)]
pub(crate) use testing::RecordedRequest;

#[derive(Debug)]
pub(crate) struct OpenAiConfig {
    pub(crate) model: String,
    pub(crate) max_tokens: u32,
    pub(crate) disable_thinking: bool,
}

pub enum LlmProvider {
    Mock,
    #[cfg(test)]
    Scripted(testing::ScriptedProvider),
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

impl LlmProvider {
    pub fn from_config(config: &Config) -> anyhow::Result<Self> {
        configuration::provider_from_config(config)
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

    pub async fn query_learning_postmortem(
        &self,
        prompt: &str,
        locale: &Locale,
    ) -> anyhow::Result<String> {
        let system_prompt = super::prompts::learning_postmortem_system_prompt(locale);
        self.query_with_system_prompt(&system_prompt, prompt, Effort::Heavy)
            .await
    }

    pub async fn query_autoplay_action(
        &self,
        prompt: &str,
        effort: Effort,
        locale: &Locale,
    ) -> anyhow::Result<String> {
        let system_prompt = super::prompts::autoplay_action_system_prompt(locale);
        self.query_with_system_prompt(&system_prompt, prompt, effort)
            .await
    }

    pub async fn query_card_reward_candidate(&self, prompt: &str) -> anyhow::Result<String> {
        self.query_with_system_prompt(
            super::prompts::card_reward_candidate_system_prompt(),
            prompt,
            Effort::Medium,
        )
        .await
    }

    pub async fn query_card_reward_comparison(&self, prompt: &str) -> anyhow::Result<String> {
        self.query_with_system_prompt(
            super::prompts::card_reward_comparison_system_prompt(),
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
        let result = match self {
            LlmProvider::Mock if prompt == "TRIGGER_LLM_ERROR" => anyhow::bail!("mock error"),
            LlmProvider::Mock if system_prompt.contains("CARD_REWARD_CANDIDATE_SELECTOR_V1") => {
                super::mock::mock_card_reward_candidate_response(prompt)
            }
            LlmProvider::Mock if system_prompt.contains("CARD_REWARD_RESULTING_STATE_JUDGE_V1") => {
                super::mock::mock_card_reward_comparison_response(prompt)
            }
            LlmProvider::Mock if system_prompt.contains("AUTO_PLAY_ACTION_PLANNER") => {
                super::mock::mock_autoplay_action_response(prompt)
            }
            LlmProvider::Mock => super::mock::mock_advice_response(system_prompt, prompt),
            #[cfg(test)]
            LlmProvider::Scripted(provider) => provider.query(system_prompt, prompt, effort)?,
            LlmProvider::OpenAiCompatible {
                base_url,
                api_key,
                temperature,
                client,
                fast,
                medium,
                heavy,
            } => {
                transport::query_openai_compatible(transport::ProviderRequest {
                    base_url,
                    api_key: Some(api_key),
                    temperature: *temperature,
                    client,
                    fast,
                    medium,
                    heavy,
                    system_prompt,
                    prompt,
                    effort,
                })
                .await?
            }
            LlmProvider::PollinationsFree {
                base_url,
                temperature,
                client,
                fast,
                medium,
                heavy,
            } => {
                transport::query_pollinations_free(transport::ProviderRequest {
                    base_url,
                    api_key: None,
                    temperature: *temperature,
                    client,
                    fast,
                    medium,
                    heavy,
                    system_prompt,
                    prompt,
                    effort,
                })
                .await?
            }
            LlmProvider::Anthropic {
                base_url,
                api_key,
                client,
                fast,
                medium,
                heavy,
            } => {
                transport::query_anthropic(transport::ProviderRequest {
                    base_url,
                    api_key: Some(api_key),
                    temperature: 0.0,
                    client,
                    fast,
                    medium,
                    heavy,
                    system_prompt,
                    prompt,
                    effort,
                })
                .await?
            }
        };

        let system_prompt = system_prompt.to_string();
        let prompt = prompt.to_string();
        let response = result.clone();
        tokio::task::spawn_blocking(move || {
            super::logging::log_prompt_with_system(&system_prompt, &prompt, &response);
        })
        .await
        .ok();
        Ok(result)
    }
}
