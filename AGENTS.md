# AGENTS.md

## Setup
- **Rust 1.85+** minimum
- **Git hooks**: `git config core.hooksPath .githooks` (once per clone)
- **Pre-commit** runs: `cargo fmt --check` → `cargo clippy -- -D warnings` → `cargo test` → `cargo build --release`
- **Clippy** is clean on both binary and tests: `cargo clippy --all-targets -- -D warnings` passes with zero warnings
- **Rust edition 2024** — all source uses edition 2024 syntax
- **`.env`**: copy `.env.example` to `.env` and place next to the binary (or in CWD for `cargo run`)
- **Version**: current release is `0.2.0`

## Build & Test
- `cargo test` — runs 1570 tests (1568 unit + 2 integration)
- `cargo build --release` — Linux binary at `target/release/slay-the-spire-copilot`
- Cross-compile Windows: `cargo build --release --target x86_64-pc-windows-gnu` (needs `mingw-w64-gcc` on Arch)
- Project uses `rustls-tls` (not OpenSSL) — no C dependencies beyond mingw on cross-compile
- Dev-test with mock provider: `echo '{"in_game":true,...}' | cargo run -- --stdin-test`

## Architecture
- **Single binary** — `main.rs` is the entrypoint. Modules are declared at the top.
- **Module map**:
  - `advice`, `config`, `gate`, `journal`, `llm`, `logging`, `postmortem`, `protocol`, `relic_counters`, `runtime`, `setup_wizard`, `startup`, `state` — top-level modules
  - `autoplay/` — action, command_state, control, planner (auto-play feature)
  - `combat/` — adviser, context, damage, effects, kill_scan (combat analysis and lethal-finding)
  - `ranker/` — mod, rules, engine, context, formula, parser, predicates, rules.json (pool-relative combat action scoring)
  - `locales/` — en, zh, ja, ko JSON locale files + `mod.rs`
  - `prompt.rs` — facade re-exporting from `prompt/builder.rs` and `prompt/routing.rs`
- **Output paths** all resolve relative to the binary's parent directory (`project_root()` in `logging.rs`):
  - `logs/sts-ai.log`, `logs/comm-mod-raw.log`, `logs/prompts.log` (with rotation: `.1`, `.2` backups)
  - `output/advice.txt`, `output/overlay.json`, `runs/<run_id>/events.jsonl`, `.env`, `rules.json`
- **Log level** default is `info`. Set `RUST_LOG=debug` for verbose. Logs are file-only (no stdout).
- **Advice gating** (`SCREEN_CONFIG`): `CARD_REWARD`, `BOSS_REWARD`, `EVENT` (>1 choice), `REST`, `SHOP_SCREEN`.
  - **Combat turns**: `CombatTurnGate` — triggers once per turn when `screen_type == "NONE"`, `action_phase == "WAITING_ON_USER"`, and monsters are present.
  - **Map advice**: `MapGate` — triggers at act entry and mid-act crossroads (when the current node has >1 child).
- **Boss card reward**: detected by `is_boss_card_reward()` — `CARD_REWARD` screen at floors 16, 33, 50.
- **Model routing** (`Effort` via `LLM_MODEL_FAST/MEDIUM/HEAVY`):
  - **Heavy**: card rewards, boss card rewards
  - **Medium**: rest, events, shop, map_suggestion, map_crossroad, boss relic
  - **Fast**: combat entry

## Tests
- **Unit tests**: inline in each module via `#[path = "tests/xxx_tests.rs"] mod tests;` — these are in `src/tests/`
- **Integration tests**: `tests/integration_test.rs`
- **Fixtures**: 21 files in `tests/fixtures/*.json` — CommunicationMod game states (card reward, combat, rest, shop) + kill-scan run/integration fixtures
- Single test: `cargo test test_name`
- Test module: `cargo test module::tests`

## Data Files
- **i18n**: `src/i18n/powers.json` — localized power name mappings used for parsing
- **Locales**: `src/locales/` — `en.json`, `zh.json`, `ja.json`, `ko.json` (4 languages). Contains all UI strings, system prompts, few-shot examples, parser keywords, and relic counter cycles per locale. Loaded via `locale::Locale::load(lang)`.

