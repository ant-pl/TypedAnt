use std::sync::Arc;

use ast::{ExprId, StmtId, expr::IntValue, node::GetToken};
use bigdecimal::BigDecimal;
use indexmap::IndexMap;
use token::token::Token;

use crate::{
    ty::TyId,
    typed_ast::{GetType, SetType, typed_expressions::ident::Ident},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedExpression {
    BigInt {
        token: Token,
        value: BigDecimal,
        ty: TyId,
    },
    Bool {
        token: Token,
        value: bool,
        ty: TyId,
    },
    Int {
        token: Token,
        value: IntValue,
        ty: TyId,
    },
    Ident(Ident, TyId),
    Block(Token, Vec<StmtId>, TyId),
    TypeHint(Ident, ExprId, TyId),
    SizeOf(Token, ExprId, TyId),
    BuildStruct(Token, Ident, IndexMap<Ident, ExprId>, TyId),
    FieldAccess(Token, ExprId, Ident, TyId),
    Infix {
        token: Token,
        op: Arc<str>,
        left: ExprId,
        right: ExprId,
        ty: TyId,
    },
    Cast {
        token: Token,
        val: ExprId,
        cast_to: ExprId,
        ty: TyId,
    },
    Prefix {
        token: Token,
        op: Arc<str>,
        right: ExprId,
        ty: TyId,
    },
    Function {
        token: Token,
        name: Option<Token>,
        params: Vec<ExprId>,
        generics_params: Vec<ExprId>,
        block: ExprId,
        ret_ty: Option<ExprId>,
        ty: TyId,
    },
    Call {
        token: Token,
        func: ExprId,
        args: Vec<ExprId>,
        func_ty: TyId,
        ret_ty: TyId,
    },
    If {
        token: Token,
        condition: ExprId,
        consequence: ExprId,
        else_block: Option<ExprId>,
        ty: TyId
    },
    Assign {
        token: Token,
        left: ExprId,
        right: ExprId,
        ty: TyId
    },
    StrLiteral {
        token: Token,
        value: Arc<str>,
        ty: TyId,
    },
    BoolAnd {
        token: Token,
        left: ExprId,
        right: ExprId,
        ty: TyId,
    },
    BoolOr {
        token: Token,
        left: ExprId,
        right: ExprId,
        ty: TyId,
    },
    TypePath {
        token: Token,
        left: Ident,
        paths: Vec<ExprId>,
        ty: TyId,
    },
}

impl GetType for TypedExpression {
    fn get_type(&self) -> TyId {
        match self {
            Self::FieldAccess(_, _, _, field_ty) => field_ty.clone(),
            Self::StrLiteral { ty, .. } => *ty,
            Self::BigInt { ty, .. } => *ty,
            Self::Int { ty, .. } => *ty,
            Self::Bool { ty, .. } => *ty,
            Self::Ident(_, ty) => *ty,
            Self::Block(_, _, ty) => *ty,
            Self::Function { ty, .. } => *ty,
            Self::Infix { ty, .. } => *ty,
            Self::Cast { ty, .. } => *ty,
            Self::Prefix { ty, .. } => *ty,
            Self::TypeHint(_, _, ty) => *ty,
            Self::If { ty, .. } => *ty,
            Self::BuildStruct(_, _, _, ty) => *ty,
            Self::Call { ret_ty, .. } => *ret_ty,
            Self::Assign { ty, .. } => *ty,
            Self::BoolAnd { ty, .. } => *ty,
            Self::BoolOr { ty, .. } => *ty,
            Self::TypePath { ty, .. } => *ty,
            Self::SizeOf(_, _, ty) => *ty,
        }
    }
}

impl SetType for TypedExpression {
    fn set_type(&mut self, new_ty: TyId) {
        match self {
            Self::FieldAccess(_, _, _, field_ty) => *field_ty = new_ty,
            Self::StrLiteral { ty, .. } => *ty = new_ty,
            Self::BigInt { ty, .. } => *ty = new_ty,
            Self::Int { ty, .. } => *ty = new_ty,
            Self::Bool { ty, .. } => *ty = new_ty,
            Self::Ident(_, ty) => *ty = new_ty,
            Self::Block(_, _, ty) => *ty = new_ty,
            Self::Function { ty, .. } => *ty = new_ty,
            Self::Infix { ty, .. } => *ty = new_ty,
            Self::Cast { ty, .. } => *ty = new_ty,
            Self::Prefix { ty, .. } => *ty = new_ty,
            Self::TypeHint(_, _, ty) => *ty = new_ty,
            Self::If { ty, .. } => *ty = new_ty,
            Self::BuildStruct(_, _, _, ty) => *ty = new_ty,
            Self::Call { ret_ty, .. } => *ret_ty = new_ty,
            Self::Assign { ty, .. } => *ty = new_ty,
            Self::BoolAnd { ty, .. } => *ty = new_ty,
            Self::BoolOr { ty, .. } => *ty = new_ty,
            Self::TypePath { ty, .. } => *ty = new_ty,
            Self::SizeOf(_, _, ty) => *ty = new_ty,
        }
    }
}

impl GetToken for TypedExpression {
    fn token(&self) -> Token {
        match self {
            TypedExpression::BigInt { token, .. } => token.clone(),
            TypedExpression::Bool { token, .. } => token.clone(),
            TypedExpression::Int { token, .. } => token.clone(),
            TypedExpression::Ident(ident, ..) => ident.token.clone(),
            TypedExpression::Block(token, ..) => token.clone(),
            TypedExpression::TypeHint(ident, ..) => ident.token.clone(),
            TypedExpression::BuildStruct(token, ..) => token.clone(),
            TypedExpression::SizeOf(token, ..) => token.clone(),
            TypedExpression::FieldAccess(token, ..) => token.clone(),
            TypedExpression::Infix { token, .. } => token.clone(),
            TypedExpression::Cast { token, .. } => token.clone(),
            TypedExpression::Prefix { token, .. } => token.clone(),
            TypedExpression::Function { token, .. } => token.clone(),
            TypedExpression::Call { token, .. } => token.clone(),
            TypedExpression::If { token, .. } => token.clone(),
            TypedExpression::Assign { token, .. } => token.clone(),
            TypedExpression::StrLiteral { token, .. } => token.clone(),
            TypedExpression::BoolAnd { token, .. } => token.clone(),
            TypedExpression::BoolOr { token, .. } => token.clone(),
            TypedExpression::TypePath { token, .. } => token.clone(),
        }
    }
}

impl TypedExpression {
    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Self::Bool { .. } | Self::Int { .. } | Self::StrLiteral { .. }
        )
    }
}
