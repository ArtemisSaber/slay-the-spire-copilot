export type TargetType = "enemy" | "allEnemies" | "randomEnemy" | "none";
export type CardType = "ATTACK" | "SKILL" | "POWER" | "STATUS" | "CURSE";
export type Stance = "Neutral" | "Calm" | "Wrath" | "Divinity";

export type CardEffect =
  | { kind: "vulnerable"; amount: number }
  | { kind: "strength"; amount: number }
  | { kind: "energy"; amount: number }
  | { kind: "enterStance"; stance: Stance }
  | { kind: "exitStance" }
  | { kind: "mantra"; amount: number }
  | { kind: "xCostAttack"; damagePerX: number };

export type Cards = {
  uuid: string;
  name: string;
  cost: number | "X";
  type: CardType;
  damage?: number;
  hits?: number;
  targetType: TargetType;
  effects?: CardEffect[];
};

export type PowerStatus = {
  id: string;
  amount?: number;
};

export type RelicStatus = {
  id: string;
  counter?: number;
};

export type Monster = {
  index: number;
  hp: number;
  block: number;
  isMinion: boolean;
  powers: PowerStatus[];
};

export type PlayerStatus = {
  stance: Stance;
  strength: number;
  mantra: number;
  powers: PowerStatus[];
  relics: RelicStatus[];
  cardsPlayedThisTurn: number;
  playLimitRemaining?: number;
  xCostBonus?: number;
};

export type KillScanCase = {
  caseName: string;
  caseReason: string;
  expectedDeterministicKill: boolean;
  handDeck: Cards[];
  energy: number;
  monsters: Monster[];
  currentPlayerStatus: PlayerStatus;
};

