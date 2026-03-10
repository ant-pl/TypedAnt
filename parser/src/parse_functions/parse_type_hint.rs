use ast::{expr::Expression, expressions::ident::Ident, node::GetToken};
use token::token_type::TokenType;

use crate::{ParseResult, Parser, precedence::Precedence};

pub fn parse_type_hint(parser: &mut Parser, left: Expression) -> ParseResult<Expression> {
    if !matches!(left, Expression::Ident(..)) {
        parser.unexpect_token_err(TokenType::Ident, left.token())?;
    }

    let token = left.token();

    parser.next_token(); // 前进到类型表达式

    Ok(Expression::TypeHint(
        Ident {
            value: token.value.clone(),
            token,
        },
        Box::new(parser.parse_type_expression(Precedence::Lowest)?),
    ))
}
