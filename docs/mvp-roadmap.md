# MVP Roadmap

## Current Product Shape

Slay the Spire Copilot is a local Rust CLI that is launched by CommunicationMod, reads line-delimited game-state JSON from stdin, asks an LLM for advice, and writes advice to local files.

The current implementation is a working prototype with a strong technical foundation:

- CommunicationMod protocol startup via `ready`
- JSON game-state parsing from stdin
- normalized game-state model
- Chinese i18n for cards, relics, monsters, powers, potions, and card descriptions
- prompt generation for combat, card rewards, rest sites, and generic states
- mock and OpenAI-compatible LLM providers
- advice caching by stable state hash
- advice output to `output/advice.txt`
- structured JSONL journal under `runs/<run_id>/events.jsonl`
- run lifecycle journal events: `run_started`, `run_ended`
- state-change journal events
- advice journal events
- unit and integration test coverage across state, prompt, i18n, LLM, protocol, startup, advice, config, and journal modules

## Current Runtime Behavior

The runtime only generates advice for `CARD_REWARD` screens.

Prompt builders already support more contexts, including combat and rest sites, but the main screen gate does not currently enable those advice paths.

The journal records normalized state changes before advice gating, so screens that do not generate advice can still appear in the run timeline.

## Current Files Of Interest

- `src/main.rs`: application loop, screen gating, stdin processing
- `src/state.rs`: normalized game-state extraction and stable hashing
- `src/prompt.rs`: prompt formatting
- `src/llm.rs`: provider abstraction and LLM calls
- `src/advice.rs`: advice cache and advice output
- `src/journal.rs`: JSONL run journal
- `src/startup.rs`: CommunicationMod config validation/fixup
- `src/i18n/`: localization data
- `tests/fixtures/`: sample game states

## MVP Goal

The MVP should support this complete loop:

1. Player starts a Slay the Spire run with CommunicationMod.
2. Copilot records the run timeline locally.
3. Copilot gives useful advice at key decision points.
4. Player finishes or stops the run.
5. Copilot can generate a basic post-mortem from the recorded journal.

## MVP Missing Features

### 1. Full-Fidelity Journal Hash

Current state-change dedupe uses `NormalizedState::stable_hash()`.

That hash is designed for advice caching, not full replay fidelity. It intentionally omits or normalizes away details that matter for post-mortems, including some pile details, card UUIDs, monster block, monster powers, and card order.

Needed:

- add a separate `observation_hash`
- base it on the full serialized normalized state
- keep `stable_hash` for advice caching
- journal both hashes:
  - `observation_hash` for state-change dedupe and replay
  - `advice_hash` or `state_hash` for advice cache linkage

### 2. Event Schema Versioning

Journal records should include a schema version before the format becomes relied on by analysis tooling.

Needed:

- add `schema_version: 1` to all journal events
- add tests that assert the field exists

### 3. More Advice Coverage

The product is too narrow if it only advises on card rewards.

Needed for MVP:

- enable rest-site advice
- enable combat-entry advice when a fight starts
- keep boss relic, shop, map, events, and grid screens as post-MVP unless they become easy wins

Combat advice should not run every turn for MVP. LLM latency is too high for the pace of combat, and repeated advice can interrupt gameplay. The MVP should give an initial fight plan when entering combat, then rely on the journal to support post-mortem analysis after the fight.

Turn-by-turn combat advice can be revisited later if latency improves or if the app gains a faster local heuristic layer.

### 4. Post-Mortem Report Command

The journal is necessary but not enough. Users need a readable report.

Needed:

- add a command or mode that reads `runs/<run_id>/events.jsonl`
- output a Markdown or plain-text summary
- include:
  - run start/end
  - last observed floor, HP, gold, deck size, relics
  - advice events by floor/screen
  - card reward choices and inferred picks where possible
  - major HP changes
  - final observed state

Example target:

```bash
slay-the-spire-copilot postmortem runs/<run_id>/events.jsonl
```

### 5. Manual/Test Run Mode

The current startup flow validates CommunicationMod config before processing stdin. That can make fixture/manual testing awkward.

Needed:

- add a way to bypass CommunicationMod setup for local testing
- possible options:
  - `--stdin-test`
  - `--no-startup-check`
  - `SKIP_COMM_CONFIG=1`
- update README examples accordingly

### 6. Run Metadata

Run journals need enough metadata to be useful later.

Needed:

- provider name
- model names
- app version
- start timestamp
- first observed character
- ascension, seed, or modifiers if CommunicationMod exposes them

### 7. Output Behavior Cleanup

`output/advice.txt` currently appends advice forever.

For live use, the current advice should be easy to read.

Needed:

- decide whether `advice.txt` should contain only latest advice
- keep historical advice in the JSONL journal
- optionally add `output/advice-history.txt` if plain-text history is still useful

## Recommended Implementation Order

1. Add `schema_version` to journal events.
2. Add full-fidelity `observation_hash` and use it for journal dedupe.
3. Add tests for pile-only, monster-block, monster-power, and UUID-only state changes.
4. Add manual stdin/test mode.
5. Enable rest advice.
6. Enable combat-entry advice only.
7. Add post-mortem report command.
8. Update README with the MVP workflow.

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
