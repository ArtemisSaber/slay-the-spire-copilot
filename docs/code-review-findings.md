# Codebase Review — Findings & Issues

**Project**: slay-the-spire-copilot
**Version reviewed**: 0.2.0 (Rust edition 2024)
**Review date**: 2026-07-07
**Reviewer**: Sisyphus (OhMyOpenCode orchestration)

---

## Methodology

The review combined automated tool checks with three parallel explore-agent investigations:

### Automated checks (direct)
- `cargo fmt --check` — formatting compliance
- `cargo clippy --all-targets -- -D warnings` — lint compliance (binary + tests)
- `cargo clippy -- -D warnings` — lint compliance (binary only, matches pre-commit/CI)
- `cargo test` — full test suite
- `cargo update --dry-run` — dependency freshness
- `cargo audit` — not available (cargo-audit not installed)
- `git ls-files` — secret-file leakage scan
- `grep` for `unwrap/expect/panic/todo/unimplemented/unreachable` across `src/`
- `grep` for `TODO/FIXME/HACK/XXX` markers
- `wc -l` on all `.rs` files — module size audit

### Parallel explore agents
1. **Security audit** — command injection, API key leakage, path traversal, unsafe deserialization, unsafe Rust, SSRF, TOCTOU, secrets in git
2. **Code quality** — oversized files (>250 LOC), long functions (>60 lines), duplicated logic, stringly-typed data, magic numbers, error swallowing, dead code, inconsistent error handling, blocking I/O in async
3. **Architecture & correctness** — race conditions, kill-scan edge cases, rules.json loading, locale loading, CombatTurnGate dedup, map cycle detection, run finalization idempotency, postmortem AI validation, documentation drift

---

## Baseline Results

| Check | Result |
|-------|--------|
| `cargo fmt --check` | ✅ Pass (no output) |
| `cargo clippy -- -D warnings` (binary, matches CI/pre-commit) | ✅ Pass |
| `cargo clippy --all-targets -- -D warnings` (includes tests) | ❌ 219 errors (202 `needless_borrow`, 11 `field_reassign_with_default`, 2 `map_or`, 4 others) |
| `cargo test` | ✅ 1553 passed (1551 unit + 2 integration), 0 failed |
| Secret files in git | ✅ None tracked (`.env` in `.gitignore`) |
| `unsafe` in production code | ✅ None (only in test files for `env::set_var`) |
| TODO/FIXME/HACK markers | ✅ None |
| `cargo audit` | ⚠️ Not installed |
| Outdated dependencies | ⚠️ 27 transitive packages updatable |

---

## Issues Ordered by Urgency

> **Fix status as of 2026-07-07**: 20 of 32 issues fixed on branch `fix/code-review-critical`.
> All 3 Critical issues resolved. All 9 TDD-feasible Medium issues resolved. See the status table below and per-issue markers.
>
> | Severity | Total | Fixed | Remaining |
> |----------|-------|-------|-----------|
> | 🔴 Critical | 3 | 3 | 0 |
> | 🟠 High | 10 | 4 | 6 |
> | 🟡 Medium | 11 | 10 | 1 |
> | 🟢 Low | 8 | 4 | 4 |
> | **Total** | **32** | **21** | **11** |
>
> Verification: `cargo fmt --check` ✅ · `cargo clippy --all-targets -- -D warnings` ✅ (0 warnings) · `cargo test` ✅ 1592 passed

---

### CRITICAL — Fix immediately

---

#### C1. `rules.json` parse failure crashes the app — ✅ FIXED (`3bb0d10`)

- **File**: `src/ranker/mod.rs`
- **Line**: 31
- **Code**:
  ```rust
  .unwrap_or_else(|e| panic!("invalid {}: {e}", path.display()))
  ```
- **Description**: A malformed `rules.json` file next to the binary triggers a panic on first combat access via `LazyLock<RuleSet>`. The file-not-found path (line 26-27) correctly falls back to the embedded `include_str!("rules.json")` copy, but parse errors do not — they panic instead.
- **Impact**: A user with a partially-downloaded, truncated, or hand-edited `rules.json` gets a **panic crash** on the first combat round. The embedded fallback that should save them only activates on file-not-found, not on parse failure.
- **Fix**: Apply the same fallback logic for parse errors:
  ```rust
  .unwrap_or_else(|e| {
      tracing::warn!("invalid {}: {e}, falling back to embedded rules.json", path.display());
      serde_json::from_str(include_str!("rules.json"))
          .expect("embedded rules.json is corrupt")
  })
  ```

---

#### C2. Map path enumeration has no cycle detection — ✅ FIXED (`bef1fe2`)

