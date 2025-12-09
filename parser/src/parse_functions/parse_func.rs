use ast::{expr::Expression, expressions::ident::Ident};
use token::token_type::TokenType;

use crate::{
    ParseResult, Parser,
    parse_functions::{
        parse_block::parse_block_stmt, parse_ident::parse_ident, parse_type_hint::parse_type_hint,
    },
};

pub fn parse_func(parser: &mut Parser) -> ParseResult<Expression> {
    let token = parser.cur_token.clone();

    let name = if parser.peek_token_is(TokenType::Ident) {
        parser.next_token(); // 前进到标识符

        Some(parser.cur_token.clone())
    } else {
        None
    };

    let mut generics_params = vec![];

    if name.is_some() && parser.peek_token_is(TokenType::Lt) {
        parser.next_token(); // 前进到 <

        generics_params = parser.parse_expression_list(TokenType::Gt)?;
    }

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

    let mut ret_type = None;

    if parser.cur_token_is(TokenType::Minus) {
        parser.expect_peek(TokenType::Gt)?;

        parser.next_token(); // 前进到 >

        parser.expect_peek(TokenType::Ident)?;

        parser.next_token(); // 前进到 Ident

        ret_type = Some(Ident {
            token: parser.cur_token.clone(),
            value: parser.cur_token.value.clone(),
        });

        parser.next_token(); // 理应前进到左大括号
    }

    let block = Box::new(parse_block_stmt(parser)?);

    Ok(Expression::Function {
        token,
        name,
        params,
        block,
        ret_ty: ret_type,
        generics_params
    })
}