export const killScanCases: KillScanCase[] = [
  {
    caseName: "curl_up_lethal_single_hit",
    caseReason:
      "Curl Up should not prevent a hit that kills immediately. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "curl-lethal-attack",
        name: "Big Strike",
        cost: 1,
        type: "ATTACK",
        damage: 12,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 1,
    monsters: [
      {
        index: 0,
        hp: 12,
        block: 0,
        isMinion: false,
        powers: [{ id: "Curl Up", amount: 6 }],
      },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "curl_up_nonlethal_blocks_later_card",
    caseReason:
      "A nonlethal attack triggers Curl Up block, so the later attack no longer kills. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "curl-first-strike",
        name: "Strike A",
        cost: 1,
        type: "ATTACK",
        damage: 5,
        hits: 1,
        targetType: "enemy",
      },
      {
        uuid: "curl-second-strike",
        name: "Strike B",
        cost: 1,
        type: "ATTACK",
        damage: 5,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 2,
    monsters: [
      {
        index: 0,
        hp: 8,
        block: 0,
        isMinion: false,
        powers: [{ id: "Curl Up", amount: 6 }],
      },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "curl_up_multihit_same_card_kills_before_block_matters",
    caseReason:
      "Curl Up block is gained after the attack's damage resolves, so one 2x5 multi-hit card kills 9 hp before the gained block can affect later cards. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "curl-multihit-attack",
        name: "Five Cuts",
        cost: 1,
        type: "ATTACK",
        damage: 2,
        hits: 5,
        targetType: "enemy",
      },
    ],
    energy: 1,
    monsters: [
      {
        index: 0,
        hp: 9,
        block: 0,
        isMinion: false,
        powers: [{ id: "Curl Up", amount: 5 }],
      },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "artifact_blocks_vulnerable_setup",
    caseReason:
      "Artifact consumes Vulnerable, so Bash plus Strike is only 14 damage. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "artifact-bash",
        name: "Bash",
        cost: 2,
        type: "ATTACK",
        damage: 8,
        hits: 1,
        targetType: "enemy",
        effects: [{ kind: "vulnerable", amount: 2 }],
      },
      {
        uuid: "artifact-strike",
        name: "Strike",
        cost: 1,
        type: "ATTACK",
        damage: 6,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 3,
    monsters: [
      {
        index: 0,
        hp: 17,
        block: 0,
        isMinion: false,
        powers: [{ id: "Artifact", amount: 1 }],
      },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "bash_vulnerable_does_not_buff_self",
    caseReason:
      "Vulnerable from Bash applies after Bash damage, not to Bash itself. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "bash-self-vulnerable",
        name: "Bash",
        cost: 2,
        type: "ATTACK",
        damage: 8,
        hits: 1,
        targetType: "enemy",
        effects: [{ kind: "vulnerable", amount: 2 }],
      },
    ],
    energy: 2,
    monsters: [
      { index: 0, hp: 12, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "vulnerable_setup_enables_later_attack",
    caseReason:
      "A debuff-only setup card should be kept when it makes later attack damage lethal. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "apply-vulnerable",
        name: "Trip",
        cost: 0,
        type: "SKILL",
        targetType: "enemy",
        effects: [{ kind: "vulnerable", amount: 2 }],
      },
      {
        uuid: "vulnerable-strike",
        name: "Heavy Strike",
        cost: 1,
        type: "ATTACK",
        damage: 10,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 1,
    monsters: [
      { index: 0, hp: 15, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "malleable_blocks_second_attack",
    caseReason:
      "Malleable adds block after unblocked attack damage, so two 6 damage attacks do not kill 12 hp. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "malleable-strike-a",
        name: "Strike A",
        cost: 1,
        type: "ATTACK",
        damage: 6,
        hits: 1,
        targetType: "enemy",
      },
      {
        uuid: "malleable-strike-b",
        name: "Strike B",
        cost: 1,
        type: "ATTACK",
        damage: 6,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 2,
    monsters: [
      {
        index: 0,
        hp: 12,
        block: 0,
        isMinion: false,
        powers: [{ id: "Malleable", amount: 3 }],
      },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "intangible_caps_single_hit",
    caseReason:
      "Intangible caps positive attack damage to 1, so one large hit cannot kill 3 hp. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "intangible-big-hit",
        name: "Big Strike",
        cost: 1,
        type: "ATTACK",
        damage: 20,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 1,
    monsters: [
      {
        index: 0,
        hp: 3,
        block: 0,
        isMinion: false,
        powers: [{ id: "Intangible", amount: 1 }],
      },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "flight_halves_attack_damage",
    caseReason:
      "Flight halves attack damage while active, so 10 displayed attack damage does not kill 10 hp. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "flight-strike",
        name: "Strike",
        cost: 1,
        type: "ATTACK",
        damage: 10,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 1,
    monsters: [
      {
        index: 0,
        hp: 10,
        block: 0,
        isMinion: false,
        powers: [{ id: "Flight", amount: 3 }],
      },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "invincible_caps_turn_damage",
    caseReason:
      "Invincible caps total hp loss this turn, so repeated large hits cannot kill through the cap. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "invincible-hit-a",
        name: "Heavy Strike A",
        cost: 1,
        type: "ATTACK",
        damage: 40,
        hits: 1,
        targetType: "enemy",
      },
      {
        uuid: "invincible-hit-b",
        name: "Heavy Strike B",
        cost: 1,
        type: "ATTACK",
        damage: 40,
        hits: 1,
        targetType: "enemy",
      },
      {
        uuid: "invincible-hit-c",
        name: "Heavy Strike C",
        cost: 1,
        type: "ATTACK",
        damage: 40,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 3,
    monsters: [
      {
        index: 0,
        hp: 100,
        block: 0,
        isMinion: false,
        powers: [{ id: "Invincible", amount: 30 }],
      },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "play_limit_prevents_many_small_attacks",
    caseReason:
      "A known remaining-play limit must cap the sequence, so four 3 damage cards cannot all be used. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "limit-strike-1",
        name: "Strike 1",
        cost: 0,
        type: "ATTACK",
        damage: 3,
        hits: 1,
        targetType: "enemy",
      },
      {
        uuid: "limit-strike-2",
        name: "Strike 2",
        cost: 0,
        type: "ATTACK",
        damage: 3,
        hits: 1,
        targetType: "enemy",
      },
      {
        uuid: "limit-strike-3",
        name: "Strike 3",
        cost: 0,
        type: "ATTACK",
        damage: 3,
        hits: 1,
        targetType: "enemy",
      },
      {
        uuid: "limit-strike-4",
        name: "Strike 4",
        cost: 0,
        type: "ATTACK",
        damage: 3,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 0,
    monsters: [
      { index: 0, hp: 10, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
      playLimitRemaining: 3,
    },
  },
  {
    caseName: "random_target_guaranteed_two_enemies",
    caseReason:
      "Three random hits of 4 are guaranteed to kill two ordinary enemies with total durability 8. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "random-guaranteed",
        name: "Sword Boomerang",
        cost: 1,
        type: "ATTACK",
        damage: 4,
        hits: 3,
        targetType: "randomEnemy",
      },
    ],
    energy: 1,
    monsters: [
      { index: 0, hp: 4, block: 0, isMinion: false, powers: [] },
      { index: 1, hp: 4, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "random_target_not_guaranteed_two_enemies",
    caseReason:
      "Three random hits of 4 are not guaranteed against two ordinary enemies with total durability 10. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "random-not-guaranteed",
        name: "Sword Boomerang",
        cost: 1,
        type: "ATTACK",
        damage: 4,
        hits: 3,
        targetType: "randomEnemy",
      },
    ],
    energy: 1,
    monsters: [
      { index: 0, hp: 5, block: 0, isMinion: false, powers: [] },
      { index: 1, hp: 5, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "random_target_guaranteed_after_setup",
    caseReason:
      "The random attack is not guaranteed at root, but becomes guaranteed after targeted setup damage. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "random-setup-strike",
        name: "Strike",
        cost: 1,
        type: "ATTACK",
        damage: 6,
        hits: 1,
        targetType: "enemy",
      },
      {
        uuid: "random-after-setup",
        name: "Sword Boomerang",
        cost: 1,
        type: "ATTACK",
        damage: 4,
        hits: 3,
        targetType: "randomEnemy",
      },
    ],
    energy: 2,
    monsters: [
      { index: 0, hp: 10, block: 0, isMinion: false, powers: [] },
      { index: 1, hp: 4, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "energy_gain_enables_big_attack",
    caseReason:
      "Energy-only setup should be kept when it enables the lethal attack. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "energy-setup",
        name: "Seeing Red",
        cost: 0,
        type: "SKILL",
        targetType: "none",
        effects: [{ kind: "energy", amount: 2 }],
      },
      {
        uuid: "energy-big-attack",
        name: "Big Attack",
        cost: 2,
        type: "ATTACK",
        damage: 18,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 1,
    monsters: [
      { index: 0, hp: 18, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "strength_setup_enables_multihit",
    caseReason:
      "Strength-only setup should be kept; +2 Strength makes Twin Strike deal 14 total. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "strength-setup",
        name: "Inflame",
        cost: 1,
        type: "POWER",
        targetType: "none",
        effects: [{ kind: "strength", amount: 2 }],
      },
      {
        uuid: "strength-twin-strike",
        name: "Twin Strike",
        cost: 1,
        type: "ATTACK",
        damage: 5,
        hits: 2,
        targetType: "enemy",
      },
    ],
    energy: 2,
    monsters: [
      { index: 0, hp: 14, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "calm_exit_energy_enables_big_attack",
    caseReason:
      "Leaving Calm grants 2 energy, enabling the second card. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "calm-exit",
        name: "Empty Fist",
        cost: 1,
        type: "ATTACK",
        damage: 9,
        hits: 1,
        targetType: "enemy",
        effects: [{ kind: "exitStance" }],
      },
      {
        uuid: "calm-exit-big-attack",
        name: "Big Attack",
        cost: 2,
        type: "ATTACK",
        damage: 20,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 1,
    monsters: [
      { index: 0, hp: 29, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Calm",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "calm_to_wrath_energy_and_damage_enable_kill",
    caseReason:
      "Entering Wrath from Calm grants Calm exit energy, and later attacks use Wrath damage. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "calm-wrath-eruption",
        name: "Eruption",
        cost: 2,
        type: "ATTACK",
        damage: 9,
        hits: 1,
        targetType: "enemy",
        effects: [{ kind: "enterStance", stance: "Wrath" }],
      },
      {
        uuid: "calm-wrath-strike",
        name: "Strike",
        cost: 1,
        type: "ATTACK",
        damage: 6,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 2,
    monsters: [
      { index: 0, hp: 21, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Calm",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "neutral_to_wrath_damage_enables_kill",
    caseReason:
      "Entering Wrath from Neutral gives no Calm energy, but later attacks should use Wrath damage. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "neutral-wrath-eruption",
        name: "Eruption",
        cost: 2,
        type: "ATTACK",
        damage: 9,
        hits: 1,
        targetType: "enemy",
        effects: [{ kind: "enterStance", stance: "Wrath" }],
      },
      {
        uuid: "neutral-wrath-strike",
        name: "Strike",
        cost: 1,
        type: "ATTACK",
        damage: 6,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 3,
    monsters: [
      { index: 0, hp: 21, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "neutral_to_wrath_still_not_enough_damage",
    caseReason:
      "Eruption damage is not retroactively buffed by Wrath; 9 plus Wrath Strike 12 leaves 22 hp alive. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "neutral-wrath-negative-eruption",
        name: "Eruption",
        cost: 2,
        type: "ATTACK",
        damage: 9,
        hits: 1,
        targetType: "enemy",
        effects: [{ kind: "enterStance", stance: "Wrath" }],
      },
      {
        uuid: "neutral-wrath-negative-strike",
        name: "Strike",
        cost: 1,
        type: "ATTACK",
        damage: 6,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 3,
    monsters: [
      { index: 0, hp: 22, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "neutral_to_divinity_damage_and_energy_enable_kill",
    caseReason:
      "Entering Divinity from Neutral grants 3 energy and later attacks use triple damage. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "neutral-divinity-pray",
        name: "Pray",
        cost: 1,
        type: "SKILL",
        targetType: "none",
        effects: [{ kind: "mantra", amount: 10 }],
      },
      {
        uuid: "neutral-divinity-strike",
        name: "Strike",
        cost: 1,
        type: "ATTACK",
        damage: 6,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 1,
    monsters: [
      { index: 0, hp: 18, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "neutral_to_divinity_still_not_enough_damage",
    caseReason:
      "Divinity triples later attack damage, but 6 times 3 does not kill 19 hp. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "neutral-divinity-negative-pray",
        name: "Pray",
        cost: 1,
        type: "SKILL",
        targetType: "none",
        effects: [{ kind: "mantra", amount: 10 }],
      },
      {
        uuid: "neutral-divinity-negative-strike",
        name: "Strike",
        cost: 1,
        type: "ATTACK",
        damage: 6,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 1,
    monsters: [
      { index: 0, hp: 19, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "calm_to_divinity_energy_and_damage_enable_kill",
    caseReason:
      "Entering Divinity from Calm grants both Calm exit energy and Divinity energy, then later attacks use triple damage. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "calm-divinity-pray",
        name: "Pray",
        cost: 1,
        type: "SKILL",
        targetType: "none",
        effects: [{ kind: "mantra", amount: 10 }],
      },
      {
        uuid: "calm-divinity-big-strike",
        name: "Big Strike",
        cost: 3,
        type: "ATTACK",
        damage: 10,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 1,
    monsters: [
      { index: 0, hp: 30, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Calm",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "calm_to_divinity_still_not_enough_damage",
    caseReason:
      "Calm exit plus Divinity energy enables the attack, but 10 times 3 does not kill 31 hp. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "calm-divinity-negative-pray",
        name: "Pray",
        cost: 1,
        type: "SKILL",
        targetType: "none",
        effects: [{ kind: "mantra", amount: 10 }],
      },
      {
        uuid: "calm-divinity-negative-big-strike",
        name: "Big Strike",
        cost: 3,
        type: "ATTACK",
        damage: 10,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 1,
    monsters: [
      { index: 0, hp: 31, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Calm",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
  {
    caseName: "x_cost_chemical_x_enables_zero_energy_kill",
    caseReason:
      "Chemical X adds 2 to X, so a zero-energy X attack can still be lethal. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "chemical-x-attack",
        name: "Whirlwind",
        cost: "X",
        type: "ATTACK",
        targetType: "allEnemies",
        effects: [{ kind: "xCostAttack", damagePerX: 4 }],
      },
    ],
    energy: 0,
    monsters: [
      { index: 0, hp: 8, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [{ id: "Chemical X" }],
      cardsPlayedThisTurn: 0,
      xCostBonus: 2,
    },
  },
  {
    caseName: "minion_left_after_leader_kill",
    caseReason:
      "Combat ends when all remaining living monsters are minions. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "kill-leader",
        name: "Strike",
        cost: 1,
        type: "ATTACK",
        damage: 5,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 1,
    monsters: [
      { index: 0, hp: 5, block: 0, isMinion: false, powers: [] },
      { index: 2, hp: 50, block: 0, isMinion: true, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
  },
];
