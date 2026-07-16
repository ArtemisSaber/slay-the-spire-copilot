# Ranker Module Architecture

Rule-engine-powered combat action ranker. Scores every available combat action
(play card, use potion, end turn) so callers can pick the highest-scoring action.

The rule engine is driven by a **JSON rule set** (`rules.json`). Rules are
self-contained objects carrying their own weights. Conditions are implicit AND
with no nesting. OR semantics come from multiple rules overriding the same base.

---

## 1. Boundaries

### In Scope

- Hand cards (targeted, AoE, random-target, X-cost, POWER, SKILL, ATTACK).
- Potions (damage, block, debuffs; no cost/energy/draw rules).
- End Turn.
- Description parsing: zh/en regex for damage, hits, block, heal, draw,
  self-damage, STR/DEX gain, enemy STR loss, poison, vulnerable, weak, energy,
  exhaust, discard, ethereal, orbs, focus, stance, mantra.
- X-cost resolution (effective hits/damage from spent energy + Chemical X).
- Overrides: a rule suppresses another rule's contribution.
- `per_target` iteration for targeted actions.
- `score_fn` dispatch to Rust for formulas too complex for `@var` expressions.

### Out of Scope

- Kill-scan integration (`can_end_fight` scanner). End-fight scoring uses a
  `compute` formula checking raw damage vs. total monster HP+block.
- Beat of Death — uniform penalty across all cards; does not affect ranking.
- Artifact — debuff mitigation does not affect ranking.
- Draw simulation and generated card simulation.
- Relic side damage.
- Full combat simulation.

---

## 2. Module Layout

```
src/ranker/
├── mod.rs           # pub fn rank() → Vec<ScoredAction>
├── rules.rs         # serde: Rule, Condition, RuleSet, ConditionScope
├── engine.rs        # evaluate loop, override suppression, sort, Avoid list
├── context.rs       # ActionContext, @var resolver, context builder
├── formula.rs       # parse/eval "@damage * @hits * @weight / max(@pool, 1)"
├── parser.rs        # zh/en regex → ParsedEffects
├── predicates.rs    # score_fn registry (hp_cost_penalty, weak_new, etc.)
└── rules.json       # canonical rule source; build.rs copies to target/, embedded fallback
```

Tests live under `src/tests/ranker_tests.rs`, matching the existing inline
test-module pattern.

Keep this module independent from prompts, LLM calls, and autoplay session state.

---

## 3. Public API

```rust
pub struct ScoredAction {
    pub action_type: ActionType,     // PlayCard, UsePotion, EndTurn
    pub target_index: Option<usize>, // Command target index (if targeted)
    pub score: i64,                  // Total score (sum of matching rules)
    pub breakdown: Vec<RuleResult>,  // which rules contributed — debug/explain
    pub is_avoid: bool,              // score == i64::MIN
}

pub enum ActionType {
    PlayCard { card_id: String, card_name: String },
    UsePotion { potion_name: String },
    EndTurn,
}

pub struct RuleResult {
    pub rule_id: String,
    pub score: i64,
    pub matched: bool,
}

/// Score every available action. Returns ranked list descending by score.
/// Actions with score == i64::MIN are segregated into the "Avoid" list
/// (positioned at the end).
pub fn rank(state: &CombatState) -> Vec<ScoredAction>;
```

`rank()` builds action contexts for each card×target and potion×target
combination, evaluates all rules against each context, suppresses overridden
contributions, sums non-suppressed scores, and returns sorted results.

---

## 4. Data Flow

```
NormalizedState + available actions
        │
        ▼
  Context Builder
  ┌──────────────────────────────────────────┐
  │  For each (action × target):             │
  │    1. Parse descriptions (zh/en regex)   │
  │       → @damage @hits @block @heal ...   │
  │    2. Resolve X-cost overrides           │
  │    3. Resolve state @vars                │
  │       (@cost @current_energy @cur_hp...) │
  │    4. Emit ActionContext                 │
  └──────────────────────────────────────────┘
        │
        ▼
  Rule Engine (per ActionContext)
  ┌──────────────────────────────────────────┐
  │  For each rule in ascending priority order  │
  │  (override rules carry lower priority       │
  │   so they are evaluated before their base): │
  │    ├─ Skip if action type not in scope   │
  │    ├─ Evaluate all conditions (AND)      │
  │    ├─ If matched:                        │
  │    │   ├─ Compute score                  │
  │    │   │   ├─ weight only → flat const   │
  │    │   │   ├─ formula   → eval @vars     │
  │    │   │   └─ score_fn  → Rust dispatch  │
  │    │   ├─ Record RuleResult              │
  │    │   └─ If override → suppress target  │
  │    └─ If suppressed (by prior override): │
  │        ├─ Skip score                     │
  │        └─ Still record as matched=false  │
  └──────────────────────────────────────────┘
        │
        ▼
  Post-process
  ┌──────────────────────────────────────────┐
  │  1. Sum all non-suppressed contributions │
  │  2. Sort by score descending             │
  │  3. Separate i64::MIN → Avoid list       │
  │  4. Return Vec<ScoredAction>             │
  └──────────────────────────────────────────┘
```

