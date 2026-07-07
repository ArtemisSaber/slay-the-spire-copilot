use crate::locales::Locale;
use crate::parsing::extract_first_integer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CounterType {
    UsesRemaining,
    Cyclic,
}

fn counter_type_for(id: &str) -> Option<CounterType> {
    match id {
        "NeowsBlessing" | "Omamori" | "Matryoshka" | "Girya" | "WingedGreaves" => {
            Some(CounterType::UsesRemaining)
        }
        "Happy Flower" | "Incense Burner" | "StoneCalendar" | "Pen Nib" | "Nunchaku" | "Kunai"
        | "Shuriken" | "Ornamental Fan" | "Letter Opener" | "Sundial" | "InkBottle" => {
            Some(CounterType::Cyclic)
        }
        _ => None,
    }
}

fn replace_first_integer(s: &str, new_val: i64) -> String {
    let mut result = String::with_capacity(s.len());
    let mut found = false;
    let mut in_digits = false;
    let mut digit_start = 0usize;

    for (i, c) in s.char_indices() {
        if !found && c.is_ascii_digit() {
            if !in_digits {
                in_digits = true;
                digit_start = i;
            }
        } else if in_digits {
            in_digits = false;
            found = true;
            result.push_str(&s[..digit_start]);
            result.push_str(&new_val.to_string());
            result.push_str(&s[i..]);
            break;
        }
    }

    if in_digits && !found {
        result.push_str(&s[..digit_start]);
        result.push_str(&new_val.to_string());
        return result;
    }

    if !found {
        return s.to_string();
    }

    result
}

pub fn rewrite_relic_description(id: &str, counter: i64, desc: &str, locale: &Locale) -> String {
    let Some(ctype) = counter_type_for(id) else {
        return desc.to_string();
    };
    match ctype {
        CounterType::UsesRemaining => replace_first_integer(desc, counter),
        CounterType::Cyclic => {
            let max = extract_first_integer(desc).unwrap_or(1);
            let remaining = (max - counter).max(0);
            locale
                .relic_counter_cycles
                .get(id)
                .map(|template| template.replace("{count}", &remaining.to_string()))
                .unwrap_or_else(|| desc.to_string())
        }
    }
}

#[cfg(test)]
#[path = "tests/relic_counters_tests.rs"]
mod tests;
