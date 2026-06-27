# Combat Action Ranker — Score Rules Reference

Each available combat action (play card, use potion, end turn) is scored by
summing all applicable rules. Unless a rule explicitly overrides another
(e.g., Manual Filters set `i64::MIN`, Intangible dominates Vulnerable), every
matching rule contributes additively to the final score. Actions matching a
**Manual Filter** receive `i64::MIN` and go to the "Avoid" list.

---

## 1. Base

| Rule | Condition | Score |
|------|-----------|-------|
| Cost penalty | Every card (not STATUS/CURSE) | `-10 × cost` |
| Status/Curse | `card_type == "STATUS"` or `"CURSE"` | `-100` |
| X-cost | `cost == -1` (not STATUS/CURSE) | `-10 × current_energy` |

### X-cost Effect Resolution

When `cost == -1`, the card spends all current energy. The effect magnitude
uses that spent energy; Chemical X adds to the effect value but does not leave
energy behind.

The description contains `"X"` instead of a number for the variable part.

| Rule | Condition | Override |
|------|-----------|----------|
| X-cost damage hits | `cost == -1` AND `card_type == "ATTACK"` | `hits = spent_energy + chemical_x_bonus` (description says "X次"/"X times" — X cannot be parsed as a digit) |
| X-cost non-damage amount | `cost == -1` AND NOT `"ATTACK"` | `effective_amount = spent_energy + chemical_x_bonus` (when parsed amount is 0 because X is non-numeric) |

These overrides ensure X-cost cards like 旋风斩 (8 dmg/X次) correctly use
all current energy and Malaise correctly uses the X value for STR loss/Weak.

---

## 2. Combat Outcome

| Rule | Condition | Score |
|------|-----------|-------|
| Ends fight | `can_end_fight(ctx_after_candidate)` succeeds (Section 18) | `+1000` |
| Ends fight (boss dead) | Kills most-maxHP monster AND all other alive monsters have power `"Minion"` | `+1000` |
| Priority kill | Kills most-maxHP monster AND non-minion monsters remain | `+500` |
| Minion kill | Target has power `"Minion"` AND would die | `+200` |

---

## 3. Core Effects

### 3a. Targeted Damage

| Rule | Condition | Score |
|------|-----------|-------|
| Damage | Parsed damage > 0 AND target does NOT have `"Intangible"` | `+1000 × damage × hits / total_monster_hp_pool` |
| Damage (Vulnerable target) | Parsed damage > 0 AND target has `"Vulnerable"` AND target does NOT have `"Intangible"` | `+1000 × damage × 1.5 × hits / total_monster_hp_pool` |
| Damage (Intangible target) | Target has `"Intangible"` (amount > 0) | `+1000 × 1 × hits / total_monster_hp_pool` |

Damage is scored as a fraction of the total enemy HP+block pool. This naturally
deprioritizes chip damage against large pools and values damage more as enemies
near death.

When both `"Intangible"` and `"Vulnerable"` are present, Intangible takes
precedence — damage is capped to 1 per hit, and the Vulnerable multiplier is
not applied.

Player-side damage modifiers (STR, Wrath, Divinity, etc.) are already baked
into the card description by the game engine. Only enemy Vulnerable and
Intangible need manual application.

### 3b. AoE Damage (cards hitting all enemies)

Detection: description contains `"所有"` / `"all"` for "all enemies".

| Rule | Condition | Score |
|------|-----------|-------|
| AoE (1 monster) | Only 1 alive monster | Same as targeted — full modifiers + kill check |
| AoE (multiple monsters) | Multiple monsters alive | `+1000 × damage × hits × monster_count / total_monster_hp_pool` + summed target priority modifiers (scaling +15/enemy, punish +10/enemy, killable +5/enemy) |

### 3c. Random-target Damage

Detection: `card.has_target == false` AND `card.card_type == "ATTACK"` AND
NOT AoE (no "all enemies" keyword).

