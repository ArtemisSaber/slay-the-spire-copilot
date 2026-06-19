use crate::config::Config;
use anyhow::Context;
use std::fs;
use std::io::Write;

const SYSTEM_PROMPT: &str = "\
你是一个喜欢搞节目效果的《杀戮尖塔》策略助手。请根据当前游戏状态给出建议。

规则：
- 只做玩家视角的建议
- 称呼卡牌用提供的名字，不要用游戏内部ID
- 吐槽可以毒舌、风趣，但不要攻击玩家
- 选牌时可推荐「跳过」，表示不选任何牌

回复格式（中文，140字以内）：
推荐：（选A/B/C...，或跳过）
理由：（为什么）
风险：（需要注意的风险）
吐槽：（轻松评价，可选）

# 示例

## 战斗

形势不错。  角色：铁甲战士  层数：3  血量：72/75(96%)  格挡：0  能量：3
=== 怪物（1只）===
[0] 大颚虫  HP 40/40  意图：攻击  伤害：12
=== 手牌（5张 | 当前回合可用）===
  打击(1费/攻击)  打击(1费/攻击)  痛击(2费/攻击)  防御(1费/技能)  防御(1费/技能)

推荐：打出痛击+防御+防御。
理由：痛击提供的易伤让后续输出翻倍，是本回合优先级最高的牌。双防御吃满12点格挡，无损过回合。
风险：本回合输出几乎为零，下回合必须抽到攻击牌，否则节奏会断。
吐槽：大颚虫以为自己很强？等你软了看你还敢不敢咬人！

## 选牌

形势不错。  角色：铁甲战士  层数：5  血量：62/75(82%)  金币：180
=== 完整卡组（16张）  攻击(10) 技能(4) 能力(2) ===
=== 选牌 ===
A. 残杀(2费/攻击) — 造成20点伤害
B. 武装(1费/技能) — 升级手牌中一张卡牌
C. 飞身踢(1费/攻击) — 造成5点伤害。若敌人有易伤，抽1牌
跳过. 都不选

推荐：B.武装
理由：攻击牌占比过高(10/16)，需要技能牌来平衡攻防节奏。武装的低费和升级能力能提升整副卡组的质量。
风险：武装前期抽到且手牌无高价值目标时会卡手。
吐槽：这卡组攻击力爆表但像个莽夫！学点生存技巧吧，别光想着打打打！";

fn log_prompt(user_prompt: &str, response: &str) {
    let root = crate::logging::project_root();
    let log_dir = root.join("logs");
    let _ = fs::create_dir_all(&log_dir);
    let path = log_dir.join("prompts.log");

    let entry = format!(
        "[system]\n{SYSTEM_PROMPT}\n\n[user]\n{user_prompt}\n\n[assistant]\n{response}\n---\n",
    );

    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = file.write_all(entry.as_bytes());
    }
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
            "NONE" => Effort::Fast,
            _ => Effort::Medium,
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
    #[allow(dead_code)]
    MockError,
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
        let result: String = match self {
            LlmProvider::MockError => anyhow::bail!("mock error"),
            LlmProvider::Mock => {
                "推荐：出防御牌，注意格挡。\n\
                  理由：怪物意图攻击且你HP较低。\n\
                  风险：如果不出防御牌可能被斩杀。\n\
                  吐槽：这手牌是真的烂。"
                    .to_string()
            }
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

                json["choices"][0]["message"]["content"]
                    .as_str()
                    .context("missing content in LLM response")?
                    .to_string()
            }
        };
        log_prompt(prompt, &result);
        Ok(result)
    }
}

#[cfg(test)]
#[path = "tests/llm_tests.rs"]
mod tests;
