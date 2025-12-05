use std::fmt::Display;

use token::token::Token;

use crate::stmt::Statement;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    Program {
        token: Token,
        statements: Vec<Statement>
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Program { statements, .. } => {
                write!(
                    f, "{}",
                    statements
                        .iter()
                        .map(|it| it.to_string())
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            }
        }
    }
}

pub trait GetToken {
    fn token(&self) -> Token;
}