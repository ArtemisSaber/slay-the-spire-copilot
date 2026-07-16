# Auto-Play Module Plan

## Goal

Implement auto-play as a separate Rust module that can be enabled, paused, resumed, and disabled from the overlay side.

The copilot should keep advice mode as the default. Auto-play is an opt-in runtime mode with explicit safety gates, command validation, journal events, and an overlay-visible status model.

## Key Design Choice

Do not let the overlay edit `output/overlay.json`.

`overlay.json` is owned by the copilot and is written atomically by the current advice layer. If the overlay also writes to it, the two processes can clobber each other. Use a second file for overlay-to-copilot control:

```text
output/overlay.json          copilot -> overlay
output/autoplay-control.json overlay -> copilot
```

The overlay writes `autoplay-control.json`; the copilot polls it on each game-state tick and optionally on a short timer before sending any command.

## Control Contract

Add a new schema file:

```text
schemas/autoplay-control.d.ts
```

Proposed shape:

```ts
export type AutoPlayMode = "off" | "advise" | "auto" | "paused";

export interface AutoPlayControl {
  schema_version: 1;
  revision: number;
  mode: AutoPlayMode;
  updated_at_ms: number;
  require_confirmation: boolean;
  allow_combat: boolean;
  allow_map: boolean;
  allow_shop: boolean;
  allow_events: boolean;
  allow_rest: boolean;
  allow_card_rewards: boolean;
  min_hp_percent: number | null;
  max_commands_per_turn: number | null;
}
```

Rules:

- `revision` must increase for every overlay write. The copilot ignores already-seen revisions.
- `mode: "off"` means no auto-play decisions are made and pending action state is cleared.
- `mode: "paused"` means no commands are executed, but pending action state may remain visible.
- `mode: "advise"` means the copilot can compute proposed actions but must not execute them.
- `mode: "auto"` means the copilot may execute validated actions when safety checks pass.
- Missing or malformed control files must fail closed to advice mode.
- The overlay should write via temp file plus rename, matching the copilot's atomic output pattern.

## Overlay Output Updates

Extend `OverlayOutput` in `src/advice.rs` and `schemas/overlay.d.ts` so the overlay can show and control auto-play state.

Proposed additional fields:

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
  command: string;
  target_label: string;
  reason: string;
  risk: string;
  requires_confirmation: boolean;
}

export interface AutoPlayState {
  mode: AutoPlayRuntimeMode;
  status: AutoPlayStatus;
  control_revision: number | null;
  last_error: string | null;
  blocked_reason: string | null;
  pending_action: AutoPlayActionView | null;
  last_executed_action: AutoPlayActionView | null;
}
```

`OverlayOutput` should add:

```ts
autoplay: AutoPlayState;
```

Keep advice fields intact for backward compatibility. Old overlays can ignore `autoplay`; new overlays can render buttons from it.

## New Rust Module

Create a new module tree:

```text
src/autoplay/
  mod.rs
  control.rs
  status.rs
  action.rs
  planner.rs
  resolver.rs
  executor.rs
  safety.rs
  prompt.rs
