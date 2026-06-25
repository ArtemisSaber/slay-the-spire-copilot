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
  {
    caseName: "block_counts_as_durability",
    caseReason:
      "Living monster block must count as durability, so 10 damage does not kill 6 hp plus 5 block. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "block-ignored-risk",
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
      { index: 0, hp: 6, block: 5, isMinion: false, powers: [] },
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
    caseName: "block_exact_damage_kills",
    caseReason:
      "Damage exactly equal to hp plus block should kill. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "block-exact-kill",
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
      { index: 0, hp: 6, block: 4, isMinion: false, powers: [] },
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
    caseName: "targeted_overkill_does_not_spill",
    caseReason:
      "Overkill on one targeted enemy does not damage another enemy. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "overkill-single-target",
        name: "Big Strike",
        cost: 1,
        type: "ATTACK",
        damage: 10,
        hits: 1,
        targetType: "enemy",
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
    caseName: "aoe_kills_multiple_non_minions",
    caseReason:
      "AoE damage should apply to every living enemy. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "aoe-kill-all",
        name: "Cleave",
        cost: 1,
        type: "ATTACK",
        damage: 6,
        hits: 1,
        targetType: "allEnemies",
      },
    ],
    energy: 1,
    monsters: [
      { index: 0, hp: 6, block: 0, isMinion: false, powers: [] },
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
    caseName: "aoe_leaves_non_minion_alive",
    caseReason:
      "AoE that leaves any non-minion alive is not a kill. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "aoe-not-enough",
        name: "Cleave",
        cost: 1,
        type: "ATTACK",
        damage: 5,
        hits: 1,
        targetType: "allEnemies",
      },
    ],
    energy: 1,
    monsters: [
      { index: 0, hp: 5, block: 0, isMinion: false, powers: [] },
      { index: 1, hp: 6, block: 0, isMinion: false, powers: [] },
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
    caseName: "random_target_hits_less_than_alive_count",
    caseReason:
      "A random attack cannot guarantee killing every living enemy when hit count is lower than living enemy count. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "random-too-few-hits",
        name: "Small Boomerang",
        cost: 1,
        type: "ATTACK",
        damage: 10,
        hits: 2,
        targetType: "randomEnemy",
      },
    ],
    energy: 1,
    monsters: [
      { index: 0, hp: 1, block: 0, isMinion: false, powers: [] },
      { index: 1, hp: 1, block: 0, isMinion: false, powers: [] },
      { index: 2, hp: 1, block: 0, isMinion: false, powers: [] },
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
    caseName: "random_target_with_mutable_power_is_not_simplified",
    caseReason:
      "The simple random-target formula should not be used when a mutable power like Curl Up can change durability. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "random-curl-up",
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
      {
        index: 0,
        hp: 4,
        block: 0,
        isMinion: false,
        powers: [{ id: "Curl Up", amount: 6 }],
      },
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
    caseName: "artifact_allows_second_vulnerable_application",
    caseReason:
      "Artifact should decrement after the first Vulnerable application, allowing a second Vulnerable card to affect later damage. Expected true.",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "artifact-trip-a",
        name: "Trip A",
        cost: 0,
        type: "SKILL",
        targetType: "enemy",
        effects: [{ kind: "vulnerable", amount: 2 }],
      },
      {
        uuid: "artifact-trip-b",
        name: "Trip B",
        cost: 0,
        type: "SKILL",
        targetType: "enemy",
        effects: [{ kind: "vulnerable", amount: 2 }],
      },
      {
        uuid: "artifact-final-strike",
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
      {
        index: 0,
        hp: 15,
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
    caseName: "x_cost_zero_energy_without_chemical_x",
    caseReason:
      "A zero-energy X attack without Chemical X has X=0 and should not kill. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "zero-x-no-chemical-x",
        name: "Whirlwind",
        cost: "X",
        type: "ATTACK",
        targetType: "allEnemies",
        effects: [{ kind: "xCostAttack", damagePerX: 4 }],
      },
    ],
    energy: 0,
    monsters: [
      { index: 0, hp: 1, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
      xCostBonus: 0,
    },
  },
  {
    caseName: "unsupported_dangerous_monster_power_fails_closed",
    caseReason:
      "A dangerous unmodeled monster power should fail closed instead of accepting apparent lethal damage. Expected false.",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "unsupported-power-lethal-looking-hit",
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
        hp: 5,
        block: 0,
        isMinion: false,
        powers: [{ id: "Mode Shift", amount: 30 }],
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
];

export type ExpectedKillPlay = {
  card: string;
  target: number | null;
};

export type KillScanIntegrationEntryPoint =
  | "find_kill_sequence_from_normalized_state"
  | "autoplay_execute_sequence"
  | "ranker_candidate_suffix"
  | "context_builder_fail_closed";

export type KillScanIntegrationCase = {
  caseName: string;
  caseReason: string;
  entryPoint: KillScanIntegrationEntryPoint;
  expectedDeterministicKill: boolean;
  handDeck: Cards[];
  energy: number;
  monsters: Monster[];
  currentPlayerStatus: PlayerStatus;
  candidatePlay?: ExpectedKillPlay;
  expectedSequence?: ExpectedKillPlay[];
  expectedSuffix?: ExpectedKillPlay[];
  expectedContextFailure?: boolean;
  executorAssertions?: string[];
};

