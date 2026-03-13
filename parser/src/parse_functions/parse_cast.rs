use ast::expr::Expression;

use crate::{ParseResult, Parser, precedence::Precedence};

pub fn parse_cast(parser: &mut Parser, left: Expression) -> ParseResult<Expression> {
    let token = parser.cur_token.clone();

    parser.next_token(); // 离开 as

    let right = parser.parse_expression(Precedence::Cast)?;

    Ok(Expression::Cast {
        val: Box::new(left),
        cast_to: Box::new(right),
        token,
    })
}