| Rule | Condition | Score |
|------|-----------|-------|
| Random (1 monster) | Only 1 alive monster | Same as targeted — full modifiers + kill check |
| Random (multiple monsters) | Multiple monsters alive | `+10 × damage × hits` (raw, no target modifiers, no kill check per target) |

**Random-target kill check**: random-target cards certify `can_end_fight()` only
when the shared scanner's closed-form overkill guarantee passes:
`damage * hits >= total_durability + (damage - 1) * (alive_count - 1)`, with
`hits >= alive_count`. Do not use expected damage or odds.
If the guarantee fails at the current DFS state, skip that random-card play;
other visible cards may still be played first so the random card can become a
guaranteed finisher later.

### 3d. Block

| Rule | Condition | Score |
|------|-----------|-------|
| Block (non-excessive) | Parsed block > 0 AND (`current_block < incoming_damage` OR has retain) | `+1000 × min(block, incoming_damage) / current_hp` |
| Block (excessive) | Parsed block > 0 AND `current_block >= incoming_damage` AND no retain | `-2000 × block / current_hp` |

Block is scored as the fraction of current HP saved, capped at incoming damage to
prevent valuing over-blocking. This naturally makes blocks higher priority at low
HP and lower priority at high HP.

The `min(block, incoming_damage)` cap ensures only the block actually needed to
absorb incoming damage is valued. Excess block beyond incoming damage gets zero
from the non-excessive rule (and is penalized by the excessive rule if you were
already over-blocked).

The penalty triggers only when already overblocked *before* playing the card
(`current_block >= incoming_damage`). Partial overblock created by the card
itself (where `current_block < incoming_damage` but
`current_block + card_block > incoming_damage`) does NOT trigger the penalty;
the excess is silently discarded.

Block retain detection: player has `"Barricade"` power or `"Calipers"` relic.
When retain is present, all block scores at `+1000 × block / current_hp` regardless of
pre-existing block (no `min()` cap since block carries over).

### 3e. Heal

| Rule | Condition | Score |
|------|-----------|-------|
| Heal | Parsed heal > 0 | `+50` |

### 3f. Draw

| Rule | Condition | Score |
|------|-----------|-------|
| Draw (has energy) | Parsed draw > 0 AND `remaining_energy > 0` after play | `+10 × draw_amount` |
| Draw (no energy, free reachable) | Parsed draw > 0 AND `remaining_energy == 0` AND free cards reachable | `+10 × draw_amount × (free_count / pile_size)` |
| Draw (no energy, no free) | Parsed draw > 0 AND `remaining_energy == 0` AND no free cards reachable | `0` |

**Free card** = 0-cost, non-STATUS, non-CURSE card.
**Pile** = draw pile if non-empty, else discard pile.

---

## 4. Target Priority (per target, for targeted cards)

| Rule | Condition | Score |
|------|-----------|-------|
| Scaling target | Target `is_scaling` | `+15` |
| Punish target | Target has `"Thorns"` or `"Curiosity"` or `"Enrage"` | `+10` |
| Killable target | Target `can_be_killed` | `+5` |

---

## 5. Setup / Scaling

| Rule | Condition | Score |
|------|-----------|-------|
| POWER card | `card_type == "POWER"` AND no `monster_id == "AwakenedOne"` | `+30` |
| Self STR/DEX | Parsed gain > 0 | `+15 × gain` |
| Poison | "Poison"/"中毒" + amount > 0 | `+10 × amount` |
| Vulnerable | "Vulnerable"/"易伤" + amount > 0 | `+10 × amount` |
| Weak (new) | No existing "Weak" power on target | `(5 + (target_total_damage × 2.5)) × weak_amount` |
| Weak (refresh) | Already has "Weak" power on target | `5 × weak_amount` |
| Enemy STR loss (attacking) | Enemy loses STR AND intent is attacking | `+10 × min(str_loss, monster.damage) × monster.hits` |
| Enemy STR loss (not attacking, permanent) | Enemy loses STR AND NOT attacking AND NOT `一回合`/`1 turn` | `+10 × str_loss` |
| Enemy STR loss (not attacking, temporary) | Enemy loses STR AND NOT attacking AND `一回合`/`1 turn` | `0` |
| Energy gain | `[E]` count in description > 0 | `+20 × amount` |
| Turn 1 | `turn_number == 1` | `+10` |
| Zero-cost | `cost == 0` | `+5` |
| Ethereal | Card is ethereal | `+5` |

