use ast::{expressions::ident::Ident, stmt::Statement};
use token::token_type::TokenType;

use crate::{ParseResult, Parser, parse_functions::parse_block::parse_block_stmt};

pub fn parse_trait(parser: &mut Parser) -> ParseResult<Statement> {
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
    })
}
