//! Scanner for rlox
use crate::error::Error;
use crate::token::Token;
use crate::token_type::TokenType;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::str::Chars;

/// Map to identify keywords
static KEYWORDS: Lazy<HashMap<&str, TokenType>> = Lazy::new(|| {
    HashMap::from([
        ("and", TokenType::And),
        ("class", TokenType::Class),
        ("else", TokenType::Else),
        ("false", TokenType::False),
        ("for", TokenType::For),
        ("fun", TokenType::Fun),
        ("if", TokenType::If),
        ("nil", TokenType::Nil),
        ("or", TokenType::Or),
        ("print", TokenType::Print),
        ("return", TokenType::Return),
        ("super", TokenType::Super),
        ("this", TokenType::This),
        ("true", TokenType::True),
        ("var", TokenType::Var),
        ("while", TokenType::While),
    ])
});

/// Scanner
pub struct Scanner<'a> {
    source: &'a String,
    tokens: Vec<Token>,
    /// The index of the token we current consider
    start: Position<'a>,
    /// The index of the char we current consider
    current: Position<'a>,
    line: usize,
}

/// An iterator to handle the stream of source code.
///
/// May support utf-8, but is not tested, so it's now only stable for ascii
#[derive(Clone)]
pub struct Position<'a> {
    iter: Chars<'a>,
    index: usize,
    end: usize,
    current: Option<char>,
}

impl<'a> Position<'a> {
    pub fn new(iter: Chars<'a>, index: usize, end: usize) -> Self {
        let mut iter = iter.clone();
        let current = iter.next();
        Self {
            iter,
            index,
            end,
            current,
        }
    }
    pub fn is_end(&self) -> bool {
        self.index >= self.end
    }
    /// Get the string of [a, b)(if a is in the front of b).
    ///
    /// And the method will consume the former iterator
    pub fn get_between(a: &mut Self, b: &mut Self) -> Option<String> {
        if a.index == a.end || b.index == b.end {
            return None;
        }
        let mut ret = String::new();
        if a.index < b.index {
            while a.index < b.index {
                ret.push(a.next().unwrap());
            }
        } else {
            while b.index < a.index {
                ret.push(b.next().unwrap());
            }
        }
        Some(ret)
    }
    /// peek that return '\0' if meet the end
    pub fn peek(&self) -> char {
        if self.current.is_none() {
            '\0'
        } else {
            self.current.unwrap()
        }
    }
    /// peek that return `None` if meet the end
    pub fn peek_opt(&self) -> Option<char> {
        self.current
    }
    pub fn peek_next(&self) -> char {
        let mut tmp = self.clone();
        tmp.next();
        tmp.peek()
    }
}

impl<'a> Iterator for Position<'a> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        let current = self.current;
        self.current = self.iter.next();
        current
    }
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a String) -> Self {
        let end = source.chars().count();
        Self {
            source,
            tokens: Vec::new(),
            start: Position::new(source.chars(), 0, end),
            current: Position::new(source.chars(), 0, end),
            line: 1,
        }
    }

    pub fn scan_tokens(&'a mut self) -> Result<Vec<Token>, Error> {
        while !self.current.is_end() {
            self.start = self.current.clone();
            if let Err(e) = self.scan_token() {
                return Err(e);
            }
        }
        self.add_token(TokenType::Eof);

        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<(), Error> {
        match self.current.next() {
            Some('(') => self.add_token(TokenType::LeftParen),
            Some(')') => self.add_token(TokenType::RightParen),
            Some('{') => self.add_token(TokenType::LeftBrace),
            Some('}') => self.add_token(TokenType::RightBrace),
            Some(',') => self.add_token(TokenType::Comma),
            Some('.') => self.add_token(TokenType::Dot),
            Some('-') => self.add_token(TokenType::Minus),
            Some('+') => self.add_token(TokenType::Plus),
            Some(';') => self.add_token(TokenType::Semicolon),
            Some('*') => self.add_token(TokenType::Star),
            Some('!') => {
                if self.is_match('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            }
            Some('=') => {
                if self.is_match('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            }
            Some('<') => {
                if self.is_match('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            }
            Some('>') => {
                if self.is_match('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            }
            Some('/') => {
                if self.is_match('/') {
                    while self.current.peek() != '\n' && !self.current.is_end() {
                        self.current.next();
                    }
                } else {
                    self.add_token(TokenType::Slash)
                }
            }
            Some(' ') | Some('\r') | Some('\t') => {}
            Some('\n') => {
                self.line += 1;
            }
            Some('"') => {
                if let Err(e) = self.take_string() {
                    return Err(e);
                }
            }
            Some(c) => {
                if c.is_ascii_digit() {
                    if let Err(e) = self.take_number(c) {
                        return Err(e);
                    }
                } else if is_valid_start(c) {
                    self.take_identifier(c);
                } else {
                    return Err(Error::new(
                        self.line,
                        "".to_string(),
                        "Unexpected character.".to_string(),
                    ));
                }
            }
            None => self.add_token(TokenType::Eof),
        }
        Ok(())
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = if token_type != TokenType::Eof {
            if let Some(t) = Position::get_between(&mut self.start, &mut self.current) {
                t
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };
        self.tokens.push(Token {
            token_type,
            lexeme: text,
            line: self.line,
        })
    }

    fn add_token_literal(&mut self, token_type: TokenType) {
        let text = Position::get_between(&mut self.start, &mut self.current).unwrap();
        self.tokens.push(Token {
            token_type,
            lexeme: text,
            line: self.line,
        });
    }

    fn is_match(&mut self, target: char) -> bool {
        if self.current.is_end() {
            return false;
        }
        if self.current.peek() == target {
            self.current.next();
            true
        } else {
            false
        }
    }

    fn take_string(&mut self) -> Result<(), Error> {
        let mut value = "".to_string();
        while self.current.peek() != '"' && !self.current.is_end() {
            if self.current.peek() == '\n' {
                self.line += 1;
            }
            if self.current.peek() != '"' {
                value.push(self.current.next().unwrap());
            }
        }

        if self.current.is_end() {
            return Err(Error::new(
                self.line,
                "".to_string(),
                "Unterminated string.".to_string(),
            ));
        }

        self.current.next();

        self.add_token_literal(TokenType::String(value));
        Ok(())
    }

    fn take_number(&mut self, current: char) -> Result<(), Error> {
        let mut value = current.to_string();

        while !self.current.is_end() && self.current.peek().is_ascii_digit() {
            value.push(self.current.next().unwrap());
        }

        if self.current.peek() == '.' && self.current.peek_next().is_ascii_digit() {
            value.push(self.current.next().unwrap());

            while self.current.peek().is_ascii_digit() {
                value.push(self.current.next().unwrap());
            }
        }
        self.add_token_literal(TokenType::Number(value.parse::<f64>().unwrap()));
        Ok(())
    }

    fn take_identifier(&mut self, current: char) {
        let mut value = current.to_string();
        while is_valid(self.current.peek()) {
            value.push(self.current.next().unwrap());
        }
        let token_type = KEYWORDS.get(value.as_str());
        if let Some(token_type) = token_type {
            self.add_token(token_type.clone())
        } else {
            self.add_token(TokenType::Identifier(value))
        }
    }
}

fn is_valid_start(a: char) -> bool {
    a.is_ascii_alphabetic() || a == '_'
}

fn is_valid(a: char) -> bool {
    a.is_ascii_alphanumeric() || a == '_'
}
