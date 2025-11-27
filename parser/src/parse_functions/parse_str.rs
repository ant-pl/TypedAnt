use ast::expr::Expression;

use crate::{ParseResult, Parser};

pub fn parse_str(parser: &mut Parser) -> ParseResult<Expression> {
    Ok(Expression::StrLiteral {
        token: parser.cur_token.clone(),
        value: parser.cur_token.value.clone(),
    })
}