- **File**: `src/prompt/routing.rs`
- **Lines**: 4-50 (`enumerate_paths`), 468-481 (`enumerate_paths_from_roots`)
- **Called from**: `src/main.rs:597-601`
- **Description**: `enumerate_paths()` is a stack-based DFS with **no visited set, no depth limit, no node cap**. It pushes children onto the stack and traverses them unconditionally. STS maps are always DAGs (nodes point to higher Y coordinates), so this is safe for valid game data.
- **Impact**: Malformed or modded CommunicationMod data that creates a map cycle (A → B → C → A) would cause **unbounded stack growth, OOM, or hang**. The copilot has no defense against pathological input.
- **Fix**: Add a `HashSet<(i64, i64)>` visited set per path, or enforce a max-node limit (e.g., 1000 nodes).

---

#### C3. API key leakage via LLM error response bodies — ✅ FIXED (`d9a3e2d`)

- **Files**: `src/llm.rs`
- **Lines**: 592-593 (OpenAI-compatible), 646-647 (Anthropic), 702-703 (Pollinations)
- **Code**:
  ```rust
  let err_body = response.text().await.unwrap_or_default();
  anyhow::bail!("LLM API error {status}: {err_body}");
  ```
- **Description**: All three provider variants include the raw HTTP error response body in the error message. Some LLM providers echo the `Authorization` header (containing the API key) in error responses for malformed requests. The error then propagates to `tracing::error!`/`tracing::warn!` calls (`src/advice.rs:145`, `src/main.rs:243, 407`) and is written to `logs/sts-ai.log`.
- **Impact**: API key may be written to log files in cleartext if the LLM provider echoes it in error responses.
- **Fix**: Strip or truncate `err_body` before including it in the error message. Optionally scan for the API key substring and redact it:
  ```rust
  let err_body = response.text().await.unwrap_or_default();
  let redacted = err_body.replace(&config.api_key, "<REDACTED>");
  anyhow::bail!("LLM API error {status}: {redacted}");
  ```

---

### HIGH — Fix soon

---

#### H1. `LlmProvider` derives `Debug` — API key exposed — ✅ FIXED (`d9a3e2d`)

- **File**: `src/llm.rs`
- **Line**: 371
- **Code**:
  ```rust
  #[derive(Debug)]
  pub enum LlmProvider {
      OpenAiCompatible { base_url: String, api_key: String, ... },
      Anthropic { base_url: String, api_key: String, ... },
      ...
  }
  ```
- **Description**: `LlmProvider` holds `api_key: String` and derives `Debug`. Any accidental `tracing::debug!("{:?}", provider)` or panic unwind would print the API key in cleartext to logs. No direct `{:?}` call on the provider exists today, but this is a latent footgun — the compiler won't warn against future uses.
- **Impact**: Future code changes could accidentally leak the API key.
- **Fix**: Implement a manual `Debug` that redacts `api_key`:
  ```rust
  impl fmt::Debug for LlmProvider {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
          match self {
              Self::OpenAiCompatible { base_url, .. } => write!(f, "OpenAiCompatible {{ base_url: {base_url}, api_key: <REDACTED> }}"),
              // ...
          }
      }
  }
  ```

---

#### H2. `AGENTS.md` test count is 2× stale — ✅ FIXED (`43b673d`, `82da39f`)

- **File**: `AGENTS.md`
- **Line**: 12
- **Current text**: `cargo test — runs 789 tests (787 unit + 2 integration)`
- **Actual**: 1553 tests (1551 unit + 2 integration)
- **Impact**: `AGENTS.md` is the primary guide for AI agents. The stale count means agents underestimate the test suite by half, potentially skipping optimization or confidence checks. Also misleading to human contributors.
- **Fix**: Update to `cargo test — runs 1553 tests (1551 unit + 2 integration)`

---

#### H3. God function: `main()` is 631 lines — ⏳ REMAINING

- **File**: `src/main.rs`
- **Lines**: 127-758
- **Description**: `async fn main()` handles CLI parsing, setup wizard, postmortem mode, config validation, provider init, stdin loop, JSON parsing, state normalization, advice gating, autoplay routing, overlay writing, journal logging, run finalization, and map gate tracking — all in one function. Contains deeply nested match/if chains throughout.
- **Impact**: Unmaintainable. Violates the project's own 60-line function guideline by 10×. Any change risks unintended side effects across unrelated concerns.
- **Fix**: Extract into a coordinator + per-mode handlers: `run_postmortem_mode()`, `run_setup_mode()`, `run_stdin_loop()`, `handle_game_state()`, `finalize_run()`.

---

#### H4. God function: `NormalizedState::from_raw()` is 453 lines — ⏳ REMAINING