---

## 5. JSON Rule Format

### Priority ranges

Rules evaluate in ascending priority order (lower = first). Override rules
carry lower priority than the base rules they suppress.

| Range | Section |
|-------|---------|
| 1000–1009 | Potion base |
| 1010–1019 | Card base (cost, status/curse, X-cost) |
| 1020–1029 | Manual filters |
| 1030–1039 | Target priority |
| 1040–1049 | Combat outcome |
| 1050–1079 | Core effects (damage, block, heal, draw) |
| 1080–1099 | Setup / scaling |
| 1100–1119 | Watcher stances |
| 1120–1139 | Orbs & focus |
| 1140–1149 | Utility / exhaust |
| 1150–1159 | Exhaust synergy |
| 1160–1169 | Punishment |
| 1170–1179 | Danger context |
| 2000–2009 | End turn |

Numbers below 1000 are reserved for future rules.

### Complete example set

```json
{
  "version": "1.0",
  "available_score_fns": [
    "hp_cost_penalty",
    "weak_new_formula",
    "weak_refresh_formula"
  ],
  "rules": [
    {
      "rule_id": "base_cost_penalty",
      "priority": 1010,
      "weight": -10,
      "applies_to": ["play_card"],
      "formula": "@weight * @cost",
      "conditions": [
        {"card": {"type_not_in": ["STATUS", "CURSE"]}}
      ]
    },
    {
      "rule_id": "base_status_curse",
      "priority": 1011,
      "weight": -100,
      "applies_to": ["play_card"],
      "conditions": [
        {"card": {"type_in": ["STATUS", "CURSE"]}}
      ]
    },
    {
      "rule_id": "base_x_cost",
      "priority": 1012,
      "weight": -10,
      "applies_to": ["play_card"],
      "formula": "@weight * @current_energy",
      "conditions": [
        {"card": {"cost_eq": -1}},
        {"card": {"type_not_in": ["STATUS", "CURSE"]}}
      ]
    },

    {
      "rule_id": "combat_ends_fight",
      "priority": 1040,
      "weight": 1000,
      "applies_to": ["play_card", "use_potion"],
      "conditions": [
        {"compute": {"formula": "@total_damage >= @monsters_total_hp_plus_block"}}
      ]
    },
    {
      "rule_id": "combat_priority_kill",
      "priority": 1041,
      "weight": 500,
      "applies_to": ["play_card", "use_potion"],
      "conditions": [
        {"target": {"is_max_hp": true}},
        {"target": {"can_be_killed": true}},
        {"monsters": {"any": {"is_not_minion": true, "exclude_target": true}}}
      ]
    },
    {
      "rule_id": "combat_minion_kill",
      "priority": 1042,
      "weight": 200,
      "applies_to": ["play_card", "use_potion"],
      "conditions": [
        {"target": {"power": "Minion"}},
        {"target": {"can_be_killed": true}}
      ]
    },

    {
      "rule_id": "core_damage_vulnerable",
      "priority": 1050,
      "weight": 1000,
      "applies_to": ["play_card"],
      "formula": "@damage * @hits * @weight * 1.5 / max(@monsters_total_hp_plus_block, 1)",
      "override": "core_damage",
      "per_target": true,
      "conditions": [
        {"parsed": {"damage_gt": 0}},
        {"target": {"power": "Vulnerable"}},
        {"target": {"power_not": "Intangible"}}
      ]
    },
    {
      "rule_id": "core_damage_intangible",
      "priority": 1051,
      "weight": 1000,
      "applies_to": ["play_card"],
      "formula": "1 * @hits * @weight / max(@monsters_total_hp_plus_block, 1)",
      "override": "core_damage",
      "per_target": true,
      "conditions": [
        {"parsed": {"damage_gt": 0}},
        {"target": {"power": "Intangible"}}
      ]
    },
    {
      "rule_id": "core_damage",
      "priority": 1052,
      "weight": 1000,
      "applies_to": ["play_card"],
      "formula": "@damage * @hits * @weight / max(@monsters_total_hp_plus_block, 1)",
      "per_target": true,
      "conditions": [
        {"parsed": {"damage_gt": 0}},
        {"target": {"power_not": "Intangible"}}
      ]
    },

    {
      "rule_id": "core_block_retain",
      "priority": 1060,
      "weight": 1000,
      "applies_to": ["play_card", "use_potion"],
      "formula": "@block * @weight / max(@current_hp, 1)",
      "override": "core_block_non_excessive",
      "conditions": [
        {"parsed": {"block_gt": 0}},
        {"player": {"power": "Barricade"}}
      ]
    },
    {
      "rule_id": "core_block_retain_relic",
      "priority": 1061,
      "weight": 1000,
      "applies_to": ["play_card", "use_potion"],
      "formula": "@block * @weight / max(@current_hp, 1)",
      "override": "core_block_non_excessive",
      "conditions": [
        {"parsed": {"block_gt": 0}},
        {"player": {"relic": "Calipers"}}
      ]
    },
    {
      "rule_id": "core_block_excessive",
      "priority": 1062,
      "weight": -2000,
      "applies_to": ["play_card", "use_potion"],
      "formula": "@block * @weight / max(@current_hp, 1)",
      "override": "core_block_non_excessive",
      "conditions": [
        {"parsed": {"block_gt": 0}},
        {"compute": {"formula": "@incoming_damage <= @current_block"}},
        {"player": {"power_not": "Barricade"}},
        {"player": {"relic_not": "Calipers"}}
      ]
    },
    {
      "rule_id": "core_block_non_excessive",
      "priority": 1063,
      "weight": 1000,
      "applies_to": ["play_card", "use_potion"],
      "formula": "min(@block, @incoming_damage) * @weight / max(@current_hp, 1)",
      "conditions": [
        {"parsed": {"block_gt": 0}},
        {"compute": {"formula": "@incoming_damage > @current_block"}}
      ]
    },

    {
      "rule_id": "core_heal",
      "priority": 1070,
      "weight": 50,
      "applies_to": ["play_card", "use_potion"],
      "conditions": [
        {"parsed": {"heal_gt": 0}}
      ]
    },

    {
      "rule_id": "core_draw_no_energy_no_free",
      "priority": 1075,
      "weight": 0,
      "applies_to": ["play_card"],
      "formula": "@draw * @weight",
      "override": "core_draw_has_energy",
      "conditions": [
        {"parsed": {"draw_gt": 0}},
        {"compute": {"formula": "@remaining_energy == 0"}},
        {"compute": {"formula": "@free_count == 0"}}
      ]
    },
    {
      "rule_id": "core_draw_has_energy",
      "priority": 1076,
      "weight": 10,
      "applies_to": ["play_card"],
      "formula": "@draw * @weight",
      "conditions": [
        {"parsed": {"draw_gt": 0}},
        {"state": {"remaining_energy_gt": 0}}
      ]
    },

    {
      "rule_id": "target_scaling",
      "priority": 1030,
      "weight": 15,
      "applies_to": ["play_card"],
      "per_target": true,
      "conditions": [
        {"target": {"is_scaling": true}}
      ]
    },
    {
      "rule_id": "target_punish",
      "priority": 1031,
      "weight": 10,
      "applies_to": ["play_card"],
      "per_target": true,
      "conditions": [
        {"target": {"any_power_in": ["Thorns", "Curiosity", "Enrage"]}}
      ]
    },
    {
      "rule_id": "target_killable",
      "priority": 1032,
      "weight": 5,
      "applies_to": ["play_card"],
      "per_target": true,
      "conditions": [
        {"target": {"can_be_killed": true}}
      ]
    },

    {
      "rule_id": "setup_power_card",
      "priority": 1080,
      "weight": 30,
      "applies_to": ["play_card"],
      "conditions": [
        {"card": {"type": "POWER"}},
        {"monsters": {"none": {"monster_id": "AwakenedOne"}}}
      ]
    },
    {
      "rule_id": "setup_self_str_gain",
      "priority": 1081,
      "weight": 15,
      "applies_to": ["play_card"],
      "formula": "@str_gain * @weight",
      "conditions": [
        {"parsed": {"str_gain_gt": 0}}
      ]
    },
    {
      "rule_id": "setup_self_dex_gain",
      "priority": 1082,
      "weight": 15,
      "applies_to": ["play_card"],
      "formula": "@dex_gain * @weight",
      "conditions": [
        {"parsed": {"dex_gain_gt": 0}}
      ]
    },
    {
      "rule_id": "setup_poison",
      "priority": 1083,
      "weight": 10,
      "applies_to": ["play_card"],
      "formula": "@poison * @weight",
      "per_target": true,
      "conditions": [
        {"parsed": {"poison_gt": 0}}
      ]
    },
    {
      "rule_id": "setup_vulnerable_apply",
      "priority": 1084,
      "weight": 10,
      "applies_to": ["play_card"],
      "formula": "@vulnerable * @weight",
      "per_target": true,
      "conditions": [
        {"parsed": {"vulnerable_gt": 0}}
      ]
    },
    {
      "rule_id": "setup_weak_new",
      "priority": 1085,
      "score_fn": "weak_new_formula",
      "applies_to": ["play_card"],
      "per_target": true,
      "conditions": [
        {"parsed": {"weak_gt": 0}},
        {"target": {"power_not": "Weak"}}
      ]
    },
    {
      "rule_id": "setup_weak_refresh",
      "priority": 1086,
      "score_fn": "weak_refresh_formula",
      "applies_to": ["play_card"],
      "per_target": true,
      "conditions": [
        {"parsed": {"weak_gt": 0}},
        {"target": {"power": "Weak"}}
      ]
    },
    {
      "rule_id": "setup_energy_gain",
      "priority": 1087,
      "weight": 20,
      "applies_to": ["play_card"],
      "formula": "@energy_gain * @weight",
      "conditions": [
        {"parsed": {"energy_gain_gt": 0}}
      ]
    },
    {
      "rule_id": "setup_turn_one",
      "priority": 1088,
      "weight": 10,
      "applies_to": ["play_card"],
      "conditions": [
        {"state": {"turn_eq": 1}}
      ]
    },
    {
      "rule_id": "setup_zero_cost",
      "priority": 1089,
      "weight": 5,
      "applies_to": ["play_card"],
      "conditions": [
        {"card": {"cost_eq": 0}}
      ]
    },
    {
      "rule_id": "setup_ethereal",
      "priority": 1090,
      "weight": 5,
      "applies_to": ["play_card"],
      "conditions": [
        {"card": {"ethereal": true}}
      ]
    },

    {
      "rule_id": "utility_exhaust_status_curse",
      "priority": 1140,
      "weight": 50,
      "applies_to": ["play_card"],
      "formula": "@weight - @card_base_score",
      "conditions": [
        {"parsed": {"exhausts_status_curse": true}}
      ]
    },

    {
      "rule_id": "punishment_skill_vs_anger",
      "priority": 1160,
      "weight": -100,
      "applies_to": ["play_card"],
      "conditions": [
        {"monsters": {"any": {"power": "Anger"}}},
        {"card": {"type": "SKILL"}}
      ]
    },
    {
      "rule_id": "punishment_power_vs_curiosity",
      "priority": 1161,
      "weight": -100,
      "applies_to": ["play_card"],
      "conditions": [
        {"monsters": {"any": {"power": "Curiosity"}}},
        {"card": {"type": "POWER"}}
      ]
    },
    {
      "rule_id": "danger_retaliatory_damage",
      "priority": 1162,
      "weight": 1,
      "score_fn": "retaliation_damage_penalty",
      "applies_to": ["play_card"],
      "conditions": [
        {"card": {"type": "ATTACK"}},
        {"compute": {"formula": "@retaliatory_damage > 0"}}
      ]
    },

    {
      "rule_id": "danger_lethal_block_saved",
      "priority": 1170,
      "weight": 200,
      "applies_to": ["play_card"],
      "conditions": [
        {"compute": {"formula": "@incoming_lethal == true"}},
        {"compute": {"formula": "@current_hp + @current_block + @block >= @incoming_damage"}}
      ]
    },
    {
      "rule_id": "danger_lethal_block",
      "priority": 1171,
      "weight": 50,
      "applies_to": ["play_card"],
      "conditions": [
        {"compute": {"formula": "@incoming_lethal == true"}},
        {"parsed": {"block_gt": 0}}
      ]
    },
    {
      "rule_id": "danger_lethal_damage",
      "priority": 1172,
      "weight": 20,
      "applies_to": ["play_card"],
      "conditions": [
        {"compute": {"formula": "@incoming_lethal == true"}},
        {"parsed": {"damage_gt": 0}}
      ]
    },
    {
      "rule_id": "danger_lethal_zero_impact",
      "priority": 1173,
      "weight": -50,
      "applies_to": ["play_card"],
      "conditions": [
        {"compute": {"formula": "@incoming_lethal == true"}},
        {"parsed": {"damage_eq": 0}},
        {"parsed": {"block_eq": 0}}
      ]
    },
    {
      "rule_id": "danger_hp_cost",
      "priority": 1174,
      "score_fn": "hp_cost_penalty",
      "applies_to": ["play_card"],
      "conditions": [
        {"parsed": {"self_damage_gt": 0}}
      ]
    },

    {
      "rule_id": "manual_filter_limit_break",
      "priority": 1020,
      "weight": "i64::MIN",
      "applies_to": ["play_card"],
      "conditions": [
        {"card": {"id": "Limit Break"}},
        {"compute": {"formula": "@player_str_amount <= 0"}}
      ]
    },
    {
      "rule_id": "manual_filter_spot_weakness",
      "priority": 1021,
      "weight": "i64::MIN",
      "applies_to": ["play_card"],
      "conditions": [
        {"card": {"id": "Spot Weakness"}},
        {"target": {"intent_not": "ATTACK"}}
      ]
    },
    {
      "rule_id": "manual_filter_blasphemy",
      "priority": 1022,
      "weight": "i64::MIN",
      "applies_to": ["play_card"],
      "conditions": [
        {"card": {"id_in": ["Blasphemy", "渎神"]}},
        {"player": {"stance_not": "Divinity"}}
      ]
    },

    {
      "rule_id": "end_turn_plays_remain",
      "priority": 2000,
      "weight": 0,
      "applies_to": ["end_turn"],
      "conditions": [
        {"compute": {"formula": "@useful_cards_in_hand > 0"}}
      ]
    },
    {
      "rule_id": "end_turn_hand_exhausted",
      "priority": 2001,
      "weight": 80,
      "applies_to": ["end_turn"],
      "conditions": [
        {"compute": {"formula": "@useful_cards_in_hand == 0"}}
      ]
    },

    {
      "rule_id": "potion_base",
      "priority": 1000,
      "weight": -20,
      "applies_to": ["use_potion"]
    },

    {
      "rule_id": "orb_channel_lightning",
      "priority": 1120,
      "weight": 10,
      "applies_to": ["play_card"],
      "formula": "@weight * (@focus + 3)",
      "conditions": [
        {"parsed": {"channel_orb": "Lightning"}}
      ]
    },
    {
      "rule_id": "orb_channel_frost",
      "priority": 1121,
      "weight": 50,
      "applies_to": ["play_card"],
      "conditions": [
        {"parsed": {"channel_orb": "Frost"}},
        {"state": {"incoming_damage_gt": 0}}
      ]
    },
    {
      "rule_id": "orb_channel_dark",
      "priority": 1122,
      "weight": 10,
      "applies_to": ["play_card"],
      "conditions": [
        {"parsed": {"channel_orb": "Dark"}}
      ]
    },
    {
      "rule_id": "orb_channel_plasma",
      "priority": 1123,
      "weight": 20,
      "applies_to": ["play_card"],
      "conditions": [
        {"parsed": {"channel_orb": "Plasma"}}
      ]
    },
    {
      "rule_id": "orb_slot_expand",
      "priority": 1124,
      "weight": 100,
      "applies_to": ["play_card"],
      "formula": "@weight * @expand_count",
      "conditions": [
        {"parsed": {"orb_slot_expand_gt": 0}}
      ]
    },
    {
      "rule_id": "orb_focus_gain",
      "priority": 1125,
      "weight": 20,
      "applies_to": ["play_card"],
      "formula": "@weight * @focus_gain",
      "conditions": [
        {"parsed": {"focus_gain_gt": 0}}
      ]
    },

    {
      "rule_id": "stance_calm_exit",
      "priority": 1100,
      "weight": 40,
      "applies_to": ["play_card"],
      "conditions": [
        {"player": {"stance": "Calm"}},
        {"parsed": {"exits_stance": true}}
      ]
    },
    {
      "rule_id": "stance_wrath_retreat_block",
      "priority": 1101,
      "weight": 10,
      "applies_to": ["play_card"],
      "formula": "@incoming_damage * 0.5 * @weight",
      "conditions": [
        {"player": {"stance": "Wrath"}},
        {"parsed": {"exits_stance": true}},
        {"state": {"incoming_damage_gt": 0}}
      ]
    },

    {
      "rule_id": "exhaust_synergy_dark_embrace",
      "priority": 1150,
      "weight": 10,
      "applies_to": ["play_card"],
      "formula": "@weight * @exhaust_count",
      "conditions": [
        {"parsed": {"exhausts_cards": true}},
        {"player": {"power": "Dark Embrace"}}
      ]
    },
    {
      "rule_id": "exhaust_synergy_charons_ashes",
      "priority": 1151,
      "weight": 30,
      "applies_to": ["play_card"],
      "formula": "@weight * @monster_count * @exhaust_count",
      "conditions": [
        {"parsed": {"exhausts_cards": true}},
        {"player": {"relic": "Charon's Ashes"}}
      ]
    }
  ]
}
```

