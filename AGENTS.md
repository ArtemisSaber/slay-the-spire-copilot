# AGENTS.md

## Setup
- **Git hooks**: `git config core.hooksPath .githooks` (once per clone)
- **Pre-commit** runs: `cargo fmt --check` → `cargo clippy -- -D warnings` → `cargo test` → `cargo build --release`
- **Rust edition 2024** — all source uses edition 2024 syntax
- **`.env`**: copy `.env.example` to `.env` and place next to the binary (or in CWD for `cargo run`)

## Build & Test
- `cargo test` — runs 195 tests
- `cargo build --release` — Linux binary at `target/release/slay-the-spire-copilot`
- Cross-compile Windows: `cargo build --release --target x86_64-pc-windows-gnu` (needs `mingw-w64-gcc` on Arch)
- Project uses `rustls-tls` (not OpenSSL) — no C dependencies beyond mingw on cross-compile
- Dev-test with mock provider: `echo '{"in_game":true,...}' | cargo run -- --stdin-test`

## Architecture
- **Single binary** — `main.rs` is the entrypoint. Modules are declared at the top.
- **Output paths** all resolve relative to the binary's parent directory (`project_root()` in `logging.rs`):
  - `logs/sts-ai.log`, `output/advice.txt`, `runs/<run_id>/events.jsonl`, `.env`
- **Log level** default is `info`. Set `RUST_LOG=debug` for verbose. Logs are file-only (no stdout).
- **Advice gating** (`SCREEN_CONFIG`): `CARD_REWARD`, `BOSS_REWARD`, `EVENT` (>1 choice), `REST`. Combat entry is separate via `AdviceGate`.
- **Boss card reward**: detected by `is_boss_card_reward()` — `CARD_REWARD` screen at floors 16, 33, 50.
- **Model routing**: Heavy (card/boss rewards), Medium (rest/events), Fast (combat entry). Configured via `LLM_MODEL_FAST/MEDIUM/HEAVY`.

## Tests
- **Unit tests**: inline in each module via `#[path = "tests/xxx_tests.rs"] mod tests;` — these are in `src/tests/`
- **Integration tests**: `tests/integration_test.rs`
- **Fixtures**: `tests/fixtures/*.json` — sample CommunicationMod game states
- Single test: `cargo test test_name`
- Test module: `cargo test module::tests`

## Data Files
- **i18n**: `src/i18n/` — `cards.json`, `relics.json`, `monsters.json`, `powers.json`, `potions.json`, `card_desc.json`
- **Card values**: `src/card_values.json` — `d/b/m` (base) and `d+/b+/m+` (upgraded). `!D!`, `!B!`, `!M!` in descriptions resolve to these. `m` is misc count (Miracles, draws, etc.). Verify against game data when editing — values like 门徒之怒 and 冥想 were previously wrong.

## Java `.properties` Gotcha
- CommunicationMod's `config.properties` uses Java `.properties` format where `\` is an escape char
- `format_command_value()` doubles backslashes when writing (`\` → `\\`)
- `parse_command_value()` normalizes doubled backslashes when reading (`\\` → `\`)
- Both functions are in `src/startup.rs`

## CLI Flags
| Flag | Effect |
|------|--------|
| `--stdin-test` | Skip config check + force mock provider |
| `--no-startup-check` | Skip config check only |
| `SKIP_COMM_CONFIG=1` | Env var alternative to `--no-startup-check` |
| `postmortem <path>` | Generate report from journal JSONL |
| `--plain` | (with postmortem) Skip AI rewrite |