## Environment Variables (`.env.example`)
| Variable | Default | Purpose |
|----------|---------|---------|
| `LLM_PROVIDER` | `pollinations-free` | Provider identifier |
| `LLM_BASE_URL` | (Pollinations URL) | API base URL |
| `LLM_API_KEY` | — | API key |
| `LLM_MODEL` | `openai-fast` | Default model |
| `LLM_MODEL_FAST` | `openai-fast` | Fast-tier model |
| `LLM_MODEL_MEDIUM` | — | Medium-tier model |
| `LLM_MODEL_HEAVY` | — | Heavy-tier model |
| `LLM_MAX_TOKENS` | `50000` | Default max completion tokens |
| `LLM_MAX_TOKENS_FAST` | `300` | Fast-tier max tokens |
| `LLM_MAX_TOKENS_MEDIUM` | — | Medium-tier max tokens |
| `LLM_MAX_TOKENS_HEAVY` | — | Heavy-tier max tokens |
| `LLM_TEMPERATURE` | `0.7` | Sampling temperature |
| `LLM_DISABLE_FAST_THINKING` | — | Disable fast-thinking (reasoning) for compatible models |
| `AUTO_PLAY` | `false` | Enable auto-play mode |

## Auto-Play
- `src/autoplay/` — LLM-based autonomous play engine
- Controlled by `AUTO_PLAY=true` in `.env`
- Runtime control via `output/autoplay-control.json`
- Planner generates actions from game state; control layer executes them via VirtualKey + mouse

## Kill-Scan
- `src/combat/kill_scan.rs` — finds lethal card play sequences in combat
- DFS-based search with memoization, deadlines, and worker thread support
- Locale-aware effect keyword patterns from `src/combat/effects.rs`
- 62 spec-driven tests + 24 combat unit tests

## Ranker
- `src/ranker/` — JSON-rule-driven combat action scoring engine
- **Pool-relative scoring**: damage scored as fraction of enemy HP pool (`damage / monsters_total_hp_pool`), block scored as fraction of self HP saved (`min(block, incoming) / current_hp`)
- Loads `rules.json` from binary directory at runtime, falls back to embedded copy on **both** file-not-found and parse error (via `load_rules_from()`)
- `build.rs` copies `src/ranker/rules.json` to `target/release/rules.json`
- `top_ranked_context()` in `src/autoplay/combat_adviser.rs` returns flat array of all non-avoided actions with `score` and `tags` (no rule breakdown in prompt)
- `include_str!("rules.json")` embeds the file at compile time as fallback
- See `docs/ranker-architecture.md` and `docs/ranker-rules.md`

## Java `.properties` Gotcha
- CommunicationMod's `config.properties` uses Java `.properties` format where `\` is an escape char
- `format_command_value()` doubles backslashes when writing (`\` → `\\`)
- `parse_command_value()` normalizes doubled backslashes when reading (`\\` → `\`)
- Both functions are in `src/startup.rs`

## CLI Flags & Subcommands
| Flag / Command | Effect |
|----------------|--------|
| `--stdin-test` | Skip config check + force mock provider |
| `--no-startup-check` | Skip config check only |
| `SKIP_COMM_CONFIG=1` | Env var alternative to `--no-startup-check` |
| `setup` / `configure` | Run interactive API setup wizard and exit |
| `postmortem <path>` | Generate report from journal JSONL file |
| `postmortem --plain <path>` | Same, but skip AI rewrite (deterministic only) |

## CI/CD
- **`.github/workflows/ci.yml`** — on push to `develop` / `feat/mvp` branches (ubuntu + windows): fmt → clippy → test → build
- **`.github/workflows/release.yml`** — on `v*` tags: builds linux, macOS, Windows binaries and uploads artifacts

## Other Directories
- **`docs/`** — 9 design docs covering auto-play, kill-scan, MVP roadmap, ranker rules, and code review findings
- **`schemas/overlay.d.ts`** — TypeScript type definition for `output/overlay.json`

## Security Hardening
- **API key redaction**: `LlmProvider` has a manual `Debug` impl that redacts `api_key` as `<REDACTED>`. LLM error response bodies are sanitized via `sanitize_err_body()` (redacts key, truncates to 500 chars) before inclusion in error messages.
- **Stdin size cap**: `MAX_STDIN_JSON_BYTES` (10 MB) in `main.rs` rejects oversized stdin input before JSON parsing to prevent stack overflow / OOM.
- **Map cycle detection**: `enumerate_paths()` in `prompt/routing.rs` skips children already in the current path to prevent infinite loops on malformed map data.
- **rules.json fallback**: `load_rules_from()` in `ranker/mod.rs` falls back to the embedded copy on both file-not-found and parse error (no panic on malformed user file).
- See `docs/code-review-findings.md` for the full security audit and issue tracking.
