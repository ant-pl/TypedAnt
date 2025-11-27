use ast::stmt::Statement;
use token::token_type::TokenType;

use crate::{ParseResult, Parser, parse_functions::parse_block::parse_block_stmt, precedence::Precedence};

pub fn parse_while(parser: &mut Parser) -> ParseResult<Statement> {
    let token = parser.cur_token.clone();

    parser.next_token(); // 离开 while 词法单元

    // 条件
    let condition = parser.parse_expression(Precedence::Lowest)?;

    parser.expect_peek(TokenType::LBrace)?;

    parser.next_token(); // 离开条件表达式

    let block = parse_block_stmt(parser)?;
    
    Ok(Statement::While { token, condition, block: Box::new(block) })
}