**Weak formula**: `target_total_damage = monster.damage × monster.hits` (0 if None).

**Enemy STR loss temporary**: detection = `"一回合"` (zh) or `"1 turn"` (en) in
description. The STR loss expires this turn — worthless when enemy isn't attacking.
**Permanent** = no such qualifier (e.g. Disarm, Malaise).

---

## 6. Utility

| Rule | Condition | Score |
|------|-----------|-------|
| Exhaust Status/Curse | Exhausts a STATUS or CURSE card | `50 - card_base_score` |
| Discard Status/Curse | Discards a STATUS or CURSE card | `50 - card_base_score` |
| Minion kill | Target has `"Minion"` AND would die | `+200` |

### 6a. Exhaust Synergy

When the player has exhaust-synergy powers or relics, exhausting cards earns
bonuses. `exhaust_count` = number of cards the action exhausts (1 if
self-exhaust only, N if "Exhaust N cards"). Defaults to 1 if exhaust keyword
is present without a count.

| Rule | Condition | Score |
|------|-----------|-------|
| Exhaust w/ Dark Embrace | Card exhausts AND player has `"Dark Embrace"` power | `+10 × exhaust_count` |
| Exhaust w/ Feel No Pain | Card exhausts AND player has `"Feel No Pain"` power | Effective block = `Feel No Pain.amount × exhaust_count`, scored via Section 3d |
| Exhaust w/ Charon's Ashes | Card exhausts AND player has `"Charon's Ashes"` relic | `+30 × monster_count × exhaust_count` |

Feel No Pain block stacks additively with any other block the card provides —
the total `card_block` feeds into the non-excessive/excessive formulas of
Section 3d, lethal-save (`+200`, §8), and Beat of Death (§13).
Dark Embrace draw may trigger Section 3f free-card-reachable checks.

These bonuses are additive to the base exhaust Status/Curse scoring.

---

## 7. Punishment

| Rule | Condition | Score |
|------|-----------|-------|
| SKILL vs Nob | `card_type == "SKILL"` AND any `monster_id == "GremlinNob"` | `-100` |
| POWER vs Awakened One | `card_type == "POWER"` AND any `monster_id == "AwakenedOne"` | `-100` |
| ATTACK vs Thorns | `card_type == "ATTACK"` AND target has `"Thorns"` (amount > 0) | `-5 × Thorns.amount × hits` |

---

## 8. Danger Context

| Rule | Condition | Score |
|------|-----------|-------|
| Lethal + block + saved | `incoming_lethal` AND `hp + block + card_block >= incoming_damage` | `+200` |
| Lethal + block | `incoming_lethal` AND parsed block > 0 | `+50` |
| Lethal + damage | `incoming_lethal` AND parsed damage > 0 | `+20` |
| Lethal + zero impact | `incoming_lethal` AND damage == 0 AND block == 0 | `-50` |
| HP-cost | Parsed self-damage > 0 | `-1_050_000 × self_damage / max(1, cur_hp)³` |

**HP-cost formula**: uses absolute `cur_hp` (not ratio), scaling as `1 / cur_hp³`.
At high HP the penalty is negligible; at ~45 HP Offering's +70 in energy/draw is
exactly cancelled. Around 5 HP the penalty approaches `i64::MIN`. The floor of
`max(1, cur_hp)` prevents division by zero.

---

## 9. Manual Filters (score = `i64::MIN` → "Avoid" list)

