use ast::{
    expressions::{ident::Ident, visibility_expr::VisibilityNode},
    stmt::Statement,
};
use token::token_type::TokenType;

use crate::{ParseResult, Parser};

#[inline(always)]
pub fn parse_enum(parser: &mut Parser) -> ParseResult<Statement> {
    parse_enum_with(parser, None)
}

pub fn parse_enum_with(
    parser: &mut Parser,
    visibility: Option<VisibilityNode>,
) -> ParseResult<Statement> {
    let token = parser.cur_token.clone();

    parser.expect_peek(TokenType::Ident)?;

    parser.next_token();

    let name = Ident {
        value: parser.cur_token.value.clone(),
        token: parser.cur_token.clone(),
    };

    let mut generics = vec![];

    if parser.peek_token_is(TokenType::Lt) {
        parser.next_token();

        generics = parser.parse_expression_list(TokenType::Gt)?;
    }

    parser.expect_peek(TokenType::LBrace)?;

    parser.next_token();

    let variants = parser.parse_expression_list(TokenType::RBrace)?;

    Ok(Statement::Enum {
        token,
        name,
        variants,
        generics,
        visibility,
    })
}
