use token::token::Token;

use crate::ty::TyId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constraint {
    pub expected: TyId,
    pub got: TyId,
    pub token: Token,
}

impl Constraint {
    pub fn new(expected: TyId, got: TyId, token: Token) -> Self {
        Self { expected, got, token }
    }
}
