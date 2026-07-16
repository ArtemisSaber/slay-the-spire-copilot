# External Strategic Memory for the LLM Planner

## Implementation Design

| Field | Value |
|---|---|
| Status | Implemented behind `MEMORY_MODE`; disabled by default |
| Last updated | 2026-07-16 |
| Target | Post-`0.2.0` |
| Review scope | Data contracts, lesson generation, lifecycle, retrieval, safety, testing, and rollout |
| Primary owner | Slay the Spire Copilot maintainers |
| Related documents | [`ranker-architecture.md`](ranker-architecture.md), [`ranker-rules.md`](ranker-rules.md), [`auto-play-design.md`](auto-play-design.md) |

## 1. Summary

This module gives the planner durable experience without training the model,
fine-tuning it, or running a reinforcement-learning loop. It records factual
decisions from completed legitimate runs, asks the existing run-end LLM call
for at most one uncertain strategic hypothesis, retrieves that hypothesis in a
later comparable run, and evaluates it using subsequent run outcomes.

The central design constraint is that there is no golden action dataset. Slay
the Spire is an oracle for what happened, not for what would have happened
after an action that was never selected. The implementation therefore divides
responsibility as follows:

- Deterministic code records facts, enforces eligibility, selects bounded
  evidence, validates references, retrieves memories, and updates lifecycle
  counters.
- The LLM interprets a multi-decision trajectory and proposes a strategic
  hypothesis such as target priority, defense versus offense, setup,
  sequencing, or resource timing.
- Later legitimate runs test whether the hypothesis is useful at the run level.
- Two consecutive comparable degraded runs retire the hypothesis and trigger
  one regeneration attempt that includes the rejected lesson and failed runs.

The model and ranker weights never change. Learning is an external,
append-only knowledge lifecycle.

### 1.1 Decision table

| Topic | Decision |
|---|---|
| Learning mechanism | Retrieval-augmented external memory |
| Source data | Completed legitimate runs with acknowledged semantic actions |
| Stored truth | Observed state, action, and outcomes only |
| LLM output | At most one uncertain strategic hypothesis per eligible run |
| Lesson evaluation | Only later comparable runs in which the lesson was reported used |
| Comparison cohort | Same character, exact Ascension, objective, and compatibility identity |
| Failure threshold | Two consecutive degraded qualifying runs |
| Failed lesson | Retained as retired provenance; never retrieved |
| Regeneration | Rejected lesson plus its benchmark and two failed trials are shown to the LLM |
| Runtime experiments | At most one lesson can influence a run |
| Ranker | Unchanged; remains a general deterministic prior |
| Additional calls | No combat-time call and no second postmortem call |
| Storage | Local append-only JSONL plus rebuildable snapshots |
| Debug Ascension `-15` | Full flow may run, but no cases or lessons enter knowledge |

## 2. Problem and motivation

The planner otherwise starts every run without durable memory. A human-readable
postmortem can explain a loss, but that explanation is not available during a
later similar situation.

A useful memory system must avoid two symmetric errors:

1. Treating a defeat as proof that the final card was the cause.
2. Treating a victory as proof that every selected action was correct.

Many meaningful mistakes span multiple decisions. Examples include attacking
the wrong enemy for several turns, taking too much chip damage before a setup
engine comes online, spending a potion too early, or failing to preserve enough
HP to bootstrap a sequence. A schema limited to “selected card plus immediate
outcome” cannot express those hypotheses.

The module therefore stores immutable action-level cases as evidence but makes
the learned unit a broader strategic policy. Its value is judged by later run
progression, not by a fabricated one-action counterfactual label.

## 3. Goals and non-goals

### 3.1 Goals

- Learn immediately from completed legitimate runs without epochs.
- Preserve facts separately from model interpretation.
- Generate a broad, actionable, explicitly uncertain strategic hypothesis.
- Allow evidence across multiple decisions rather than forcing a terminal
  action explanation.
- Store the origin character, exact Ascension, objective, final floor, victory,
  and compatibility identity locally from trusted telemetry.
