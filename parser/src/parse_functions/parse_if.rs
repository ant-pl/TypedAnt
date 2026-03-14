use ast::expr::Expression;
use token::token_type::TokenType;

use crate::{
    ParseResult, Parser,
    precedence::Precedence,
};

pub fn parse_if(parser: &mut Parser) -> ParseResult<Expression> {
    let token = parser.cur_token.clone();

    parser.next_token(); // 离开 if 词法单元

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
