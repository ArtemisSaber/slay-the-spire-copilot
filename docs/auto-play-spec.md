# Auto-Play Specification

## Purpose

This document specifies auto-play mode for Slay the Spire Copilot.

Auto-play must be implemented as an isolated module that:

- Reads game state from the existing CommunicationMod stdin loop.
- Plans typed actions separately from human advice.
- Sends validated commands to CommunicationMod stdout.
- Can be controlled by the overlay at runtime.
- Fails closed to advice-only behavior when state, control, or command validation is uncertain.

The current advice experience must remain the default and must continue to work when auto-play is off.

## Status

This is a target specification. It intentionally includes screens that are not yet parsed by the current codebase, including `COMBAT_REWARD`.

Local run journals under `target/release/runs/` contain observed `COMBAT_REWARD` state transitions. Those journals preserve normalized state, not raw CommunicationMod command metadata, so they help define reward parsing behavior but do not fully answer command syntax.

## Non-Goals

- No auto-play is enabled by default.
- No direct overlay writes to `output/overlay.json`.
- No command execution from localized free-text advice.
- No full combat batching in v1. Combat should send at most one command per fresh state.
- No unsupported screen guessing. Unknown screens must block auto-play and surface a reason.

## File Ownership

Use two files for overlay communication:

```text
output/overlay.json             copilot -> overlay
output/autoplay-control.json    overlay -> copilot
```

`overlay.json` remains copilot-owned. The overlay must never modify it.

The overlay owns `autoplay-control.json` and must write it atomically through a temp file and rename.

## Overlay Control Schema

Add:

```text
schemas/autoplay-control.d.ts
```

Schema:

```ts
export type AutoPlayMode = "off" | "advise" | "auto" | "paused";

export interface AutoPlayControl {
  schema_version: 1;
  revision: number;
  mode: AutoPlayMode;
  updated_at_ms: number;

  require_confirmation: boolean;

  allow_card_rewards: boolean;
  allow_combat_rewards: boolean;
  allow_boss_rewards: boolean;
  allow_rest: boolean;
  allow_events: boolean;
  allow_map: boolean;
  allow_shop: boolean;
  allow_combat: boolean;
  allow_selection_screens: boolean;

  min_hp_percent: number | null;
  max_commands_per_turn: number | null;
  max_commands_per_floor: number | null;
}
```

Rules:

- `revision` must increase on every overlay write.
- The copilot must ignore an already-seen `revision`.
- Missing control file means `mode = "off"`.
- Malformed control file means `mode = "off"` and overlay status `blocked`.
- `mode: "off"` disables auto-play and clears pending action state.
- `mode: "paused"` prevents command execution but may keep pending action state visible.
- `mode: "advise"` may propose actions but must not execute.
- `mode: "auto"` may execute only after all safety checks pass.

## Overlay Output Schema

Extend `OverlayOutput` in `src/advice.rs` and `schemas/overlay.d.ts`.

Add:

```ts
export type AutoPlayRuntimeMode = "off" | "advise" | "auto" | "paused";
export type AutoPlayStatus =
  | "inactive"
  | "ready"
  | "thinking"
  | "proposed"
  | "executing"
  | "blocked"
  | "error";

export interface AutoPlayActionView {
  id: string;
  screen_type: string | null;
  command: string | null;
  target_label: string;
  reason: string;
  risk: string;
  requires_confirmation: boolean;
}

export interface AutoPlayState {
  mode: AutoPlayRuntimeMode;
  status: AutoPlayStatus;
  control_revision: number | null;
  blocked_reason: string | null;
  last_error: string | null;
  pending_action: AutoPlayActionView | null;
  last_executed_action: AutoPlayActionView | null;
  supported_screen: boolean;
  ready_for_command: boolean | null;
  available_commands: string[];
}
```

Then add:

```ts
autoplay: AutoPlayState;
```

to `OverlayOutput`.

Backward compatibility:

