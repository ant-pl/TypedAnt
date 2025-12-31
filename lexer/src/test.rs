#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use token::{token::Token, token_type::TokenType};
    use utils::assert_eq;

    use crate::{Lexer, error::LexerErrorKind};

    fn expected_token_types(expected: Vec<TokenType>, got: Vec<Token>) {
        let on_failure_function = || panic!("expected {expected:#?}, got {got:#?}");

        for (expected, got) in expected.iter().zip(&got) {
            assert_eq(expected, &got.token_type, on_failure_function);
        }
    }

    #[test]
    fn test_lexer() {
        let mut l = Lexer::new(
            "{let a = 1 + 2 * 3 / 4}".into(),
            Arc::from("__test_lexer__"),
        );
        let tokens = l.get_tokens();

        let expected = vec![
            TokenType::LBrace,
            TokenType::Let,
            TokenType::Ident,
            TokenType::Assign,
            TokenType::IntegerBig,
            TokenType::Plus,
            TokenType::IntegerBig,
            TokenType::Asterisk,
            TokenType::IntegerBig,
            TokenType::Slash,
            TokenType::IntegerBig,
            TokenType::RBrace,
        ];

        // 验证词法单元
        expected_token_types(expected, tokens);
    }

    #[test]
    fn test_lexer_unicode() {
        let mut l = Lexer::new(
            "let ♿ = \"otto\"; let 你好 = \"Hello\"".into(),
            Arc::from("__test_lexer_unicode__"),
        );

        let tokens = l.get_tokens();

        let expected = vec![
            TokenType::Let,
            TokenType::Ident,
            TokenType::Assign,
            TokenType::String,
            TokenType::Semicolon,
            TokenType::Let,
            TokenType::Ident,
            TokenType::Assign,
            TokenType::String,
        ];

        // 验证词法单元
        expected_token_types(expected, tokens);
    }

    #[test]
    fn test_lexer_comment() {
        let mut l = Lexer::new(
            "// test comment".into(),
            Arc::from("__test_lexer_comment__"),
        );
        let tokens = l.get_tokens();

        let expected = vec![];

        // 验证词法单元
        expected_token_types(expected, tokens);
    }

    #[test]
    fn test_lexer_test_print_token() {
        let mut l = Lexer::new(
            "TestPrint n".into(),
            Arc::from("__test_lexer_test_print_token__"),
        );

        let tokens = l.get_tokens();

        let expected = vec![TokenType::TestPrint, TokenType::Ident];

        // 验证词法单元
        expected_token_types(expected, tokens);
    }

    #[test]
    fn test_lexer_error() {
        use utils::assert_eq;

        let mut l = Lexer::new(r#""  """#.into(), Arc::from("test_lexer_error"));

        let _tokens = l.get_tokens();

        assert_eq(l.contains_error(), true, || {
            panic!("expected error, got none")
        });

        let expected_kind = LexerErrorKind::UnClosedString;
        let got_kind = l.errors()[0].kind;

        assert_eq(
            expected_kind, got_kind,
            || panic!("expected {expected_kind:#?}, got: {got_kind:#?}")
        );

        l.print_errors();
    }
}
