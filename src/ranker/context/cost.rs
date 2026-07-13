use crate::ranker::parser::ParsedEffects;
use crate::state::CardInfo;

pub(super) fn resolve_x_cost(
    card: &CardInfo,
    energy: i64,
    chemical_x: f64,
) -> (i64, Option<i64>, Option<i64>) {
    if card.cost != -1 {
        return (card.cost, None, None);
    }
    let spent = energy as f64 + chemical_x;
    if card.card_type == "ATTACK" {
        (energy, Some(spent as i64), None)
    } else {
        (energy, None, Some(spent as i64))
    }
}

pub(super) fn apply_x_cost_amount(parsed: &mut ParsedEffects, amount: i64) {
    if parsed.damage.is_none() || parsed.damage == Some(0) {
        parsed.damage = Some(amount);
    }
    if parsed.block.is_none() || parsed.block == Some(0) {
        parsed.block = Some(amount);
    }
    if parsed.str_loss.is_none() || parsed.str_loss == Some(0) {
        parsed.str_loss = Some(amount);
    }
    if parsed.weak.is_none() || parsed.weak == Some(0) {
        parsed.weak = Some(amount);
    }
}