- Test a lesson only when it was actually exposed and reported used.
- Isolate experiments so one run is attributed to at most one strategic lesson.
- Retire a lesson after two consecutive comparable degraded runs.
- Regenerate with the failed lesson and evidence in the prompt.
- Keep retrieval deterministic, local, bounded, and inspectable.
- Preserve current action validation, kill-scan, and ranker authority.
- Provide status and completed-run review through the terminal control center.

### 3.2 Non-goals

- Reinforcement learning, policy gradients, self-play, or optimizer epochs.
- LLM fine-tuning or a learned value network.
- Automatically modifying `rules.json`, ranker weights, or ranker predicates.
- Proving that an unselected action would have won.
- Requiring Undo the Spire, save-state branching, or a callable mod API.
- Embeddings, a vector database, or a remote memory service in this version.
- Mixing different Ascension levels in one lesson experiment.
- Letting memory invent or authorize executable actions.
- Claiming convergence or global optimality.

## 4. Correctness invariants

These invariants are release-blocking.

1. Factual cases are never rewritten by LLM output.
2. Counterfactuals remain hypotheses and cannot be stored as observed facts.
3. Ascension `-15`, synthetic, stdin-test, incomplete, and detected
   restore/undo runs are knowledge-ineligible.
4. Current state and server-resolved `available_actions` remain the only
   execution authority; prompt-scoped refs cannot invent an action.
5. Retrieval cannot override kill-scan, ranker avoidance, or command validation.
6. Same-seed factual cases are not retrieved.
7. A strategic lesson is evaluated only if its ID appears in validated
   `memory_ids_used` on at least one committed case from the run.
8. If zero or multiple strategic lessons are attributed, no lifecycle result
   is assigned to either lesson.
9. A qualifying run must match character, exact Ascension, objective, and
   compatibility identity.
10. At most one strategic lesson is returned in a decision prompt, and the
    first used lesson is locked for the rest of the run.
11. Retired or contested lessons are never retrieved.
12. A malformed critic response can still yield a human fallback report but
    cannot mutate knowledge.
13. Append-only cases and lesson events are the source of truth; snapshots are
    disposable derived artifacts.
14. Learning adds no combat-time LLM request.

## 5. Architecture

### 5.1 Data flow

```text
LEGITIMATE RUN

NormalizedState + current candidates + ranker output
                        |
                        v
                  planner decision
                        |
                        v
         semantic action + observed outcomes
                        |
                        v
              terminal eligibility gate
                        |
                        v
               append factual cases
                        |
                        v
      existing postmortem request + critic V3
                        |
                        v
       append at most one strategic lesson

LATER COMPARABLE RUN

public situation + exact Ascension + different seed
                        |
                        v
             deterministic retrieval
                        |
                        v
       at most one experimental lesson in prompt
                        |
                        v
      validated model self-report: memory_ids_used
                        |
                        v
          completed-run progression comparison
                        |
          +-------------+-------------+
          |                           |
   retained/reset              second degraded run
                                      |
                                      v
                         retire and regenerate once
```

### 5.2 Source layout

