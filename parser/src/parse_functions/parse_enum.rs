use ast::{expressions::ident::Ident, stmt::Statement};
use token::token_type::TokenType;

use crate::{ParseResult, Parser};

/// 解析 enum 定义
/// 例如:
/// enum Option { Some, None }
/// enum Color { Red, Green, Blue }
pub fn parse_enum(parser: &mut Parser) -> ParseResult<Statement> {
    let token = parser.cur_token.clone();

    parser.expect_peek(TokenType::Ident)?;

    parser.next_token(); // 前进到标识符

    let name = Ident {
        value: parser.cur_token.value.clone(),
        token: parser.cur_token.clone(),
    };

    parser.expect_peek(TokenType::LBrace)?;

    parser.next_token(); // 前进到 {

    // 解析枚举项列表
    let mut variants = vec![];

    loop {
        if parser.peek_token_is(TokenType::RBrace) {
            break;
        }

        parser.expect_peek(TokenType::Ident)?;

        parser.next_token(); // 前进到标识符

        let variant = Ident {
            value: parser.cur_token.value.clone(),
            token: parser.cur_token.clone(),
        };

        variants.push(variant);

        // 检查是否有逗号
        if parser.peek_token_is(TokenType::Comma) {
            parser.next_token(); // 跳过逗号
        }
    }

    parser.expect_peek(TokenType::RBrace)?;

    Ok(Statement::Enum {
        token,
        name,
        variants,
    })
}
