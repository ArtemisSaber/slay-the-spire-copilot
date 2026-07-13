use super::*;

#[test]
fn is_scan_relevant_all_false() {
    let e = CardEffect {
        damage: None,
        energy_gain: 0,
        strength_gain: 0,
        vulnerable: None,
        stance: StanceEffect::None,
        mantra_gain: 0,
        execute: None,
        exhaust: ExhaustKind::None,
        x_cost: false,
    };
    assert!(!is_scan_relevant(&e));
}

#[test]
fn is_scan_relevant_damage() {
    let e = CardEffect {
        damage: Some(DamageEffect {
            amount: 6,
            hits: HitCount::Fixed(1),
            target_type: TargetType::Targeted,
        }),
        energy_gain: 0,
        strength_gain: 0,
        vulnerable: None,
        stance: StanceEffect::None,
        mantra_gain: 0,
        execute: None,
        exhaust: ExhaustKind::None,
        x_cost: false,
    };
    assert!(is_scan_relevant(&e));
}

#[test]
fn is_scan_relevant_energy_gain() {
    let e = CardEffect {
        damage: None,
        energy_gain: 2,
        strength_gain: 0,
        vulnerable: None,
        stance: StanceEffect::None,
        mantra_gain: 0,
        execute: None,
        exhaust: ExhaustKind::None,
        x_cost: false,
    };
    assert!(is_scan_relevant(&e));
}

#[test]
fn is_scan_relevant_strength_gain() {
    let e = CardEffect {
        damage: None,
        energy_gain: 0,
        strength_gain: 3,
        vulnerable: None,
        stance: StanceEffect::None,
        mantra_gain: 0,
        execute: None,
        exhaust: ExhaustKind::None,
        x_cost: false,
    };
    assert!(is_scan_relevant(&e));
}

#[test]
fn is_scan_relevant_vulnerable() {
    let e = CardEffect {
        damage: None,
        energy_gain: 0,
        strength_gain: 0,
        vulnerable: Some(2),
        stance: StanceEffect::None,
        mantra_gain: 0,
        execute: None,
        exhaust: ExhaustKind::None,
        x_cost: false,
    };
    assert!(is_scan_relevant(&e));
}

#[test]
fn is_scan_relevant_stance() {
    let e = CardEffect {
        damage: None,
        energy_gain: 0,
        strength_gain: 0,
        vulnerable: None,
        stance: StanceEffect::EnterWrath,
        mantra_gain: 0,
        execute: None,
        exhaust: ExhaustKind::None,
        x_cost: false,
    };
    assert!(is_scan_relevant(&e));
}

#[test]
fn is_scan_relevant_mantra() {
    let e = CardEffect {
        damage: None,
        energy_gain: 0,
        strength_gain: 0,
        vulnerable: None,
        stance: StanceEffect::None,
        mantra_gain: 2,
        execute: None,
        exhaust: ExhaustKind::None,
        x_cost: false,
    };
    assert!(is_scan_relevant(&e));
}

#[test]
fn is_scan_relevant_execute() {
    let e = CardEffect {
        damage: None,
        energy_gain: 0,
        strength_gain: 0,
        vulnerable: None,
        stance: StanceEffect::None,
        mantra_gain: 0,
        execute: Some(30),
        exhaust: ExhaustKind::None,
        x_cost: false,
    };
    assert!(is_scan_relevant(&e));
}

#[test]
fn is_scan_relevant_exhaust() {
    let e = CardEffect {
        damage: None,
        energy_gain: 0,
        strength_gain: 0,
        vulnerable: None,
        stance: StanceEffect::None,
        mantra_gain: 0,
        execute: None,
        exhaust: ExhaustKind::Self_,
        x_cost: false,
    };
    assert!(is_scan_relevant(&e));
}

#[test]
fn is_scan_relevant_x_cost() {
    let e = CardEffect {
        damage: None,
        energy_gain: 0,
        strength_gain: 0,
        vulnerable: None,
        stance: StanceEffect::None,
        mantra_gain: 0,
        execute: None,
        exhaust: ExhaustKind::None,
        x_cost: true,
    };
    assert!(is_scan_relevant(&e));
}