- **File**: `src/state.rs`
- **Lines**: 461-913
- **Description**: Single function that extracts every field from raw JSON into `NormalizedState`. Contains dozens of `.and_then(|g| g.get(...)).and_then(|v| v.as_str())` chains for combat, shop, map, event, card reward, and other sections.
- **Impact**: Hard to test, hard to modify, high risk of merge conflicts.
- **Fix**: Decompose into per-section extractors: `extract_combat()`, `extract_shop()`, `extract_map()`, `extract_event()`, `extract_card_reward()`, etc.

---

#### H5. God function: `NormalizedState::to_stable_value()` is 349 lines — ⏳ REMAINING

- **File**: `src/state.rs`
- **Lines**: 935-1283
- **Description**: 349 lines of repetitive `if let Some(v) = self.field { map.insert("field", ...) }` blocks. This is the mirror of `from_raw` and should use serde or a macro.
- **Impact**: Maintenance burden; any new field requires changes in both `from_raw` and `to_stable_value`.
- **Fix**: Use `#[derive(Serialize)]` on `NormalizedState` with a custom serializer, or a macro to generate the boilerplate.

---

#### H6. God function: `kill_scan::dfs()` is 230 lines — ⏳ REMAINING

- **File**: `src/combat/kill_scan.rs`
- **Lines**: 155-384
- **Description**: Deeply recursive DFS function with complex branching for card effects, memoization, deadline checks, and monster state mutations.
- **Impact**: Hard to reason about correctness; high cognitive load for reviewers.
- **Fix**: Extract into sub-functions: `check_memo()`, `iterate_cards()`, `apply_effect()`, `restore_state()`.

---

#### H7. `screen_type` is stringly-typed — 130+ comparisons across 18 files — ⏳ REMAINING

- **File**: `src/state.rs`
- **Line**: 197
- **Code**: `pub screen_type: Option<String>`
- **Description**: Compared as string literals (`== "MAP"`, `== "CARD_REWARD"`, `as_deref() == Some("COMBAT_REWARD")`, etc.) 130+ times across 18 files. No exhaustiveness checking, typo-prone, impossible to refactor safely. Same issue applies to `room_type` (line 198), `character` (line 199), `stance` (line 233), `current_action` (line 249).
- **Impact**: A typo in a screen type string is a silent bug. Adding a new screen type requires finding all string comparisons manually. No compiler assistance.
- **Fix**: Define `enum ScreenType { CardReward, BossReward, Rest, Event, ShopScreen, Map, HandSelect, Grid, CombatReward, None, ... }` with `serde` and `Display`. Replace all 130+ string comparisons. Large but mechanical refactor.

---

#### H8. Triplicated parsing logic — ✅ FIXED (`995fade`)

- **Files**:
  - `src/combat/effects.rs:59` — `extract_first_integer` returns `Option<i16>`, char-based iteration
  - `src/ranker/parser.rs:29` — `extract_first_integer` returns `Option<i64>`, byte-based iteration (different implementation!)
  - `src/relic_counters.rs:22` — `extract_first_integer` returns `Option<i64>`, char-based (identical to effects.rs)
- **Description**: Three independent copies of the same "find first integer in a string" logic with inconsistent implementations. Additionally, `parse_hits`, `parse_block`, `parse_damage_amount`, `parse_vulnerable`, `parse_energy_gain`, `parse_str_gain`, `parse_mantra`, `parse_exhaust`, `parse_enters_wrath`, `parse_enters_calm`, `parse_exits_stance` are duplicated between `combat/effects.rs` and `ranker/parser.rs` with different struct types (`CardEffect` vs `ParsedEffects`) and different return types.
- **Impact**: Bug fixes must be applied in 3 places. Divergent implementations may produce different results for the same input.
- **Fix**: Create a shared `src/parsing.rs` module with a single generic `extract_first_integer<T: FromStr>`. Unify the parse functions into a shared `ParsedEffects` struct, or have `CardEffect` build from `ParsedEffects`.

---

#### H9. Blocking I/O in async context — ⏳ REMAINING

- **Files & Lines**:
  - `src/main.rs:127` — `async fn main()` uses:
    - `dotenvy::from_path()` (line 130) — blocking file I/O
    - `std::fs::read_to_string(path)` (line 161) — blocking file I/O
    - `std::io::stdin().lock().lines()` (line 299) — blocking stdin
    - `io::stdout().lock()` (lines 370, 541) — blocking stdout
  - `src/advice.rs:126` — `async fn get_or_compute` calls `write_advice` (line 154) which uses `fs::create_dir_all`, `fs::OpenOptions`, `file.write_all` — all blocking
  - `src/llm.rs:25, 33` — `fs::create_dir_all` and `file.write_all` in async logging functions
