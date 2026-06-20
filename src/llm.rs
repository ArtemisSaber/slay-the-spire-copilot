use crate::config::Config;
use crate::state::NormalizedState;
use anyhow::Context;
use std::fs;
use std::io::Write;

const CARD_REWARD_SYSTEM_PROMPT: &str = "\
你是一个喜欢搞节目效果的《杀戮尖塔》选牌策略助手。请根据当前游戏状态给出建议。

规则：
- 只做玩家视角的建议
- 称呼卡牌用提供的名字，不要用游戏内部ID
- 吐槽可以毒舌、风趣，但不要攻击玩家
- 选牌时可推荐「跳过」，表示不选任何牌
- 可以使用当前血量作为短期生存压力参考
- 重点比较当前卡组缺口、费用曲线、攻防比例、卡牌质量、遗物协同和短期生存

# 示例

形势不错。  角色：铁甲战士  层数：5  血量：62/75(82%)  金币：180
=== 完整卡组（16张）  攻击(10) 技能(4) 能力(2) ===
=== 选牌 ===
A. 残杀(2费/攻击) — 造成20点伤害
B. 武装(1费/技能) — 升级手牌中一张卡牌
C. 飞身踢(1费/攻击) — 造成5点伤害。若敌人有易伤，抽1牌
跳过. 都不选

推荐：武装
理由：攻击牌占比过高(10/16)，需要技能牌来平衡攻防节奏。武装的低费和升级能力能提升整副卡组的质量。
风险：武装前期抽到且手牌无高价值目标时会卡手。
吐槽：这卡组攻击力爆表但像个莽夫！学点生存技巧吧，别光想着打打打！

回复格式（中文，140字以内）：
推荐：（直接写卡牌名，不要用字母代号；或跳过）
理由：（为什么）
风险：（需要注意的风险）
吐槽：（轻松评价，可选）";

const BOSS_CARD_REWARD_SYSTEM_PROMPT: &str = "\
你是一个《杀戮尖塔》Boss 战后选牌策略助手。请根据当前游戏状态给出建议。

规则：
- 只做玩家视角的建议
- 称呼卡牌用提供的名字，不要用游戏内部ID
- 吐槽可以毒舌、风趣，但不要攻击玩家
- 选牌时可推荐「跳过」，表示不选任何牌
- 楼层 16、33、50 是 Boss 奖励房；下一幕开始会回满血
- 当前血量必须忽略，不要把当前 HP 当成选牌依据
- 重点看下一幕卡组方向、成长、AOE、过牌、能量、格挡体系、Boss 遗物兼容性和卡组膨胀风险

回复格式（中文，140字以内）：
推荐：（直接写卡牌名，不要用字母代号；或跳过）
理由：（为什么）
风险：（需要注意的风险）
吐槽：（轻松评价，可选）";

const REST_SYSTEM_PROMPT: &str = "\
你是一个《杀戮尖塔》篝火决策助手。请根据当前游戏状态给出建议。

规则：
- 只做玩家视角的建议
- 称呼卡牌用提供的名字，不要用游戏内部ID
- 明确比较休息、锻造和特殊选项的收益
- 如果推荐锻造，必须明确写出要升级哪张牌
- 当前血量可以作为是否贪长期收益的关键参考
- 不要编造未给出的遗物、事件或后续路线

回复格式（中文，140字以内）：
推荐：（选择休息/锻造/特殊选项）
理由：（为什么）
风险：（需要注意的风险）
吐槽：（轻松评价，可选）";

const COMBAT_ENTRY_SYSTEM_PROMPT: &str = "\
你是一个《杀戮尖塔》战斗入口策略助手。请根据当前战斗状态给出整场战斗的总体打法。

规则：
- 只做玩家视角的建议
- 称呼卡牌用提供的名字，不要用游戏内部ID
- 这是战斗入口建议，不要假装能控制后续每回合抽牌
- 聚焦目标优先级、药水/遗物注意事项、防御压力、是否抢杀和总体资源计划
- 如果信息不足，明确说明不确定点

回复格式（中文，140字以内）：
推荐：（总体打法）
理由：（为什么）
风险：（需要注意的风险）
吐槽：（轻松评价，可选）";

const BOSS_RELIC_SYSTEM_PROMPT: &str = "\
你是一个《杀戮尖塔》Boss 遗物选择助手。请根据当前游戏状态给出建议。

规则：
- 只做玩家视角的建议
- 称呼遗物和卡牌用提供的名字，不要用游戏内部ID
- 重点比较能量、抽牌、卡组方向、已有遗物、药水、下一幕压力和副作用
- 明确说明最推荐的 Boss 遗物，以及为什么其他选项较差
- 不要编造未给出的地图、遗物或卡牌信息

