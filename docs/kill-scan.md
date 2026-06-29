# Kill Scan — Deterministic Kill Sequence Specification

Architecture: [`kill-scan-architecture.md`](kill-scan-architecture.md).

Deterministic visible-hand kill scanner. Given the current hand, energy, player
stance, and monster state, answer:

> can the visible hand guarantee ending combat this turn?

This is a no-brain certifier, not a planner. It does not reason through draw,
generated cards, odds, potions, relic side damage, poison, or future turns. If a
line depends on an unsupported mechanic, return `false`.

```
find_kill_sequence(ctx: CombatScanContext) -> Option<Vec<KillPlay>>
can_end_fight(ctx) = find_kill_sequence(ctx).is_some()

KillPlay {
    card: String,              // CardInfo.uuid
    target: Option<usize>,     // command target index, null for non-targeted cards
}
```

`Some(sequence)` means the modeled visible-hand sequence is guaranteed to end
combat. The sequence is ordered and should be playable as-is. `None` can mean
either no kill exists or the kill depends on unsupported mechanics.

Serialized/command-facing shape:

```
{ "card": "<uuid>", "target": 0 }
{ "card": "<uuid>", "target": null }
```

For callers that only need card order, derive `Vec<CardUUID>` from
`sequence.iter().map(|play| play.card)`. Targeted cards still need `target` to
be executable. `target` is `MonsterInfo.index`, the value sent to the
CommunicationMod `play` command; it is not the scanner's internal living-vector
position.

---

## 1. Inputs

```
CombatScanContext {
    cards: Vec<CardInfo>,
    energy: i16,
    initial_stance: Stance,     // stance when CommunicationMod rendered hand text
    current_stance: Stance,     // mutable during DFS
    strength_delta: i16,        // Strength gained/lost after render, usually 0
    x_cost_bonus: i16,          // Chemical X = 2, else 0
    monsters: Vec<MonsterSnapshot>,
    remaining_card_plays: usize,
}

enum Stance {
    Neutral,
    Calm,
    Wrath,
    Divinity,
}

MonsterSnapshot {
    command_index: usize,       // MonsterInfo.index; value sent to `play`
    hp: i16,
    block: i16,
    powers: Vec<PowerState>,
    is_minion: bool,
}

PowerState {
    id: String,
    amount: i16,
    triggered: bool,            // only for one-shot powers such as Curl Up
}
```

It is fair for supported/base-game scanner states to assume hp, block, energy,
Strength, and modeled power amounts fit in `i16`. Convert from `NormalizedState`
at the scanner boundary; out-of-range values are unsupported and should fail
closed.

Use `remaining_card_plays = min(hand_size, active_limit)`.

| Limit | Remaining plays |
|-------|-----------------|
| Normal | `cards.len()` |
| Velvet Choker | `6 - cards_played_this_turn` |
| Time Eater | `12 - Time_Warp.amount` |
| Normality | `3 - cards_played_this_turn` |

Clamp the result at `0`. The 12th card against Time Eater is allowed to resolve,
so check for combat end after that card.

If the state does not expose enough information to compute an active limiter
exactly, fail closed. In particular, Velvet Choker, Time Eater, and Normality
need the number of cards already played this turn or an equivalent normalized
remaining-play count. Do not guess.

---

## 2. Scan-Relevant Cards

Parse cards once, then discard cards that cannot help prove a kill.

Keep only cards with at least one modeled kill-enabling effect:

- direct attack damage: targeted, AoE, or simple random-target
- execute effects such as Judgment
- Vulnerable application
- Strength gain
- energy gain
- stance change, Calm exit, Divinity entry, or Mantra
- X-cost attack damage
- deterministic exhaust/consume that changes known remaining hand state

Discard cards with only block, Dexterity, Weak, Frail, draw, discard, retain,
orb setup, poison, defensive powers, or other utility effects. This is safe for
the certifier: it can create false negatives, not false positives.

Do not discard a damage card just because it also exhausts/consumes cards. If
the exhaust effect is deterministic, update `remaining_cards` and continue. If
the hand mutation is random, choice-based, or otherwise unsupported, the card can
still be used as a final play: resolve its modeled damage, and if combat has not
ended immediately afterward, stop that branch instead of assuming later cards
remain valid.

