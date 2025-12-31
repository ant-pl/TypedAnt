#[cfg(test)]
mod tests {
    use token::{token::Token, token_type::TokenType};

    use crate::{Parser, precedence::Precedence};

    #[test]
    fn test_parse_expr() {
        let file_name: std::sync::Arc<str> = "__test_parse_expr__".into();

        let mut parser = Parser::new(vec![
            Token::new("1".into(), TokenType::Integer64, file_name.clone(), 1, 1),
            Token::new("+".into(), TokenType::Plus, file_name.clone(), 1, 2),
            Token::new("2".into(), TokenType::Integer64, file_name.clone(), 1, 3),
            Token::new("*".into(), TokenType::Asterisk, file_name.clone(), 1, 4),
            Token::new("3".into(), TokenType::Integer64, file_name.clone(), 1, 5),
        ]);

        let expr = parser.parse_expression(Precedence::Lowest);

        match expr {
            Ok(it) => println!("{it}"),
            Err(err) => panic!("{err}")
        }
    }
}