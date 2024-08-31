use crate::Value;

#[derive(Debug, Clone)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier(String),
    String(String),
    Number(f64),

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

impl PartialEq for TokenType {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *(self as *const Self as *const isize) == *(other as *const Self as *const isize) }
    }
}

impl Eq for TokenType {}

impl TokenType {
    pub fn value(&self) -> Option<Box<Value>> {
        match self {
            TokenType::Number(n) => Some(Box::new(Value::Number(*n))),
            TokenType::String(s) => Some(Box::new(Value::String(s.clone()))),
            TokenType::True => Some(Box::new(Value::Boolean(true))),
            TokenType::False => Some(Box::new(Value::Boolean(false))),
            TokenType::Nil => Some(Box::new(Value::Nil)),
            _ => None,
        }
    }
}
