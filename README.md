# Slay the Spire AI Copilot

A local CLI copilot for Slay the Spire. It reads game state from CommunicationMod, asks an LLM for advice, writes the latest suggestion to `output/advice.txt`, and records structured run history under `runs/<run_id>/events.jsonl` for postmortem analysis.

本项目是一个本地运行的《杀戮尖塔》AI 助手。它通过 CommunicationMod 读取游戏状态，调用 LLM 生成建议，把最新建议写入 `output/advice.txt`，并将结构化运行记录保存到 `runs/<run_id>/events.jsonl`，方便之后复盘。

## Requirements / 环境要求

- Rust 1.85+ (edition 2024), if building from source
- Slay the Spire
- ModTheSpire
- CommunicationMod

## Download Or Build / 下载或构建

### Download A Release / 下载 Release

Download the archive for your operating system from GitHub Releases, then extract it:

- `slay-the-spire-copilot-linux-x86_64.tar.gz`
- `slay-the-spire-copilot-macos.tar.gz`
- `slay-the-spire-copilot-windows-x86_64.zip`

从 GitHub Releases 下载对应系统的压缩包并解压：

- Linux: `slay-the-spire-copilot-linux-x86_64.tar.gz`
- macOS: `slay-the-spire-copilot-macos.tar.gz`
- Windows: `slay-the-spire-copilot-windows-x86_64.zip`

### Build From Source / 从源码构建

```bash
cargo build --release
```

The binary will be at:

```text
target/release/slay-the-spire-copilot
```

Windows builds produce:

```text
target/release/slay-the-spire-copilot.exe
```

### Developer Git Hook / 开发者 Git Hook

Enable the versioned pre-commit hook once per clone:

每次克隆仓库后执行一次，启用仓库内置的 pre-commit hook：

```bash
git config core.hooksPath .githooks
```

Before each commit, the hook runs the same local CI flow as GitHub Actions:

每次提交前，hook 会运行与 GitHub Actions 相同的本地 CI 流程：

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

## Environment File / 创建环境变量文件

Copy `.env.example` to `.env` in the same directory where you run the binary.

将 `.env.example` 复制为 `.env`。`.env` 应放在运行二进制文件时所在的项目目录中。

```bash
cp .env.example .env
```

Mock mode works without a real API key:

Mock 模式不需要真实 API key：

```env
LLM_PROVIDER=mock
```

For OpenAI or any OpenAI-compatible `/chat/completions` API:

如果使用 OpenAI 或任何兼容 OpenAI `/chat/completions` 的服务：

```env
LLM_PROVIDER=openai-compatible
LLM_BASE_URL=https://api.openai.com/v1
LLM_API_KEY=sk-your-key
LLM_MODEL=gpt-4o-mini
```

Optional model tiers:

可选的分层模型配置：

```env
LLM_MODEL_FAST=gpt-4o-mini
LLM_MODEL_MEDIUM=gpt-4o
LLM_MODEL_HEAVY=gpt-4o
LLM_MAX_TOKENS_FAST=3000
LLM_MAX_TOKENS_MEDIUM=10000
LLM_MAX_TOKENS_HEAVY=50000
LLM_TEMPERATURE=0.7
```

## Game Configuration / 游戏配置

Install ModTheSpire and Communication Mod CJK first. The original CommunicationMod is still supported, but the CJK build is recommended for Chinese/Japanese/Korean game text.

请先安装 ModTheSpire 和 Communication Mod CJK。原版 CommunicationMod 仍然兼容，但如果需要中文/日文/韩文游戏文本，推荐使用 CJK 版本。

1. Place `CommunicationModCJK.jar` in your ModTheSpire mods directory.
2. Start the game through ModTheSpire once with Communication Mod CJK enabled.
3. Edit Communication Mod CJK's `config.properties`.
4. Set `command` to the full path of this copilot binary.
5. Set `runAtGameStart=true`.

步骤：

1. 将 `CommunicationModCJK.jar` 放入 ModTheSpire 的 mods 目录。
2. 通过 ModTheSpire 启动一次游戏，并启用 Communication Mod CJK。
3. 编辑 Communication Mod CJK 的 `config.properties`。
4. 将 `command` 设置为本项目二进制文件的完整路径。
5. 设置 `runAtGameStart=true`。

Example / 示例：

```properties
command=/absolute/path/to/slay-the-spire-copilot
runAtGameStart=true
```

Windows example / Windows 示例：

```properties
command=C:\path\to\slay-the-spire-copilot.exe
runAtGameStart=true
```

Common config locations / 常见配置位置：

- Linux: `~/.config/ModTheSpire/CommunicationModCJK/config.properties`
- macOS: `~/Library/Preferences/ModTheSpire/CommunicationModCJK/config.properties`
- Windows: `%LOCALAPPDATA%\ModTheSpire\CommunicationModCJK\config.properties`

