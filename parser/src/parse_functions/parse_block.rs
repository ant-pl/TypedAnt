use ast::{expr::Expression, stmt::Statement};
use token::token_type::TokenType;

use crate::{ParseResult, Parser};

pub fn parse_block_expr(parser: &mut Parser) -> ParseResult<Expression> {
    let token = parser.cur_token.clone();

    parser.expect_cur(TokenType::LBrace)?;

    parser.next_token(); // 离开左括号

    let mut statements = vec![];


    while !parser.cur_token_is(TokenType::RBrace) && !parser.cur_token_is(TokenType::Eof) {
        let statement = parser.parse_statement();

        statements.push(statement?);

        parser.next_token() // 离开语句
    }

    // WARNING: 有需要离开右括号的情况自行处理

    Ok(Expression::Block(token, statements))
}

pub fn parse_block_stmt(parser: &mut Parser) -> ParseResult<Statement> {
    parser.expect_cur(TokenType::LBrace)?;

    let token = parser.cur_token.clone();

    parser.next_token(); // 离开左括号

    let mut statements = vec![];


    while !parser.cur_token_is(TokenType::RBrace) && !parser.cur_token_is(TokenType::Eof) {
        let statement = parser.parse_statement();

        statements.push(statement?);

        parser.next_token() // 离开语句
    }

    // WARNING: 有需要离开右括号的情况自行处理

    Ok(Statement::Block { token, statements })
}