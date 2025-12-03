use ast::{expressions::ident::Ident, stmt::Statement};
use token::token_type::TokenType;

use crate::{
    ParseResult, Parser,
    parse_functions::{parse_ident::parse_ident, parse_type_hint::parse_type_hint},
};

pub fn parse_extern(parser: &mut Parser) -> ParseResult<Statement> {
    let token = parser.cur_token.clone(); // extern

    parser.expect_peek(TokenType::String)?;

    parser.next_token(); // 前进到字符串 Token

    let abi = parser.cur_token.clone();

    parser.expect_peek(TokenType::Func)?;
    parser.next_token(); // 前进到 func

    parser.expect_peek(TokenType::Ident)?;
    parser.next_token(); // 前进到标识符

    let name = parser.cur_token.clone();

    parser.expect_peek(TokenType::LParen)?;

    parser.next_token(); // 前进到左括号

    // WARNING: 非十足把握请勿模仿动态注入表达式解析表

    // 注入 TypeHint 解析函数
    parser
        .prefix_parse_fn_map
        .insert(TokenType::Ident, parse_type_hint);

    let params = parser.parse_expression_list(TokenType::RParen)?;

    // 移除 TypeHint 解析函数
    parser
        .prefix_parse_fn_map
        .insert(TokenType::Ident, parse_ident);

    parser.next_token(); // 离开右括号 (正常应前进到左大括号 或者 '->' )

    parser.expect_cur(TokenType::Minus)?;
    parser.expect_peek(TokenType::Gt)?;

    parser.next_token(); // 前进到 >

    parser.expect_peek(TokenType::Ident)?;

    parser.next_token(); // 前进到 Ident

    let ret_type = Ident {
        token: parser.cur_token.clone(),
        value: parser.cur_token.value.clone(),
    };

    let alias = if parser.peek_token_is(TokenType::As) {
        parser.next_token(); // 前进到 As

        parser.expect_peek(TokenType::Ident)?;
        parser.next_token(); // 前进到标识符

        parser.cur_token.clone()
    } else {
        name.clone()
    };

    if parser.peek_token_is(TokenType::Semicolon) {
        parser.next_token();
    }

    Ok(Statement::Extern {
        token,
        abi,
        extern_func_name: name,
        params,
        ret_ty: ret_type,
        alias,
    })
}