```text
src/learning/
├── action.rs                    # stable cross-run semantic actions
├── bundle.rs                    # verified repository bundle
├── case.rs                      # immutable factual decision cases
├── cli.rs                       # status, verify, inspect, and maintenance
├── compatibility.rs             # automatic runtime compatibility identity
├── config.rs                    # off/collect/shadow/on and hard bounds
├── context.rs                   # bounded planner experience context
├── descriptor.rs               # public-information situation descriptor
├── eligibility.rs              # fail-closed run classification
├── lesson.rs                    # shared lesson schema and facade
├── lesson/
│   ├── event.rs                 # append-only lesson state events
│   ├── identity.rs              # stable lesson/family identity
│   ├── legacy.rs                # schema-v1 compatibility and support
│   ├── matching.rs              # applicability and legacy association logic
│   ├── strategy.rs              # schema-v2 hypothesis and trial lifecycle
│   └── validation.rs            # legacy coherence checks
├── retrieval.rs                # filters, ranking, and diversity
├── retrieval/similarity.rs     # deterministic integer similarity
├── session.rs                  # run-scoped facade and attribution lock
├── session/
│   ├── bootstrap.rs            # store and snapshot initialization
│   ├── capture.rs              # pending cases and trusted run benchmark
│   ├── capture/observation.rs  # terminal facts and encounter detection
│   ├── capture/retention.rs    # bounded case retention
│   ├── critic.rs               # critic V3 ingest
│   ├── critic/contract.rs      # strict response contract
│   ├── critic/evidence.rs      # bounded multi-decision evidence selection
│   ├── critic/mode.rs          # initial/report-only/regenerate state
│   ├── critic/prompt.rs        # critic V3 prompt appendix
│   ├── finalize.rs             # case commit and lifecycle evaluation
│   ├── lifecycle.rs            # trial events, retirement, regeneration result
│   ├── prepare.rs              # retrieval and context preparation
│   └── resume.rs               # pinned snapshot restoration
├── snapshot.rs                 # immutable verified retrieval view
├── store.rs                    # append-only logs and atomic snapshots
└── telemetry.rs                # auditable planner decision records
```

The prior deterministic terminal-action audit is intentionally removed. The
critic receives a bounded trajectory and may reason about earlier decisions.

## 6. Factual cases

A `DecisionCase` is immutable evidence for one acknowledged combat decision.
It includes:

- run and stable decision IDs;
- hashed seed scoped to compatibility identity;
- public situation descriptor;
- exact Ascension for lifecycle reconstruction, validated against the
  descriptor's Ascension band;
- selected semantic action and bounded available actions;
- bounded ranker suggestions and tags;
- memory IDs exposed to the planner and reported used (the journal separately
  retains shadow-retrieved IDs);
- action, turn, combat, and terminal run outcomes;
- app, prompt, ranker, model, compatibility, and snapshot provenance.

It excludes card UUIDs, hand indices, raw localized descriptions, API secrets,
ordered draw piles, and future RNG state. An observed outcome means only “this
happened after the selected action.”

Cases are kept in memory until terminal eligibility succeeds. A normal
completed victory or defeat can be eligible. Debug, synthetic, incomplete, or
inconsistent runs are discarded from knowledge.

## 7. Strategic lesson schema

### 7.1 Hypothesis

Schema-v2 lessons add a `StrategicHypothesis`:

```json
{
  "text": "Prioritize the enemy whose continued presence creates the most near-term pressure.",
  "applies_when": "Multiple targets are alive and target choice changes future pressure.",
  "expected_effect": "This may preserve HP and setup time in comparable fights.",
  "evidence": [
    {
      "run_id": "run-2026-07-16",
      "decision_ids": ["run-2026-07-16:12:2:14", "run-2026-07-16:12:3:18"],
      "observed_chain": "Damage remained split while both enemies continued contributing pressure."
    }
  ],
  "uncertainty": "The alternative target sequence was not played, so its result is unknown."
}
```

The fields are deliberately strategic rather than a card/outcome tuple. The
model may describe target priority, defense, setup, sequencing, resource use,
deck construction, or pathing when the supplied evidence supports discussion.

### 7.2 Trusted benchmark

The model does not supply the benchmark. The runtime attaches it from the
completed eligible run:

```json
{
  "origin_run_id": "run-2026-07-16",
  "character": "IRONCLAD",
  "ascension_level": 20,
  "objective": "act3_victory",
  "final_floor": 37,
  "victory": false,
  "compatibility_sha256": "sha256:..."
}
```

Exact Ascension is retained even though situation retrieval also uses an
Ascension band. Lifecycle comparison always uses the exact value.

### 7.3 Lifecycle state

```json
{
  "benchmark": { "...": "trusted fields above" },
  "parent_lesson_id": null,
  "generation": 1,
  "qualifying_runs": 0,
  "consecutive_degraded_runs": 0,
  "recent_trials": [],
  "regeneration_resolved": false
}
```

Only the latest two trial summaries are retained in the lesson state. Full
factual cases remain in append-only case storage. Lifecycle counters are not
part of lesson identity, so an evaluated event updates the same lesson ID.

