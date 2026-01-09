use std::{fmt::Display, sync::Arc};

use ast::{expr::IntValue, node::GetToken};
use bigdecimal::BigDecimal;
use indexmap::IndexMap;
use token::token::Token;

use crate::{
    Ty,
    typed_ast::{GetType, typed_expressions::ident::Ident, typed_stmt::TypedStatement},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedExpression {
    BigInt {
        token: Token,
        value: BigDecimal,
        ty: Ty,
    },
    Bool {
        token: Token,
        value: bool,
        ty: Ty,
    },
    Int {
        token: Token,
        value: IntValue,
        ty: Ty,
    },
    Ident(Ident, Ty),
    Block(Token, Vec<TypedStatement>, Ty),
    TypeHint(Ident, Ident, Ty),
    BuildStruct(Token, Ident, IndexMap<Ident, TypedExpression>, Ty),
    FieldAccess(Box<TypedExpression>, Ident, Ty),
    Infix {
        token: Token,
        op: Arc<str>,
        left: Box<TypedExpression>,
        right: Box<TypedExpression>,
        ty: Ty,
    },
    Function {
        token: Token,
        name: Option<Token>,
        params: Vec<Box<TypedExpression>>,
        generics_params: Vec<Box<TypedExpression>>,
        block: Box<TypedExpression>,
        ret_ty: Option<Ident>,
        ty: Ty,
    },
    Call {
        token: Token,
        func: Box<TypedExpression>,
        args: Vec<Box<TypedExpression>>,
        func_ty: Ty,
    },
    If {
        token: Token,
        condition: Box<TypedExpression>,
        consequence: Box<TypedExpression>,
        else_block: Option<Box<TypedExpression>>,
    },
    Assign {
        token: Token,
        left: Box<TypedExpression>,
        right: Box<TypedExpression>,
    },
    StrLiteral {
        token: Token,
        value: Arc<str>,
        ty: Ty,
    },
}

impl Display for TypedExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BuildStruct(_, struct_name, block, _) => write!(
                f,
                "{struct_name} {{\n{}\n}}",
                block
                    .iter()
                    .map(|(name, val_expr)| format!("\t{name} = {val_expr}"))
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            Self::FieldAccess(obj, field, _) => write!(f, "{obj}.{field}"),
            Self::StrLiteral { value, .. } => write!(f, "\"{value}\""),
            Self::Assign { left, right, .. } => write!(f, "{left} = {right}"),
            Self::Call { func, args, .. } => write!(
                f,
                "{func}({})",
                args.iter()
                    .map(|it| it.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::If {
                condition,
                consequence,
                else_block,
                ..
            } => write!(
                f,
                "if {condition} {consequence}{}",
                if let Some(it) = else_block {
                    format!(" else {it}")
                } else {
                    "".to_string()
                }
            ),
            Self::BigInt { value, .. } => write!(f, "{}", value),
            Self::Bool { value, .. } => write!(f, "{}", value),
            Self::Int { value, .. } => write!(f, "{}", value),
            Self::Ident(ident, _) => write!(f, "{}", ident),
            Self::Block(_, it, _) => write!(
                f,
                "{{\n{}\n}}",
                it.iter()
                    .map(|it| it
                        .to_string()
                        .split("\n")
                        .map(|it| "\t".to_owned() + it)
                        .collect::<Vec<String>>()
                        .join("\n"))
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            Self::TypeHint(ident, ty, _) => write!(f, "{}: {}", ident, ty),
            Self::Function {
                params,
                name,
                block,
                ret_ty,
                generics_params,
                ..
            } => write!(
                f,
                "func {}{}({}){}{}",
                name.as_ref()
                    .map_or_else(|| "".into(), |it| it.value.clone()),
                if generics_params.is_empty() {
                    "".to_owned()
                } else {
                    "<".to_owned()
                        + &generics_params
                            .iter()
                            .map(|it| it.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                        + ">"
                },
                params
                    .iter()
                    .map(|it| it.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
                ret_ty
                    .as_ref()
                    .map_or_else(|| " ".into(), |it| format!(" -> {it} ")),
                block.to_string()
            ),
            Self::Infix {
                op, left, right, ..
            } => write!(f, "({}{}{})", left, op, right),
        }
    }
}

impl GetType for TypedExpression {
    fn get_type(&self) -> Ty {
        match self {
            Self::FieldAccess(_, _, field_ty) => field_ty.clone(),
            Self::StrLiteral { ty, .. } => ty.clone(),
            Self::BigInt { ty, .. } => ty.clone(),
            Self::Int { ty, .. } => ty.clone(),
            Self::Bool { ty, .. } => ty.clone(),
            Self::Ident(_, ty) => ty.clone(),
            Self::Block(_, _, ty) => ty.clone(),
            Self::Function { ty, .. } => ty.clone(),
            Self::Infix { ty, .. } => ty.clone(),
            Self::TypeHint(_, _, ty) => ty.clone(),
            Self::If { consequence, .. } => consequence.get_type(),
            Self::BuildStruct(_, _, _, ty) => ty.clone(),
            Self::Call { func_ty, .. } => match func_ty {
                Ty::Function { ret_type, .. } => *ret_type.clone(),
                _ => unreachable!(),
            },
            Self::Assign { right, .. } => right.get_type(),
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
            TypedExpression::FieldAccess(typed_expression, ..) => typed_expression.token(),
            TypedExpression::Infix { token, .. } => token.clone(),
            TypedExpression::Function { token, .. } => token.clone(),
            TypedExpression::Call { token, .. } => token.clone(),
            TypedExpression::If { token, .. } => token.clone(),
            TypedExpression::Assign { token, .. } => token.clone(),
            TypedExpression::StrLiteral { token, .. } => token.clone(),
        }
    }
}
