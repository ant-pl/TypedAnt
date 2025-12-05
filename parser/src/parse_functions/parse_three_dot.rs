use ast::expr::Expression;

use crate::{ParseResult, Parser};

pub fn parse_three_dot(parser: &mut Parser) -> ParseResult<Expression> {
    Ok(Expression::ThreeDot(parser.cur_token.clone()))
}