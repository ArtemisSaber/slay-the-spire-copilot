use crate::config::Config;
use anyhow::Context;

const SYSTEM_PROMPT: &str = "\
你是一个《杀戮尖塔》策略助手。根据当前游戏状态给出简洁建议。

回复格式（中文，120字以内）：
推荐：（具体行动建议）
理由：（为什么）
风险：（需要注意的风险）
吐槽：（轻松评价，可选）";

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
            "NONE" => Effort::Fast,
            _ => Effort::Medium,
        }
    }
}

pub(crate) struct OpenAiConfig {
    model: String,
    max_tokens: u32,
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

    pub async fn query(&self, prompt: &str, effort: Effort) -> anyhow::Result<String> {
        match self {
            LlmProvider::Mock => Ok("推荐：出防御牌，注意格挡。\n\
                 理由：怪物意图攻击且你HP较低。\n\
                 风险：如果不出防御牌可能被斩杀。\n\
                 吐槽：这手牌是真的烂。"
                .to_string()),
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
                        {"role": "system", "content": SYSTEM_PROMPT},
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

                let content = json["choices"][0]["message"]["content"]
                    .as_str()
                    .context("missing content in LLM response")?;

                Ok(content.to_string())
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/llm_tests.rs"]
mod tests;