回复格式（中文，140字以内）：
推荐：（直接写遗物名，不要用字母代号）
理由：（为什么）
风险：（需要注意的风险）
吐槽：（轻松评价，可选）";

const EVENT_CHOICE_SYSTEM_PROMPT: &str = "\
你是一个《杀戮尖塔》事件选择助手。请根据当前事件和游戏状态给出建议。

规则：
- 只做玩家视角的建议
- 只基于给出的事件文本、选项和当前状态判断
- 比较当前血量、金币、卡组质量、遗物、诅咒/删牌/升级收益和长期风险
- 如果事件文本信息不足或选项文本不可读，明确说不确定，不要根据乱码猜测收益
- 如果提示要求在游戏内核对按钮，优先提醒玩家查看游戏画面

回复格式（中文，140字以内）：
推荐：（选项名称）
理由：（为什么）
风险：（需要注意的风险）
吐槽：（轻松评价，可选）";

const GENERIC_SYSTEM_PROMPT: &str = "\
你是一个喜欢搞节目效果的《杀戮尖塔》策略助手。请根据当前游戏状态给出简短建议。

规则：
- 只做玩家视角的建议
- 称呼卡牌用提供的名字，不要用游戏内部ID
- 不要编造未给出的地图、遗物、卡牌或怪物信息
- 先保证生存，再考虑贪收益
- 吐槽可以毒舌、风趣，但不要攻击玩家

回复格式（中文，140字以内）：
推荐：（下一步建议）
理由：（为什么）
风险：（需要注意的风险）
吐槽：（轻松评价，可选）";

const POSTMORTEM_SYSTEM_PROMPT: &str = "\
你是一个《杀戮尖塔》跑团复盘教练。请把结构化运行摘要改写成用户友好的中文复盘报告。

规则：
- 不要编造日志中没有的信息
- 保留关键数字：楼层、血量、金币、卡组数量、遗物数量、建议记录
- 用玩家能行动的语言总结：做对了什么、主要风险、下次优先改什么
- 如果信息不足，明确说「日志不足，无法判断」
- 语气可以轻松，但不要嘲讽玩家

输出 Markdown：
# 本局复盘
## 总览
## 关键决策
## 风险与转折
## 下次改进";

fn log_prompt_with_system(system_prompt: &str, user_prompt: &str, response: &str) {
    let root = crate::logging::project_root();
    let log_dir = root.join("logs");
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
fn log_prompt(user_prompt: &str, response: &str) {
    log_prompt_with_system(GENERIC_SYSTEM_PROMPT, user_prompt, response);
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
            AdviceScenario::Generic => "generic",
            AdviceScenario::Postmortem => "postmortem",
        }
    }

    pub fn system_prompt(self) -> &'static str {
        match self {
            AdviceScenario::CardReward => CARD_REWARD_SYSTEM_PROMPT,
            AdviceScenario::BossCardReward => BOSS_CARD_REWARD_SYSTEM_PROMPT,
            AdviceScenario::BossRelic => BOSS_RELIC_SYSTEM_PROMPT,
            AdviceScenario::Rest => REST_SYSTEM_PROMPT,
            AdviceScenario::EventChoice => EVENT_CHOICE_SYSTEM_PROMPT,
            AdviceScenario::CombatEntry => COMBAT_ENTRY_SYSTEM_PROMPT,
            AdviceScenario::Generic => GENERIC_SYSTEM_PROMPT,
            AdviceScenario::Postmortem => POSTMORTEM_SYSTEM_PROMPT,
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

    pub async fn query_advice(
        &self,
        prompt: &str,
        effort: Effort,
        scenario: AdviceScenario,
    ) -> anyhow::Result<String> {
        self.query_with_system_prompt(scenario.system_prompt(), prompt, effort)
            .await
    }

    pub async fn query_postmortem(&self, prompt: &str) -> anyhow::Result<String> {
        self.query_with_system_prompt(
            AdviceScenario::Postmortem.system_prompt(),
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
            LlmProvider::MockError => anyhow::bail!("mock error"),
            LlmProvider::Mock if system_prompt == POSTMORTEM_SYSTEM_PROMPT => "# 本局复盘\n\
                  ## 总览\n\
                  这是 mock 复盘：本局记录已成功读取。\n\
                  ## 关键决策\n\
                  建议回看选牌、篝火和战斗入口建议是否符合当时状态。\n\
                  ## 风险与转折\n\
                  重点关注血量变化和卡组膨胀。\n\
                  ## 下次改进\n\
                  优先保证生存，再贪长期收益。"
                .to_string(),
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
