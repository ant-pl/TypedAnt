use ast::{expr::Expression, expressions::ident::Ident};
use indexmap::IndexMap;
use token::token_type::TokenType;

use crate::{ParseResult, Parser, error::ParserErrorKind};

pub fn parse_build_struct(parser: &mut Parser, left: Expression) -> ParseResult<Expression> {
    let token = parser.cur_token.clone();

    parser.expect_cur(TokenType::LBrace)?;

    // WARNING: 非十足把握请勿模仿动态注入表达式解析表

    let fields = parser.parse_expression_list(TokenType::RBrace)?;

    let mut new_fields = IndexMap::new();

    for field in fields {
        let Expression::Assign { left, right, .. } = *field else {
            Err(parser.make_error(ParserErrorKind::Other, Some(format!("not an assign expression").into())))?
        };

        let Expression::Ident(name) = *left else {
            Err(parser.make_error(ParserErrorKind::Other, Some(format!("not an ident").into())))?
        };

        new_fields.insert(name, *right);
    }

    Ok(Expression::BuildStruct(Ident {
        value: left.to_string().into(),
        token: token
    }, new_fields))
}
