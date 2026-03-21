use std::{num::IntErrorKind, str::FromStr};

use ast::expr::{Expression, FloatValue};
use bigdecimal::BigDecimal;
use token::token_type::TokenType;

use crate::{
    ParseResult, Parser,
    error::{ParseIntErrorKind, ParserErrorKind},
};

macro_rules! impl_parse_num {
    ($name:ident, $ty:ident) => {
        pub fn $name(parser: &mut Parser) -> ParseResult<Expression> {
            Ok(Expression::Int {
                token: parser.cur_token.clone(),
                value: match $ty::from_str(&parser.cur_token.value) {
                    Ok(it) => it.into(),
                    Err(it) => {
                        return Err(parser.make_error(
                            ParserErrorKind::ParseIntError(match it.kind() {
                                IntErrorKind::Empty => ParseIntErrorKind::Empty,
                                IntErrorKind::InvalidDigit => ParseIntErrorKind::InvalidDigit,
                                IntErrorKind::PosOverflow => ParseIntErrorKind::PosOverflow,
                                IntErrorKind::NegOverflow => ParseIntErrorKind::NegOverflow,
                                _ => unreachable!(),
                            }),
                            None,
                        ))
                    }
                },
            })
        }
    };
}

impl_parse_num!(parse_i64, i64);
impl_parse_num!(parse_i32, i32);
impl_parse_num!(parse_i16, i16);
impl_parse_num!(parse_i8, i8);
impl_parse_num!(parse_u64, u64);
impl_parse_num!(parse_u32, u32);
impl_parse_num!(parse_u16, u16);
impl_parse_num!(parse_u8, u8);
impl_parse_num!(parse_usize, usize);
impl_parse_num!(parse_isize, isize);

pub fn parse_int(parser: &mut Parser) -> ParseResult<Expression> {
    let token = parser.cur_token.clone();

    if parser.peek_token_is(TokenType::Dot) {
        parser.next_token(); // 前进至 .(点号)
        parser.next_token();

        let s = &format!("{}.{}", token.value, parser.cur_token.value);
        let parse_result = BigDecimal::from_str(s);

        let Ok(it) = parse_result else {
            return Err(Parser::make_error_with_token(
                ParserErrorKind::ParseIntError(ParseIntErrorKind::InvalidDigit),
                Some(format!("could not parse `{}` as float", s).into()),
                token,
            ));
        };

        return Ok(Expression::Float {
            token,
            value: FloatValue::F32(it),
        });
    }

    let parse_result = BigDecimal::from_str(&token.value);

    let Ok(it) = parse_result else {
        return Err(Parser::make_error_with_token(
            ParserErrorKind::ParseIntError(ParseIntErrorKind::InvalidDigit),
            Some(format!("could not parse `{}` as integer", token.value).into()),
            token,
        ));
    };

    Ok(Expression::UnknownTypeInt { token, value: it })
}