### Field reference

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `rule_id` | string | yes | Unique identifier |
| `priority` | int | yes | Evaluation order (ascending; lower = first). Override rules get lower priority than the rules they override |
| `weight` | i64 or `"i64::MIN"` | yes | Flat constant, or coefficient in formula |
| `formula` | string | no | Expression with `@vars`, weight via `@weight` |
| `score_fn` | string | no | Rust function dispatch (mutually exclusive with formula). Must match an entry in `available_score_fns` |
| `override` | string | no | `rule_id` to suppress when this rule matches |
| `applies_to` | [string] | yes | Action types: `play_card`, `use_potion`, `end_turn` |
| `per_target` | bool | no | Iterate per-target; sum scores together |
| `conditions` | [Condition] | yes | All must be true (implicit AND) |

---

## 6. Condition Vocabulary

Each condition is a single-scope object. The key names the scope; the value
specifies one or more predicates within that scope.

| Scope | Predicate | Value type | Example |
|-------|-----------|------------|---------|
| `card` | `type` | string | `{"card": {"type": "SKILL"}}` |
| | `type_in` | [string] | `{"card": {"type_in": ["STATUS", "CURSE"]}}` |
| | `type_not_in` | [string] | `{"card": {"type_not_in": ["STATUS", "CURSE"]}}` |
| | `cost_eq` | int | `{"card": {"cost_eq": -1}}` |
| | `ethereal` | bool | `{"card": {"ethereal": true}}` |
| | `id` | string | `{"card": {"id": "Limit Break"}}` |
| | `id_in` | [string] | `{"card": {"id_in": ["Blasphemy", "渎神"]}}` |
| `parsed` | `damage_gt` | int | `{"parsed": {"damage_gt": 0}}` |
| | `damage_eq` | int | `{"parsed": {"damage_eq": 0}}` |
| | `block_gt` | int | `{"parsed": {"block_gt": 0}}` |
| | `block_eq` | int | `{"parsed": {"block_eq": 0}}` |
| | `heal_gt` | int | `{"parsed": {"heal_gt": 0}}` |
| | `draw_gt` | int | `{"parsed": {"draw_gt": 0}}` |
| | `self_damage_gt` | int | `{"parsed": {"self_damage_gt": 0}}` |
| | `str_gain_gt` | int | `{"parsed": {"str_gain_gt": 0}}` |
| | `dex_gain_gt` | int | `{"parsed": {"dex_gain_gt": 0}}` |
| | `poison_gt` | int | `{"parsed": {"poison_gt": 0}}` |
| | `vulnerable_gt` | int | `{"parsed": {"vulnerable_gt": 0}}` |
| | `weak_gt` | int | `{"parsed": {"weak_gt": 0}}` |
| | `energy_gain_gt` | int | `{"parsed": {"energy_gain_gt": 0}}` |
| | `focus_gain_gt` | int | `{"parsed": {"focus_gain_gt": 0}}` |
| | `str_loss_gt` | int | `{"parsed": {"str_loss_gt": 0}}` |
| | `str_loss_temp` | bool | `{"parsed": {"str_loss_temp": true}}` |
| | `mantra_gt` | int | `{"parsed": {"mantra_gt": 0}}` |
| | `exhausts_cards` | bool | `{"parsed": {"exhausts_cards": true}}` |
| | `exhausts_status_curse` | bool | `{"parsed": {"exhausts_status_curse": true}}` |
| | `channel_orb` | string | `{"parsed": {"channel_orb": "Lightning"}}` |
| | `orb_slot_expand_gt` | int | `{"parsed": {"orb_slot_expand_gt": 0}}` |
| | `exits_stance` | bool | `{"parsed": {"exits_stance": true}}` |
| `target` | `power` | string | `{"target": {"power": "Vulnerable"}}` |
| | `power_not` | string | `{"target": {"power_not": "Intangible"}}` |
| | `any_power_in` | [string] | `{"target": {"any_power_in": ["Thorns", "Curiosity", "Enrage"]}}` |
| | `is_scaling` | bool | `{"target": {"is_scaling": true}}` |
| | `is_minion` | bool | `{"target": {"is_minion": true}}` |
| | `is_max_hp` | bool | `{"target": {"is_max_hp": true}}` |
| | `can_be_killed` | bool | `{"target": {"can_be_killed": true}}` |
| | `intent` | string | `{"target": {"intent": "ATTACK"}}` |
| | `intent_not` | string | `{"target": {"intent_not": "ATTACK"}}` |
| `monsters` | `any` | Predicate | Any monster matches this predicate |
| | `none` | Predicate | No monster matches this predicate |
| | (within any/none) `monster_id` | string | `{"monster_id": "GremlinNob"}` |
| | (within any/none) `is_scaling` | bool | |
| | (within any/none) `is_not_minion` | bool | |
| | (within any/none) `exclude_target` | bool | Exclude the current action's target |
| `player` | `power` | string | `{"player": {"power": "Barricade"}}` |
| | `power_not` | string | `{"player": {"power_not": "Barricade"}}` |
| | `relic` | string | `{"player": {"relic": "Calipers"}}` |
| | `relic_not` | string | `{"player": {"relic_not": "Calipers"}}` |
| | `stance` | string | `{"player": {"stance": "Wrath"}}` |
| | `stance_not` | string | `{"player": {"stance_not": "Divinity"}}` |
| `state` | `turn_eq` | int | `{"state": {"turn_eq": 1}}` |
| | `remaining_energy_gt` | int | `{"state": {"remaining_energy_gt": 0}}` |
| | `incoming_damage_gt` | int | `{"state": {"incoming_damage_gt": 0}}` |
| `compute` | `formula` | string | `{"compute": {"formula": "@incoming_damage > @current_block"}}` |

