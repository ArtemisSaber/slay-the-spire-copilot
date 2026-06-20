# MVP Roadmap

## Current Product Shape

Slay the Spire Copilot is a local Rust CLI that is launched by CommunicationMod, reads line-delimited game-state JSON from stdin, asks an LLM for advice, and writes advice to local files.

The current implementation is a working prototype with a strong technical foundation:

- CommunicationMod protocol startup via `ready`
- JSON game-state parsing from stdin
- normalized game-state model
- Chinese i18n for cards, relics, monsters, powers, potions, and card descriptions
- `card_values.json` for resolving description variables (`!D!`, `!B!`, `!M!`)
- prompt generation for combat, card rewards, boss rewards, rest sites, events, and generic states
- danger assessment (`DangerFlags`, `DangerLevel`) for context-aware prompts
- mock and OpenAI-compatible LLM providers
- effort-based model routing (Fast/Medium/Heavy) per screen type
- advice caching by stable state hash
- combat identity deduplication (advice once per floor:room_type per run)
- advice output to `output/advice.txt`
- structured JSONL journal under `runs/<run_id>/events.jsonl`
- schema versioning (`schema_version: 1`) on all journal events
- journal event types: `run_metadata`, `state_changed`, `run_started`, `run_ended`, `advice`
- state-change deduplication via `observation_hash`
- advice cache linkage via `state_hash`
- CommunicationMod startup check, config validation, and auto-fix
- CJK language detection and Communication Mod CJK recommendation
- postmortem report generation (deterministic + AI-enhanced), with `--plain` flag
- manual test modes: `--stdin-test`, `--no-startup-check`, `SKIP_COMM_CONFIG=1`
- unit and integration test coverage across state, prompt, i18n, LLM, protocol, startup, advice, config, and journal modules

## Current Runtime Behavior

The runtime generates advice for these screens:

| Screen | Effort | Notes |
|---|---|---|
| `CARD_REWARD` | Heavy | Standard card picks |
| `BOSS_REWARD` | Heavy | Boss card picks |
| `EVENT` | Medium | Only when >1 available (non-disabled) choices exist |
| `REST` | Medium | Campfire decisions (rest, smith, toke, dig, lift, recall, girya) |
| `NONE` (combat) | Fast | Once per unique floor:room_type (entry-only, not per turn) |

The journal records normalized state changes before advice gating, so screens that do not generate advice can still appear in the run timeline.

Combat advice is intentionally entry-only. Turn-by-turn advice is deferred to Post-MVP (see below).

Run end is detected on `GAME_OVER` screen type or when `in_game` transitions from `true` to `false`, triggering automatic postmortem generation.

## Current Files Of Interest

- `src/main.rs`: application loop, screen gating, stdin processing
- `src/state.rs`: normalized game-state extraction, danger assessment, stable/observation hashing
- `src/prompt.rs`: prompt formatting
- `src/llm.rs`: provider abstraction, effort routing, system prompts
- `src/advice.rs`: advice cache and advice output
- `src/journal.rs`: JSONL run journal with schema versioning
- `src/postmortem.rs`: deterministic and AI-enhanced postmortem reports
- `src/startup.rs`: CommunicationMod config validation, auto-fix, CJK detection
- `src/protocol.rs`: CommunicationMod protocol messages
- `src/config.rs`: environment variable loading
- `src/logging.rs`: file-based tracing/logging
- `src/i18n/`: localization data (cards, relics, monsters, powers, potions, card_desc)
- `src/card_values.json`: numeric values for card description variables
- `src/tests/`: embedded unit test modules
- `tests/integration_test.rs`: integration test
- `tests/fixtures/`: sample game states

## MVP Goal

The MVP should support this complete loop:

1. Player starts a Slay the Spire run with CommunicationMod.
2. Copilot records the run timeline locally.
3. Copilot gives useful advice at key decision points.
4. Player finishes or stops the run.
5. Copilot can generate a basic post-mortem from the recorded journal.

**Status: Achieved.** All five steps are functional.

## MVP Completed Features

### 1. Full-Fidelity Journal Hash

- Added `NormalizedState::observation_hash()` based on full serialized normalized state
- `stable_hash()` is kept for advice caching only
- Journal deduplication uses `observation_hash`
- `state_hash` is recorded in advice events for cache linkage
- Tests cover pile-only, monster-block, monster-power, and UUID-only state changes

### 2. Event Schema Versioning

- `SCHEMA_VERSION: u32 = 1` defined in `journal.rs`
- All journal events include `schema_version: 1`

### 3. More Advice Coverage

- Enabled `CARD_REWARD` advice (Heavy effort)
- Enabled `BOSS_REWARD` advice (Heavy effort)
- Enabled `REST` site advice (Medium effort)
- Enabled `EVENT` advice for >1 available choice (Medium effort)
- Enabled combat-entry advice on new fights (Fast effort, deduplicated by floor:room_type)
- Combat advice is entry-only; per-turn advice deferred to Post-MVP

### 4. Post-Mortem Report Command

- Added `postmortem` CLI command that reads `runs/<run_id>/events.jsonl`
- Outputs a Markdown summary including:
  - run start/end
  - last observed floor, HP, gold, deck size, relics
  - advice events by floor/screen
  - card reward choices and inferred picks
  - major HP changes
  - final observed state
- Added `--plain` flag to skip AI rewrite and print deterministic report directly
- AI-enhanced reports use a coaching-style Chinese prompt for 2-4 improvement suggestions
- Automatically generated when a run ends during normal operation

### 5. Manual/Test Run Mode

- `--stdin-test`: skips CommunicationMod startup check and forces mock provider
- `--no-startup-check`: skips CommunicationMod startup check only
- `SKIP_COMM_CONFIG=1`: environment variable alternative for bypassing startup check

### 6. Run Metadata

- `log_run_started_with_config` captures:
  - provider name
  - model names (fast, medium, heavy)
  - app version
  - start timestamp
  - first observed character, ascension, seed (when CommunicationMod exposes them)

### 7. Output Behavior Cleanup

- `output/advice.txt` now contains only the latest advice (overwrite, not append)
- Full historical advice is preserved in the JSONL journal

## Post-MVP Ideas

- combat simulator integration
- optional turn-by-turn combat advice
- action reconstruction from pile deltas
- richer card reward inference
- shop/event/map advice
- run comparison across multiple journals
- local HTML report viewer
- GUI overlay or live dashboard
- raw CommunicationMod event logging behind an opt-in debug flag
