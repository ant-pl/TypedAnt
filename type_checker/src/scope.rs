use token::token::Token;

use ty::TyId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScopeKind {
    Global,
    Function,
    Class,
    Trait,
    Module
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckScope {
    pub kind: ScopeKind,
    pub collect_return_types: Vec<(TyId, Token)>
}