use ast::expr::Expression;
use token::token_type::TokenType;

use crate::{ParseResult, Parser};

pub fn parse_bool(parser: &mut Parser) -> ParseResult<Expression> {
    Ok(Expression::Bool{
        token: parser.cur_token.clone(),
        value: parser.cur_token.token_type == TokenType::BoolTrue
    })
}
