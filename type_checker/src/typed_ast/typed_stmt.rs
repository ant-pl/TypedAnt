use std::{fmt::Display, sync::Arc};

use ast::node::GetToken;
use indexmap::IndexMap;
use token::token::Token;

use crate::{
    ty::TyId,
    typed_ast::{GetType, typed_expr::TypedExpression, typed_expressions::ident::Ident},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedStatement {
    ExpressionStatement(TypedExpression),
    Return {
        token: Token,
        expr: TypedExpression,
        ty: TyId,
    },
    Block {
        token: Token,
        statements: Vec<TypedStatement>,
        ty: TyId,
    },
    Let {
        token: Token,
        name: Ident,
        var_type: Option<Ident>,
        value: TypedExpression,
        ty: TyId,
    },
    Const {
        token: Token,
        name: Ident,
        var_type: Option<Ident>,
        value: TypedExpression,
        ty: TyId,
    },
    While {
        token: Token,
        condition: TypedExpression,
        block: Box<TypedStatement>,
        ty: TyId,
    },
    Struct {
        token: Token,
        name: Ident,
        fields: Vec<TypedExpression>,
        ty: TyId,
        generics: Vec<Box<TypedExpression>>,
    },
    Trait {
        token: Token,
        name: Ident,
        block: Box<TypedStatement>,
        ty: TyId,
    },
    Extern {
        token: Token,
        abi: Token,
        extern_func_name: Token,
        alias: Token,
        params: Vec<Box<TypedExpression>>,
        ret_ty: Ident,
        ty: TyId,
        vararg: bool,
    },
    FuncDecl {
        token: Token,
        name: Token,
        params: Vec<Box<TypedExpression>>,
        generics_params: Vec<Box<TypedExpression>>,
        ret_ty: Option<Ident>,
        ty: TyId,
    },
    Impl {
        token: Token,
        impl_: Ident,
        for_: Option<Ident>,
        block: Box<TypedStatement>,
        new_fields: IndexMap<Arc<str>, TyId>,
        ty: TyId,
    },
}

impl Display for TypedStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FuncDecl {
                params,
                name,
                ret_ty,
                generics_params,
                ..
            } => write!(
                f,
                "func {}{}({}){};",
                name.value,
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
            ),
            Self::Extern {
                abi,
                extern_func_name,
                params,
                ret_ty,
                alias,
                vararg,
                ..
            } => write!(
                f,
                "extern \"{abi}\" {extern_func_name}({}{}) -> {ret_ty} as {alias}",
                params
                    .iter()
                    .map(|it| it.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
                if *vararg { ", ..." } else { "" }
            ),
            Self::Struct {
                name,
                fields,
                generics,
                ..
            } => write!(
                f,
                "struct {name}{} {}",
                if generics.is_empty() {
                    "".to_string()
                } else {
                    "<".to_owned()
                        + &generics
                            .iter()
                            .map(|it| it.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                        + ">"
                },
                if fields.is_empty() {
                    "{}".to_string()
                } else {
                    format!(
                        "{{\n{}\n}}",
                        fields
                            .iter()
                            .map(|it| "\t".to_owned() + &it.to_string())
                            .collect::<Vec<String>>()
                            .join("\n")
                    )
                }
            ),
            Self::Trait { name, block, .. } => write!(f, "trait {name} {block}",),
            Self::While {
                condition, block, ..
            } => write!(f, "while {condition} {block} "),
            Self::ExpressionStatement(expr) => expr.fmt(f),
            Self::Return { expr, .. } => write!(f, "return {expr}"),
            Self::Let {
                name,
                var_type,
                value,
                ..
            } => match var_type {
                Some(ty) => write!(f, "let {name}: {ty} = {value}",),
                None => write!(f, "let {name} = {value}"),
            },
            Self::Const {
                name,
                var_type,
                value,
                ..
            } => match var_type {
                Some(ty) => write!(f, "const {name}: {ty} = {value}",),
                None => write!(f, "const {name} = {value}"),
            },
            Self::Block { statements, .. } => write!(
                f,
                "{{\n{}\n}}",
                statements
                    .iter()
                    .map(|it| it
                        .to_string()
                        .split("\n")
                        .map(|it| "\t".to_owned() + it)
                        .collect::<Vec<String>>()
                        .join("\n"))
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            Self::Impl {
                impl_, block, for_, ..
            } => write!(
                f,
                "impl {impl_}{} {block}",
                for_.as_ref()
                    .map_or_else(String::new, |it| format!(" for {}", it.value.clone()))
            ),
        }
    }
}

impl GetType for TypedStatement {
    fn get_type(&self) -> TyId {
        match self {
            Self::Extern { ty, .. } => *ty,
            Self::Block { ty, .. } => *ty,
            Self::ExpressionStatement(expr) => expr.get_type(),
            Self::Return { ty, .. } => *ty,
            Self::Let { ty, .. } => *ty,
            Self::Const { ty, .. } => *ty,
            Self::Struct { ty, .. } => *ty,
            Self::FuncDecl { ty, .. } => *ty,
            Self::Trait { ty, .. } => *ty,
            Self::While { ty, .. } => *ty,
            Self::Impl { ty, .. } => *ty
        }
    }
}

impl GetToken for TypedStatement {
    fn token(&self) -> Token {
        match self {
            TypedStatement::ExpressionStatement(typed_expression) => typed_expression.token(),
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
