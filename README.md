# Slay the Spire AI Copilot

A local CLI copilot for Slay the Spire. It reads game state from Communication Mod CJK (recommended) or the original CommunicationMod, asks an LLM for advice, writes the latest suggestion to `output/advice.txt` (plain text) and `output/overlay.json` (structured JSON), and records structured run history under `runs/<run_id>/events.jsonl` for postmortem analysis.

本项目是一个本地运行的《杀戮尖塔》AI 助手。它通过 Communication Mod CJK（推荐）或原版 CommunicationMod 读取游戏状态，调用 LLM 生成建议，把最新建议写入 `output/advice.txt`（纯文本）和 `output/overlay.json`（结构化 JSON），并将结构化运行记录保存到 `runs/<run_id>/events.jsonl`，方便之后复盘。

## Requirements / 环境要求

- Rust 1.85+ (edition 2024), if building from source
- Slay the Spire
- ModTheSpire
- Communication Mod CJK (recommended) or CommunicationMod

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
LLM_MAX_TOKENS_FAST=300
LLM_MAX_TOKENS_MEDIUM=10000
LLM_MAX_TOKENS_HEAVY=50000
LLM_TEMPERATURE=0.7
LLM_DISABLE_FAST_THINKING=true  # DeepSeek: force non-thinking mode for combat/fast advice
```

## Game Configuration / 游戏配置

Install ModTheSpire and Communication Mod CJK first. The original CommunicationMod is still supported, but the CJK build is recommended for Chinese/Japanese/Korean game text.

请先安装 ModTheSpire 和 Communication Mod CJK。原版 CommunicationMod 仍然兼容，但如果需要中文/日文/韩文游戏文本，推荐使用 CJK 版本。

Communication Mod CJK is available from:

- Steam Workshop: <https://steamcommunity.com/sharedfiles/filedetails/?id=3748153752>
- GitHub releases: <https://github.com/ArtemisSaber/CommunicationMod/releases>

Communication Mod CJK 可以从这里获取：

- Steam 创意工坊：<https://steamcommunity.com/sharedfiles/filedetails/?id=3748153752>
- GitHub Releases：<https://github.com/ArtemisSaber/CommunicationMod/releases>

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
command="C:\path with spaces\slay-the-spire-copilot.exe"
runAtGameStart=true
```

Quote the Windows `command=` value if the path contains spaces. The startup checker also understands quoted paths and unquoted `.exe` paths with spaces.

如果 Windows 路径里有空格，请给 `command=` 的值加引号。启动检查也支持带引号的路径，以及未加引号但以 `.exe` 结尾的 Windows 路径。

Common config locations / 常见配置位置：

- Linux: `~/.config/ModTheSpire/CommunicationModCJK/config.properties`
- macOS: `~/Library/Preferences/ModTheSpire/CommunicationModCJK/config.properties`
- Windows: `%LOCALAPPDATA%\ModTheSpire\CommunicationModCJK\config.properties`

The app checks `CommunicationModCJK` first, then falls back to the original `CommunicationMod` config directory.

程序会优先检查 `CommunicationModCJK` 配置目录，然后再回退检查原版 `CommunicationMod` 配置目录。

If the game language is Chinese, Japanese, or Korean, the app will prompt you to use Communication Mod CJK instead of the original CommunicationMod. It checks Slay the Spire's `preferences/STSGameplaySettings` file first, especially the `LANGUAGE` value such as `ZHS`, `ZHT`, `JPN`, or `KOR`.

如果游戏语言是中文、日文或韩文，程序会提示你改用 Communication Mod CJK，而不是原版 CommunicationMod。程序会优先读取 Slay the Spire 的 `preferences/STSGameplaySettings` 文件，尤其是其中的 `LANGUAGE` 值，例如 `ZHS`、`ZHT`、`JPN` 或 `KOR`。

For non-standard install locations, set `SLAY_THE_SPIRE_DIR` to the game install directory. For manual testing, `SLAY_THE_SPIRE_LANGUAGE=ZHS` can force the CJK-language startup check.

如果游戏安装在非标准位置，可以将 `SLAY_THE_SPIRE_DIR` 设置为游戏安装目录。手动测试时，也可以用 `SLAY_THE_SPIRE_LANGUAGE=ZHS` 强制触发 CJK 语言启动检查。

PowerShell example / PowerShell 示例：