The app checks `CommunicationModCJK` first, then falls back to the original `CommunicationMod` config directory.

程序会优先检查 `CommunicationModCJK` 配置目录，然后再回退检查原版 `CommunicationMod` 配置目录。

The app also tries to detect and repair an empty or wrong `command=` value at startup.

程序启动时也会尝试检测并修复空的或错误的 `command=` 配置。

## Local Smoke Test / 本地快速测试

Use `--stdin-test` to bypass CommunicationMod setup and force the mock provider:

使用 `--stdin-test` 可以跳过 CommunicationMod 配置检查，并强制使用 mock provider：

```bash
echo '{"in_game":true,"game_state":{"screen_type":"CARD_REWARD","screen_state":{"cards":[{"id":"Uppercut","name":"Uppercut","cost":2,"type":"ATTACK","upgrades":0}],"skip_available":true},"class":"IRONCLAD","floor":1,"current_hp":68,"max_hp":75,"gold":99}}' \
  | cargo run --release -- --stdin-test
```

Or with a built binary:

也可以直接运行已构建的二进制：

```bash
echo '{"in_game":true,"game_state":{"screen_type":"CARD_REWARD","screen_state":{"cards":[{"id":"Uppercut","name":"Uppercut","cost":2,"type":"ATTACK","upgrades":0}],"skip_available":true},"class":"IRONCLAD","floor":1,"current_hp":68,"max_hp":75,"gold":99}}' \
  | ./target/release/slay-the-spire-copilot --stdin-test
```

Check the latest advice:

查看最新建议：

```text
output/advice.txt
```

## Runtime Behavior / 运行行为

- stdout is reserved for CommunicationMod protocol messages.
- logs go to `logs/sts-ai.log`.
- prompts and LLM responses are logged to `logs/prompts.log`.
- latest advice is written to `output/advice.txt`.
- durable run history is written to `runs/<run_id>/events.jsonl`.
- advice is generated for card rewards, rest sites, and once when entering combat.
- combat advice is intentionally entry-only for MVP to avoid high latency every turn.

运行行为：

- stdout 专门用于 CommunicationMod 协议消息。
- 普通日志写入 `logs/sts-ai.log`。
- prompt 和 LLM 回复写入 `logs/prompts.log`。
- 最新建议写入 `output/advice.txt`。
- 持久化运行记录写入 `runs/<run_id>/events.jsonl`。
- 当前会在选牌、篝火、进入战斗时生成建议。
- MVP 阶段战斗建议只在进入战斗时生成一次，避免每回合 LLM 延迟影响游戏节奏。

## Postmortem / 复盘

Generate a Markdown-style postmortem from a run journal:

从运行日志生成 Markdown 风格复盘。默认会调用当前配置的 LLM，把机器摘要改写成更适合玩家阅读的中文复盘：

```bash
slay-the-spire-copilot postmortem runs/<run_id>/events.jsonl
```

When running from source:

从源码运行：

```bash
cargo run --release -- postmortem runs/<run_id>/events.jsonl
```

To skip the AI rewrite and print the deterministic report directly:

如果想跳过 AI 改写，只输出确定性的机器复盘：

```bash
slay-the-spire-copilot postmortem --plain runs/<run_id>/events.jsonl
```

The report uses the deterministic summary as source material, then asks the LLM to produce a user-friendly Chinese review. If the AI call fails, the app falls back to the deterministic report.

复盘会先生成确定性的机器摘要，再让 LLM 改写为更友好的中文复盘。如果 AI 调用失败，程序会回退到机器摘要。

## Output Format / 建议格式

AI advice is written in Chinese:

AI 建议使用中文格式：

```text
推荐：（推荐行动）
理由：（理由）
风险：（需要注意的风险）
吐槽：（轻松评价，可选）
```

## Release Process / 发布流程

GitHub Actions automatically creates a release when a tag matching `v*` is pushed.

当推送 `v*` 格式的 tag 时，GitHub Actions 会自动创建 Release。

```bash
# Update Cargo.toml version first if needed.
git tag v0.1.0
git push origin v0.1.0
```

The release workflow builds and uploads downloadable archives for Linux, macOS, and Windows.

Release workflow 会构建并上传 Linux、macOS、Windows 的可下载压缩包。

## Project Structure / 项目结构

```text
src/
  main.rs        main loop, CLI modes, screen gating
  config.rs      environment variable loading
  protocol.rs    CommunicationMod protocol messages
  state.rs       normalized game state, advice hash, observation hash
  prompt.rs      Chinese LLM prompt builder
  llm.rs         LLM provider abstraction
  advice.rs      latest advice file output
  journal.rs     JSONL run journal
  postmortem.rs  postmortem report generation
  logging.rs     file logger
tests/
  fixtures/      sample CommunicationMod JSON states
```

## License / 许可证

MIT