export const killScanIntegrationCases: KillScanIntegrationCase[] = [
  {
    caseName: "integration_command_target_index_gap_is_preserved",
    caseReason:
      "NormalizedState can contain living monsters whose command indexes are not dense. Scanner output must use MonsterInfo.index, not living-vector position.",
    entryPoint: "find_kill_sequence_from_normalized_state",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "kill-command-index-two",
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
      { index: 0, hp: 50, block: 0, isMinion: true, powers: [] },
      { index: 2, hp: 5, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
    expectedSequence: [{ card: "kill-command-index-two", target: 2 }],
    executorAssertions: [
      "Returned target is command index 2, not living-monster vector position 1.",
      "Autoplay validates target against current state.monsters[].index and sends target 2 unchanged.",
    ],
  },
  {
    caseName: "integration_executor_recomputes_hand_index_from_uuid",
    caseReason:
      "The scanner returns UUIDs, while autoplay must resolve each UUID to the current hand index after prior plays shift the hand.",
    entryPoint: "autoplay_execute_sequence",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "uuid-energy-setup",
        name: "Seeing Red",
        cost: 0,
        type: "SKILL",
        targetType: "none",
        effects: [{ kind: "energy", amount: 1 }],
      },
      {
        uuid: "uuid-shifted-lethal-attack",
        name: "Strike",
        cost: 1,
        type: "ATTACK",
        damage: 5,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 0,
    monsters: [
      { index: 0, hp: 5, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
    expectedSequence: [
      { card: "uuid-energy-setup", target: null },
      { card: "uuid-shifted-lethal-attack", target: 0 },
    ],
    executorAssertions: [
      "Resolve uuid-energy-setup to its current hand index before the first play.",
      "After the first play, resolve uuid-shifted-lethal-attack again; do not reuse its original hand index.",
    ],
  },
  {
    caseName: "integration_ranker_candidate_play_returns_suffix",
    caseReason:
      "Ranker scoring can apply a candidate play first, then ask the scanner for a deterministic suffix from the updated context.",
    entryPoint: "ranker_candidate_suffix",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "candidate-bash",
        name: "Bash",
        cost: 2,
        type: "ATTACK",
        damage: 8,
        hits: 1,
        targetType: "enemy",
        effects: [{ kind: "vulnerable", amount: 2 }],
      },
      {
        uuid: "candidate-suffix-strike",
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
      { index: 0, hp: 17, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
    candidatePlay: { card: "candidate-bash", target: 0 },
    expectedSuffix: [{ card: "candidate-suffix-strike", target: 0 }],
    executorAssertions: [
      "Candidate Bash damage resolves before Vulnerable applies to later attacks.",
      "The suffix scanner sees 9 damage from Strike because Vulnerable is active after the candidate play.",
    ],
  },
  {
    caseName: "integration_time_warp_missing_amount_fails_closed",
    caseReason:
      "If Time Warp is present but the remaining play count cannot be computed exactly, context building must fail closed.",
    entryPoint: "context_builder_fail_closed",
    expectedDeterministicKill: false,
    handDeck: [
      {
        uuid: "time-warp-strike-a",
        name: "Strike A",
        cost: 0,
        type: "ATTACK",
        damage: 3,
        hits: 1,
        targetType: "enemy",
      },
      {
        uuid: "time-warp-strike-b",
        name: "Strike B",
        cost: 0,
        type: "ATTACK",
        damage: 3,
        hits: 1,
        targetType: "enemy",
      },
      {
        uuid: "time-warp-strike-c",
        name: "Strike C",
        cost: 0,
        type: "ATTACK",
        damage: 3,
        hits: 1,
        targetType: "enemy",
      },
      {
        uuid: "time-warp-strike-d",
        name: "Strike D",
        cost: 0,
        type: "ATTACK",
        damage: 3,
        hits: 1,
        targetType: "enemy",
      },
    ],
    energy: 0,
    monsters: [
      {
        index: 0,
        hp: 10,
        block: 0,
        isMinion: false,
        powers: [{ id: "Time Warp" }],
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
    expectedContextFailure: true,
    executorAssertions: [
      "The source state has Time Warp active but no exact counter in the normalized test input.",
      "The scanner must return None/false rather than assume enough plays remain.",
    ],
  },
  {
    caseName: "integration_sequence_revalidation_aborts_on_illegal_target",
    caseReason:
      "Autoplay must revalidate scanner output immediately before sending commands, because monster indexes can disappear after state changes.",
    entryPoint: "autoplay_execute_sequence",
    expectedDeterministicKill: true,
    handDeck: [
      {
        uuid: "target-revalidation-strike",
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
      { index: 2, hp: 5, block: 0, isMinion: false, powers: [] },
    ],
    currentPlayerStatus: {
      stance: "Neutral",
      strength: 0,
      mantra: 0,
      powers: [],
      relics: [],
      cardsPlayedThisTurn: 0,
    },
    expectedSequence: [{ card: "target-revalidation-strike", target: 2 }],
    executorAssertions: [
      "Before execution, validate target 2 against the latest state.monsters[].index list.",
      "If target 2 is no longer legal, abort the scanner sequence and fall back to normal planning.",
    ],
  },
];