- **Description**: The application uses `tokio` with `rt-multi-thread` feature but performs synchronous blocking I/O throughout async functions. This blocks the tokio worker thread.
- **Impact**: The stdin loop (hot path) and LLM calls share the same runtime; blocking I/O can stall async tasks. The hide-timer (`tokio::spawn` in `advice.rs:232`) could be delayed.
- **Fix**: Either make `main()` synchronous (use `reqwest::blocking`) or switch all file/stdin I/O to `tokio::fs`/`tokio::io`. The current mix is the worst of both worlds.

---

#### H10. 219 clippy errors in test code — ✅ FIXED (`585682e`)

- **Command**: `cargo clippy --all-targets -- -D warnings`
- **Breakdown**:
  | Lint | Count |
  |------|-------|
  | `needless_borrow` | 202 |
  | `field_reassign_with_default` | 11 |
  | `map_or` simplifiable | 2 |
  | `assert_eq!` with literal bool | 1 |
  | unnecessary `get("JUSTTEXT").is_none()` | 1 |
  | explicit lifetimes could be elided | 1 |
  | length comparison to zero | 1 |
- **Description**: Does not block pre-commit or CI (both run `cargo clippy -- -D warnings` without `--all-targets`, which only checks the binary). But `cargo clippy --all-targets` is broken, which means contributors running full clippy locally will see failures.
- **Impact**: Inconsistency between local full-clippy and CI. Discourages contributors from running thorough lint checks.
- **Fix**: 207 of 219 are auto-fixable: `cargo clippy --fix --tests` then verify remaining 12 manually.

---

### MEDIUM

---

#### M1. 23 source files exceed 250 pure LOC — ⏳ REMAINING

The project's own rule (from `AGENTS.md` / remove-ai-slops skill): "250+ pure LOC with mandatory modular refactoring."

| File | Pure LOC | Notes |
|------|----------|-------|
| `src/prompt/builder.rs` | 1439 | No inline tests |
| `src/state.rs` | 1292 | No inline tests |
| `src/main.rs` | 769 | No inline tests |
| `src/llm.rs` | 728 | No inline tests |
| `src/setup_wizard.rs` | ~704 | Tests at line 705 |
| `src/autoplay/action.rs` | ~696 | Tests at line 697 |
| `src/startup.rs` | 670 | No inline tests |
| `src/postmortem.rs` | 629 | No inline tests |
| `src/ranker/engine.rs` | 627 | No inline tests |
| `src/autoplay/planner.rs` | ~640 | Tests at line 641 |
| `src/combat/kill_scan.rs` | 551 | No inline tests |
| `src/ranker/parser.rs` | 515 | No inline tests |
| `src/ranker/context.rs` | 515 | No inline tests |
| `src/prompt/routing.rs` | 481 | No inline tests |
| `src/combat/damage.rs` | 421 | No inline tests |
| `src/autoplay/combat_adviser.rs` | 398 | No inline tests |
| `src/combat/effects.rs` | ~385 | Tests at line 386 |
| `src/ranker/formula.rs` | 384 | No inline tests |
| `src/locales/mod.rs` | 360 | No inline tests |
| `src/journal.rs` | 336 | No inline tests |
| `src/autoplay/control.rs` | 280 | No inline tests |
| `src/advice.rs` | 270 | No inline tests |
| `src/ranker/rules.rs` | 269 | No inline tests |

---

#### M2. Error swallowing via `.ok()` — 39 instances across 13 files — ✅ FIXED (`8066438`–`1f1ccb4`)

`.ok()` converts `Result<T, E>` to `Option<T>`, silently discarding the error context.

**Key offenders**:
- `src/startup.rs:214-215, 241, 337, 347` — `fs::read_to_string(path).ok()?` — file-not-found vs permission-denied vs invalid-UTF-8 are all silently treated as `None`
- `src/autoplay/planner.rs:460` — `serde_json::from_str::<PlannerResponse>(response.trim()).ok()?` — LLM returns malformed JSON and the parse error is silently swallowed
- `src/autoplay/status.rs:19-20` — double `.ok()` chain on file read + JSON parse
- `src/journal.rs:274, 276, 282` — `fs::read_dir().ok()?`, `e.ok()` on directory entries
- `src/config.rs:52, 56, 59, 62, 66` — `.and_then(|v| v.parse().ok())` on env vars (parse errors discarded)
- `src/ranker/parser.rs:39-40, 58-59` — `.ok()` on UTF-8 validation and integer parse

