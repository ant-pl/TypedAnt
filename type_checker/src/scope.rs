use crate::ty::Ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScopeKind {
    Global,
    Function,
    Class
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckScope {
    pub kind: ScopeKind,
    pub collect_return_types: Vec<Ty>
}