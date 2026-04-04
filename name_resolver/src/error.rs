use std::{fmt::Display, sync::Arc};

use parser::error::ParserErrorKind;
use token::token::Token;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NameResolverErrorKind {
    Unresolvedimport,
    SymbolIsPrivate,
    ParserError(ParserErrorKind),
    Other
}

impl Display for NameResolverErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Unresolvedimport => "unresolved import",
            Self::SymbolIsPrivate => "the symbol is private",
            Self::ParserError(it) => &it.to_string(),
            Self::Other => "",
        };

        write!(f, "{s}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameResolverError {
    pub kind: NameResolverErrorKind,
    pub token: Token,
    pub message: Option<Arc<str>>
}