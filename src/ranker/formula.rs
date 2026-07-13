use std::collections::HashMap;

use lexer::Token;
use parser::Parser;

mod lexer;
mod parser;

pub fn evaluate(expr: &str, vars: &HashMap<String, f64>) -> Result<i64, String> {
    let parser = Parser::new(expr)?;
    let resolved = parser
        .into_tokens()
        .into_iter()
        .map(|token| match token {
            Token::Variable(name) => Token::Number(vars.get(&name).copied().unwrap_or(0.0)),
            token => token,
        })
        .collect();
    let mut parser = Parser::from_tokens(resolved);
    let result = parser.parse_expr()?;
    if !parser.is_complete() {
        return Err("unexpected trailing content".to_string());
    }
    Ok(clamp_to_i64(result))
}

fn clamp_to_i64(value: f64) -> i64 {
    if value.is_nan() || value.is_infinite() {
        return 0;
    }
    if value > i64::MAX as f64 {
        i64::MAX
    } else if value < i64::MIN as f64 {
        i64::MIN
    } else {
        value as i64
    }
}

#[cfg(test)]
#[path = "tests/formula_tests.rs"]
mod tests;