Use this filtered list for DFS depth:

```
D = min(scan_relevant_cards.len(), remaining_card_plays)
```

---

## 3. Playability

CommunicationMod's `CardInfo.playable` reflects the current state. Include:

- cards with `playable == true`
- fixed-cost cards that are currently unplayable only because `cost > energy`
  and no known hard blocker is present

Exclude hard-blocked cards, such as attacks while Entangled, Clash with
non-attacks in hand, STATUS, CURSE, or any card whose unplayability cannot be
safely explained by energy alone.

For X-cost cards, `playable == false` is hard-blocked unless a known rule says
the card can be played at zero energy.

---

## 4. Modeled Effects

Keep the parser small and explicit.

```
CardEffect {
    damage: Option<DamageEffect>,
    energy_gain: i16,
    strength_gain: i16,
    vulnerable: Option<i16>,
    stance: StanceEffect,
    mantra_gain: i16,
    execute: Option<i16>,       // hp threshold, e.g. Judgment 30
    exhaust: ExhaustEffect,
}
```

Useful description patterns:

| Effect | zh | en |
|--------|----|----|
| AoE | `所有` | `all enemies` / `ALL enemies` |
| Vulnerable | `易伤 N` | `Vulnerable N` |
| Energy | `[E]` | `[E]` |
| Strength gain | `获得 N 点 力量` | `Gain N Strength` |
| Enter Wrath | `进入.*愤怒` | `Enter.*Wrath` |
| Enter Calm | `进入.*宁静` | `Enter.*Calm` |
| Exit stance | `退出.*姿态` / `结束.*姿态` | `Exit.*stance` / `End.*stance` |
| Mantra | `真言 N` | `Mantra N` |
| Execute | `生命值.*小于等于 N` / `斩杀` | `HP is N or less` / `set its HP to 0` |
| Exhaust/consume | `消耗` | `Exhaust` |

Conditional cards use current visible state. For example, Fear No Evil enters
Calm only when the chosen target intends to attack.

### Exhaust / Consume

Draw is out of scope because it introduces unknown future cards. Exhaust/consume
can be in scope when it removes known visible cards deterministically.

| Exhaust effect | Scan behavior |
|----------------|---------------|
| Exhaust self | No extra handling; played cards already leave `remaining_cards`. |
| Exhaust all cards in hand | Deterministic: clear `remaining_cards` after the card resolves. |
| Exhaust all non-Attack cards | Deterministic: remove matching visible cards after the card resolves. |
| Exhaust all Attack cards | Deterministic: remove matching visible cards after the card resolves. |
| Exhaust random card(s) | Unsupported unless combat ends immediately after this card. |
| Exhaust chosen card(s) | Deterministic only when exactly one legal choice exists; otherwise unsupported unless combat ends immediately after this card. |

Fiend Fire-style damage that scales with exhausted visible cards may be modeled
deterministically: compute the hit count from the known cards that will be
exhausted, resolve the damage, then remove those cards.

---

## 5. Modeled Monster Powers

Only model powers needed for common no-false-positive damage checks.

| Power | Effect |
|-------|--------|
| `Artifact` | Blocks Vulnerable application; decrement amount. |
| `Curl Up` | Before first attack hit on that monster, add `amount` block once. |
| `Flight` | While active, attack damage is halved, rounded down; decrement on attack damage. |
| `Intangible` | Cap positive attack damage to `1` per hit. |
| `Invincible` | Cap total hp loss this turn to remaining `amount`; decrement by hp loss. |
| `Malleable` | After unblocked attack damage, add current `amount` block, then increment amount by 1. |
| `Slow` | Damage taken is increased by `10% * amount`; increment after each card resolves. |
| `Vulnerable` | Attack damage is multiplied by 1.5, rounded down. |
| `Minion` | Used only by combat-end check. |
| `Time Warp` | Used only by play-limit construction. |

If a living monster has an unmodeled power that can reduce damage, add block,
prevent death, revive, or phase-change, return `false` for the scan. The goal is
not to reject common deterministic mechanics; add them to this table when they
show up often enough to cause false negatives.