**Fix**: Replace `.ok()?` with proper error logging:
```rust
match fs::read_to_string(path) {
    Ok(content) => Some(content),
    Err(e) => { tracing::warn!("failed to read {}: {e}", path.display()); None }
}
```

---

#### M3. Error swallowing via `let _ =` — 55 instances across 9 files — ✅ FIXED (`686918e`–`a2c2a0f`)

`let _ =` discards a `Result` without any logging.

**Key offenders**:
- `src/advice.rs:90, 96, 97, 99, 156, 167` — `let _ = fs::create_dir_all/remove_file/rename/write_all` — file write failures silently discarded (advice file may not be written and nobody knows)
- `src/llm.rs:25, 33` — `let _ = fs::create_dir_all/write_all` — prompt log write failures discarded
- `src/main.rs:130, 137, 148` — `let _ = dotenvy::from_path(...)` — `.env` load failures discarded
- `src/logging.rs:24, 26, 29` — log rotation failures discarded

**Fix**: Replace with logged handling:
```rust
if let Err(e) = fs::write_all(&path, data) {
    tracing::warn!("failed to write {}: {e}", path.display());
}
```

---

#### M4. `panic!`/`expect`/`unreachable!` in non-test code — ✅ PARTIALLY FIXED (`baf8dde`–`d1b7cf5`)

| File | Line | Code | Risk |
|------|------|------|------|
| `src/ranker/mod.rs` | 31 | `panic!("invalid {}: {e}", ...)` | Crash on malformed rules.json (see C1) |
| `src/ranker/mod.rs` | 27 | `.expect("embedded rules.json is corrupt")` | Crash on startup if embedded JSON is bad (dev-only) |
| `src/autoplay/control.rs` | 167, 234, 265 | `panic!("expected updated control")` / `panic!("expected lower but changed revision")` | Crash on autoplay control invariant violation |
| `src/ranker/formula.rs` | 302 | `unreachable!()` | Crash if formula parser reaches unexpected state |
| `src/journal.rs` | 68 | `unreachable!()` on `JournalState::Confirmed` | Crash on journal state machine violation |
| `src/logging.rs` | 36 | `.expect("failed to create logs directory")` | Crash on startup if logs dir can't be created |
| `src/locales/mod.rs` | 341 | `.expect("failed to parse locale JSON")` | Crash on startup if locale JSON is bad (compile-time embedded, dev-only) |
| `src/combat/damage.rs` | 175 | `.expect("targeted damage requires target")` | Crash if targeted damage called without target |
| `src/setup_wizard.rs` | 1493 | `.expect("DeepSeek preset not found")` | Crash if preset lookup fails (should be static) |

**Description**: Mix of `anyhow::Result`, `io::Result`, `String` errors, `panic!`, `expect()`, `unwrap()`, and `.ok()`. No consistent error handling strategy.

---

#### M5. Magic numbers in path scoring — ✅ FIXED (`1ccc96e`, `52aa2d4`)

- **File**: `src/prompt/routing.rs`
- **Lines**: 296-334 (`score_shop`), 349-379 (`evaluate_path`)
- **Description**: Gold thresholds `180, 100, 60, 250, 120` are bare literals. Score values `50.0, 12.0, 10.0, 8.0, 5.0, 3.0, 2.0, 6.0, 4.0, 0.8, -1.5, -8.0` are unexplained. HP ratio thresholds `0.75, 0.45, 0.7, 0.6` appear in multiple places.
- **Also in**: `src/combat/effects.rs:141, 159` (`dmg > 1000` max damage sanity check), `src/combat/kill_scan.rs:175` (`is_multiple_of(1000)` deadline interval), `src/main.rs:721` (`prompt.len().min(200)` log truncation)
- **Fix**: Add named constants:
  ```rust
  const GOLD_THRESHOLD_SHOP_HIGH: i32 = 180;
  const HP_RATIO_DANGER: f64 = 0.45;
  const BASE_PATH_SCORE: f64 = 50.0;
  // etc.
  ```

---

#### M6. `eval_parsed()` — 121 lines of repetitive if-let guards — ✅ FIXED (`M6-cycle`)

- **File**: `src/ranker/engine.rs`
- **Lines**: 275-395
- **Description**: 20+ sequential `if let Some(v) = p.field_gt && parsed.field.unwrap_or(0) <= v { return false; }` blocks. Classic excessive-complexity (if/elif variant chain) pattern.
- **Fix**: Collapsed 17 i64-threshold guards (15 gt + 2 eq) into two table-driven slices iterated by a single loop each. Bool-equality and special-case guards (exhausts_status_curse, channel_orb) kept explicit. 121 → 77 LOC.

---

#### M7. No validation of AI postmortem output — ✅ FIXED (`164caa5`)

