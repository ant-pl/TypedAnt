use ast::expr::Expression;
use token::token_type::TokenType;

use crate::{
    ParseResult, Parser,
    parse_functions::{
        parse_static_member_access::parse_static_member_access, parse_turbo_fish::parse_turbo_fish,
    },
};

pub fn parse_static_member_access_or_turbo_fish(
    parser: &mut Parser,
    left: Expression,
) -> ParseResult<Expression> {
    parser.expect_peek_token_in(&[TokenType::Ident, TokenType::Lt])?;

    if parser.peek_token_is(TokenType::Ident) {
        return parse_static_member_access(parser, left);
    } else if parser.peek_token_is(TokenType::Lt) {
        return parse_turbo_fish(parser, left);
    }

    unreachable!()
}