- Existing fields remain unchanged.
- Old overlays may ignore `autoplay`.
- New overlays should use `autoplay.mode`, `autoplay.status`, and `autoplay.pending_action` for controls.

## Rust Module Layout

Add:

```text
src/autoplay/
  mod.rs
  control.rs
  status.rs
  command_state.rs
  action.rs
  prompt.rs
  planner.rs
  resolver.rs
  executor.rs
  safety.rs
```

Responsibilities:

| File | Responsibility |
|---|---|
| `mod.rs` | Public facade used by `main.rs`. |
| `control.rs` | Parse overlay control file and track revisions. |
| `status.rs` | Maintain overlay-facing auto-play status. |
| `command_state.rs` | Extract `ready_for_command`, `available_commands`, and `choice_list` from raw JSON. |
| `action.rs` | Define typed action schemas and parse LLM JSON. |
| `prompt.rs` | Build executor-oriented action prompts with stable IDs. |
| `planner.rs` | Query LLM for action JSON. |
| `resolver.rs` | Resolve action IDs to current command arguments. |
| `executor.rs` | Write commands to stdout. |
| `safety.rs` | Enforce mode, screen allow-list, HP, readiness, budgets, and exact resolution. |

`main.rs` must call only `AutoPlayController`.

## Public API

```rust
pub struct AutoPlayController {
    // private fields
}

pub enum AutoPlayOutcome {
    Inactive,
    AdviceOnly,
    Proposed(AutoPlayAction),
    Executed(AutoPlayAction),
    Blocked { reason: String },
    Error { message: String },
}

impl AutoPlayController {
    pub fn new(project_root: PathBuf) -> Self;

    pub async fn handle_state(
        &mut self,
        raw: &serde_json::Value,
        normalized: &NormalizedState,
        provider: &LlmProvider,
        locale: &Locale,
        journal: &Journal,
    ) -> AutoPlayOutcome;

    pub fn overlay_state(&self) -> AutoPlayState;
}
```

`handle_state` must not write human advice. It only updates auto-play state, proposes actions, executes actions, and logs auto-play events.

## Main Loop Integration

Current advice flow remains in place. Add auto-play before advice output:

```text
read stdin line
parse raw JSON
normalize state
log state change
update auto-play control

if auto-play mode is active:
  outcome = autoplay.handle_state(...)
  write overlay with current autoplay state
  if outcome is Executed:
    skip normal advice for this state
  if outcome is Proposed and mode is advise:
    continue normal advice
  if outcome is Blocked:
    continue normal advice when advice is configured for this screen

run existing advice path when needed
```

Action decisions must not use `AdviceCache`. Advice caching is text-oriented and may replay stale commands. Auto-play can cache nothing in v1.

## Command State Extraction

Add a struct:

```rust
pub struct CommandState {
    pub ready_for_command: bool,
    pub available_commands: Vec<String>,
    pub choice_list: Vec<String>,
}
```

Extraction:

- `ready_for_command` from `/ready_for_command`.
- `available_commands` from `/available_commands`.
- `choice_list` from `/game_state/choice_list`, default empty.

This can live in `src/autoplay/command_state.rs` rather than `NormalizedState` if command metadata should stay separate from strategy state.

## Protocol Writers

Add production protocol functions with testable writer variants:

```rust
send_choose_to(writer, index)
send_play_to(writer, hand_index, target_index: Option<usize>)
send_end_to(writer)
send_proceed_to(writer)
send_skip_to(writer)
send_leave_to(writer)
send_return_to(writer)
send_wait_to(writer, ms)
send_state_to(writer)
```

All executor commands must first verify that `available_commands` contains the command name.

Open protocol questions that must be verified before implementation:

- Exact case for `wait` and `state`. Current test helpers emit `WAIT 30` and `STATE`.
- Exact combat `play` syntax, especially target format.
- Exact reward-screen `proceed` syntax.
- Whether `skip` is a command or a `choose` index on card reward screens.
- Whether combat reward uses `choose`, `proceed`, click coordinates, or mixed commands.

