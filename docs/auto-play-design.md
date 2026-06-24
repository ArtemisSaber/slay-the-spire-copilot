# Auto-Play Mode: Design & Feasibility

## Verdict: Highly Feasible (Moderate Effort)

Auto-play is achievable with **~800-1200 lines of new/modified Rust code**. Most infrastructure already exists — the core gap is that the LLM outputs free-text advice instead of executable commands.

---

## 1. Current State: What Auto-Play Can Leverage

| Capability | Status | Notes |
|---|---|---|
| **Game communication** | Done | CommunicationMod sends game state via stdin, accepts commands via stdout |
| **Command protocol** | Done | `choose <idx>`, `key`, `click`, `wait`, `state` — already supported by CommunicationMod |
| **Game state parsing** | Done | `NormalizedState::from_raw()` maps raw JSON to typed struct with 30+ fields |
| **Screen detection** | Done | Screen gating via `SCREEN_CONFIG` + `CombatTurnGate` for combat turns |
| **Decision context** | Done | Prompt builders craft locale-aware LLM prompts per screen type |
| **LLM pipeline** | Done | Effort routing (Fast/Medium/Heavy), scenario detection, caching by state hash |
| **Run lifecycle** | Done | Journal, postmortem, run end detection |
| **Multi-language** | Done | en/zh/ja/ko locale system with system prompts + few-shot examples |

**The only missing piece is the "close the loop" — taking LLM output and sending it as a game command.**

---

## 2. What Needs to Be Built

### A. Config Toggle (trivial, ~10 lines)
- Add `AUTO_PLAY=true` (or `PLAY_MODE=auto` vs `advice`) to `.env` / `Config`
- When enabled, skip overlay output and instead execute commands
- Keep advice mode as the default — auto-play activates only when explicitly enabled

### B. Protocol Layer Expansion (trivial, ~30 lines)
- Add `send_choose(index: usize)`, `send_key(key: &str)`, `send_click(x, y)`, `send_wait(ms)` to `protocol.rs`
- Pattern: identical to existing `send_ready_to()` — `writeln!(stdout, "choose {index}")` + `flush()`
- These functions already exist in test-only form (`send_wait_to`, `send_state_to`)

### C. Structured Action Decision (core work, ~300-400 lines)

**Two viable approaches:**

**Approach 1 (Recommended): Modified LLM Prompts**
- When `AUTO_PLAY=true`, use a different set of system prompts that instruct the LLM to output a structured action format instead of advice text. Example:
  ```
  ACTION: choose 2
  REASON: Shrug It Off adds block and draw
  ```
- Add a new `parse_action_response()` function that extracts `ACTION:` lines
- Best accuracy (LLM understands game context) but higher latency/cost
- Reuses existing scenario-based prompt infrastructure — just needs new locale strings

**Approach 2: Deterministic Post-Processor**
- After receiving normal advice text, use a lightweight LLM call (or regex) to extract the choice index
- Lower latency but less reliable for complex scenarios (combat sequences, shop purchases)

**Approach 1 is recommended** — the existing `AdviceScenario` system, effort routing, and locale prompts are already perfectly set up to be extended with action-oriented variants.

### D. Action Executor (moderate, ~150-200 lines)

Screen-specific command mapping:

| Screen Type | Command | Notes |
|---|---|---|
| `CARD_REWARD` | `choose <0..N-1>` | Index into `card_reward_choices` |
| `CARD_REWARD` (skip) | `choose N` or `skip` | When LLM recommends Skip; verify `skip_available` |
| `BOSS_REWARD` | `choose <0..N-1>` | Index into `boss_relic_choices` |
| `REST` | `choose <0..N-1>` | Index into `rest_options` (Rest/Smith/Toke/Dig/Lift/Recall) |
| `EVENT` | `choose <0..N-1>` | Index into `event_choices` |
| `SHOP_SCREEN` | Sequence of `choose` | Buy/skip/purge decisions — may need confirmation handling |
| `MAP` | `choose <0..N-1>` | Select next node from available paths |
| `NONE` (combat) | `choose <0..N-1>` + `key end` | Play card from hand; `key` for end turn |
| `HAND_SELECT` | `choose <0..N-1>` | Discard/exhaust selection screens |

Key challenge: **index resolution**. The LLM doesn't know about array indices — it selects by name. The executor must:
1. Parse the LLM's action (e.g., "pick Shrug It Off")
2. Look up the index in `card_reward_choices`, `boss_relic_choices`, `event_choices`, etc.
3. Send `choose <index>`

For combat, the LLM must reference cards by **name** or **uuid** (which are already in the prompt), and the executor maps these back to hand indices.

### E. Command Readiness Tracking (small, ~50 lines)
- Extract `available_commands` and `ready_for_command` from the raw JSON (currently not parsed)
- Add to `NormalizedState` or a separate `CommandState` struct
- Only send commands when `ready_for_command: true`
- The `available_commands` field tells you what the game is prepared to accept for this screen

