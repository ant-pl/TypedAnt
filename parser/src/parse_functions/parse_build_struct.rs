use ast::{expr::Expression, expressions::ident::Ident};
use indexmap::IndexMap;
use token::token_type::TokenType;

use crate::{ParseResult, Parser, error::ParserErrorKind};

pub fn parse_build_struct(parser: &mut Parser) -> ParseResult<Expression> {
    let token = parser.cur_token.clone();
    
    parser.next_token(); // 离开 new 前进到 Ident

    parser.expect_cur(TokenType::Ident)?;

    let ident = Ident {
        token: parser.cur_token.clone(),
        value: parser.cur_token.value.clone()
    };

    parser.next_token(); // 离开标识符

    parser.expect_cur(TokenType::LBrace)?;

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

    Ok(Expression::BuildStruct(token, ident, new_fields))
}