---

## 7. `@variables` Reference

All fields available in formulas (`formula` and `compute→formula`).

### Parsed effects (from description parser)

| Variable | Type | Source |
|----------|------|--------|
| `@damage` | i64 | Parsed damage per hit |
| `@hits` | i64 | Parsed hit count (default 1) |
| `@block` | i64 | Parsed block value |
| `@heal` | i64 | Parsed heal amount |
| `@draw` | i64 | Parsed cards drawn |
| `@self_damage` | i64 | Parsed HP loss |
| `@str_gain` | i64 | Parsed self-STR gain |
| `@dex_gain` | i64 | Parsed self-DEX gain |
| `@poison` | i64 | Parsed poison amount |
| `@vulnerable` | i64 | Parsed vulnerable amount |
| `@weak` | i64 | Parsed weak amount |
| `@energy_gain` | i64 | Parsed energy gain (from `[E]`) |
| `@str_loss` | i64 | Parsed enemy STR loss |
| `@focus_gain` | i64 | Parsed focus gain |
| `@expand_count` | i64 | Parsed orb slot expand count |
| `@exhaust_count` | i64 | Cards exhausted by this action |
| `@card_base_score` | i64 | Pre-computed base score (for exhaust formula) |

### State values (resolved from combat state)

| Variable | Type | Source |
|----------|------|--------|
| `@cost` | i64 | Card cost (post X-cost resolution) |
| `@current_energy` | i64 | Energy before playing the card |
| `@remaining_energy` | i64 | Energy after playing the card |
| `@current_block` | i64 | Player block before the action |
| `@current_hp` | i64 | Player current HP |
| `@incoming_damage` | i64 | Total incoming monster damage this turn: the saturating sum of each positive per-hit `move_adjusted_damage * move_hits` (`move_hits` defaults to 1 when absent; non-positive damage and `NONE` intent are excluded) |
| `@incoming_lethal` | bool | Would incoming damage kill the player? |
| `@total_damage` | i64 | Sum of this action's damage across all targets |
| `@monsters_total_hp_plus_block` | i64 | Sum of all alive monster HP + block |
| `@monster_count` | i64 | Number of alive monsters |
| `@useful_cards_in_hand` | i64 | Count of playable non-status non-curse cards |
| `@free_count` | i64 | Count of 0-cost non-status non-curse cards in draw/discard |
| `@pile_size` | i64 | Size of draw pile (or discard if draw empty) |
| `@thorns` | i64 | Thorns amount on current target |
| `@retaliatory_damage` | i64 | Target's retaliation damage from Thorns or Sharp Hide before Block |
| `@focus` | i64 | Player focus amount (from Focus power) |
| `@player_str_amount` | i64 | Player Strength power amount |

