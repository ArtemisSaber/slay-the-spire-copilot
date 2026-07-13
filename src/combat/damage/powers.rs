use crate::combat::effects::StanceEffect;
use crate::combat::{MonsterSnapshot, PowerState, Stance};

pub(crate) fn apply_vulnerable_through_artifact(monster: &mut MonsterSnapshot, amount: i16) {
    let artifact_idx = monster
        .powers
        .iter()
        .position(|p| p.id == "Artifact" && p.amount > 0);
    if let Some(ai) = artifact_idx {
        monster.powers[ai].amount -= 1;
        if monster.powers[ai].amount < 0 {
            monster.powers[ai].amount = 0;
        }
        return;
    }

    if let Some(p) = monster
        .powers
        .iter_mut()
        .find(|p| p.id == "Vulnerable" || p.id == "易伤")
    {
        p.amount += amount;
    } else {
        monster.powers.push(PowerState {
            id: "Vulnerable".into(),
            amount,
            triggered: false,
        });
    }
}

pub(crate) fn apply_after_card_powers(monsters: &mut [MonsterSnapshot]) {
    for m in monsters.iter_mut() {
        if let Some(p) = m
            .powers
            .iter_mut()
            .find(|p| (p.id == "Slow" || p.id == "缓慢") && p.amount > 0)
        {
            p.amount += 1;
        }
    }
}

pub(crate) fn apply_stance_effect(effect: &StanceEffect, current: Stance) -> Stance {
    match effect {
        StanceEffect::None => current,
        StanceEffect::EnterWrath => Stance::Wrath,
        StanceEffect::EnterCalm => Stance::Calm,
        StanceEffect::ExitStance => Stance::Neutral,
        StanceEffect::EnterDivinity => Stance::Divinity,
    }
}

pub(crate) fn stance_energy_delta(from: Stance, to: Stance) -> i16 {
    let mut delta = 0i16;

    if from == Stance::Calm && to != Stance::Calm {
        delta += 2;
    }

    if to == Stance::Divinity {
        delta += 3;
    }

    delta
}
