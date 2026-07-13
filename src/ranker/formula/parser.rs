use super::lexer::{Lexer, Token};

pub(super) struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub(super) fn new(input: &str) -> Result<Self, String> {
        Ok(Self::from_tokens(Lexer::new(input).tokenize()?))
    }

    pub(super) fn from_tokens(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    pub(super) fn into_tokens(self) -> Vec<Token> {
        self.tokens
    }

    pub(super) fn is_complete(&self) -> bool {
        self.pos == self.tokens.len()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.pos];
        self.pos += 1;
        token
    }

    pub(super) fn parse_expr(&mut self) -> Result<f64, String> {
        let mut left = self.parse_term()?;
        while let Some(operator) = self.peek().cloned() {
            let result = match operator {
                Token::Plus => self.parse_binary(left, |left, right| left + right),
                Token::Minus => self.parse_binary(left, |left, right| left - right),
                Token::Gt => self.parse_binary(left, compare(|left, right| left > right)),
                Token::Lt => self.parse_binary(left, compare(|left, right| left < right)),
                Token::Gte => self.parse_binary(left, compare(|left, right| left >= right)),
                Token::Lte => self.parse_binary(left, compare(|left, right| left <= right)),
                Token::Eq => self.parse_binary(left, |left, right| {
                    ((left - right).abs() < 1e-9) as u8 as f64
                }),
                _ => break,
            };
            left = result?;
        }
        Ok(left)
    }

    fn parse_binary(
        &mut self,
        left: f64,
        operation: impl FnOnce(f64, f64) -> f64,
    ) -> Result<f64, String> {
        self.advance();
        Ok(operation(left, self.parse_term()?))
    }

    fn parse_term(&mut self) -> Result<f64, String> {
        let mut left = self.parse_factor()?;
        loop {
            let result = match self.peek() {
                Some(Token::Star) => self.parse_term_binary(left, |left, right| left * right),
                Some(Token::Slash) => self.divide(left),
                _ => break,
            };
            left = result?;
        }
        Ok(left)
    }

    fn divide(&mut self, left: f64) -> Result<f64, String> {
        self.advance();
        let right = self.parse_factor()?;
        if right == 0.0 {
            Err("division by zero".to_string())
        } else {
            Ok(left / right)
        }
    }

    fn parse_term_binary(
        &mut self,
        left: f64,
        operation: impl FnOnce(f64, f64) -> f64,
    ) -> Result<f64, String> {
        self.advance();
        Ok(operation(left, self.parse_factor()?))
    }

    fn parse_factor(&mut self) -> Result<f64, String> {
        match self.peek().cloned() {
            Some(Token::Number(number)) => {
                self.advance();
                Ok(number)
            }
            Some(Token::Variable(_)) => {
                self.advance();
                Ok(0.0)
            }
            Some(Token::Func(_)) => self.parse_function(),
            Some(Token::Minus) => {
                self.advance();
                Ok(-self.parse_factor()?)
            }
            Some(Token::LParen) => {
                self.advance();
                let expression = self.parse_expr()?;
                self.expect(Token::RParen, "expected ')'")?;
                Ok(expression)
            }
            _ => Err("unexpected token".to_string()),
        }
    }

    fn parse_function(&mut self) -> Result<f64, String> {
        let Token::Func(name) = self.advance().clone() else {
            return Err("internal parser error: expected function token after peek".to_string());
        };
        self.expect(Token::LParen, "expected '('")?;
        let first = self.parse_expr()?;
        if self.peek() != Some(&Token::Comma) {
            self.expect(Token::RParen, "expected ')'")?;
            return Err(format!("{name} requires 2 arguments"));
        }
        self.advance();
        let second = self.parse_expr()?;
        self.expect(Token::RParen, "expected ')'")?;
        match name.as_str() {
            "max" => Ok(first.max(second)),
            "min" => Ok(first.min(second)),
            _ => Err(format!("unknown function: {name}")),
        }
    }

    fn expect(&mut self, expected: Token, message: &str) -> Result<(), String> {
        if self.peek() == Some(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(message.to_string())
        }
    }
}

fn compare(predicate: impl FnOnce(f64, f64) -> bool) -> impl FnOnce(f64, f64) -> f64 {
    move |left, right| predicate(left, right) as u8 as f64
}
