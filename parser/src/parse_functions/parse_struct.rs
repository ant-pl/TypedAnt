use ast::{expressions::ident::Ident, stmt::Statement};
use token::token_type::TokenType;

use crate::{ParseResult, Parser, parse_functions::{parse_ident::parse_ident, parse_type_hint::parse_type_hint}};

pub fn parse_struct(parser: &mut Parser) -> ParseResult<Statement> {
    let mut generics = vec![];

    let token = parser.cur_token.clone();

    parser.expect_peek(TokenType::Ident)?;

    parser.next_token(); // 前进到标识符

    let name = Ident {
        value: parser.cur_token.value.clone(),
        token: parser.cur_token.clone(),
    };

    if parser.peek_token_is(TokenType::Lt) {
        parser.next_token(); // 前进到 <

        generics = parser.parse_expression_list(TokenType::Gt)?;
    }

    parser.expect_peek(TokenType::LBrace)?;

    parser.next_token(); // 前进到左括号

    // WARNING: 非十足把握请勿模仿动态注入表达式解析表

    // 注入 TypeHint 解析函数
    parser.prefix_parse_fn_map.insert(TokenType::Ident, parse_type_hint);

    let fields = parser.parse_expression_list(TokenType::RBrace)?;

    // 移除 TypeHint 解析函数
    parser.prefix_parse_fn_map.insert(TokenType::Ident, parse_ident);

    Ok(Statement::Struct { token, name, fields, generics })
}
