use crate::locales::EffectParserLocale;
use crate::state::CardInfo;

mod attributes;
mod damage;

use attributes::{
    parse_energy_gain, parse_execute, parse_exhaust, parse_mantra, parse_stance,
    parse_strength_gain, parse_vulnerable,
};
use damage::parse_damage;

#[cfg(test)]
use crate::parsing::extract_first_integer;
#[cfg(test)]
use damage::{parse_damage_multiplier_hits, parse_hits_suffix, parse_target_type};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetType {
    Targeted,
    AoE,
    RandomTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HitCount {
    Fixed(i16),
    XTimes,
    XPlus(i16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageEffect {
    pub amount: i16,
    pub hits: HitCount,
    pub target_type: TargetType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExhaustKind {
    None,
    Self_,
    All,
    NonAttacks,
    Attacks,
    Random,
    Chosen,
    ExhaustAllDamagePerCard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardEffect {
    pub damage: Option<DamageEffect>,
    pub energy_gain: i16,
    pub strength_gain: i16,
    pub vulnerable: Option<i16>,
    pub stance: StanceEffect,
    pub mantra_gain: i16,
    pub execute: Option<i16>,
    pub exhaust: ExhaustKind,
    pub x_cost: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StanceEffect {
    None,
    EnterWrath,
    EnterCalm,
    ExitStance,
    EnterDivinity,
}

pub fn parse_card_effect(card: &CardInfo, locale: &EffectParserLocale) -> Option<CardEffect> {
    let desc = &card.description;

    let effect = CardEffect {
        damage: parse_damage(desc, &card.card_type, locale),
        energy_gain: parse_energy_gain(desc),
        strength_gain: parse_strength_gain(desc, locale),
        vulnerable: parse_vulnerable(desc, locale),
        stance: parse_stance(desc, locale),
        mantra_gain: parse_mantra(desc, locale),
        execute: parse_execute(desc, locale),
        exhaust: parse_exhaust(desc, locale),
        x_cost: desc.contains('X')
            || desc.to_lowercase().contains("x次")
            || desc.to_lowercase().contains("x times")
            || desc.contains("花费所有")
            || desc.to_lowercase().contains("spend all"),
    };

    if is_scan_relevant(&effect) {
        Some(effect)
    } else {
        None
    }
}

pub fn is_scan_relevant(effect: &CardEffect) -> bool {
    effect.damage.is_some()
        || effect.energy_gain > 0
        || effect.strength_gain > 0
        || effect.vulnerable.is_some()
        || effect.stance != StanceEffect::None
        || effect.mantra_gain > 0
        || effect.execute.is_some()
        || effect.exhaust != ExhaustKind::None
        || effect.x_cost
}

#[cfg(test)]
#[path = "../tests/combat_effects_tests.rs"]
mod tests;
