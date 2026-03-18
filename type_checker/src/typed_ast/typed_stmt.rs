use std::sync::Arc;

use ast::{ExprId, StmtId, node::GetToken};
use indexmap::IndexMap;
use token::token::Token;

use crate::{
    ty::TyId,
    typed_ast::{GetType, SetType, typed_expressions::ident::Ident},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedStatement {
    ExpressionStatement(Token, ExprId, TyId),
    Return {
        token: Token,
        expr: Option<ExprId>,
        ty: TyId,
    },
    Block {
        token: Token,
        statements: Vec<StmtId>,
        ty: TyId,
    },
    Let {
        token: Token,
        name: Ident,
        var_type: Option<Ident>,
        value: ExprId,
        ty: TyId,
    },
    Const {
        token: Token,
        name: Ident,
        var_type: Option<Ident>,
        value: ExprId,
        ty: TyId,
    },
    While {
        token: Token,
        condition: ExprId,
        block: StmtId,
        ty: TyId,
    },
    Struct {
        token: Token,
        name: Ident,
        fields: Vec<ExprId>,
        ty: TyId,
        generics: Vec<ExprId>,
    },
    Trait {
        token: Token,
        name: Ident,
        block: StmtId,
        ty: TyId,
    },
    Extern {
        token: Token,
        abi: Token,
        extern_func_name: Token,
        alias: Token,
        params: Vec<ExprId>,
        ret_ty: Option<ExprId>,
        ty: TyId,
        vararg: bool,
    },
    FuncDecl {
        token: Token,
        name: Token,
        params: Vec<ExprId>,
        generics_params: Vec<ExprId>,
        ret_ty: Option<Ident>,
        ty: TyId,
    },
    Impl {
        token: Token,
        impl_: Ident,
        for_: Option<Ident>,
        block: StmtId,
        new_fields: IndexMap<Arc<str>, TyId>,
        ty: TyId,
    },
}

impl GetType for TypedStatement {
    fn get_type(&self) -> TyId {
        match self {
            Self::Extern { ty, .. } => *ty,
            Self::Block { ty, .. } => *ty,
            Self::ExpressionStatement(_, _, ty) => *ty,
            Self::Return { ty, .. } => *ty,
            Self::Let { ty, .. } => *ty,
            Self::Const { ty, .. } => *ty,
            Self::Struct { ty, .. } => *ty,
            Self::FuncDecl { ty, .. } => *ty,
            Self::Trait { ty, .. } => *ty,
            Self::While { ty, .. } => *ty,
            Self::Impl { ty, .. } => *ty,
        }
    }
}

impl SetType for TypedStatement {
    fn set_type(&mut self, new_ty: TyId) {
        match self {
            Self::Extern { ty, .. } => *ty = new_ty,
            Self::Block { ty, .. } => *ty = new_ty,
            Self::ExpressionStatement(_, _, ty) => *ty = new_ty,
            Self::Return { ty, .. } => *ty = new_ty,
            Self::Let { ty, .. } => *ty = new_ty,
            Self::Const { ty, .. } => *ty = new_ty,
            Self::Struct { ty, .. } => *ty = new_ty,
            Self::FuncDecl { ty, .. } => *ty = new_ty,
            Self::Trait { ty, .. } => *ty = new_ty,
            Self::While { ty, .. } => *ty = new_ty,
            Self::Impl { ty, .. } => *ty = new_ty,
        }
    }
}

impl GetToken for TypedStatement {
    fn token(&self) -> Token {
        match self {
            TypedStatement::ExpressionStatement(token, ..) => token.clone(),
            TypedStatement::Return { token, .. } => token.clone(),
            TypedStatement::Block { token, .. } => token.clone(),
            TypedStatement::Let { token, .. } => token.clone(),
            TypedStatement::Const { token, .. } => token.clone(),
            TypedStatement::While { token, .. } => token.clone(),
            TypedStatement::Struct { token, .. } => token.clone(),
            TypedStatement::Trait { token, .. } => token.clone(),
            TypedStatement::Extern { token, .. } => token.clone(),
            TypedStatement::FuncDecl { token, .. } => token.clone(),
            TypedStatement::Impl { token, .. } => token.clone(),
        }
    }
}
