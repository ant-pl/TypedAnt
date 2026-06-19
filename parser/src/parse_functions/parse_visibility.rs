use ast::{
    expressions::visibility_expr::{VisibilityNode, VisibilityNodeKind, VisibilityNodeShorthandKind},
    stmt::Statement,
};
use token::token_type::TokenType;

use crate::{
    ParseResult, Parser, error::ParserErrorKind, parse_functions::{parse_const::parse_const_with, parse_struct::parse_struct_with, parse_trait::parse_trait_with},
};

pub fn parse_restricted_visibility(parser: &mut Parser) -> ParseResult<VisibilityNodeKind> {
    // 因为该方法非自动调用, 所以要检查防止犯蠢
    parser.expect_cur(TokenType::LParen)?;

    if parser.peek_token_is(TokenType::Crate) {
        parser.next_token();
        parser.expect_peek(TokenType::RParen)?;
        parser.next_token(); // 前进到括号

        return Ok(VisibilityNodeKind::Restricted {
            path: vec![],
            shorthand: VisibilityNodeShorthandKind::Crate,
        });
    }

    if parser.peek_token_is(TokenType::Super) {
        parser.next_token();
        parser.expect_peek(TokenType::RParen)?;
        parser.next_token(); // 前进到括号

        return Ok(VisibilityNodeKind::Restricted {
            path: vec![],
            shorthand: VisibilityNodeShorthandKind::Super,
        });
    }

    parser.next_token();

    parser.expect_cur_token_in(&[TokenType::Crate, TokenType::Super])?;
    parser.expect_peek(TokenType::RParen)?;

    parser.next_token();

    unreachable!()
}

pub fn parse_visibility(parser: &mut Parser) -> ParseResult<Statement> {
    let token = parser.cur_token.clone();

    let visibility_node_kind;

    if parser.cur_token_is(TokenType::Public) && parser.peek_token_is(TokenType::LParen) {
        parser.next_token();

        visibility_node_kind = parse_restricted_visibility(parser)?;
    } else if parser.cur_token_is(TokenType::Public) {
        visibility_node_kind = VisibilityNodeKind::Public
    } else {
        visibility_node_kind = VisibilityNodeKind::Inherited
    }

    let visibility = VisibilityNode {
        token,
        visibility: visibility_node_kind
    };

    parser.next_token();

    match parser.cur_token.token_type {
        TokenType::Struct => parse_struct_with(parser, Some(visibility)),
        TokenType::Trait => parse_trait_with(parser, Some(visibility)),
        TokenType::Const => parse_const_with(parser, Some(visibility)),

        _ => Err({
            parser.next_token();
            parser.make_error(
                ParserErrorKind::ExpectedItem,
                Some("expected an item".into()),
            )
        }),
    }
}