```

Module responsibilities:

| File | Responsibility |
|---|---|
| `control.rs` | Read and validate `output/autoplay-control.json`; track latest revision. |
| `status.rs` | Produce overlay-facing auto-play state and errors. |
| `action.rs` | Define typed action enums and parser for LLM JSON/action output. |
| `prompt.rs` | Build the screen-aware structured scenario and prompt-scoped action refs without exposing execution-local IDs. |
| `planner.rs` | Ask LLM for an action or action sequence; no stdout writes. |
| `resolver.rs` | Map chosen refs through authoritative internal action IDs to current game-state choices, hand indices, targets, or map nodes. |
| `executor.rs` | Send validated CommunicationMod commands to stdout. |
| `safety.rs` | Enforce mode, HP threshold, allowed screen flags, confirmation, command budget, and readiness. |
| `mod.rs` | Public `AutoPlayController` facade used by `main.rs`. |

`main.rs` should call only the facade, not the internal files.

## Public API Sketch

```rust
pub struct AutoPlayController {
    control: AutoPlayControlState,
    status: AutoPlayStatusState,
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

`handle_state()` should:

1. Read latest overlay control.
2. Fail closed if mode is off, malformed, or safety-blocked.
3. Check `ready_for_command` and `available_commands`.
4. Build an auto-play prompt only for supported screens.
5. Parse a typed action.
6. Resolve it against the current state.
7. Execute only if mode is `auto` and confirmation is not required.
8. Record journal events.
9. Update overlay status.

## Required Existing File Updates

| File | Update |
|---|---|
| `src/main.rs` | Instantiate `AutoPlayController`; route eligible states through it before advice output. |
| `src/config.rs` | Add conservative defaults such as `AUTOPLAY_ENABLED_DEFAULT=false`; overlay remains runtime authority. |
| `src/state.rs` | Add command metadata or create `CommandState` from raw JSON: `ready_for_command`, `available_commands`, `choice_list`. |
| `src/protocol.rs` | Add production command writers for `choose`, `play`, `end`, `leave`, `return`, `skip`, `wait`, `state`, and maybe `key`/`click` after protocol verification. |
| `src/advice.rs` | Extend overlay JSON with `autoplay`; keep existing advice fields. |
| `src/journal.rs` | Add auto-play events: proposed, executed, blocked, failed, control_changed. |
| `src/llm.rs` | Add an action-query path or allow planner to call `query_with_system_prompt` using action prompts. |
| `src/prompt/builder.rs` | Do not overload human advice prompts; add auto-play prompt builders under `src/autoplay/prompt.rs`. |
| `schemas/overlay.d.ts` | Add `AutoPlayState`. |
| `schemas/autoplay-control.d.ts` | New overlay-to-copilot control schema. |
| `README.md` | Document overlay control file and runtime safety behavior once implemented. |

## Command Protocol Gap

Before implementing executor logic, verify CommunicationMod command grammar in a live/manual test or upstream docs.

Fixtures currently show:

| Fixture | Available commands |
|---|---|
| `card-reward-state.json` | `choose`, `proceed`, `skip`, `key`, `click`, `wait`, `state` |
| `rest-state.json` | `choose`, `return`, `key`, `click`, `wait`, `state` |
| `shop-state.json` | `choose`, `leave`, `key`, `click`, `wait`, `state` |
| `combat-state.json` | `play`, `end`, `key`, `click`, `wait`, `state` |

That means combat should not be designed around `choose <hand_index>` until verified. The first executor should dispatch from `available_commands`, not from hard-coded screen assumptions.

## Action Schema

Prefer structured JSON from the LLM, not localized `ACTION:` text.

Example:

```json
{
  "schema_version": 2,
  "actions": [
    {
      "ref": "A1",
      "reason": "Cheap damage helps Act 1 hallway fights.",
      "risk": "Adds deck size."
    }
  ]
}
```

The prompt builder exposes temporary `A0..An` references and the LLM echoes
one reference. The resolver maps it to an internal action ID. This avoids fuzzy
localized-name matching without exposing UUIDs to the model. References remain
stable across retries for one state and have no meaning on later states.

The model receives a single structured `scenario` object rather than parallel
localized prose and compact-state representations. Card, relic, potion,
monster, pile, event, shop, selection, and route facts are attached to their
typed screen section. Each prompt action may additionally carry a structured
source object and an index that links it to that scenario.

Examples:

| Screen | Internal action IDs hidden behind refs |
|---|---|
| Card reward | `card_reward:0`, `card_reward:1`, `card_reward:skip` |
| Boss relic | `boss_relic:0`, `boss_relic:1` |
| Rest | `rest:rest`, `rest:smith`, `rest:toke`, `rest:recall` |
| Event | `event:0`, `event:1` |
| Shop | `shop:choice:0`, `shop:choice:1`, matching raw `choice_list` |
| Map | `map:child:x:y` plus a resolved command index |
| Combat | `combat:play:<card_uuid>` plus optional `target:<monster_index>` |

## Screen Support Plan

### Phase 1: Overlay Control and Non-Combat Actions

Implement:

- `autoplay-control.json` reader.
- Overlay `autoplay` status.
- `CARD_REWARD`, `BOSS_REWARD`, `REST`, basic `EVENT`.
- Journal events.
- Safety gates and pause/off control.

Do not implement combat, shop, map, hand-select, or grid selection yet.

This phase gives a useful, testable auto-play loop without touching the hardest game mechanics.

### Phase 2: Shop and Map

Implement:

- Shop via raw `choice_list` indices, not separate card/relic/potion letter labels.
- Single transaction per state tick.
- Explicit `leave` handling only after planned purchases are complete.
- Map child selection with a preserved child-to-command mapping.

### Phase 3: Combat MVP

Implement:

- One action per state tick.
- Use `play`/`end` only if present in `available_commands`.
- Resolve cards by UUID, not name.
- Resolve monster targets by current monster index.
- Stop after `max_commands_per_turn`.
- Re-read control before every command so the overlay can pause quickly.

Do not batch a full combat sequence initially. State changes after every card can invalidate hand indices, energy, and targets.

### Phase 4: Selection Screens

Implement:

- `HAND_SELECT`, `GRID`, and other generated screens.
- Context tracking from the previous action that caused the selection.
- Conservative fallback to blocked/advice when context is unknown.

These screens are mandatory for full unattended runs because many cards, relics, events, and shop purges create follow-up selections.

## Main Loop Integration

The main loop should keep the current advice path but add an auto-play path before advice output:

```text
read raw state
normalize state
log state change
read overlay control

if auto-play is active:
  outcome = autoplay.handle_state(...)
  write overlay status
  if outcome executed/proposed/blocked:
    skip normal advice unless mode is "advise"

if advice should be generated:
  existing advice flow
```

Important: action execution should not use `AdviceCache`. Advice caching is useful for text suggestions, but command decisions are stateful and should use either no cache or a stricter key that includes command readiness, raw choice order, card UUIDs, monster indices, and control revision.

## Safety Rules

Auto-play should fail closed. It should not send a command when:

- `ready_for_command` is false.
- `available_commands` does not contain the command.
- The control file is missing, malformed, stale, or says off/paused.
- HP is below `min_hp_percent`.
- The screen type is not enabled by overlay flags.
- The action cannot be resolved exactly.
- The action requires a follow-up selection that is unsupported.
- The LLM output has more than one action on a screen that only supports one safe action.
- The per-turn command budget is reached.
- The last CommunicationMod message was an error.

If blocked, write the reason into overlay state and continue advice mode if possible.

## Journal Events

Add events:

```json
{"event":"autoplay_control_changed","revision":12,"mode":"auto"}
{"event":"autoplay_action_proposed","state_hash":"...","action":{...}}
{"event":"autoplay_action_executed","state_hash":"...","command":"choose 1","action":{...}}
{"event":"autoplay_action_blocked","state_hash":"...","reason":"ready_for_command=false","action":{...}}
{"event":"autoplay_action_failed","state_hash":"...","error":"CommunicationMod returned error"}
```

Postmortem can later summarize these as decisions made by auto-play versus advice shown to the user.

## Testing Plan

Unit tests:

- Control JSON parsing, stale revisions, malformed file fallback.
- Safety gates for each block reason.
- Action parser distinguishes invalid JSON from invalid schemas and rejects unknown refs and ambiguous choices.
- Resolver maps each supported screen to the correct command.
- Protocol command writers produce exact expected lines.

Integration tests:

- Fixture-driven auto-play proposed action for card reward/rest/event.
- Fixture-driven execution path with a fake writer instead of stdout.
- Overlay status contains `pending_action`, `last_executed_action`, and block reasons.
- Advice mode remains unchanged when auto-play is off.

Manual tests:

- Start in advice mode, enable auto from overlay, verify one basic command.
- Pause before a pending command, verify no stdout command is sent.
- Stop during combat, verify immediate fail-closed behavior.
- Test malformed control file, stale revision, and missing file.

## Implementation Order

1. Add schemas for control and extended overlay output.
2. Add `src/autoplay/control.rs` and `status.rs`.
3. Extend `AdviceCache`/overlay writer to include auto-play status.
4. Add command metadata extraction from raw state.
5. Add protocol writers behind tests.
6. Add action schema, parser, and resolver for Phase 1 screens.
7. Add `AutoPlayController` facade and integrate into `main.rs`.
8. Add journal events.
9. Add README notes for overlay control.

This keeps auto-play isolated while still allowing the overlay to control it dynamically.