### Rule reference

| Variable | Type | Description |
|----------|------|-------------|
| `@weight` | i64 | The rule's own weight field |

---

## 8. Override Mechanism

### Semantics

When a rule with `override: "target_rule_id"` matches, the target rule is
suppressed — its score contribution is excluded from the total. The overriding
rule's own score is added normally.

Multiple rules can target the same base rule (e.g. `core_block_retain` and
`core_block_excessive` both override `core_block_non_excessive`). All matching
overrides contribute their scores; the base is simply skipped.

Evaluation order: rules are processed in **ascending** `priority` order. Override
rules must carry a lower priority number than the base rule they suppress, so
they are evaluated first. A rule evaluates
its conditions. If matched and it carries `override`, it marks the target rule
as suppressed. When the engine later encounters the suppressed rule, it skips
score computation but still records it in the breakdown as `matched: false`.

If two rules try to override the same target, both match independently — they
both contribute scores, and the base is suppressed once.

### When NOT to use override

- Non-conflicting rules (e.g. two independent damage bonuses) — just let both
  add normally. No override needed.
- Rules that always apply together — split into separate rules, each with its
  own conditions, no override.

---

## 9. `score_fn` Registry

Complex formulas not expressible with `@vars` + arithmetic dispatch to Rust
functions via the `score_fn` field. The top-level `available_score_fns` array
declares all valid identifiers. At load time, if a rule references a `score_fn`
not in that list, the engine emits a `warn!` log and discards the rule.

