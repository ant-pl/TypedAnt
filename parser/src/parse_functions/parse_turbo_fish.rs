use ast::expr::Expression;
use token::token_type::TokenType;

use crate::{ParseResult, Parser};

pub fn parse_turbo_fish(parser: &mut Parser, left: Expression) -> ParseResult<Expression> {
    let token = parser.cur_token.clone();

    parser.expect_peek(TokenType::Lt)?;
    
    parser.next_token();

    let paths = parser.parse_type_expression_list(TokenType::Gt)?;

    Ok(Expression::GenericInstance {
        token,
        left: Box::new(left),
        paths,
    })
}