```powershell
$env:SLAY_THE_SPIRE_DIR="C:\Program Files (x86)\Steam\steamapps\common\SlayTheSpire"
$env:SLAY_THE_SPIRE_LANGUAGE="ZHS"
```

The app also tries to detect and repair an empty or wrong `command=` value at startup.

程序启动时也会尝试检测并修复空的或错误的 `command=` 配置。

## Local Smoke Test / 本地快速测试

Use `--stdin-test` to bypass CommunicationMod setup and force the mock provider. Use `--no-startup-check` to skip config validation only, or set `SKIP_COMM_CONFIG=1`.

使用 `--stdin-test` 可以跳过 CommunicationMod 配置检查，并强制使用 mock provider。使用 `--no-startup-check` 仅跳过配置校验，或设置 `SKIP_COMM_CONFIG=1`。

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
output/advice.txt    (plain text, backward compatible)
output/overlay.json  (structured JSON, for overlay mod)
```

## Runtime Behavior / 运行行为

- stdout is reserved for CommunicationMod protocol messages.
- logs go to `logs/sts-ai.log`.
- prompts and LLM responses are logged to `logs/prompts.log`.
- latest advice is written to `output/advice.txt` (plain text) and `output/overlay.json` (structured JSON).
- durable run history is written to `runs/<run_id>/events.jsonl`.
- when the run ends, a postmortem report is automatically written to `runs/<run_id>/postmortem.md`.
- advice is generated for card rewards, boss card rewards, boss relics, rest sites, events (with multiple choices), and once when entering combat.
- map route advice is generated at act entry (full route analysis) and at mid-act crossroads (immediate next-node decisions).
- advice uses tiered model routing: Heavy for card/boss rewards, Medium for rest/events/map routes, Fast for combat entry.
- combat advice is intentionally entry-only to avoid high latency every turn.

运行行为：

- stdout 专门用于 CommunicationMod 协议消息。
- 普通日志写入 `logs/sts-ai.log`。
- prompt 和 LLM 回复写入 `logs/prompts.log`。
- 最新建议写入 `output/advice.txt`（纯文本）和 `output/overlay.json`（结构化 JSON）。
- 持久化运行记录写入 `runs/<run_id>/events.jsonl`。
- 本局结束时会自动生成复盘报告：`runs/<run_id>/postmortem.md`。
- 当前会在选牌、Boss 选牌、Boss 遗物、篝火、事件（多选项时）、进入战斗时生成建议。
- 地图路线建议会在每幕入口（完整路线分析）和路口分叉（即时下一节点决策）时触发。
- 建议按场景分层使用不同模型：Heavy（选牌/Boss 选牌）、Medium（篝火/事件/地图路线）、Fast（进入战斗）。
- 战斗建议只在进入战斗时生成一次，避免每回合 LLM 延迟影响游戏节奏。

## Copilot Overlay Mod / 游戏内悬浮窗

The [Copilot Overlay Mod](https://github.com/ArtemisSaber/slay-the-spire-copilot-overlay-mod) shows AI advice directly inside the game as a semi-transparent overlay in the top-right corner.

[Copilot Overlay Mod](https://github.com/ArtemisSaber/slay-the-spire-copilot-overlay-mod) 是一个游戏内悬浮窗模组，会在画面右上角半透明显示 AI 建议。

### Installation / 安装

1. Download `CopilotOverlay.jar` from the [overlay mod releases](https://github.com/ArtemisSaber/slay-the-spire-copilot-overlay-mod/releases).
2. Place it in your ModTheSpire mods directory.
3. Enable **Copilot Overlay** alongside Communication Mod CJK when launching through ModTheSpire (requires BaseMod).

安装步骤：
1. 从[悬浮窗模组 Releases](https://github.com/ArtemisSaber/slay-the-spire-copilot-overlay-mod/releases)下载 `CopilotOverlay.jar`。
2. 放入 ModTheSpire 的 mods 目录。
3. 通过 ModTheSpire 启动游戏时，同时启用 **Copilot Overlay** 和 Communication Mod CJK（需要 BaseMod）。

### File Path / 文件路径

The overlay polls `output/overlay.json` every 500ms. Resolution order:

悬浮窗每 500ms 读取一次 `output/overlay.json`，路径查找顺序：

1. `COPILOT_ADVICE_PATH` environment variable (absolute path to `output/overlay.json`)
2. `output/overlay.json` relative to the game's working directory

When used with CommunicationMod, the working directory is the Slay the Spire install directory — the same directory where the copilot writes `output/overlay.json` by default.

通过 CommunicationMod 使用时，工作目录就是 Slay the Spire 安装目录，与 copilot 默认写入 `output/overlay.json` 的目录一致。

### Behavior / 行为

- The copilot controls visibility via the `overlay_visibility` field in `output/overlay.json`.
- `status: "loading"` — the copilot is waiting for the LLM; overlay may show a spinner.
- `status: "ok"` — fresh advice available with structured `advice` fields (`recommendation`, `reason`, `risk`, `commentary`).
- After 30 seconds, the copilot sets `overlay_visibility: false`; the overlay should hide.
- Parsing Chinese labels (`推荐`/`理由`/`风险`/`吐槽`) from plain text is no longer needed — the JSON schema (`schemas/overlay.d.ts`) provides structured fields directly.

行为：
- copilot 通过 `output/overlay.json` 中的 `overlay_visibility` 字段控制悬浮窗显隐。
- `status: "loading"` — copilot 正在等待 LLM 回复，悬浮窗可显示加载状态。
- `status: "ok"` — 新建议已就绪，`advice` 对象包含结构化字段（`recommendation`、`reason`、`risk`、`commentary`）。
- 30 秒后，copilot 会将 `overlay_visibility` 设为 `false`，悬浮窗应隐藏。
- 不再需要从纯文本中解析中文标签（`推荐`/`理由`/`风险`/`吐槽`）——JSON schema（`schemas/overlay.d.ts`）直接提供结构化字段。

## Postmortem / 复盘

When launched by CommunicationMod, the app automatically generates a Markdown postmortem when the run ends. The report is saved next to the journal:

通过 CommunicationMod 启动时，程序会在本局结束时自动生成 Markdown 复盘。复盘会保存在对应日志旁边：

```text
runs/<run_id>/postmortem.md
```

The app first builds a deterministic report from `events.jsonl`, then asks the configured LLM to rewrite it into a more user-friendly Chinese report. If the AI call fails, the deterministic report is saved instead.

程序会先根据 `events.jsonl` 生成确定性的机器摘要，再调用当前配置的 LLM 改写成更适合玩家阅读的中文复盘。如果 AI 调用失败，会保存机器摘要作为兜底。

You can also manually regenerate a Markdown-style postmortem from a run journal:

也可以手动从运行日志重新生成 Markdown 风格复盘：

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

## Output Format / 建议格式

AI advice uses a structured format parsed from the LLM response. For map routes, the recommendation includes a position label and compact route chain.

AI 建议使用结构化格式，从 LLM 回复中解析。地图路线建议包含位置标签和路线链。

```text
Recommendation: Root 1 (leftmost) — M→$→M→R — early shop, safe elite
Reason: Shop early with gold, rest before the only elite
Risk: Only one elite this act
Comment: Play it safe!