### Required functions

#### `hp_cost_penalty`

```
score = -1_050_000 * @self_damage / max(1, @current_hp)^3
```

At ~45 HP, Offering's cost (+70 energy/draw) is exactly cancelled.
At ~5 HP, penalty approaches `i64::MIN`.

#### `weak_new_formula`

```
score = (5 + target_total_damage * 2.5) * @weak
  where target_total_damage = monster.damage * monster.hits
```

Applies when no existing Weak power on target.

#### `weak_refresh_formula`

```
score = 5 * @weak
```

Applies when target already has Weak power.

#### `feel_no_pain_block`

Computes effective block from Feel No Pain power × exhaust_count. This block
feeds into the Section 3d block rules (non-excessive/excessive). Implemented as
a context pre-computation rather than a separate rule — the context builder adds
FNP block to `@block` when applicable.

---

## 10. Context Building

`context.rs` transforms `CombatState` into one or more `ActionContext`
instances.

### Input

- `CombatState` (from `src/state.rs`): hand, draw pile, discard pile, energy,
  player HP/block/powers/relics/orbs/stance, monsters, turn number.
- List of available potions.

### Output per candidate

```rust
struct ActionContext {
    action_type: ActionType,         // PlayCard, UsePotion, EndTurn
    card: Option<CardRef>,           // The card being played (if any)
    potion: Option<PotionRef>,       // The potion being used (if any)
    target_index: Option<usize>,     // Command target index (if targeted)
    target: Option<TargetSnapshot>,  // The targeted monster's state
    parsed: ParsedEffects,           // All extracted numbers and flags
    vars: HashMap<String, Value>,    // @variable → value table
}
```

