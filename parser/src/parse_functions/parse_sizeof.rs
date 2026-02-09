use ast::expr::Expression;

use crate::{ParseResult, Parser, precedence::Precedence};

pub fn parse_sizeof(parser: &mut Parser) -> ParseResult<Expression> {
    Ok(Expression::SizeOf(
        parser.cur_token.clone(),
        {
            parser.next_token();
            Box::new(parser.parse_expression(Precedence::Lowest)?)
        }
    ))
}
