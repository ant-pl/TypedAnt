use std::fmt::Display;

use token::token::Token;

use crate::{expr::Expression, expressions::ident::Ident, node::GetToken};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Statement {
    ExpressionStatement(Expression),
    Return {
        token: Token,
        expr: Expression,
    },
    Block {
        token: Token,
        statements: Vec<Statement>,
    },
    While {
        token: Token,
        condition: Expression,
        block: Box<Statement>,
    },
    Let {
        token: Token,
        name: Ident,
        var_type: Option<Ident>,
        value: Expression,
    },
    Const {
        token: Token,
        name: Ident,
        var_type: Option<Ident>,
        value: Expression,
    },
    Struct {
        token: Token,
        name: Ident,
        fields: Vec<Box<Expression>>,
        generics: Vec<Box<Expression>>,
    },
    Trait {
        token: Token,
        name: Ident,
        block: Box<Statement>,
    },
    Impl {
        token: Token,
        impl_: Ident,
        for_: Option<Ident>,
        block: Box<Statement>,
    },
    Extern {
        token: Token,
        abi: Token,
        vararg: bool,
        extern_func_name: Token,
        alias: Token,
        params: Vec<Box<Expression>>,
        ret_ty: Ident,
    },
    FuncDecl {
        token: Token,
        name: Token,
        params: Vec<Box<Expression>>,
        generics_params: Vec<Box<Expression>>,
        ret_ty: Option<Ident>,
    },
}

impl Display for Statement {
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
                    "<".to_owned() + &generics
                        .iter()
                        .map(|it| it.to_string())
                        .collect::<Vec<String>>()
                        .join(", ") + ">"
                },
                if fields.is_empty() {
                    "{}".to_string()
                } else {
                    format!(
                        "\n{}\n",
                        fields
                            .iter()
                            .map(|it| "\t".to_owned() + &it.to_string())
                            .collect::<Vec<String>>()
                            .join("\n")
                    )
                }
            ),
            Self::Trait { name, block, .. } => write!(f, "trait {name} {block}"),
            Self::Impl {
                impl_, block, for_, ..
            } => write!(
                f,
                "impl {impl_}{} {block}",
                for_.as_ref()
                    .map_or_else(String::new, |it| format!(" for {}", it.value.clone()))
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
                    .map(|it| "\t".to_owned() + &it.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
        }
    }
}

impl GetToken for Statement {
    fn token(&self) -> Token {
        match self {
            Statement::ExpressionStatement(expression) => expression.token(),
            Statement::FuncDecl { token, .. } => token.clone(),
            Statement::Return { token, .. } => token.clone(),
            Statement::Block { token, .. } => token.clone(),
            Statement::While { token, .. } => token.clone(),
            Statement::Let { token, .. } => token.clone(),
            Statement::Const { token, .. } => token.clone(),
            Statement::Struct { token, .. } => token.clone(),
            Statement::Trait { token, .. } => token.clone(),
            Statement::Extern { token, .. } => token.clone(),
            Statement::Impl { token, .. } => token.clone(),
        }
    }
}