### 7.4 Compatibility carrier fields

The shared `Lesson` record still contains the schema-v1 action-association
fields so existing snapshots deserialize without a destructive migration.
Schema-v2 lessons use explicit neutral values:

- `action_pattern.kind = strategic_policy`
- `outcome_code = run_progression`
- `guidance.kind = experimental`

Runtime context for a schema-v2 lesson reads `strategy` and `lifecycle`, not the
legacy carrier fields. This avoids fabricating an `end_turn` or
`combat_death` association.

### 7.5 Identity and lineage

The strategic lesson ID includes immutable content: scope, hypothesis,
evidence case IDs, critic model profile, confidence, benchmark, parent ID, and
generation. Mutable trial counters are excluded. A replacement is a new lesson
whose `parent_lesson_id` identifies the retired lesson and whose generation is
one greater.

## 8. Critic V3 generation

### 8.1 Why the LLM is necessary

Deterministic code can identify observed damage, progression, ranker
disagreement, and the final outcome. It cannot determine from one trajectory
whether the better strategy was to focus another monster, defend earlier,
delay an attack, save a potion, or change setup sequencing. Encoding those
answers deterministically would simply move an unverified policy into code.

The LLM is used only for the uncertain interpretation step. Deterministic code
continues to own evidence integrity and lifecycle decisions.

### 8.2 Modes

| Mode | Condition | Allowed lesson result |
|---|---|---|
| `initial` | Eligible run has no lesson or trial already associated with it | One new hypothesis or `no_lesson` |
| `report_only` | Run already originated or evaluated a lesson | `no_lesson` only |
| `regenerate` | This run is the second consecutive degraded trial for a retired lesson and no child/resolution exists | Materially different child hypothesis or `no_lesson` |

This prevents one completed run from generating an unlimited sequence of
lessons. A valid `no_lesson` regeneration marks the parent resolved so repeated
reviews do not continually ask for a replacement.

### 8.3 Evidence selection

The prompt contains up to 14 cases from the current run and up to six cases
from each related origin/trial run. Selection is deterministic:

1. Preserve the first and last decisions.
2. Preserve a terminal window of the last four decisions.
3. Rank high observed-damage and ranker-regret signals.
4. Include neighboring decisions around high-signal points.
5. Fill remaining capacity with evenly spaced trajectory coverage.

The selection does not label any case as the cause. Each case includes current
public situation, selected and available actions, ranker evaluation, and
observed action/turn/combat/run outcomes. The LLM cites stable decision IDs,
not internal SHA IDs that a user must manually enter.

The deterministic report prefix is capped at 8,000 bytes and the complete
critic prompt at 40,000 bytes. If the appendix cannot fit, normal postmortem
behavior remains available and no lesson is created.

### 8.4 Prompt contract

The system prompt explicitly states:

- analyze a multi-decision trajectory;
- do not assume the final action is the strategic root cause;
- distinguish observed facts from untested alternatives;
- never claim an unplayed action would certainly have won;
- produce at most one lesson;
- in regeneration, do not paraphrase or merely negate the failed lesson.

The response is exactly one JSON object:

```json
{
  "schema_version": 3,
  "report_markdown": "# Human-readable postmortem\n\n...",
  "result": "lesson",
  "lesson": {
    "text": "Preserve enough HP to establish setup before committing to a long damage sequence.",
    "applies_when": "The deck needs setup turns and current HP cannot absorb an extended race.",
    "expected_effect": "This may improve the chance of reaching the stable sequence.",
    "evidence": [
      {
        "run_id": "trial-two",
        "decision_ids": ["trial-two:39:2:7"],
        "observed_chain": "The prior lesson was used and the run ended below its benchmark."
      }
    ],
    "uncertainty": "The defensive sequence was not played.",
    "confidence_millis": 720
  },
  "rejected_lesson_analysis": "Required only in regenerate mode; explain why the prior lesson failed."
}
```

For `no_lesson`, `result` is `no_lesson` and `lesson` is `null`.

### 8.5 Validation

