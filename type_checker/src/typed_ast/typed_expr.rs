use std::{fmt::Display, rc::Rc};

use ast::expr::IntValue;
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
    Block(Vec<TypedStatement>, Ty),
    TypeHint(Ident, Ident, Ty),
    BuildStruct(Ident, IndexMap<Ident, TypedExpression>, Ty),
    FieldAccess(Box<TypedExpression>, Ident, Ty),
    Infix {
        token: Token,
        op: Rc<str>,
        left: Box<TypedExpression>,
        right: Box<TypedExpression>,
        ty: Ty,
    },
    Function {
        token: Token,
        name: Option<Token>,
        params: Vec<Box<TypedExpression>>,
        block: Box<TypedStatement>,
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
        value: Rc<str>,
        ty: Ty,
    },
}

impl Display for TypedExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BuildStruct(struct_name, block, _) => write!(
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
                    &format!(" else {it}")
                } else {
                    ""
                }
            ),
            Self::BigInt { value, .. } => write!(f, "{}", value),
            Self::Bool { value, .. } => write!(f, "{}", value),
            Self::Int { value, .. } => write!(f, "{}", value),
            Self::Ident(ident, _) => write!(f, "{}", ident),
            Self::Block(it, _) => write!(
                f,
                "{{\n{}\n}}",
                it.iter()
                    .map(|it| "\t".to_owned() + &it.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            Self::TypeHint(ident, ty, _) => write!(f, "{}: {}", ident, ty),
            Self::Function {
                params,
                name,
                block,
                ret_ty,
                ..
            } => write!(
                f,
                "func {}({}){}{{{}}}",
                name.as_ref()
                    .map_or_else(|| "".into(), |it| it.value.clone()),
                params
                    .iter()
                    .map(|it| it.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
                ret_ty
                    .as_ref()
                    .map_or_else(|| "".into(), |it| format!(" -> {it} ")),
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
            Self::Block(_, ty) => ty.clone(),
            Self::Function { ty, .. } => ty.clone(),
            Self::Infix { ty, .. } => ty.clone(),
            Self::TypeHint(_, _, ty) => ty.clone(),
            Self::If { consequence, .. } => consequence.get_type(),
            Self::BuildStruct(_, _, ty) => ty.clone(),
            Self::Call { func_ty, .. } => match func_ty {
                Ty::Function { ret_type, .. } => *ret_type.clone(),
                _ => unreachable!(),
            },
            Self::Assign { right, .. } => right.get_type(),
        }
    }
}
