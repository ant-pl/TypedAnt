use ast::stmt::Statement;
use token::token_type::TokenType;

use crate::{ParseResult, Parser};

pub fn parse_use(parser: &mut Parser) -> ParseResult<Statement> {
    let token = parser.cur_token.clone();

    parser.expect_peek(TokenType::Ident)?;

    parser.next_token(); // 前进到第一个标识符

    let name_token = parser.cur_token.clone();

    let mut full_path = vec![parser.cur_token.clone()];

    if !parser.peek_token_is(TokenType::Semicolon) {
        while parser.peek_token_is(TokenType::TwoColon) {
            parser.next_token(); // 离开标识符
            parser.next_token(); // 离开逗号

            parser.expect_cur(TokenType::Ident)?;
            full_path.push(parser.cur_token.clone());
        }

        // 前进到 结束/As 的词法单元
        parser.next_token();
    } else {
        // 前进到结束的词法单元
        parser.next_token();

        return Ok(Statement::Use {
            token,
            full_path: full_path.iter().map(|it| it.value.clone()).collect(),
            alias: name_token,
        });
    }

    let mut alias = name_token;

    if parser.cur_token_is(TokenType::As) {
        parser.expect_peek(TokenType::Ident)?;
        parser.next_token(); // 前进到标识符

        alias = parser.cur_token.clone()
    }

    if parser.peek_token_is(TokenType::Semicolon) {
        parser.next_token();
    }

    Ok(Statement::Use {
        token,
        full_path: full_path.iter().map(|it| it.value.clone()).collect(),
        alias: full_path.last().unwrap_or(&alias).clone(),
    })
}
