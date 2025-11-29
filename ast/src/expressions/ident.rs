use std::{fmt::Display, rc::Rc};

use token::token::Token;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ident {
    pub token: Token,
    pub value: Rc<str>,
}

impl Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}