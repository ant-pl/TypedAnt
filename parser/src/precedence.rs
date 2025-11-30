use num_enum::{IntoPrimitive, TryFromPrimitive};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::Sub;

use token::token_type::TokenType;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, TryFromPrimitive, IntoPrimitive)]
#[repr(i32)]
pub enum Precedence {
    Lowest,
    Assignment,  // a = 1
    AndOr,       // Or | And
    Equals,      // ==
    LessGreater, // > | <
    Sum,         // +
    Product,     // *
    Prefix,      // -X | !X
    Call,        // myFunction(X) array[index] person.Name 
    Highest,
}

pub static TOKEN_PRECEDENCES: Lazy<HashMap<TokenType, Precedence>> = Lazy::new(|| {
    let mut m = HashMap::new();

    m.insert(TokenType::Eq, Precedence::Equals);
    m.insert(TokenType::Colon, Precedence::Lowest);
    m.insert(TokenType::NotEq, Precedence::Equals);
    m.insert(TokenType::Lt, Precedence::LessGreater);
    m.insert(TokenType::Gt, Precedence::LessGreater);
    m.insert(TokenType::Plus, Precedence::Sum);
    m.insert(TokenType::Minus, Precedence::Sum);
    m.insert(TokenType::Slash, Precedence::Product);
    m.insert(TokenType::Asterisk, Precedence::Product);
    m.insert(TokenType::LParen, Precedence::Call);
    m.insert(TokenType::LBracket, Precedence::Call);
    m.insert(TokenType::Assign, Precedence::Assignment);
    m.insert(TokenType::Dot, Precedence::Call);
    m.insert(TokenType::GetClassMember, Precedence::Call);
    m.insert(TokenType::BoolOr, Precedence::AndOr);
    m.insert(TokenType::BoolAnd, Precedence::AndOr);
    
    m
});

impl Sub<i32> for Precedence {
    type Output = Precedence;

    fn sub(self, rhs: i32) -> Self::Output {
        let num: i32 = self.into();
        Precedence::try_from(num - rhs).unwrap()
    }
}

pub fn get_token_precedence(token_type: TokenType) -> Precedence {
    match TOKEN_PRECEDENCES.get(&token_type.clone()).cloned() {
        None => Precedence::Lowest,
        Some(it) => it,
    }
}