## Action JSON Schema

The LLM must return strict JSON.

```json
{
  "schema_version": 1,
  "actions": [
    {
      "kind": "choose",
      "action_id": "card_reward:1",
      "label": "Anger",
      "reason": "Cheap damage improves hallway fights.",
      "risk": "Adds deck size."
    }
  ]
}
```

General action fields:

| Field | Required | Meaning |
|---|---|---|
| `kind` | yes | `choose`, `skip`, `proceed`, `play`, `end`, `leave`, `return`, or `wait`. |
| `action_id` | yes | Stable ID generated by prompt builder. |
| `label` | yes | Human-readable target for overlay. |
| `reason` | yes | Short reason. |
| `risk` | yes | Short risk or empty string. |

The resolver must reject:

- Non-JSON output.
- Unknown schema versions.
- Empty actions.
- More than one action on screens that support only one safe action.
- Unknown `action_id`.
- Any action that cannot be mapped exactly to the current state.

## Screen Requirements

### `CARD_REWARD`

Supported in Phase 1.

Action IDs:

- `card_reward:0..N-1`
- `card_reward:skip` when `skip_available` is true

Resolution:

- Pick by index into current `card_reward_choices`.
- Skip only when `skip_available` is true and `available_commands` includes either `skip` or a verified skip command path.

Execution:

- `choose <idx>` for card picks after protocol verification.
- `skip` or verified equivalent for skip.

### Boss Card Reward

Supported in Phase 1 as a variant of `CARD_REWARD`.

Detection remains `CARD_REWARD` on floors 16, 33, 50.

Use action IDs:

- `boss_card_reward:0..N-1`
- `boss_card_reward:skip`

### `BOSS_REWARD`

Supported in Phase 1.

Action IDs:

- `boss_relic:0..N-1`

Execution:

- `choose <idx>` after protocol verification.

### `COMBAT_REWARD`

Required for unattended runs. Support should start in Phase 1 if the payload and command grammar are known; otherwise it is the first Phase 2 item.

Why it matters:

- After every combat, the run can stall on gold, relic, potion, card reward, or proceed choices.
- Card reward screens often appear after selecting a reward item from `COMBAT_REWARD`.
- Auto-play that handles combat but not `COMBAT_REWARD` cannot advance between fights.

Required parser additions:

```rust
pub struct CombatRewardState {
    pub rewards: Vec<CombatRewardItem>,
    pub can_proceed: bool,
    pub source: CombatRewardSource,
}

pub enum CombatRewardItem {
    Gold { index: usize, amount: Option<i64>, label: String },
    Relic { index: usize, id: String, name: String, description: String },
    Potion { index: usize, id: String, name: String, description: String },
    CardReward { index: usize, label: String },
    Unknown { index: usize, label: String },
}

pub enum CombatRewardSource {
    RawScreenState,
    ChoiceList,
    NormalizedEventChoices,
}
```

Observed journal behavior:

- The current normalized model stores combat reward labels in `event_choices` because `NormalizedState::from_raw()` falls back to `/game_state/choice_list`.
- Across the local journals there are 103 `COMBAT_REWARD` state changes.
- Observed reward labels are generic English strings: `gold`, `relic`, `potion`, `card`.
- Reward collection appears as a shrinking queue:
  - `gold|card` -> `card` -> empty
  - `gold|potion|card` -> `potion|card` -> `card` -> empty
  - `gold|relic|potion|card` -> `relic|potion|card` -> `potion|card` -> `card` -> empty
  - `relic` -> empty for treasure rooms
- State deltas identify what was collected:
  - `gold` disappears and `gold` increases.
  - `relic` disappears and relic count increases.
  - `potion` disappears and potion count may increase, but may remain when potion slots are full.
  - `card` usually leads to `CARD_REWARD`, then returns to `COMBAT_REWARD` with `card` removed.

Implementation implication:

- `COMBAT_REWARD` parser can initially use ordered `choice_list` / normalized `event_choices` labels as reward candidates.
- Reward labels are type-level, not item-level. The parser should classify `gold`, `relic`, `potion`, and `card`, but it should not pretend to know the specific relic, potion, or card from this label alone.
- Specific item details, if needed, must come from raw `screen_state` or follow-up state deltas.

Discovery requirement:

- Add at least one `tests/fixtures/combat-reward-state.json` captured from CommunicationMod CJK.
- Confirm whether rewards appear in `screen_state`, `choice_list`, or another field.
- Confirm `available_commands` for `COMBAT_REWARD`.
- Confirm exact command syntax for claiming each reward type and for proceeding when the queue is empty.

Action IDs:

- `combat_reward:gold:<index>`
- `combat_reward:relic:<index>`
- `combat_reward:potion:<index>`
- `combat_reward:card:<index>`
- `combat_reward:proceed`
- `combat_reward:unknown:<index>` only allowed in advise mode, never auto mode

Execution rules:

- Gold rewards may be collected automatically when exact command mapping is known.
- Relic rewards may be collected automatically when exact command mapping is known, but the overlay should display the action as "Collect relic" unless the raw payload exposes the relic identity.
- Potion reward collection should be blocked if potion slots are full unless the state clearly supports taking it.
- Card reward item should be selected to open `CARD_REWARD`; the actual card pick is handled by `CARD_REWARD`.
- `proceed` is allowed only when no known useful rewards remain, or when rewards are unsupported and the user has allowed skipping unknown rewards.
- Unknown reward items block auto mode and show `blocked_reason`.

Safety:

- Never skip an unparsed relic, potion, or card reward silently.
- Never proceed from `COMBAT_REWARD` while any known collectible reward remains.
- If only gold remains and gold amount is unknown, collecting is allowed if the command mapping is exact.
- If only `potion` remains and potion count did not increase after a previous potion attempt, block instead of retrying forever.

### `REST`

Supported in Phase 1.

Action IDs:

- `rest:rest`
- `rest:smith`
- `rest:toke`
- `rest:dig`
- `rest:lift`
- `rest:recall`
- `rest:girya`

Resolution:

- Map ID to index in `rest_options`.

Important limitation:

- `smith`, `toke`, and similar options may open a follow-up selection screen. If follow-up selection screens are disabled, these actions require confirmation or must be blocked.

### `EVENT`

Supported in Phase 1 for readable choices.

Action IDs:

- `event:0..N-1`

Resolution:

- Map to non-disabled event choice index.

Safety:

- If event choice text is unreadable fallback text, block in auto mode unless the overlay explicitly allows unreadable event choices.
- Single-choice events may auto-proceed only after protocol verification and only when the choice is purely continuation.

### `MAP`

Phase 2.

Action IDs:

- `map:child:<x>:<y>`

Resolution:

- Map child coordinate to the immediate selectable node index.
- Do not resolve by pretty route labels.

Execution:

- Verified `choose <idx>` or equivalent.

### `SHOP_SCREEN`

Phase 2.

Action IDs:

- `shop:choice:<idx>` using raw `choice_list`
- `shop:leave`

Execution:

- One transaction per fresh state.
- Re-plan after every purchase because gold and inventory change.
- Leave only when planner chooses `shop:leave` and no mandatory action remains.

Safety:

- Purge may trigger a `GRID` selection. Block unless selection screens are enabled.
- Do not use separate card/relic/potion letter labels for resolution because current prompt labels restart at `A` in each shop section.

### `NONE` Combat

Phase 3.

Action IDs:

- `combat:play:<card_uuid>`
- `combat:end`

Targeted cards:

- Include `target:<monster_index>` in action payload.

Execution:

- Use `play` only when `available_commands` includes `play`.
- Use `end` only when `available_commands` includes `end`.
- Send at most one command per fresh state.
- Re-read overlay control immediately before command execution.

Safety:

- Resolve cards by UUID, not localized name.
- Reject actions for cards no longer in hand.
- Reject target indices not present in current active monsters.
- Stop after `max_commands_per_turn`.