---

## 6. Damage and Stance

CommunicationMod renders card text for the current player state. A Strike in
Wrath already says 12 damage, so do not multiply the current card by current
stance again.

Rendered text can include one-shot player damage modifiers. If effects such as
Vigor, Wreath of Flame, Pen Nib, or other "next attack" bonuses may be present,
the scanner must model them as consumable modifiers. If it cannot, fail closed.

For later cards after a simulated stance change:

```
stance_ratio = current_stance.mult() / initial_stance.mult()
base = floor(card.parsed_damage * stance_ratio)
base += strength_delta * current_stance.mult()
base = max(0, base)
```

Per attack hit:

1. Trigger Curl Up if present and untriggered.
2. Apply damage modifiers in the same order the game applies target powers. If
   that order is not available, use a conservative lower damage value or return
   `false` for that branch. Known modifiers:
   - Slow: `floor(damage * (1.0 + 0.1 * amount))`
   - Vulnerable: `floor(damage * 1.5)`
   - Flight: `floor(damage * 0.5)` while active
   - Intangible: cap positive damage to `1`
3. Apply block.
4. Apply Invincible to cap hp loss.
5. Subtract unblocked damage from hp.
6. If unblocked attack damage was dealt:
   - decrement Flight if present
   - trigger Malleable

After the card:

- apply Vulnerable through Artifact
- apply energy gain
- apply Strength gain
- apply stance/Mantra changes
- increment Slow on living monsters that have Slow
- apply deterministic exhaust/consume to `remaining_cards`
- if the card has unsupported draw/discard/exhaust/hand mutation and combat has
  not ended, stop this branch

Bash-style cards are therefore correct: Bash damage does not benefit from the
Vulnerable that Bash applies, but later attacks do.

### Stance Energy

| Change | New stance | Energy delta |
|--------|------------|--------------|
| No stance change | unchanged | 0 |
| Enter Wrath | Wrath | Calm exit bonus if leaving Calm |
| Enter Calm | Calm | 0 |
| Exit stance | Neutral | Calm exit bonus if leaving Calm |
| Enter Divinity | Divinity | `+3` plus Calm exit bonus if leaving Calm |
| Mantra reaches 10 | Divinity | `+3` plus Calm exit bonus if leaving Calm |

Calm exit bonus is `+2` exactly once when moving from Calm to non-Calm.

X-cost cards spend all current energy. Their X value is:

```
x = spent_energy + x_cost_bonus
```

---

## 7. DFS

```
fn dfs(state):
    if combat_ended(state): return Some([])
    if state.depth >= remaining_card_plays: return None
    if state_budget_exceeded: return None

    key = canonicalize(state)
    if memo contains key: return memo[key]

    for card in remaining_cards:
        for play in legal_plays(card, state):
            if let Some(child) = resolve_play(play, state):
                if let Some(mut suffix) = dfs(child):
                    suffix.prepend(KillPlay {
                        card: card.uuid,
                        target: play.target,
                    })
                    return memo[key] = Some(suffix)

    memo[key] = None
    return None
```

DFS returns any deterministic kill sequence that satisfies the modeled rules.
Use a fixed local traversal for reproducibility and easier debugging, but the
public API does not promise the globally first sequence:

1. lower hand index first
2. for targeted cards, lower command target index first
3. deterministic non-target cards before equivalent unsupported-final-play
   variants

| Card type | Branches |
|-----------|----------|
| Targeted damage / execute / targeted debuff | one per living target |
| AoE | one |
| Energy / Strength / stance only | one |
| Deterministic exhaust/consume | one, updating `remaining_cards` |
| X-cost | one, spending all current energy |
| Random-target | one only if the guarantee formula below passes |

If a random-target card fails the formula at the current state, skip that play.
Do not simulate partial random damage. DFS may still play other cards first and
try the random-target card again later.

---

## 8. Random-Target Guarantee

For simple random-target attacks where every hit has the same effective damage
against every living monster and no modeled power changes durability during the
card:

```
hits >= alive_count
total_damage = damage_per_hit * hits
total_durability = sum(living_monster.hp + living_monster.block)

total_damage >= total_durability + (damage_per_hit - 1) * (alive_count - 1)
```