### F. Combat Turn Management (small, ~50 lines)
- The existing `CombatTurnGate` already detects `action_phase=WAITING_ON_USER`
- Auto-play in combat must:
  1. Receive combat state at turn start
  2. Send `choose <card_index>` commands to play cards
  3. Send `key end` to end the turn
  4. **Crucially: wait for the next game state** from CommunicationMod after each command — the game sends updated state after each action

### G. Safety/Failsafe (small, ~100 lines)
- **Time budget**: Optional max-run-duration or max-turn-duration config
- **HP threshold**: Optionally pause auto-play when HP drops below a threshold (e.g., 20%)
- **Boss fights**: Opt-in/out for boss auto-play (high stakes)
- **Error handling**: If CommunicationMod returns an error, fall back to advice mode
- **Emergency stop**: Watch for a specific output file as a kill-switch (e.g., user writes "STOP" to a control file)

---

## 3. Architecture Changes

```
main.rs loop:
  ┌─ Receive game state from stdin
  ├─ Normalize state (existing)
  ├─ Gate: should we act? (existing)
  ├─ IF AUTO_PLAY:
  │   ├─ Build auto-play prompt (new, variant of existing prompt builders)
  │   ├─ LLM query → structured action response (modified)
  │   ├─ Parse action from response (new)
  │   ├─ Validate & resolve index (new)
  │   ├─ Send command to stdout (new protocol functions)
  │   └─ Continue loop (game sends back updated state)
  └─ ELSE (advice mode):
      ├─ Build advice prompt (existing)
      ├─ LLM query → advice text (existing)
      └─ Write to overlay/advice.txt (existing)
```

### Files changed:
| File | Change |
|---|---|
| `src/protocol.rs` | Add `send_choose()`, `send_key()`, `send_click()`, `send_wait()` |
| `src/main.rs` | Add auto-play branch in main loop; toggle based on config |
| `src/config.rs` | Add `auto_play: bool` field |
| `src/state.rs` | Optionally add `available_commands`, `ready_for_command` fields |
| `src/llm.rs` | Add auto-play system prompt variants or parameterize existing ones |
| `src/prompt/builder.rs` | Add `#ACTION` output format to prompts when auto-play |
| `src/llm.rs` / new `src/action.rs` | Add `parse_action_response()` + `ActionCommand` enum |
| New: `src/executor.rs` | Screen-specific command resolution and execution |
| New: `src/safety.rs` | Safety guards (HP threshold, time limits, kill-switch) |
| `src/locales/en.json` (x4 zh/ja/ko) | Add `system_prompts.auto_play.*` and `system_prompts.combat_turn_action` |

---

## 4. Risks and Mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| **LLM mis-parses action index** | Medium | Validate index against actual array length; log mismatch; fall back to advice-only |
| **LLM hallucinates non-existent card name** | Low | Fuzzy match card names; log failure; skip turn if unresolvable |
| **High LLM latency in combat** | Medium | Use `Effort::Fast` model for combat; combat decisions are simple card plays; could batch multiple card plays in one prompt |
| **LLM makes bad decisions → run dies** | Low | This is the whole point of auto-play — accept suboptimal play as a tradeoff for automation. Log everything for postmortem. |
| **Game state changes between command and response** | Low | CommunicationMod is synchronous — it won't send new state until the copilot responds, so no race condition |
| **SHOP_SCREEN complexity** | Medium | Shops require multiple decisions (buy cards, relics, potions, purge). LLM prompt must enumerate all options with indices. Start with a simpler "shop advice → single transaction" flow. |

---

## 5. Phased Implementation Plan

### Phase 1 — Basic Decision Screens (2-3 hours)
- Config toggle + protocol functions
- CARD_REWARD, BOSS_REWARD, REST, EVENT auto-selection
- No combat, no shop, no map

### Phase 2 — Combat (2-3 hours)
- Combat turn handling: play cards until end turn
- `CombatTurnGate` already detects turn start
- LLM outputs card play sequence → executor sends `choose` commands one-by-one → waits for updated state → loops until turn end

### Phase 3 — Map & Shop (1-2 hours)
- Map node selection (next floor choice)
- Shop purchasing logic

### Phase 4 — Polish & Safety (1-2 hours)
- Error handling, index validation
- HP threshold, kill-switch
- Journal logging of auto-play actions
- Postmortem integration

---

## 6. Summary

Auto-play is not only feasible but well-supported by the existing architecture. The CommunicationMod protocol already accepts commands, the game state model is comprehensive, and the LLM pipeline is already designed per-screen-type. The primary work is:

1. **Prompt engineering** — teaching the LLM to output structured actions instead of advice
2. **Action executor** — mapping LLM output to game commands with index resolution
3. **Combat loop** — sending commands and waiting for state updates between each action

Estimated total effort: **8-12 hours** for a working v1 covering all screen types.
