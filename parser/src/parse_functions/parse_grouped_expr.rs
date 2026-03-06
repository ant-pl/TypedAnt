use ast::expr::Expression;
use token::token_type::TokenType;

use crate::{ParseResult, Parser, precedence::Precedence};

pub fn parse_grouped_expr(parser: &mut Parser) -> ParseResult<Expression> {
    parser.next_token();
    let expr = parser.parse_expression(Precedence::Lowest)?;
    parser.expect_peek(TokenType::RParen)?;
    parser.next_token();
    Ok(expr)
}
