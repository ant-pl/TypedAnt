use ast::{expr::Expression, expressions::ident::Ident};
use token::token_type::TokenType;

use crate::{ParseResult, Parser};

pub fn parse_field_access(parser: &mut Parser, left: Expression) -> ParseResult<Expression> {
    parser.expect_peek(TokenType::Ident)?;
    parser.next_token(); // 前进到标识符

    let ident = Ident {
        value: parser.cur_token.value.clone(),
        token: parser.cur_token.clone()
    };

    Ok(Expression::FieldAccess(Box::new(left), ident))
}
