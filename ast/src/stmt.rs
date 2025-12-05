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
    Struct {
        token: Token,
        name: Ident,
        fields: Vec<Box<Expression>>,
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
}

impl Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
            Statement::Return { token, .. } => token.clone(),
            Statement::Block { token, .. } => token.clone(),
            Statement::While { token, .. } => token.clone(),
            Statement::Let { token, .. } => token.clone(),
            Statement::Struct { token, .. } => token.clone(),
            Statement::Extern { token, .. } => token.clone(),
        }
    }
}