The ingest path rejects knowledge mutation unless all applicable checks pass:

- top-level and nested objects contain only declared fields;
- schema version is exactly 3;
- result and nullable lesson shape agree;
- the human report is non-empty, bounded, and free of disallowed controls;
- hypothesis text fields are non-empty, bounded, and reject URLs, code fences,
  and script tags;
- evidence contains one to three entries and no more than 12 decision IDs per
  entry;
- every cited run and decision was supplied in the prompt;
- at least one cited decision belongs to the current run;
- model confidence is an integer in `0..=1000`;
- `report_only` cannot create a lesson;
- regeneration includes a safe rejected-lesson analysis;
- a replacement's normalized text is not identical to the rejected lesson.

Text-difference validation is a guard against exact restatement, not a proof of
semantic novelty. The prompt, lineage, later trials, and optional human review
provide the remaining defense.

The runtime assigns language, scope, source case IDs, benchmark, parent,
generation, compatibility, and critic profile. The model cannot override them.

## 9. Lesson experiment lifecycle

### 9.1 Attribution

The planner may return `memory_ids_used`. The runtime intersects those IDs with
the IDs actually serialized into `experience_context`, sorts and deduplicates
them, and discards unknown IDs.

When the first strategic lesson is reported used, `RunCapture` locks that
lesson ID. Later decision preparation filters out any other strategic lesson,
and later memory-use reports are reduced to the locked lesson. This implements
one experimental lesson per run.

Self-reported use is imperfect, but it is stricter than treating mere retrieval
as exposure. It is diagnostic attribution, not causal proof.

### 9.2 Comparable runs

A later run qualifies only when all are equal to the origin benchmark:

- character;
- exact Ascension level;
- objective;
- compatibility identity.

The run must independently pass normal knowledge eligibility and contain
committed cases using exactly one active strategic lesson. Origin and duplicate
run IDs are ignored.

Different Ascension levels are not ordered as better or worse. Mixing them
would confound strategy quality with difficulty, so they form separate cohorts.

### 9.3 Degradation rule

The implemented comparator is intentionally small and auditable:

```text
if origin was a victory:
    degraded = later run is a defeat
else:
    degraded = later run is a defeat and final_floor < origin final_floor
```

A non-degraded qualifying run resets `consecutive_degraded_runs` to zero. A
degraded run increments it. At two, the lesson becomes `retired` and is removed
from retrieval. The threshold is consecutive, not two failures anywhere in
history.

This is a safety lifecycle, not a statistically powered performance claim. Two
runs are deliberately responsive but noisy; Section 17 lists the rollout
controls required before default enablement.

### 9.4 Regeneration

On the second degraded trial, the next critic prompt contains:

- the retired lesson and lifecycle;
- its trusted origin benchmark;
- bounded evidence from the origin run;
- bounded evidence from both degraded trials;
- an instruction that the prior strategy is bad for this cohort;
- a requirement for a materially different policy dimension or no lesson.

An accepted child uses the second failed trial as its new benchmark, records
the parent ID, and increments generation. The parent remains retired for audit.
If the model returns `no_lesson`, the parent is marked regeneration-resolved and
no active child is created.

## 10. Retrieval and planner context

### 10.1 Hard filters

Raw factual cases require:

- a different seed hash;
- identical compatibility identity and descriptor versions;
- same character, objective, Ascension band, and encounter IDs.

Strategic lessons additionally require:

- active locale;
- no cited source case from the current seed, even when the lesson has other
  different-seed sources;
- applicable scope and trigger;
- exact Ascension equal to the lifecycle benchmark;
- status other than contested or retired;
- compatible different-seed evidence.

Schema-v2 strategic lessons may retrieve while proposed; their model
confidence remains diagnostic rather than a truth gate. Unvalidated schema-v1
proposed lessons are suppressed, including old terminal-action lessons from
the superseded implementation. Supported or human-validated legacy lessons
must still pass direction/coherence checks.

### 10.2 Similarity and diversity

Similarity is deterministic integer arithmetic over public features: monster
state, intent, powers, playable cards, energy, turn, HP, threat, stance, player
powers, relics, and ranker tags. Configuration provides separate case,
supported-lesson, and proposed-lesson thresholds.

