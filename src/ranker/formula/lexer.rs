#[derive(Debug, Clone, PartialEq)]
pub(super) enum Token {
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

pub(super) struct Lexer {
    chars: Vec<char>,
    pos: usize,
    error: bool,
}

impl Lexer {
    pub(super) fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            error: false,
        }
    }

    pub(super) fn tokenize(mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            tokens.push(token);
        }
        if self.error {
            Err("unrecognized character in expression".to_string())
        } else if tokens.is_empty() {
            Err("empty expression".to_string())
        } else {
            Ok(tokens)
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|char| char.is_ascii_whitespace()) {
            self.advance();
        }
    }

    fn read_number(&mut self) -> f64 {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.advance();
        }
        while self.peek().is_some_and(|char| char.is_ascii_digit()) {
            self.advance();
        }
        if self.peek() == Some('.') {
            self.advance();
            while self.peek().is_some_and(|char| char.is_ascii_digit()) {
                self.advance();
            }
        }
        self.chars[start..self.pos]
            .iter()
            .collect::<String>()
            .parse()
            .unwrap_or(0.0)
    }

    fn read_ident(&mut self) -> String {
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|char| char.is_ascii_alphanumeric() || char == '_')
        {
            self.advance();
        }
        self.chars[start..self.pos].iter().collect()
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let char = self.peek()?;
        if char.is_ascii_digit()
            || (char == '-'
                && self.pos + 1 < self.chars.len()
                && self.chars[self.pos + 1].is_ascii_digit())
        {
            return Some(Token::Number(self.read_number()));
        }
        match char {
            '+' => self.single(Token::Plus),
            '-' => self.single(Token::Minus),
            '*' => self.single(Token::Star),
            '/' => self.single(Token::Slash),
            '>' => self.comparison(Token::Gt, Token::Gte),
            '<' => self.comparison(Token::Lt, Token::Lte),
            '=' => self.equals(),
            '(' => self.single(Token::LParen),
            ')' => self.single(Token::RParen),
            ',' => self.single(Token::Comma),
            '@' => {
                self.advance();
                Some(Token::Variable(self.read_ident()))
            }
            char if char.is_ascii_alphabetic() => {
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

    fn single(&mut self, token: Token) -> Option<Token> {
        self.advance();
        Some(token)
    }

    fn comparison(&mut self, single: Token, inclusive: Token) -> Option<Token> {
        self.advance();
        if self.peek() == Some('=') {
            self.advance();
            Some(inclusive)
        } else {
            Some(single)
        }
    }

    fn equals(&mut self) -> Option<Token> {
        self.advance();
        if self.peek() == Some('=') {
            self.advance();
            Some(Token::Eq)
        } else {
            self.error = true;
            None
        }
    }
}
