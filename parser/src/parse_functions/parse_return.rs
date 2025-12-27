use ast::stmt::Statement;

use crate::{ParseResult, Parser, precedence::Precedence};

pub fn parse_return(parser: &mut Parser) -> ParseResult<Statement> {
    let token = parser.cur_token.clone();

    parser.next_token(); // 离开 return

    Ok(Statement::Return { token, expr: parser.parse_expression(Precedence::Lowest)? })
}
