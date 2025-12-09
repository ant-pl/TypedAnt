use std::{fmt::Display, rc::Rc};

use bigdecimal::BigDecimal;
use indexmap::IndexMap;
use token::token::Token;

use crate::{expressions::ident::Ident, node::GetToken, stmt::Statement};

#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub enum IntValue {
    I64(i64),
    I32(i32),
    I16(i16),
    I8(i8),
    ISize(isize),
    U64(u64),
    U32(u32),
    U16(u16),
    U8(u8),
    USize(usize),
}

impl Display for IntValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntValue::I64(it) => write!(f, "{it}"),
            IntValue::I32(it) => write!(f, "{it}"),
            IntValue::I16(it) => write!(f, "{it}"),
            IntValue::I8(it) => write!(f, "{it}"),
            IntValue::ISize(it) => write!(f, "{it}"),
            IntValue::U32(it) => write!(f, "{it}"),
            IntValue::U64(it) => write!(f, "{it}"),
            IntValue::U16(it) => write!(f, "{it}"),
            IntValue::U8(it) => write!(f, "{it}"),
            IntValue::USize(it) => write!(f, "{it}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    BigInt {
        token: Token,
        value: BigDecimal,
    },
    Int {
        token: Token,
        value: IntValue,
    },
    Bool {
        token: Token,
        value: bool,
    },
    Ident(Ident),
    TypeHint(Ident, Ident),
    Block(Vec<Statement>),
    BuildStruct(Ident, IndexMap<Ident, Expression>),
    FieldAccess(Box<Expression>, Ident),
    Infix {
        token: Token,
        op: Rc<str>,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Function {
        token: Token,
        name: Option<Token>,
        params: Vec<Box<Expression>>,
        generics_params: Vec<Box<Expression>>,
        block: Box<Statement>,
        ret_ty: Option<Ident>,
    },
    If {
        token: Token,
        condition: Box<Expression>,
        consequence: Box<Expression>,
        else_block: Option<Box<Expression>>,
    },
    Call {
        token: Token,
        func: Box<Expression>,
        args: Vec<Box<Expression>>,
    },
    Assign {
        token: Token,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    StrLiteral {
        token: Token,
        value: Rc<str>,
    },
    ThreeDot(Token),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ThreeDot(token) => write!(f, "{}", token.value),
            Self::BuildStruct(struct_name, block) => write!(
                f,
                "{struct_name} {{\n{}\n}}",
                block
                    .iter()
                    .map(|(name, val_expr)| format!("\t{name} = {val_expr}"))
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
            Self::FieldAccess(obj, field) => write!(f, "{obj}.{field}"),
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
            Self::BigInt { value, .. } => write!(f, "{}", value),
            Self::Bool { value, .. } => write!(f, "{}", value),
            Self::Int { value, .. } => write!(f, "{}", value),
            Self::TypeHint(ident, ty) => write!(f, "{ident}: {ty}"),
            Self::Ident(ident) => write!(f, "{}", ident),
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
            Self::Block(it) => write!(
                f,
                "{{\n{}\n}}",
                it.iter()
                    .map(|it| "\t".to_owned() + &it.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            ),
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
                    .map_or_else(|| "".into(), |it| format!(" -> {it} ")),
                block.to_string()
            ),
            Self::Infix {
                op, left, right, ..
            } => write!(f, "({}{}{})", left, op, right),
        }
    }
}

impl GetToken for Expression {
    fn token(&self) -> Token {
        match self {
            Expression::BigInt { token, .. } => token.clone(),
            Expression::Int { token, .. } => token.clone(),
            Expression::Bool { token, .. } => token.clone(),
            Expression::Ident(ident) => ident.token.clone(),
            Expression::TypeHint(ident, ..) => ident.token.clone(),
            Expression::Block(statements) => {
                if !statements.is_empty() {
                    statements[0].token()
                } else {
                    Token::eof("unknown".into(), 0, 0)
                }
            }
            Expression::BuildStruct(ident, ..) => ident.token.clone(),
            Expression::FieldAccess(expression, ..) => expression.token(),
            Expression::Infix { token, .. } => token.clone(),
            Expression::Function { token, .. } => token.clone(),
            Expression::If { token, .. } => token.clone(),
            Expression::Call { token, .. } => token.clone(),
            Expression::Assign { token, .. } => token.clone(),
            Expression::StrLiteral { token, .. } => token.clone(),
            Expression::ThreeDot(token) => token.clone(),
        }
    }
}