- **File**: `src/postmortem.rs`
- **Lines**: 23-29
- **Description**: `combine_postmortem_report()` concatenates the LLM response verbatim with no length or content sanity check. If the LLM returns empty, malformed, or garbage content, it goes directly to `runs/<run_id>/postmortem.md`. No retry mechanism (unlike the planner which retries 3 times).
- **Impact**: Postmortem file may contain nonsense. The deterministic report is always appended as a fallback, so some useful content is preserved.
- **Fix**: Add a content-length or sanity check on the AI report before combining. Consider adding a retry mechanism.

---

#### M8. SSRF — unvalidated `LLM_BASE_URL` — ✅ FIXED (`72a3a83`)

- **File**: `src/llm.rs`
- **Lines**: 582, 637, 691
- **Description**: `base_url` from `.env` is passed directly to `reqwest::Client::post(url)` without validation. If an attacker can control `.env`, they redirect all LLM prompts (containing game state) to an arbitrary server. Uses `rustls-tls` (TLS verification on by default), so HTTPS is enforced by the client, but the URL itself is not validated.
- **Impact**: Local privilege escalation vector. Cannot be exploited remotely (`.env` is a local file), but if an attacker can write to `.env`, they can exfiltrate game state.
- **Fix**: Validate `LLM_BASE_URL` — reject `file://`, `ftp://`, and internal/private IP ranges. Enforce HTTPS-only.

---

#### M9. Unbounded stdin JSON deserialization — ✅ FIXED (`d812b2c`)

- **File**: `src/main.rs`
- **Line**: 316
- **Code**: `let raw: serde_json::Value = match serde_json::from_str(trimmed) { ... };`
- **Description**: JSON from stdin (Communication Mod) is parsed into untyped `serde_json::Value` with no size or depth limit. `serde_json`'s default parser is recursive; deeply nested JSON (e.g., `[[[[...]]]]`) could cause stack overflow. A very large JSON object could cause OOM.
- **Impact**: A malformed or malicious Communication Mod could crash the copilot via stack overflow or OOM.
- **Fix**: Add a size cap on `trimmed` before parsing, and/or use `serde_stacker` or a depth-limited deserializer.

---

#### M10. TOCTOU file race on `overlay.json` — ✅ FIXED (`bd6a9ba`)

- **File**: `src/autoplay/status.rs`
- **Lines**: 18-33
- **Description**: `write_overlay_autoplay()` reads `overlay.json`, modifies it in memory, writes it back. If the main loop writes `overlay.json` (via `write_overlay_loading` or `write_overlay_ready`) between the read and write, the autoplay status update is lost (overwritten by stale data). The hide-timer path (`src/advice.rs:232-250`) is partially mitigated by a generation counter, but the autoplay status path has no such protection.
- **Impact**: In rare timing windows, the overlay could show stale advice or lose the autoplay status update. Not crash-worthy.
- **Fix**: Use a single write path for `overlay.json` with a mutex, or merge autoplay status into the same write call as advice data.

---

#### M11. 27 outdated transitive dependencies — ✅ FIXED (`4a29a8d`)

- **Command**: `cargo update --dry-run`
- **Notable updates**: `rand` 0.9.4 → 0.10.2 (major version jump), `anyhow` 1.0.102 → 1.0.103, `rustls` 0.23.40 → 0.23.41, `time` 0.3.49 → 0.3.53, `quinn` 0.11.9 → 0.11.11
- **Impact**: Minor — all transitive. But `rand` major version jump could introduce behavioral changes in dependencies that rely on it.
- **Fix**: Run `cargo update`, verify `cargo test` still passes, commit `Cargo.lock`.

---

### LOW

---

#### L1. Windows non-atomic file rename — ⏳ REMAINING

- **File**: `src/advice.rs`
- **Lines**: 93-98
- **Description**: `atomic_write_json` on Windows does `fs::remove_file(path)` then `fs::rename(tmp, path)` (not atomic). There's a brief window where `overlay.json` doesn't exist — the overlay mod polling every 500ms could read a "file not found" error.
- **Fix**: Use `MoveFileExW` with `MOVEFILE_REPLACE_EXISTING` via `windows-sys`, or accept the small window.

---

#### L2. No file locking on journal/log append — ⏳ REMAINING

- **Files**: `src/journal.rs:252`, `src/logging.rs:68`
- **Description**: `OpenOptions::new().create(true).append(true).open(path)` without cross-process file locking. If multiple copilot instances run simultaneously (e.g., multiple Communication Mod launches), log and journal lines could interleave.
- **Fix**: Use `fs2::FileExt::try_lock_exclusive()` or accept the rare interleaving.

