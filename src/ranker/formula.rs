use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Variable(String),
    Plus,
    Minus,
    Star,
    Slash,
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
    LParen,
    RParen,
    Comma,
    Func(String),
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
    error: bool,
}

impl Lexer {
    fn new(input: &str) -> Self {
        Lexer {
            chars: input.chars().collect(),
            pos: 0,
            error: false,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|c| c.is_ascii_whitespace()) {
            self.advance();
        }
    }

    fn read_number(&mut self) -> f64 {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.advance();
        }
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.advance();
        }
        if self.peek() == Some('.') {
            self.advance();
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.advance();
            }
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        s.parse().unwrap_or(0.0)
    }

    fn read_ident(&mut self) -> String {
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            self.advance();
        }
        self.chars[start..self.pos].iter().collect()
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let c = self.peek()?;

        if c.is_ascii_digit()
            || (c == '-'
                && self.pos + 1 < self.chars.len()
                && self.chars[self.pos + 1].is_ascii_digit())
        {
            return Some(Token::Number(self.read_number()));
        }

        match c {
            '+' => {
                self.advance();
                Some(Token::Plus)
            }
            '-' => {
                self.advance();
                Some(Token::Minus)
            }
            '*' => {
                self.advance();
                Some(Token::Star)
            }
            '/' => {
                self.advance();
                Some(Token::Slash)
            }
            '>' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::Gte)
                } else {
                    Some(Token::Gt)
                }
            }
            '<' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::Lte)
                } else {
                    Some(Token::Lt)
                }
            }
            '=' => {
                self.advance();
                if self.peek() == Some('=') {
                    self.advance();
                    Some(Token::Eq)
                } else {
                    self.error = true;
                    None
                }
            }
            '(' => {
                self.advance();
                Some(Token::LParen)
            }
            ')' => {
                self.advance();
                Some(Token::RParen)
            }
            ',' => {
                self.advance();
                Some(Token::Comma)
            }
            '@' => {
                self.advance();
                let ident = self.read_ident();
                Some(Token::Variable(ident))
            }
            c if c.is_ascii_alphabetic() => {
                let ident = self.read_ident();
                if self.peek() == Some('(') {
                    Some(Token::Func(ident))
                } else {
                    self.error = true;
                    None
                }
            }
            _ => {
                self.error = true;
                None
            }
        }
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Result<Self, String> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        while let Some(tok) = lexer.next_token() {
            tokens.push(tok);
        }
        if lexer.error {
            return Err("unrecognized character in expression".to_string());
        }
        if tokens.is_empty() {
            return Err("empty expression".to_string());
        }
        Ok(Parser { tokens, pos: 0 })
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        self.pos += 1;
        tok
    }

    fn parse_expr(&mut self) -> Result<f64, String> {
        let mut left = self.parse_term()?;
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.advance();
                    let right = self.parse_term()?;
                    left += right;
                }
                Some(Token::Minus) => {
                    self.advance();
                    let right = self.parse_term()?;
                    left -= right;
                }
                Some(Token::Gt) => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = if left > right { 1.0 } else { 0.0 };
                }
                Some(Token::Lt) => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = if left < right { 1.0 } else { 0.0 };
                }
                Some(Token::Gte) => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = if left >= right { 1.0 } else { 0.0 };
                }
                Some(Token::Lte) => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = if left <= right { 1.0 } else { 0.0 };
                }
                Some(Token::Eq) => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = if (left - right).abs() < 1e-9 {
                        1.0
                    } else {
                        0.0
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<f64, String> {
        let mut left = self.parse_factor()?;
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.advance();
                    let right = self.parse_factor()?;
                    left *= right;
                }
                Some(Token::Slash) => {
                    self.advance();
                    let right = self.parse_factor()?;
                    if right == 0.0 {
                        return Err("division by zero".to_string());
                    }
                    left /= right;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<f64, String> {
        match self.peek().cloned() {
            Some(Token::Number(n)) => {
                self.advance();
                Ok(n)
            }
            Some(Token::Variable(_)) => {
                let _ = self.advance();
                Ok(0.0)
            }
            Some(Token::Func(_)) => {
                if let Token::Func(name) = self.advance().clone() {
                    self.expect_lparen()?;
                    let a = self.parse_expr()?;
                    if self.peek() == Some(&Token::Comma) {
                        self.advance();
                        let b = self.parse_expr()?;
                        self.expect_rparen()?;
                        match name.as_str() {
                            "max" => Ok(a.max(b)),
                            "min" => Ok(a.min(b)),
                            _ => Err(format!("unknown function: {name}")),
                        }
                    } else {
                        self.expect_rparen()?;
                        Err(format!("{name} requires 2 arguments"))
                    }
                } else {
                    Err("internal parser error: expected function token after peek".to_string())
                }
            }
            Some(Token::Minus) => {
                self.advance();
                let factor = self.parse_factor()?;
                Ok(-factor)
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect_rparen()?;
                Ok(expr)
            }
            _ => Err("unexpected token".to_string()),
        }
    }

    fn expect_lparen(&mut self) -> Result<(), String> {
        match self.peek() {
            Some(Token::LParen) => {
                self.advance();
                Ok(())
            }
            _ => Err("expected '('".to_string()),
        }
    }

    fn expect_rparen(&mut self) -> Result<(), String> {
        match self.peek() {
            Some(Token::RParen) => {
                self.advance();
                Ok(())
            }
            _ => Err("expected ')'".to_string()),
        }
    }
}

pub fn evaluate(expr: &str, vars: &HashMap<String, f64>) -> Result<i64, String> {
    let parser = Parser::new(expr)?;
    let tokens = parser.tokens.clone();

    let resolved: Vec<Token> = tokens
        .iter()
        .map(|t| match t {
            Token::Variable(name) => {
                let v = vars.get(name).copied().unwrap_or(0.0);
                Token::Number(v)
            }
            other => other.clone(),
        })
        .collect();

    let mut parser = Parser {
        tokens: resolved,
        pos: 0,
    };

    let result = parser.parse_expr()?;
    if parser.pos != parser.tokens.len() {
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