| Card ID | Condition | Reason |
|---------|-----------|--------|
| `Limit Break` | No `"Strength"` power on player OR amount ≤ 0 | `STR=0, no effect` |
| `Spot Weakness` | Target monster `intent != "ATTACK"` | `target not attacking` |
| `Blasphemy` / `渎神` | NOT in `"Divinity"` AND `can_end_fight(ctx_after_blasphemy)` | Scored as `+1000` instead — fight ends, death irrelevant |
| `Blasphemy` / `渎神` | NOT in `"Divinity"` AND cannot end fight | `suicide` |
| `Blasphemy` / `渎神` | Already has `"Divinity"` power | `−1000` — death for nothing: ×3 already active, no +3 energy from re-entering |

---

## 10. End Turn

**Useful card**: a card is "useful" if `playable == true`. CommunicationMod
already accounts for all hard blocks (Entangled, status, etc.) in the
`playable` flag.

| Rule | Condition | Score |
|------|-----------|-------|
| Plays remain | Useful cards exist in hand | `max(0, 40 - best_card_score)` |
| Hand exhausted | No useful cards in hand | `80` |

---

## 11. Potions

Potions use the same parsing and heuristics as cards, but with their own base
score. Potions with `can_use == false` are skipped entirely. Potions with
`requires_target` are scored per valid target.

| Rule | Condition | Score |
|------|-----------|-------|
| Potion base | Every usable potion | `-20` |
| Damage | Parsed damage > 0 | `+1000 × damage × hits / total_monster_hp_pool` (same pool-relative formula as cards) |
| Damage (Vulnerable target) | Parsed damage > 0 AND target has `"Vulnerable"` | `+1000 × damage × 1.5 × hits / total_monster_hp_pool` |
| Damage (Intangible target) | Target has `"Intangible"` | `+1000 × 1 × hits / total_monster_hp_pool` |
| Block | Parsed block > 0 | `+1000 × min(block, incoming_damage) / current_hp` (non-excessive) or `-2000 × block / current_hp` (excessive), same rules as §3d |
| Heal | Parsed heal > 0 | `+50` |
| Debuffs (Poison/Vuln/Weak/STR loss) | Same formulas as Section 5 | Same scores |
| Ends fight | `total_damage >= sum_of_all_monster_hp_and_block` | `+1000` |

Cost penalty, energy, draw, ethereal, turn-1, and Beat of Death rules do NOT
apply to potions.

---

## 12. Artifact (Debuff Mitigation)

When a target has `"Artifact"` power (amount > 0), debuff effects of each type
are blocked one stack per type — the first debuff type consumes one Artifact
stack, the second debuff type consumes another, and so on. Blocked types score
0; any remaining types after Artifact stacks are exhausted score normally.

| Rule | Condition | Score |
|------|-----------|-------|
| Debuff type blocked | Card/potion applies debuff type AND target has `"Artifact"` (amount > number of prior debuff types applied) | Set that debuff type's score to `0` |
| Artifact consumed | Above — one Artifact stack is consumed per debuff type blocked | `+10` per stack consumed |

Net effect: a card applying both Vulnerable and Weak to a target with 1 Artifact
block Vulnerable (0), but Weak scores normally. Total: `+10` for the consumed
stack + full Weak score.

---

## 13. Beat of Death

The Heart's Beat of Death — lose 1 HP per card played. Applied as a global
penalty to every card action. Does NOT apply to potions or End Turn.

| Rule | Condition | Score |
|------|-----------|-------|
| Beat of Death (with block) | Any monster has `"BeatOfDeath"` AND (`current_block > 0` OR `card_block > 0`) | `-10` (block absorbs the hit) |
| Beat of Death (no block) | Any monster has `"BeatOfDeath"` AND `current_block == 0` AND `card_block == 0` | `-20` (direct HP loss — treated as self-damage) |

**Block reduction**: if the card provides block, the first point of that block
is consumed by Beat of Death. Reduce `card_block` by 1 before computing the
non-excessive/excessive block score via Section 3d. If
`card_block` drops from 1 to 0, treat as no block (the `-20` BoD penalty and
excessive/non-excessive checks still apply based on pre-reduction `card_block >
0` for the BoD-with-block `-10` tier).

---

