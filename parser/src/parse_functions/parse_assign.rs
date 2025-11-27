use ast::expr::Expression;

use crate::{ParseResult, Parser, precedence::Precedence};

pub fn parse_assign(parser: &mut Parser, left: Expression) -> ParseResult<Expression> {
    let token = parser.cur_token.clone();
    let left = Box::new(left);

    parser.next_token(); // 离开等于号

    let value = Box::new(parser.parse_expression(Precedence::Lowest)?);

    Ok(Expression::Assign { token, left, right: value })
}