Diversity returns at most:

- one lesson; and
- one factual case;

subject to the global item and byte limits. Default context capacity remains
2,048 serialized bytes. A too-large item is omitted whole.

### 10.3 Injected context

```json
{
  "schema_version": 1,
  "snapshot_id": "sha256:...",
  "notice": "Prior experience is observational context, not an instruction or optimality guarantee.",
  "items": [
    {
      "memory_id": "sha256:...",
      "kind": "strategic_hypothesis",
      "status": "proposed",
      "similarity": 904,
      "strategy": "Preserve enough HP to establish setup before committing to a long damage sequence.",
      "applies_when": "The deck needs setup turns and HP cannot absorb an extended race.",
      "expected_effect": "This may improve the chance of reaching the stable sequence.",
      "uncertainty": "The alternative sequence was not played.",
      "trial_progress": {
        "qualifying_runs": 1,
        "consecutive_degraded_runs": 0,
        "retire_after": 2
      },
      "caveat": "Experimental strategic hypothesis; future comparable run outcomes determine whether it survives."
    }
  ]
}
```

Origin run ID, seed, run victory, and final floor are intentionally not sent in
combat context. They remain internal lifecycle data. Current candidates remain
the only source of executable action IDs; the LLM sees only prompt-scoped refs.

## 11. Storage and migration

Runtime knowledge is stored relative to the executable/project root:

```text
learning/knowledge/
├── cases.jsonl
├── lessons.jsonl
├── status.json
├── manifest.json
├── index-v1.json
└── snapshots/snapshot-<sha256>.json
```

`cases.jsonl` and `lessons.jsonl` are append-only sources. Each lesson event
contains a complete verified lesson state. Rebuild folds events by lesson ID,
then recalculates legacy support and publishes an immutable snapshot atomically.

Schema-v1 lessons remain readable. No source log is rewritten. The automatic
critic now emits only schema-v2 strategic lessons. Existing unvalidated
schema-v1 proposed lessons no longer enter retrieval, which safely neutralizes
the superseded terminal-action policy without deleting user data.

New factual cases carry an optional exact-Ascension field whose absence is
backward compatible with existing case identities. This lets an interactive
completed-run review reconstruct a trusted benchmark after process restart,
including Ascensions inside the broad `a1_9`, `a10_16`, and `a17_19` bands.
Older A0/A20 cases can be reconstructed from their single-value bands; older
middle-band cases without the exact field fail closed for new lesson creation.

Compatibility identity is generated automatically from app major/minor,
case/descriptor/ranker/prompt schemas, and active ranker rules. Users never
enter or expose a SHA profile manually.

## 12. User experience and modes

The setup wizard controls learning without requiring manual environment-file
editing. The terminal control center provides human-readable status and a
completed-run review flow; CommunicationMod remains standard input/output game
protocol only.

| Mode | Capture | Retrieve | Inject into planner |
|---|---:|---:|---:|
| `off` | No | No | No |
| `collect` | Yes | No | No |
| `shadow` | Yes | Yes | No |
| `on` | Yes | Yes | Yes |

Default is `off`. The terminal reports saved case/lesson counts and whether the
last run was learned in user-facing language. Internal IDs remain available to
maintainer inspect/verify tooling but are not required for normal operation.

## 13. Failure handling

| Failure | Behavior |
|---|---|
| Missing or corrupt local knowledge | Rebuild or fall back to the verified embedded bundle |
| Invalid case or event identity | Reject the source; do not partially publish it |
| Descriptor cannot be built | Omit capture/retrieval for that decision |
| Unknown or ambiguous stable action/target | Do not create a cross-run case for it |
| Same-seed evidence | Exclude before similarity scoring |
| Context item exceeds byte limit | Omit the whole item |
| Critic prompt exceeds 40,000 bytes | Use ordinary report behavior; create no lesson |
| Invalid critic JSON or references | Preserve report fallback and factual cases; create no lesson |
| Store append fails | Preserve the run journal; do not publish a new snapshot |
| Continued-run snapshot is missing | Disable memory for that run rather than substitute a newer snapshot |
| Unknown `memory_ids_used` | Discard it and validate the selected action normally |
| Zero or multiple used strategic lessons | Commit factual cases but evaluate no lesson |
| Regeneration finds no defensible child | Resolve parent with no replacement |

