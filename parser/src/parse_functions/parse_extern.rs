use ast::{expr::Expression, node::GetToken, stmt::Statement};
use token::token_type::TokenType;

use crate::{
    ParseResult, Parser,
    error::{ParserError, ParserErrorKind},
    parse_functions::parse_three_dot::parse_three_dot,
    precedence::Precedence,
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

    // 注入 ThreeDot 解析函数
    parser
        .prefix_parse_fn_map
        .insert(TokenType::ThreeDot, parse_three_dot);

    let params = parser.parse_type_expression_list(TokenType::RParen)?;

    // 移除 ThreeDot 解析函数
    parser.prefix_parse_fn_map.remove(&TokenType::ThreeDot);

    parser.next_token(); // 离开右括号 (正常应前进到左大括号 或者 '->' )

    parser.expect_cur(TokenType::Minus)?;
    parser.expect_peek(TokenType::Gt)?;

    parser.next_token(); // 前进到 >

    parser.next_token(); // 前进到 类型表达式

    let ret_type = parser.parse_type_expression(Precedence::Lowest)?;

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

    // 检查 params
    let three_dots = params
        .iter()
        .enumerate()
        .filter(|it| matches!(it.1.as_ref(), Expression::ThreeDot(_)))
        .collect::<Vec<(usize, &Box<Expression>)>>();

    if three_dots.is_empty() {
        return Ok(Statement::Extern {
            token,
            abi,
            extern_func_name: name,
            params: params
                .into_iter()
                .filter(|it| !matches!(it.as_ref(), Expression::ThreeDot(_)))
                .collect::<Vec<Box<Expression>>>(),
            ret_ty: Box::new(ret_type),
            alias,
            vararg: false,
        });
    }

    if three_dots.len() > 1 {
        Err(ParserError {
            token: three_dots[0].1.token(),
            kind: ParserErrorKind::ExpectedNothing,
            message: Some(format!("so many '...'").into()),
        })?
    }

    if three_dots[0].0 != params.len() - 1 {
        Err(ParserError {
            token: three_dots[0].1.token(),
            kind: ParserErrorKind::NotExpectedPosition,
            message: Some(format!("'...' must be the last argument").into()),
        })?
    }

    Ok(Statement::Extern {
        token,
        abi,
        extern_func_name: name,
        params: params
            .into_iter()
            .filter(|it| !matches!(it.as_ref(), Expression::ThreeDot(_)))
            .collect::<Vec<Box<Expression>>>(),
        ret_ty: Box::new(ret_type),
        alias,
        vararg: true,
    })
}
