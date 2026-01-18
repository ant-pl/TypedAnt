use ast::{expressions::ident::Ident, stmt::Statement};
use token::token_type::TokenType;

use crate::{ParseResult, Parser, error::ParserErrorKind, precedence::Precedence};

pub fn parse_const(parser: &mut Parser) -> ParseResult<Statement> {
    let token = parser.cur_token.clone(); // token 'const'

    parser.next_token(); // 离开 'const'

    parser.expect_cur(TokenType::Ident)?;

    let name = Ident {
        token: parser.cur_token.clone(),
        value: parser.cur_token.value.clone(),
    };

    parser.next_token(); // 离开标识符

    let mut var_type = None;

    if parser.cur_token_is(TokenType::Colon) {
        if !parser.peek_token_is(TokenType::Ident) {
            Err(parser.make_error(ParserErrorKind::ExpectedType, None))?
        }

        var_type = Some(Ident {
            token: parser.peek_token.clone(),
            value: parser.peek_token.value.clone(),
        });
    
        parser.next_token(); // 前进到类型标识符
        parser.next_token(); // 离开类型标识符
    }

    parser.expect_cur(TokenType::Assign)?;

    parser.next_token(); // 离开等号

    let value = parser.parse_expression(Precedence::Lowest)?;

    if parser.peek_token_is(TokenType::Semicolon) {
        parser.next_token();
    }

    Ok(Statement::Const { token, name, var_type, value })
}