### Build steps

1. **Parse description** — call `parser.rs` on the card/potion description.
   Extract all numeric fields and effect flags.
2. **Resolve X-cost** — if `card.cost == -1`:
   - `@cost = @current_energy` (the card spends all current energy)
   - If ATTACK: `@hits = @current_energy + chemical_x_bonus`
   - If non-ATTACK: use `@current_energy + chemical_x_bonus` as the effect
     amount (replacing parsed 0-fallback).
3. **Resolve state @vars** — populate the `vars` table from combat state.
4. **Resolve computed @vars** — compute derived fields like
   `@incoming_lethal`, `@total_damage`, `@remaining_energy`, etc.
5. **Emit ActionContext** — one per (action × target) combination.

---

## 11. Description Parser

`parser.rs` implements Section 16 of `ranker-rules.md`. It takes a card or
potion description string and produces `ParsedEffects`.

### Design

A two-pass approach:
1. **Locale-aware regex** — try the current locale's patterns first, fall back
   to English.
2. **Extract all numbers** — each effect has one primary numeric value.
   Multi-hit additionally extracts hit count. Energy counts `[E]` occurrences.

### Extracted fields

```rust
struct ParsedEffects {
    damage: Option<i64>,           // damage per hit
    hits: i64,                     // hit count (default 1)
    block: Option<i64>,
    heal: Option<i64>,
    draw: Option<i64>,
    self_damage: Option<i64>,
    str_gain: Option<i64>,
    dex_gain: Option<i64>,
    poison: Option<i64>,
    vulnerable: Option<i64>,
    weak: Option<i64>,
    energy_gain: Option<i64>,
    str_loss: Option<i64>,
    str_loss_temp: bool,
    mantra: Option<i64>,
    focus_gain: Option<i64>,
    exhaust_count: i64,
    exhausts_status_curse: bool,
    ethereal: bool,
    channel_orb: Option<String>,
    evoke_orb: Option<String>,
    orb_slot_expand: i64,
    exits_stance: bool,
    enters_wrath: bool,
    enters_calm: bool,
}
```