### Selection Screens: `HAND_SELECT`, `GRID`, and Similar

Phase 4.

These are required for full unattended runs, but should not be part of the initial auto mode.

Requirement:

- Track the previous action that caused the selection.
- Parse selectable cards/items with stable IDs.
- Block when context is unknown.

Examples:

- Smith target after `rest:smith`.
- Card removal after shop purge or event.
- Discard/exhaust/card-in-deck selections from combat cards.

## Safety Requirements

Auto-play must not send a command when:

- Control mode is off or paused.
- Control file is missing or malformed.
- `ready_for_command` is false.
- `available_commands` does not include the command.
- Screen type is not allowed by control flags.
- HP is below `min_hp_percent`.
- The action cannot be resolved exactly.
- The action would enter an unsupported follow-up screen.
- The planner returns invalid JSON.
- The planner returns multiple actions for a single-action screen.
- Command budget is exceeded.
- CommunicationMod emitted an error on the previous tick.

When blocked:

- Set `autoplay.status = "blocked"`.
- Set `autoplay.blocked_reason`.
- Do not execute.
- Continue normal advice flow when available.

## Journal Events

Add these event types:

```json
{"event":"autoplay_control_changed","revision":1,"mode":"auto"}
{"event":"autoplay_action_proposed","state_hash":"...","screen_type":"CARD_REWARD","action":{}}
{"event":"autoplay_action_executed","state_hash":"...","screen_type":"CARD_REWARD","command":"choose 1","action":{}}
{"event":"autoplay_action_blocked","state_hash":"...","screen_type":"COMBAT_REWARD","reason":"unknown reward item","action":{}}
{"event":"autoplay_action_failed","state_hash":"...","screen_type":"NONE","error":"CommunicationMod returned error"}
```

All events must include `schema_version`, `ts_ms`, and `run_id`.

## Tests

Unit tests:

- Control parser handles missing, malformed, stale, off, paused, and mode changes.
- Command state extraction handles missing fields and fixture fields.
- Protocol writers emit exact expected command strings.
- Action parser rejects non-JSON, unknown schema, unknown IDs, and invalid action counts.
- Resolver maps each Phase 1 screen correctly.
- Safety blocks unsupported screens and unsafe follow-up screens.

Fixture tests:

- Existing `card-reward-state.json`.
- Existing `rest-state.json`.
- Existing `shop-state.json` when shop enters Phase 2.
- Existing `combat-state.json` when combat enters Phase 3.
- New `combat-reward-state.json`.

Integration tests:

- Auto-play off leaves current advice behavior unchanged.
- Advise mode proposes but does not execute.
- Auto mode executes one validated Phase 1 action.
- Malformed control file blocks auto-play.
- `COMBAT_REWARD` with unknown reward payload blocks and surfaces reason.
- `COMBAT_REWARD` with known collectible rewards collects before proceeding.

Manual tests:

- Overlay enable, pause, resume, and disable.
- Card reward pick.
- Combat reward with gold plus card reward.
- Combat reward with relic.
- Combat reward with full potion slots.
- Pause/off control update immediately before executor sends a command.

## Acceptance Criteria

Phase 1 is complete when:

- Auto-play is off by default.
- Overlay can enable, pause, resume, and disable through `autoplay-control.json`.
- Overlay can display auto-play status from `overlay.json`.
- Card rewards, boss card rewards, boss relics, rest, basic events, and verified combat reward cases work.
- Unsupported combat reward payloads block safely.
- No command is sent without `ready_for_command` and matching `available_commands`.
- All auto-play decisions and executions are journaled.
- Existing advice tests still pass.

Full auto-play is complete when:

- `COMBAT_REWARD`, map, shop, combat turns, and selection screens are supported.
- The run can advance across combats, rewards, map nodes, shops, events, and rest sites without manual input when overlay settings allow it.
- Every unsupported or ambiguous state produces a clear overlay block reason instead of guessing.
