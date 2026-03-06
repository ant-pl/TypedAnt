use ast::expr::Expression;

use crate::{ParseResult, Parser, precedence::Precedence};

pub fn parse_prefix(parser: &mut Parser) -> ParseResult<Expression> {
    let token = parser.cur_token.clone();

    parser.next_token(); // 离开 前缀运算符

    let right = parser.parse_expression(Precedence::Prefix)?;

    Ok(Expression::Prefix {
        op: token.value.clone(),
        right: Box::new(right),
        token,
    })
}