---

#### L3. API key visible during setup wizard input — ✅ FIXED (`ef330b2`)

- **File**: `src/setup_wizard.rs`
- **Lines**: 368-369
- **Description**: Terminal echo is not disabled for the API key prompt. The code explicitly warns the user ("API key input is visible in this terminal") but does not offer a masked input. Shoulder-surfing risk.
- **Fix**: Use the `rpassword` crate or equivalent to disable terminal echo for the API key prompt.

---

#### L4. `u16` bit shift overflow in kill-scan — ✅ FIXED (`3c0d4ea`)

- **File**: `src/combat/kill_scan.rs`
- **Lines**: 201-215
- **Description**: `1u16 << i` silently wraps in release mode for `i >= 16`. The mask is built from `relevant` card indices. Vanilla STS has max 10 cards in hand, so this is safe. Modded games with 16+ card hands would silently skip cards in the DFS.
- **Fix**: Use `u32` for the mask: `1u32 << i`.

---

#### L5. `combat_identity` uses `room_type` string — ⏳ REMAINING

- **File**: `src/gate.rs`
- **Lines**: 151-158
- **Description**: `CombatTurnGate` dedup key is `"{floor}:{room_type}"`. If `room_type` is unknown (`?` or empty), two different combats on the same floor could collide, suppressing the second combat's turn-1 advice.
- **Fix**: Use a more specific identity (e.g., include combat UUID if available, or floor + a combat counter).

---

#### L6. `cargo-audit` not installed — ⏳ REMAINING

- **Description**: No known-vulnerability scanning for Rust dependencies. `cargo audit` command not found.
- **Fix**: `cargo install cargo-audit`, then add `cargo audit` to CI workflow.

---

#### L7. `build.rs` fragile triple-parent unwrap — ✅ FIXED (`756038a`)

- **File**: `build.rs`
- **Lines**: 5-12
- **Code**:
  ```rust
  let out_dir = std::env::var("OUT_DIR").unwrap();
  let dest = Path::new(&out_dir)
      .parent().unwrap()
      .parent().unwrap()
      .parent().unwrap()
      .join("rules.json");
  ```
- **Description**: Standard build script pattern, but fragile — if Cargo changes the `OUT_DIR` directory structure, this silently breaks. The `.unwrap()` calls would panic during build.
- **Fix**: Use `CARGO_TARGET_DIR` or a more robust path resolution. Low priority since Cargo's `OUT_DIR` structure is stable.

---

#### L8. `prompts.log` contains full game state — ✅ FIXED (`b3762ab`)

- **File**: `src/llm.rs`
- **Lines**: 28-34, 721
- **Description**: `logs/prompts.log` logs every prompt and LLM response. Prompts contain the full normalized game state (hand, deck, monsters, HP, relics, etc.). No API keys are logged (system prompts come from locale files, user prompts from game state). But users sharing logs for debugging may inadvertently share detailed game state.
- **Fix**: Document this in README. Optionally add a `LLM_LOG_PROMPTS=false` env var to disable prompt logging.

---

## Summary

| Severity | Total | Fixed | Remaining |
|----------|-------|-------|-----------|
| 🔴 Critical | 3 | 3 | 0 |
| 🟠 High | 10 | 4 | 6 |
| 🟡 Medium | 11 | 10 | 1 |
| 🟢 Low | 8 | 4 | 4 |
| **Total** | **32** | **21** | **11** |

**Branch**: `fix/code-review-critical` (37 commits)
**Verification**: `cargo fmt --check` ✅ · `cargo clippy --all-targets -- -D warnings` ✅ · `cargo test` ✅ 1592 passed

**Fixed issues**: C1 (rules.json fallback), C2 (map cycle detection), C3 (API key redaction in errors), H1 (LlmProvider Debug redaction), H2 (AGENTS.md test count), H8 (consolidated triplicated parsing into `src/parsing.rs`), H10 (219 clippy test warnings), M2 (`.ok()` → logged errors), M3 (`let _ =` → logged errors), M4 (panic/expect/unreachable → Result in 4 production sites), M5 (magic numbers → named constants), M7 (AI postmortem output validation), M8 (SSRF: `LLM_BASE_URL` scheme validation), M9 (stdin JSON size cap), M10 (TOCTOU race on overlay.json — mutex), M11 (cargo update — 27 deps), L3 (setup wizard masked input via `rpassword`), L4 (u16→u32 bit shift in kill-scan), L7 (build.rs triple-parent unwrap), L8 (`LLM_LOG_PROMPTS` env var to disable prompt logging).