## 14. Orbs & Focus (Defect)

Orb data is available in `combat_state.player.orbs` from CommunicationMod.
Each orb has `id` (`"Lightning"`, `"Frost"`, `"Dark"`, `"Plasma"`) and
`amount` (potency). Focus is a player power (`"Focus"`).

**Focus** = sum of player's `"Focus"` power amounts (0 if no Focus). Used as an
additive bonus in orb formulas below.

### Orb Channels (passive placement)

| Rule | Condition | Score |
|------|-----------|-------|
| Channel Lightning | Channels a Lightning orb | `+10 × (focus + 3)` |
| Channel Frost | Channels a Frost orb | `+50` (effective block = `focus + 2`, if `incoming_damage > 0`) |
| Channel Dark | Channels a Dark orb | `+10` (accumulates 6 damage/turn, hard to value immediately) |
| Channel Plasma | Channels a Plasma orb | `+20` (generates 1 energy/turn) |

### Orb Evocations (active removal)

All evocations have a `-10` base penalty (cost of losing the orb's passive slot
benefit). This `-10` is applied ONCE per card, not per-evoke.

| Rule | Condition | Score |
|------|-----------|-------|
| Evoke Lightning N times | Evokes N Lightning orbs | `-10 + (10 × (focus + 8)) × N` |
| Evoke Frost N times | Evokes N Frost orbs | `-10 + 50 × N` (effective block = `focus + 5` per evoke, if `incoming_damage > 0`) |
| Evoke Dark N times | Evokes N Dark orbs | `-10 + (10 × accumulated_damage) × N` |
| Evoke Plasma N times | Evokes N Plasma orbs | `-10 + 40 × N` (2 energy per evoke × 20/energy) |

**Dark accumulated damage**: read from the Dark orb's `amount` field.

### Other Orb Rules

| Rule | Condition | Score |
|------|-----------|-------|
| Expand orb slot | Card expands orb slots | `+100 × expand_count` |
| Focus gain | Card grants Focus (player has at least 1 orb slot) | `+20 × focus_amount` |

---

## 15. New State Fields Required

| Field | Type | Source | Used For |
|-------|------|--------|----------|
| `MonsterInfo.monster_id` | `String` | raw `"id"` in monster JSON | Punishment triggers (GremlinNob, AwakenedOne) |
| `NormalizedState.turn_number` | `Option<i64>` | raw `"combat_state.turn"` | Turn 1 bonus |
| Orb data | `Vec<Orb>` | raw `"combat_state.player.orbs"` | Orb channel/evoke scoring |
| Player Focus | `Option<i64>` | raw `"combat_state.player.powers[Focus]"` | Orb damage/block calculations |

---

## 16. Description Parsing

Numbers are extracted via regex from `CardInfo.description`. All numbers must
be `> 0` to qualify for scoring. Each effect has both a Chinese and English
pattern.

| Effect | zh pattern | en pattern | Extracts |
|--------|-----------|-----------|----------|
| Damage | `造成 N 点伤害` | `Deal N damage` | damage per hit |
| Multi-hit | `N 次` / `两次` | `N times` / `twice` | hit count |
| Block | `获得 N 点 格挡` | `Gain N Block` | block value |
| Heal | `回复 N` | `Heal N` | heal amount |
| Draw | `抽 N 张牌` | `Draw N card(s)` | cards drawn |
| Self-damage (permanent) | `失去 N 点生命` | `Lose N HP` | HP loss |
| STR gain (self) | `获得 N 点 力量` | `Gain N Strength` | gain amount |
| STR gain (temp) | `获得.*力量.*你的回合结束时.*失去` | `Gain.*Strength.*end of.*turn.*lose` | gain (temporary) |
| DEX gain | `获得 N 点 敏捷` | `Gain N Dexterity` | gain amount |
| Enemy STR loss (permanent) | `敌人.*失去.*力量` | `enemy.*lose.*Strength` | loss amount |
| Enemy STR loss (temp) | `一回合` in context | `1 turn` in context | loss (temporary) |
| Poison | `中毒 N` | `Poison N` | amount |
| Vulnerable | `易伤 N` | `Vulnerable N` | amount |
| Weak | `虚弱 N` | `Weak N` | amount |
| Energy | `[E]` | `[E]` | count |
| Exhaust | `消耗` | `Exhaust` | present + count |
| Discard | `丢弃` | `Discard` | present + count |
| Ethereal | `虚无` | `Ethereal` | present |
| Channel orb | `充能球` | `Channel` | orb type + count |
| Evoke orb | `激发` | `Evoke` | orb type + count |
| Expand orb slot | `充能球栏位` | `orb slot` | expand count |
| Focus | `集中` | `Focus` | amount |
| Enter Wrath | `进入.*愤怒` | `Enter.*Wrath` | present |
| Enter Calm | `进入.*宁静` | `Enter.*Calm` | present |
| Exit stance | `退出.*姿态` / `结束.*姿态` | `Exit.*stance` / `End.*stance` | present |
| Mantra | `真言` | `Mantra` | amount |

