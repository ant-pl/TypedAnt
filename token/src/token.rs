use std::{fmt::Display, sync::Arc};

use crate::token_type::TokenType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub value: Arc<str>,
    pub token_type: TokenType,

    pub line: usize,
    pub column: usize,
    pub file: Arc<str>,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Token {
    pub fn new(
        value: Arc<str>,
        token_type: TokenType,

        file: Arc<str>,
        line: usize,
        column: usize,
    ) -> Token {
        Token {
            value,
            token_type,
            line,
            column,
            file,
        }
    }

    pub fn eof(file: Arc<str>, line: usize, column: usize) -> Token {
        Token::new("\0".into(), TokenType::Eof, file, line, column)
    }
}