For two enemies this is:

```
total_durability + damage_per_hit - 1 <= total_damage
```

The extra term is worst-case overkill waste. Dead monsters cannot be targeted by
later hits, so the worst waste is at most `damage_per_hit - 1` for each monster
that dies before the final survivor.

Only use this shortcut for uniform simple damage. If target-specific modifiers
or mutable powers make damage/durability non-uniform, skip the random card at
that state.

---

## 9. Combat-End Check

For production `can_end_fight`, combat ends when every living monster is a
minion:

```
all monsters with hp > 0 have is_minion == true
```

Dead monsters' block is irrelevant. Living monsters' block counts as durability.

---

## 10. State-Space Budget

Depth:

```
D = min(scan_relevant_cards.len(), remaining_card_plays)
```

Normally `D <= 10`, and the relevant-card filter often lowers it to `3-6`.

Monster count:

- boss/elite fights: usually `M = 1-3`
- hallway fights: usually `M = 1-4`
- `M = 5` is a rare/worst-case cap

Immediate branch bound:

```
remaining_cards * max(1, M)
```

Usually this is `10-30` for boss/elite and `10-40` for hallway. The raw no-memo
targeted-only leaf bound is still bad:

```
D! * M^D
```

At `D = 10`, `M = 5`, this is about `3.5e13`, so use memoization and a hard
budget.

Production budget:

| Budget | Value | On exceed |
|--------|-------|-----------|
| Wall-clock hard deadline | `2s` per scan | return `false` |
| Worker threads | up to `8` | split root/early DFS branches |
| Expanded DFS states | implementation cap, e.g. `2_000_000-10_000_000` global | return `false` |
| Memo entries | memory cap, tune from profiling | return `false` |

The wall-clock deadline is the authoritative cap. State/memo caps protect
memory and catch pathological branches before the full deadline. Cap or timeout
hits are safe false negatives. Log original hand size, relevant-card count,
monster count, targeted-card count, expanded states, memo entries, elapsed time,
and whether mutable monster powers were present.

Parallelization should be coarse-grained:

- Generate legal root plays, or legal first-two-depth prefixes if root branching
  is too small.
- Evaluate prefixes on up to 8 workers.
- Stop as soon as any worker returns a valid kill sequence.
- Use a shared deadline and cancellation flag.
- Prefer per-worker memo tables unless shared memoization clearly wins in
  profiling; lock contention can cost more than duplicate work at this scale.

Expected time with compact parsed effects and integer memo keys:

| Situation | Expected time |
|-----------|---------------|
| Normal hallway/common boss line | microseconds to low milliseconds |
| Larger elite/boss hand | single-digit to tens of milliseconds |
| Pathological but bounded search | hundreds of milliseconds up to `2s` |

Any scan reaching the `2s` deadline returns `false`.

---

## 11. Tests

Minimum tests before wiring into ranker/autoplay:

- pure block/utility cards are removed before DFS
- scanner returns a valid ordered kill sequence as `{card, target}` steps
- targeted sequence steps include command target index in `target`
- targeted overkill does not spill to another monster
- Bash Vulnerable applies after Bash damage and through Artifact
- Curl Up triggers once
- Malleable adds/increments after unblocked attack damage
- Flight halves damage and is decremented by attack damage
- Slow increases damage on later cards
- Invincible caps total hp loss for the scan
- Intangible caps to 1 per hit
- damage modifier ordering/rounding cannot overestimate damage
- Calm exit grants 2 energy; neutral-to-Wrath does not
- X-cost spends all current energy and includes Chemical X
- Judgment ignores block and kills only at/below threshold
- deterministic exhaust/consume removes known remaining cards and can continue
- chosen exhaust is deterministic only with exactly one legal choice
- Fiend Fire-style damage counts known exhausted cards
- random or choice-based exhaust can certify only if combat ends immediately
- random-target formula accepts `4 x 3` vs two ordinary enemies with total
  durability `8`
- random-target formula rejects the same card at total durability `10`
- damage cards with unsupported draw/discard/hand mutation can certify only if combat ends
  immediately after that card
- unsupported dangerous powers return `false`