推荐：Root 1 (leftmost) — M→$→M→R — 早期商店，安全精英
理由：早期商店消耗金币，精英前有休息
风险：本幕只有一个精英
吐槽：稳扎稳打！
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
  main.rs          main loop, CLI modes, screen gating, MapGate (map crossroads/act-entry gating)
  config.rs        environment variable loading
  protocol.rs      CommunicationMod protocol messages
  state.rs         normalized game state, MapCoord, advice hash, observation hash, danger assessment
  prompt.rs        LLM prompt builder — card, relic, rest, event, combat, map_suggestion, map_crossroad; describe_path risk analysis; path enumeration
  llm.rs           LLM provider abstraction, effort routing, AdviceScenario (MapSuggestion, MapCrossroad, etc.)
  advice.rs        latest advice file output and cache, overlay JSON
  journal.rs       JSONL run journal with schema versioning
  postmortem.rs    postmortem report generation
  startup.rs       CommunicationMod config validation, auto-fix, CJK detection
  logging.rs       file logger
  i18n/            translations (cards, relics, monsters, powers, potions, card descriptions)
  card_values.json numeric values for card description variables (!D!, !B!, !M!)
  tests/           embedded unit test modules
tests/
  fixtures/        sample CommunicationMod JSON states
  integration_test.rs
docs/
  mvp-roadmap.md
```

## License / 许可证

MIT
