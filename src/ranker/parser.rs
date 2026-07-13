#[derive(Debug, Clone, Default)]
pub struct ParsedEffects {
    pub damage: Option<i64>,
    pub hits: i64,
    pub block: Option<i64>,
    pub heal: Option<i64>,
    pub draw: Option<i64>,
    pub self_damage: Option<i64>,
    pub str_gain: Option<i64>,
    pub dex_gain: Option<i64>,
    pub poison: Option<i64>,
    pub vulnerable: Option<i64>,
    pub weak: Option<i64>,
    pub energy_gain: Option<i64>,
    pub str_loss: Option<i64>,
    pub str_loss_temp: bool,
    pub mantra: Option<i64>,
    pub focus_gain: Option<i64>,
    pub exhaust_count: i64,
    pub ethereal: bool,
    pub channel_orb: Option<String>,
    pub evoke_orb: Option<String>,
    pub orb_slot_expand: i64,
    pub exits_stance: bool,
    pub enters_wrath: bool,
    pub enters_calm: bool,
}

mod attributes;
mod basic;
mod helpers;
mod special;

pub fn parse_description(desc: &str) -> ParsedEffects {
    let (str_loss, str_loss_temp) = attributes::parse_str_loss(desc);
    ParsedEffects {
        damage: basic::parse_damage_amount(desc),
        hits: basic::parse_hits(desc),
        block: basic::parse_block(desc),
        heal: basic::parse_heal(desc),
        draw: basic::parse_draw(desc),
        self_damage: basic::parse_self_damage(desc),
        str_gain: attributes::parse_str_gain(desc),
        dex_gain: attributes::parse_dex_gain(desc),
        poison: attributes::parse_poison(desc),
        vulnerable: attributes::parse_vulnerable(desc),
        weak: attributes::parse_weak(desc),
        energy_gain: attributes::parse_energy_gain(desc),
        str_loss,
        str_loss_temp,
        mantra: attributes::parse_mantra(desc),
        focus_gain: attributes::parse_focus_gain(desc),
        exhaust_count: special::parse_exhaust(desc),
        ethereal: special::parse_ethereal(desc),
        channel_orb: special::parse_channel_orb(desc),
        evoke_orb: special::parse_evoke_orb(desc),
        orb_slot_expand: special::parse_orb_slot_expand(desc),
        exits_stance: special::parse_exits_stance(desc),
        enters_wrath: special::parse_enters_wrath(desc),
        enters_calm: special::parse_enters_calm(desc),
    }
}

#[cfg(test)]
#[path = "tests/parser_tests.rs"]
mod tests;
