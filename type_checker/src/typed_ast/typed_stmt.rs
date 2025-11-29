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
}

impl Display for TypedStatement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
                "{}",
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
            Self::Block { ty, .. } => ty.clone(),
            Self::ExpressionStatement(expr) => expr.get_type(),
            Self::Return { ty, .. } => ty.clone(),
            Self::Let { ty, .. } => ty.clone(),
            Self::Struct { ty, .. } => ty.clone(),
            Self::While { .. } => Ty::Unit,
        }
    }
}