---

## 17. Watcher Stances

**Core principle**: card descriptions already reflect the current stance.
CommunicationMod adjusts them: a Strike reads `造成 12 点伤害` in Wrath.
The ranker does NOT re-apply multipliers to cards played in their current
stance. Multipliers are only applied when evaluating *other* cards in hand
that will be played after a stance change.

**Incoming damage**: CommunicationMod already applies Wrath ×2 to
`incoming_damage`. The ranker does NOT re-apply incoming multipliers. All
block/lethal/danger calculations use state values as-is.

### Stance Powers (from `state.powers`)

| Power ID | Meaning |
|----------|---------|
| `"Wrath"` | Deal ×2, take ×2 (already in CommMod data) |
| `"Calm"` | Gain 2 energy on exit |
| `"Divinity"` | Deal ×3, gained 3 energy on entry |
| `"Mantra"` | At 10 stacks, auto-enter Divinity |

### Calm Exit

| Rule | Condition | Score |
|------|-----------|-------|
| Calm exit | Has `"Calm"` AND card exits Calm (enters Wrath or neutral) | `+40` (2 energy × 20) |

### Wrath Retreat

Casting a card that exits Wrath while monsters are attacking.

| Rule | Condition | Score |
|------|-----------|-------|
| Retreat block | Has `"Wrath"` AND card exits Wrath AND `incoming_damage > 0` | Effective block = `incoming_damage × 0.5` (state is already ×2, exiting halves incoming). Scored via Section 3d: feeds non-excessive/excessive block scoring and lethal-save. |
| Retreat penalty | `can_end_fight(ctx_without_retreat_card)` succeeds while staying in Wrath | `-100` (staying in Wrath kills all — should kill, not retreat) |
| Retreat (no incoming) | `incoming_damage == 0` | `0` |

### Mantra

| Rule | Condition | Score |
|------|-----------|-------|
| Mantra gain | Card grants `"Mantra"` | `+10 × mantra_amount` |
| Divinity trigger | `mantra_before + mantra_gain >= 10` | `+50` (free Divinity entry — +3 energy + ×3 damage) |

---

## 18. Shared `can_end_fight()` Scanner

Use the dedicated scanner spec in [`docs/kill-scan.md`](kill-scan.md).

Integration contract for ranker rules:

- `can_end_fight(ctx)` means a visible-hand sequence can **guarantee** combat
  ends this turn. Draws, generated cards, and odds-based random lines do not
  qualify. Simple random-target cards qualify only when the closed-form overkill
  guarantee passes.
- The scanner owns stance, X-cost, monster-power, and per-monster damage logic.
  Ranker rules should not duplicate that arithmetic.
- For scanner purposes, remaining minions are acceptable when all non-minion
  enemies are dead.
- A `true` result is worth `+1000` for "Ends fight". A `false` result can still
  be a false negative when the line depends on an unsupported beneficial effect
  such as poison, orb damage, draw, generated cards, or relic side damage.
