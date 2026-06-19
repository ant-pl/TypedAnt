use ast::{
    expressions::{ident::Ident, visibility_expr::VisibilityNode},
    stmt::Statement,
};
use token::token_type::TokenType;

use crate::{ParseResult, Parser, parse_functions::parse_block::parse_block_stmt};

#[inline(always)]
pub fn parse_trait(parser: &mut Parser) -> ParseResult<Statement> {
    parse_trait_with(parser, None)
}

pub fn parse_trait_with(
    parser: &mut Parser,
    visibility: Option<VisibilityNode>,
) -> ParseResult<Statement> {
    let token = parser.cur_token.clone();

    parser.expect_peek(TokenType::Ident)?;

    parser.next_token(); // 前进到标识符

    let name = Ident {
        value: parser.cur_token.value.clone(),
        token: parser.cur_token.clone(),
    };

    parser.expect_peek(TokenType::LBrace)?;

    parser.next_token(); // 前进到左括号

    Ok(Statement::Trait {
        token,
        name,
        block: Box::new(parse_block_stmt(parser)?),
        visibility,
    })
}
