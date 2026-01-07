use ast::{expressions::ident::Ident, stmt::Statement};
use token::token_type::TokenType;

use crate::{ParseResult, Parser, parse_functions::parse_block::parse_block_stmt};

pub fn parse_impl(parser: &mut Parser) -> ParseResult<Statement> {
    let token = parser.cur_token.clone();

    parser.expect_peek(TokenType::Ident)?;

    parser.next_token(); // 前进到标识符 (impl_)

    let impl_ = Ident {
        value: parser.cur_token.value.clone(),
        token: parser.cur_token.clone(),
    };

    let mut for_ = None;

    if parser.peek_token_is(TokenType::For) {
        parser.next_token(); // 前进到 For
        parser.next_token(); // 前进到标识符 (for_)

        for_ = Some(Ident {
            value: parser.cur_token.value.clone(),
            token: parser.cur_token.clone(),
        });
    }

    parser.expect_peek(TokenType::LBrace)?;

    parser.next_token(); // 前进到左括号

    Ok(Statement::Impl {
        token,
        impl_,
        for_,
        block: Box::new(parse_block_stmt(parser)?),
    })
}