## 14. Security and privacy

- Model text is untrusted and length-bounded.
- URLs, code fences, script tags, and disallowed controls are rejected from
  persisted hypotheses.
- Memory is serialized as data beside current actions, not concatenated as an
  executable command.
- Planner output is still parsed against current candidate IDs.
- API keys are absent from cases, lessons, compatibility IDs, and prompts.
- Seed values are hashed with compatibility identity and are not shown to the
  planner.
- Raw localized card/relic descriptions and ordered piles are not persisted in
  retrieval descriptors.
- Repository publication of local knowledge remains an explicit maintainer
  operation.

Persistent prompt injection remains possible in principle through seemingly
benign lesson prose. Structured fields, sanitization, a clear untrusted notice,
one-lesson limits, action validation, retirement, and review reduce this risk;
they do not make arbitrary model text authoritative.

## 15. Observability

The implementation exposes or records:

- memory mode, case count, lesson count, and last-run eligibility;
- retrieved, exposed, and model-reported-used memory IDs;
- snapshot ID pinned to a run;
- appended/skipped case counts;
- evaluated and retired lesson counts at finalization;
- lesson benchmark, generation, parent, trial count, and degraded streak;
- complete critic prompts/responses when prompt logging is enabled;
- append-only proposed, evaluated, retired, and human status events.

Recommended production metrics include retrieval hit/relevance rate, context
bytes/tokens, invalid action/retry rate, lesson-use rate, retirement rate,
regeneration acceptance rate, and final-floor/victory distributions by fixed
cohort.

## 16. Test strategy and evidence

### 16.1 Unit coverage

Tests cover:

- eligibility, including Ascension `-15` exclusion;
- semantic action identity and outcome capture;
- strict critic V3 parsing and unknown-decision rejection;
- multi-decision strategic lessons that do not encode terminal action blame;
- trusted exact-Ascension/floor benchmark attachment;
- exact comparable-run filters;
- duplicate and incompatible trial rejection;
- degradation streak increment and reset;
- two-run retirement with stable lesson identity;
- strict-lower-floor behavior for defeat benchmarks;
- exact-Ascension strategic retrieval;
- suppression of high-confidence unvalidated legacy lessons;
- context privacy and byte bounds;
- V3 system prompt and mock-provider envelope.

### 16.2 End-to-end lifecycle coverage

The lifecycle E2E tests cross the actual store, snapshot, retrieval, context,
planner attribution, case capture, finalization, lesson events, critic prompt,
and critic ingest boundaries. They verify:

1. The lesson is injected and reported used.
2. The first degraded run evaluates but retains it.
3. The second degraded run retires it.
4. The regeneration prompt contains the rejected parent and failed evidence.
5. A different strategy becomes a generation-2 child with the second run's
   trusted benchmark.
6. `no_lesson` resolves regeneration and prevents repeated regenerate mode.

### 16.3 Repository gates

Every change must pass:

