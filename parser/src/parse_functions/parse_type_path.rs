use ast::{expr::Expression, expressions::ident::Ident, node::GetToken};
use token::token_type::TokenType;

use crate::{ParseResult, Parser};

pub fn parse_type_path(parser: &mut Parser, left: Expression) -> ParseResult<Expression> {
    if !matches!(left, Expression::Ident(..)) {
        parser.unexpect_token_err(TokenType::Ident, left.token())?;
    }

    let token = parser.cur_token.clone();

    let paths = parser.parse_type_expression_list(TokenType::Gt)?;

    Ok(Expression::TypePath {
        token,
        left: Ident {
            token: left.token(),
            value: left.token().value,
        },
        paths,
    })
}
