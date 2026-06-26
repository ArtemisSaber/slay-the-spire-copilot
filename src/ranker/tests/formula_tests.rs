use super::*;
use std::collections::HashMap;

fn vars() -> HashMap<String, f64> {
    let mut m = HashMap::new();
    m.insert("damage".to_string(), 6.0);
    m.insert("hits".to_string(), 2.0);
    m.insert("weight".to_string(), 10.0);
    m.insert("block".to_string(), 5.0);
    m.insert("cost".to_string(), 2.0);
    m.insert("incoming_damage".to_string(), 8.0);
    m.insert("current_block".to_string(), 4.0);
    m
}

#[test]
fn constant_literal() {
    assert_eq!(evaluate("1000", &HashMap::new()), Ok(1000));
    assert_eq!(evaluate("-100", &HashMap::new()), Ok(-100));
}

#[test]
fn single_variable() {
    let v = vars();
    assert_eq!(evaluate("@damage", &v), Ok(6));
    assert_eq!(evaluate("@hits", &v), Ok(2));
    assert_eq!(evaluate("@weight", &v), Ok(10));
}

#[test]
fn missing_variable_returns_zero() {
    let v = vars();
    assert_eq!(evaluate("@nope", &v), Ok(0));
}

#[test]
fn multiply_two_vars() {
    let v = vars();
    assert_eq!(evaluate("@damage * @hits", &v), Ok(12));
}

#[test]
fn multiply_chain() {
    let v = vars();
    assert_eq!(evaluate("@damage * @hits * @weight", &v), Ok(120));
}

#[test]
fn multiply_var_and_literal() {
    let v = vars();
    assert_eq!(evaluate("@damage * 1.5", &v), Ok(9));
}

#[test]
fn multiply_chain_with_literals() {
    let v = vars();
    assert_eq!(evaluate("@damage * @hits * @weight * 1.5", &v), Ok(180));
}

#[test]
fn subtraction() {
    let v = vars();
    assert_eq!(evaluate("10 - 3", &v), Ok(7));
}

#[test]
fn addition() {
    let v = vars();
    assert_eq!(evaluate("10 + 3", &v), Ok(13));
}

#[test]
fn addition_and_multiplication_precedence() {
    let v = vars();
    assert_eq!(evaluate("@damage + @hits * @weight", &v), Ok(26));
}

#[test]
fn parenthesized_precedence() {
    let v = vars();
    assert_eq!(evaluate("(@damage + @hits) * @weight", &v), Ok(80));
}

#[test]
fn max_function() {
    let v = vars();
    assert_eq!(evaluate("max(0, 42)", &v), Ok(42));
    assert_eq!(evaluate("max(10, 3)", &v), Ok(10));
}

#[test]
fn min_function() {
    let v = vars();
    assert_eq!(evaluate("min(3, 10)", &v), Ok(3));
}

#[test]
fn max_with_variables() {
    let v = vars();
    assert_eq!(
        evaluate("max(0, @incoming_damage - @current_block)", &v),
        Ok(4)
    );
}

#[test]
fn compute_formula_returns_bool_like_f64() {
    let v = vars();
    let result = evaluate("@incoming_damage > @current_block", &v).unwrap();
    assert_eq!(result, 1);
    let result2 = evaluate("@incoming_damage < @current_block", &v).unwrap();
    assert_eq!(result2, 0);
}

#[test]
fn division() {
    let v = vars();
    assert_eq!(evaluate("10 / 2", &v), Ok(5));
    assert_eq!(evaluate("@damage / 2", &v), Ok(3));
}

#[test]
fn negative_self_damage_reference() {
    let mut v = vars();
    v.insert("self_damage".to_string(), 3.0);
    v.insert("current_hp".to_string(), 45.0);
    let result = evaluate("-1050000 * @self_damage / max(1, @current_hp)", &v).unwrap();
    assert_eq!(result, -70000);
}

#[test]
fn whitespace_insensitive() {
    let v = vars();
    assert_eq!(evaluate("  @damage * @hits  ", &v), Ok(12));
}

#[test]
fn invalid_expression_returns_error() {
    assert!(evaluate("@damage ? @hits", &HashMap::new()).is_err());
    assert!(evaluate("(", &HashMap::new()).is_err());
}