```text
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

The repository build also enforces a 250-logical-LOC ceiling per Rust file.

## 17. Rollout and evaluation

### 17.1 Rollout stages

1. `off`: establish baseline behavior and verify no side effects.
2. `collect`: gather legitimate factual cases and inspect critic quality.
3. `shadow`: measure relevance without changing planner prompts.
4. `on` canary: enable for a small fixed cohort with immediate `off` rollback.
5. Fixed-snapshot comparison: compare complete runs with memory off/on using a
   pinned model, prompt, compatibility identity, character, Ascension, and
   objective.

### 17.2 Launch gates

Before default enablement:

- review at least 100 shadow retrievals with at least 90% situation relevance;
- inspect a real-provider sample of initial and regeneration prompts;
- complete at least 10 memory-on canary runs with no safety invariant failure;
- observe no material command-rejection or retry regression;
- report lesson use, retirement, and regeneration rates;
- run a predeclared larger fixed-snapshot comparison, or label outcome evidence
  inconclusive.

Two-run retirement is a guardrail, not evidence of convergence. A performance
claim needs substantially more runs and confidence intervals.

## 18. Alternatives considered

### Reinforcement learning

Rejected for this phase because it requires a reward definition, many episodes,
stable environment control, and substantially greater compute and operational
complexity.

### Fine-tuning

Rejected because there is no golden label set, iteration is slow, and a
fine-tune would not provide the transparent per-lesson lifecycle required here.

### Deterministic lesson generation

Rejected as the sole mechanism. Code can find facts and correlations but would
need hand-authored assumptions to choose among target priority, defense, setup,
and sequencing explanations. That recreates the missing oracle inside the
validator.

### Immediate-action death lessons

Removed. They overfit the last observed action and cannot represent strategic
causes distributed across a trajectory.

### Undo-based counterfactuals

Deferred. Undo the Spire currently offers no reliable callable interface for
this application, and manual branching would contaminate unattended run data.

### Embeddings or a vector database

Deferred until deterministic retrieval fails measured relevance or latency
requirements. The current corpus and descriptor are small enough for local
integer scoring.

### Direct ranker updates

Rejected. A poor lesson must be removable without changing general ranker
behavior, and model prose should never become an unreviewed numeric rule.

## 19. Risks and mitigations

| Risk | Mitigation and remaining limitation |
|---|---|
| LLM proposes a plausible but wrong strategy | Experimental status, one-lesson isolation, two-run retirement, regeneration, and optional human review; two runs are still noisy |
| Model self-report omits or falsely claims use | IDs are limited to exposed memory; attribution remains diagnostic rather than proof |
| Final floor is a noisy metric | Exact cohort matching, victory precedence, consecutive failures, and fixed-snapshot evaluation; no claim of causality |
| Exact Ascension creates sparse data | Prevents difficulty confounding but slows evaluation; widening requires a future schema/experiment |
| Replacement merely rephrases the parent | Exact normalized restatement is rejected and prompt requires a new policy dimension; semantic equivalence can still escape detection |
| One lesson masks interaction effects | Intentional isolation improves attribution; multi-lesson experiments require a separately designed factorial policy |
| Old bad lessons remain on disk | Unvalidated schema-v1 proposed lessons are no longer retrieved; provenance is preserved for audit |
| Same-seed future leakage | Hard different-seed filter and hidden-data-free descriptor |
| Persistent prompt injection | Structured safe text, untrusted context, hard action validation, and retirement |
| Debug or restored data contaminates learning | Fail-closed eligibility where signals exist; upstream rollback observability remains a limitation |
| JSONL grows indefinitely | Compact schemas and snapshots now; add compaction only after measured need |

## 20. Acceptance criteria

The implementation is code-complete when:

- no RL trainer, optimization epoch, fine-tune, or Undo dependency exists;
- legitimate completed runs persist factual cases while Ascension `-15` and
  synthetic runs persist none;
- the critic emits at most one schema-v2 strategic hypothesis through V3;
- the human-readable report is a JSON field and is extracted normally;
- the prompt permits multi-decision causes and rejects terminal-action
  certainty;
- trusted benchmark and lineage fields cannot be supplied by the model;
- only an actually used, single strategic lesson is evaluated;
- exact character/Ascension/objective/compatibility matching is enforced;
- two consecutive degraded trials retire the lesson;
- retirement removes the lesson from retrieval without deleting provenance;
- regeneration includes the rejected lesson and failed trials;
- a child is materially different at least at normalized text level, or no
  lesson is stored and regeneration is resolved;
- old unvalidated tactical proposed lessons cannot influence play;
- retrieval remains bounded to one lesson plus one factual case and 2,048
  bytes by default;
- malformed knowledge, prompts, or responses fail safely;
- setup and terminal control remain user-facing and require no manual hash;
- all repository quality gates pass.

Production default-on remains a separate decision gated by Section 17.
