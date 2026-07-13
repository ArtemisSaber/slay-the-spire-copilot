use crate::locales::Locale;
use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct AdviceFields {
    pub recommendation: String,
    pub reason: String,
    pub risk: String,
    pub commentary: String,
}

pub fn parse_advice_response(raw: &str, locale: &Locale) -> AdviceFields {
    let mut recommendation = String::new();
    let mut reason = String::new();
    let mut risk = String::new();
    let mut commentary = String::new();

    let mut current: Option<&mut String> = None;

    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix(locale.parser.recommendation.as_str()) {
            recommendation.push_str(rest);
            current = Some(&mut recommendation);
        } else if let Some(rest) = line.strip_prefix(locale.parser.reason.as_str()) {
            reason.push_str(rest);
            current = Some(&mut reason);
        } else if let Some(rest) = line.strip_prefix(locale.parser.risk.as_str()) {
            risk.push_str(rest);
            current = Some(&mut risk);
        } else if let Some(rest) = line.strip_prefix(locale.parser.commentary.as_str()) {
            commentary.push_str(rest);
            current = Some(&mut commentary);
        } else if let Some(ref mut field) = current
            && !line.is_empty()
        {
            if !field.is_empty() {
                field.push('\n');
            }
            field.push_str(line);
        }
    }

    AdviceFields {
        recommendation,
        reason,
        risk,
        commentary,
    }
}
