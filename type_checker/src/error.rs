use std::{fmt::Display, sync::Arc};

use token::token::Token;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeCheckerErrorKind {
    VariableNotFound,
    TypeNotFound,
    TypeMismatch,
    Other,
}

impl Display for TypeCheckerErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            TypeCheckerErrorKind::VariableNotFound => "variable not found",
            TypeCheckerErrorKind::TypeNotFound => "type not found",
            TypeCheckerErrorKind::TypeMismatch => "type mismatch",
            TypeCheckerErrorKind::Other => "other",
        };

        write!(f, "{s}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeCheckerError {
    pub kind: TypeCheckerErrorKind,
    pub token: Token,
    pub message: Option<Arc<str>>
}