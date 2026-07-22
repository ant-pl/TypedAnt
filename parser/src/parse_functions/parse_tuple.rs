use ast::expr::Expression;
use token::token_type::TokenType;

use crate::{ParseResult, Parser};

pub fn parse_tuple(parser: &mut Parser) -> ParseResult<Expression> {
    let token = parser.cur_token.clone();

    let mut exprs = parser.parse_expression_list(TokenType::RParen)?;
    
    if exprs.is_empty() {
        return Ok(Expression::Unit(token))
    }

    if exprs.len() == 1 {
        // 已确认 exprs 不为空可直接取表达式
        return Ok(*exprs.pop().unwrap())
    }

    return Ok(Expression::Tuple(token, exprs))
}
