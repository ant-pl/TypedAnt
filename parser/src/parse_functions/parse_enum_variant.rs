use ast::{expr::Expression, expressions::ident::Ident};
use token::token_type::TokenType;

use crate::{ParseResult, Parser};

/// 解析 枚举体=>枚举项 语法
/// 例如: Option=>Some, Result=>Ok, MyEnum=>Variant
pub fn parse_enum_variant(parser: &mut Parser, left: Expression) -> ParseResult<Expression> {
    let token = parser.cur_token.clone();

    // 当前 token 是 =>，左边的表达式应该是枚举体标识符
    // 验证左边确实是标识符
    match left {
        Expression::Ident(ident) => {
            let enum_name = ident;

            // 期望下一个 token 是标识符（枚举项）
            parser.expect_peek(TokenType::Ident)?;
            parser.next_token();

            let variant_name = Ident {
                value: parser.cur_token.value.clone(),
                token: parser.cur_token.clone(),
            };

            Ok(Expression::EnumVariant {
                token,
                enum_name,
                variant_name,
            })
        }
        _ => {
            return Err(parser.make_error(
                crate::error::ParserErrorKind::NotExpectedTokenType,
                Some("expected enum name identifier before '=>'".into()),
            ));
        }
    }
}
