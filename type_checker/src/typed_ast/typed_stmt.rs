use std::fmt::Display;

use token::token::Token;

use crate::{
    Ty,
    typed_ast::{GetType, typed_expr::TypedExpression, typed_expressions::ident::Ident},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedStatement {
    ExpressionStatement(TypedExpression),
    Return {
        token: Token,
        expr: TypedExpression,
        ty: Ty,
    },
    Block {
        token: Token,
        statements: Vec<TypedStatement>,
        ty: Ty,
    },
    Let {
        token: Token,
        name: Ident,
        var_type: Option<Ident>,
        value: TypedExpression,
        ty: Ty,
    },
    While {
        token: Token,
        condition: TypedExpression,
        block: Box<TypedStatement>,
    },
    Struct {
        token: Token,
        name: Ident,
        fields: Vec<TypedExpression>,
        ty: Ty,
    },
    Trait {
        token: Token,
        name: Ident,
        block: Box<TypedStatement>,
        ty: Ty,
    },
    Extern {
        token: Token,
        abi: Token,
        extern_func_name: Token,
        alias: Token,
        params: Vec<Box<TypedExpression>>,
        ret_ty: Ident,
        ty: Ty,
        vararg: bool,
    },
    FuncDecl {
        token: Token,
        name: Token,
        params: Vec<Box<TypedExpression>>,
        generics_params: Vec<Box<TypedExpression>>,
        ret_ty: Option<Ident>,
        ty: Ty
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
                    "<".to_owned() + 
                    &generics_params
                        .iter()
                        .map(|it| it.to_string())
                        .collect::<Vec<String>>()
                        .join(", ") + 
                    ">"
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
                f, "extern \"{abi}\" {extern_func_name}({}{}) -> {ret_ty} as {alias}",
                params
                    .iter()
                    .map(|it| it.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
                if *vararg { ", ..." } else { "" }
                
            ),
            Self::Struct { name, fields, .. } => write!(
                f,
                "struct {name} {}",
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
            Self::Trait { name, block, .. } => write!(
                f,
                "trait {name} {block}",
            ),
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
            Self::Block { statements, .. } => write!(
                f,
                "{{{}}}",
                statements
                    .iter()
                    .map(|it| it.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
        }
    }
}

impl GetType for TypedStatement {
    fn get_type(&self) -> Ty {
        match self {
            Self::Extern { ty, .. } => ty.clone(),
            Self::Block { ty, .. } => ty.clone(),
            Self::ExpressionStatement(expr) => expr.get_type(),
            Self::Return { ty, .. } => ty.clone(),
            Self::Let { ty, .. } => ty.clone(),
            Self::Struct { ty, .. } => ty.clone(),
            Self::FuncDecl { ty, .. } => ty.clone(),
            Self::Trait { ty, .. } => ty.clone(),
            Self::While { .. } => Ty::Unit,
        }
    }
}