---

## 12. Formula Evaluator

`formula.rs` parses and evaluates expressions of the form:

```
"@block * @weight / max(@current_hp, 1)"
"@damage * @hits * @weight / max(@monsters_total_hp_plus_block, 1)"
"min(@block, @incoming_damage) * @weight / max(@current_hp, 1)"
```

### Grammar (informal)

```
expr     → term | term ("+" | "-") expr
term     → factor | factor ("*" | "/") term
factor   → NUMBER | "@" IDENT | FUNCTION "(" expr ("," expr)* ")" | "(" expr ")"
IDENT    → [a-z_]+
FUNCTION → "max" | "min"
NUMBER   → [0-9]+ ( "." [0-9]+ )?
```

### Evaluation

All operations use `f64`. The final result is cast to `i64` with saturating
clamping to avoid overflow.

- `@IDENT` resolves against the `ActionContext.vars` table. Missing variables
  evaluate to `0` (the effect is not present) and emit a `warn!` log.
- `NUMBER` literals are inline coefficients (e.g. `1.5` for Vulnerable
  multiplier).
- `max(a, b)` and `min(a, b)` are the only supported functions.

### Compute formulas (conditions)

For `compute: { formula: "..." }` in conditions, the same evaluator runs.
The result (as `f64`) is compared to `0.0`:
- `> 0.0` → condition true
- `<= 0.0` → condition false

Expressions in compute formulas return the comparison result directly
(e.g. `@incoming_damage > @current_block` evaluates to `1.0` if true,
`0.0` if false).

---

## 13. Integration Points

### Kill Scanner

Not integrated in v1. The `combat_ends_fight` rule uses a simple `compute`
formula comparing visible damage vs. total monster HP+block. When the kill
scanner is ready, replace that rule with one that calls
`find_kill_sequence_from_context` via a `score_fn` dispatch.

### Autoplay

`top_ranked_context_with_refs()` in `src/autoplay/combat_adviser.rs` calls
`rank()` and returns a flat array of all non-avoided actions with the same
prompt-scoped refs used by `available_actions`, plus `score` and `tags`
(derived from matched rule IDs: `damage`, `block`, `excessive`, `killable`,
`lethal`, `priority_kill`, `heal`, `power`, `draw`, `potion`). Avoided actions
(`is_avoid == true`) and suggestions that cannot be mapped unambiguously to an
available action are excluded.

### Prompt Builder

The LLM prompt receives `ranked_suggestions` as a flat array of
`{ref, label, target, score, tags}` objects. Execution-local UUIDs and internal
action IDs are not exposed through ranker suggestions. There is no rule
breakdown or suggested/other/avoided split.

---

## 14. Implementation Order

1. Create `src/ranker/mod.rs` skeleton, wire into `main.rs`.
2. Implement `rules.rs` — serde types for `Rule`, `Condition`, `RuleSet`.
3. Implement `formula.rs` — expression parser and evaluator.
4. Implement `parser.rs` — zh/en description parsing → `ParsedEffects`.
5. Implement `context.rs` — `ActionContext` builder with `@var` resolution.
6. Implement `predicates.rs` — `score_fn` registry with `hp_cost_penalty`,
   `weak_new_formula`, `weak_refresh_formula`.
7. Implement `engine.rs` — evaluate loop, override suppression, sorting,
   Avoid list.
8. Author `rules.json` — translate all 18 sections from `ranker-rules.md`.
9. Write `src/tests/ranker_tests.rs` — rule-by-rule scoring tests using
   fixture game states.

---

## 15. Test Strategy

### Unit tests per component

- `formula.rs`: test expression parsing and evaluation with various `@var`
  tables.
- `parser.rs`: test each zh/en pattern against known card descriptions.
- `context.rs`: test context building from fixture states.
- `engine.rs`: test override suppression, per-target summation, Avoid list.

### Integration tests

- Load `rules.json`, feed fixture combat states, verify that known-good
  actions score highest and known-bad actions score lowest.
- Test edge cases: X-cost cards, random-target vs. single target, damage to
  Intangible target, block with/without retain, lethal-danger scenarios.
- Test manual filters produce `i64::MIN`.
