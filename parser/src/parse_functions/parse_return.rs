use ast::stmt::Statement;
use token::token_type::TokenType;

use crate::{ParseResult, Parser, precedence::Precedence};

pub fn parse_return(parser: &mut Parser) -> ParseResult<Statement> {
    let token = parser.cur_token.clone();

    parser.next_token(); // 离开 return

    Ok(Statement::Return {
        token,
        expr: if !parser
            .prefix_parse_fn_map
            .contains_key(&parser.cur_token.token_type)
            || parser.cur_token_is(TokenType::Semicolon)
        {
            None
        } else {
            Some(parser.parse_expression(Precedence::Lowest)?)
        },
    })
}
