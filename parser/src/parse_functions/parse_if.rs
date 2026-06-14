use ast::expr::Expression;
use token::token_type::TokenType;
use token::token::Token;

use crate::{ParseResult, Parser, precedence::Precedence};

pub fn parse_if(parser: &mut Parser) -> ParseResult<Expression> {
    let token = parser.cur_token.clone();

    parser.next_token(); // 离开 'if' 词法单元

    // 检测 if let 模式
    if parser.cur_token_is(TokenType::Let) {
        return parse_if_let(parser, token);
    }

    // 普通 if 表达式
    let condition = Box::new(parser.parse_expression(Precedence::Lowest)?);

    parser.next_token(); // 离开表达式 (正常应跳转到左大括号)

    let consequence = Box::new(parser.parse_expression(Precedence::Lowest)?);

    parser.expect_cur(TokenType::RBrace)?;

    if parser.peek_token_is(TokenType::Else) {
        parser.next_token(); // 离开右大括号
        parser.next_token(); // 离开 else 词法单元

        let else_block = Box::new(parser.parse_expression(Precedence::Lowest)?);

        Ok(Expression::If {
            token,
            condition,
            consequence,
            else_block: Some(else_block),
        })
    } else {
        Ok(Expression::If {
            token,
            condition,
            consequence,
            else_block: None,
        })
    }
}

fn parse_if_let(parser: &mut Parser, token: Token) -> ParseResult<Expression> {
    // 当前已在 'let' 处，前进到模式起始处
    parser.next_token();

    // 解析模式：使用 Assignment 优先级以防止 '=' 被当作赋值运算符
    let pattern = Box::new(parser.parse_expression(Precedence::Assignment)?);

    parser.expect_peek(TokenType::Assign)?;

    parser.next_token(); // 前进到 '='

    parser.next_token(); // 前进到被匹配的值(scrutinee)

    // 解析被匹配的值
    let scrutinee = Box::new(parser.parse_expression(Precedence::Lowest)?);

    parser.next_token(); // 前进到 '{'

    let consequence = Box::new(parser.parse_expression(Precedence::Lowest)?);

    parser.expect_cur(TokenType::RBrace)?;

    // 可选 else 块
    if parser.peek_token_is(TokenType::Else) {
        parser.next_token(); // 离开右大括号
        parser.next_token(); // 离开 else 词法单元

        let else_block = Box::new(parser.parse_expression(Precedence::Lowest)?);

        Ok(Expression::IfLet {
            token,
            pattern,
            scrutinee,
            consequence,
            else_block: Some(else_block),
        })
    } else {
        Ok(Expression::IfLet {
            token,
            pattern,
            scrutinee,
            consequence,
            else_block: None,
        })
    }
}