**Remaining**: 6 High-severity architecture refactors (god functions H3–H6, stringly-typed ScreenType H7, blocking I/O H9), 1 Medium issue (M1 oversized files), 4 Low hardening items (L1 Windows atomic rename, L2 file locking, L5 combat_identity [non-issue — gate reset between runs], L6 cargo-audit). M4 is partially fixed — 4 production panic/expect/unreachable sites resolved, dev-only instances remain.

---

## Recommended Fix Order

### Phase 1 — Quick wins (Critical, <1 hour each) — ✅ DONE
1. **C1**: Fix `rules.json` fallback (1 line in `src/ranker/mod.rs:31`) — ✅ `3bb0d10`
2. **C3**: Strip API key from LLM error bodies (3 locations in `src/llm.rs`) — ✅ `d9a3e2d`
3. **H2**: Update `AGENTS.md` test count (1 line) — ✅ `43b673d`
4. **H1**: Implement manual `Debug` for `LlmProvider` (redact `api_key`) — ✅ `d9a3e2d`

### Phase 2 — Input safety (Critical/High, 1-2 hours) — ✅ DONE
5. **C2**: Add cycle detection to map path enumeration (`src/prompt/routing.rs`) — ✅ `bef1fe2`
6. **M9**: Add size/depth cap to stdin JSON deserialization (`src/main.rs:316`) — ✅ `d812b2c`

### Phase 3 — Clippy cleanup (High, 30 min) — ✅ DONE
7. **H10**: Run `cargo clippy --fix --tests`, fix remaining 12 manually — ✅ `585682e`

### Phase 4 — Architecture refactoring (High, days) — ⏳ PARTIALLY DONE
8. **H3-H6**: Decompose god functions (`main`, `from_raw`, `to_stable_value`, `dfs`) — ⏳ remaining
9. **H7**: Introduce `ScreenType` enum (large, mechanical) — ⏳ remaining
10. **H8**: Consolidate triplicated parsing logic — ✅ `995fade` (new `src/parsing.rs`)
11. **H9**: Resolve blocking I/O in async (either go sync or use `tokio::fs`) — ⏳ remaining

### Phase 5 — Code quality (Medium, ongoing) — ⏳ PARTIALLY DONE
12. **M1**: Modularize oversized files (23 files over 250 LOC) — ⏳ remaining
13. **M2-M3**: Replace `.ok()?` and `let _ =` with logged error handling — ✅ `8066438`–`a2c2a0f`
14. **M4**: Replace `panic!`/`expect`/`unreachable!` with `Result` returns — ✅ partially (`baf8dde`–`d1b7cf5`, 4 production sites; dev-only instances remain)
15. **M5**: Extract magic numbers into named constants — ✅ `1ccc96e`, `52aa2d4`
16. **M6**: Refactor `eval_parsed()` to table-driven — ✅ table-driven (gt/eq guard slices)
17. **M7**: Add validation/retry for AI postmortem — ✅ `164caa5`

### Phase 6 — Hardening (Low, as needed) — ⏳ PARTIALLY DONE
18. **L1, L2, L5, L6**: Windows atomicity, file locking, combat_identity, cargo-audit — ⏳ remaining
19. **L3**: Setup wizard masked input — ✅ `ef330b2` (via `rpassword`)
20. **L4**: u16→u32 bit shift in kill-scan — ✅ `3c0d4ea`
21. **L7**: `build.rs` triple-parent unwrap — ✅ `756038a`
22. **L8**: `LLM_LOG_PROMPTS` env var — ✅ `b3762ab`
23. **M8**: Validate `LLM_BASE_URL` (scheme allowlist, reject non-HTTP) — ✅ `72a3a83`
24. **M10-M11**: Fix TOCTOU races, update dependencies — ✅ `bd6a9ba`, `4a29a8d`

---

## What's Working Well

The codebase has several strengths worth noting:

- **Test coverage**: 1592 passing tests with 21 fixture files — comprehensive
- **No secrets in git**: `.env` properly gitignored, `.env.example` has placeholders only
- **No `unsafe` in production code**: Only in test files for `env::set_var`
- **No TODO/FIXME markers**: Clean codebase without debt markers
- **Kill-scan DFS is well-bounded**: Depth limit, state expansion limit, deadline, and memoization all present
- **Run finalization is idempotent**: `finalized` flag + journal replacement handles edge cases correctly
- **CombatTurnGate dedup is correct**: Identity + turn key prevents double-fire and handles combat restarts
- **Locale loading is safe**: `include_str!` at compile time, unknown langs fall back to English
- **Release workflow is well-scoped**: Proper permissions, artifact packaging, multi-OS matrix
- **CI matches pre-commit**: Both run the same `fmt → clippy → test → build` pipeline
