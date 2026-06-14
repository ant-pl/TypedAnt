use ast::{expr::Expression, expressions::ident::Ident, node::GetToken};
use token::token_type::TokenType;

use crate::{ParseResult, Parser};

pub fn parse_turbo_fish(parser: &mut Parser, left: Expression) -> ParseResult<Expression> {
    let token = parser.cur_token.clone();

    // 情况 1: Color::Red  (枚举变体访问)
    if parser.peek_token_is(TokenType::Ident) {
        if !matches!(left, Expression::Ident(_)) {
            parser.unexpect_token_err(TokenType::Ident, left.token())?;
        }

        let enum_name = Ident {
            value: left.token().value,
            token: left.token(),
        };

        parser.next_token(); // 前进到变体标识符

        let variant = Ident {
            value: parser.cur_token.value.clone(),
            token: parser.cur_token.clone(),
        };

        return Ok(Expression::EnumVariant {
            token,
            enum_name,
            variant,
            args: vec![],
        });
    }

    // 情况 2: Foo::<T1, T2> 原有的 turbofish 语法
    parser.expect_peek(TokenType::Lt)?;

    parser.next_token();

    let paths = parser.parse_type_expression_list(TokenType::Gt)?;

    Ok(Expression::GenericInstance {
        token,
        left: Box::new(left),
        paths,
    })
}
