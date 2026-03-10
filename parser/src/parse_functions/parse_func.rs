use ast::expr::Expression;
use token::token_type::TokenType;

use crate::{ParseResult, Parser, precedence::Precedence};

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

    let params = parser.parse_type_expression_list(TokenType::RParen)?;

    parser.next_token(); // 离开右括号 (正常应前进到左大括号 或者 '->' )

    let mut ret_type = None;

    if parser.cur_token_is(TokenType::Minus) {
        parser.expect_peek(TokenType::Gt)?;

        parser.next_token(); // 前进到 >

        parser.next_token(); // 前进到 类型表达式

        ret_type = Some(Box::new(parser.parse_type_expression(Precedence::Lowest)?));

        parser.next_token(); // 理应前进到左大括号
    }

    let block = Box::new(parser.parse_expression(Precedence::Lowest)?);

    Ok(Expression::Function {
        token,
        name,
        params,
        block,
        ret_ty: ret_type,
        generics_params,
    })
}
