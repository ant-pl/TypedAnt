#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bigdecimal::BigDecimal;
    use token::{token::Token, token_type::TokenType};

    use crate::{
        ty::Ty, TypeChecker,
        table::TypeTable,
        typed_ast::{GetType, typed_expr::TypedExpression, typed_stmt::TypedStatement},
    };

    fn empty_table() -> Arc<Mutex<TypeTable>> {
        Arc::new(Mutex::new(TypeTable::new().init()))
    }

    #[test]
    fn test_checker_var_get() {
        let file: Arc<str> = "__test_checker_var_get__".into();

        let table = empty_table();

        table.lock().unwrap().define_var("a", Ty::BigInt);

        let checker = &mut TypeChecker::new(table);

        let ident_raw = ast::expressions::ident::Ident {
            token: Token::new("a".into(), TokenType::Ident, file.clone(), 1, 1),
            value: "a".into(),
        };

        let result = checker
            .check_expr(ast::expr::Expression::Ident(ident_raw))
            .unwrap();

        assert!(matches!(result, TypedExpression::Ident(..)));

        let (ident, ty) = match result {
            TypedExpression::Ident(it, ty) => (it, ty),
            _ => unreachable!(),
        };

        assert!(ident.value == "a".into());
        assert!(ty == Ty::BigInt);

        println!("ok! ident.value: {}, ty: {ty:#?}", ident.value)
    }

    #[test]
    fn test_checker_var_def() {
        let file: Arc<str> = "__test_checker_var_def__".into();

        let table = empty_table();

        let checker = &mut TypeChecker::new(table.clone());

        let let_stmt_raw = ast::stmt::Statement::Let {
            name: ast::expressions::ident::Ident {
                token: Token::new("a".into(), TokenType::Ident, file.clone(), 1, 5),
                value: "a".into(),
            },
            value: ast::expr::Expression::BigInt {
                token: Token::new("1".into(), TokenType::IntegerBig, file.clone(), 1, 17),
                value: BigDecimal::from(1),
            },
            var_type: Some(ast::expressions::ident::Ident {
                token: Token::new("BigInt".into(), TokenType::IntegerBig, file.clone(), 1, 8),
                value: "BigInt".into(),
            }),
            token: Token::new("let".into(), TokenType::Let, file.clone(), 1, 1),
        };

        let result = checker.check_statement(let_stmt_raw).unwrap();

        assert!(matches!(result, TypedStatement::Let { .. }));

        let (ident, ty) = match result {
            TypedStatement::Let { name, ty, .. } => (name, ty),
            _ => unreachable!(),
        };

        assert!(ident.value == "a".into());
        assert!(ty == Ty::BigInt);

        let get_symbol_result = table.lock().unwrap().get("a");
        let get_symbol_result_ref = get_symbol_result.as_ref();

        assert!(get_symbol_result.is_some());
        assert!(unsafe { get_symbol_result_ref.unwrap_unchecked() }.name == "a".into());
        assert!(
            unsafe { get_symbol_result_ref.unwrap_unchecked() }
                .ty
                .get_type()
                == Ty::BigInt
        );

        println!(
            "ok! symbol name: {}, symbol ty: {:#?}",
            unsafe { get_symbol_result_ref.unwrap_unchecked() }.name,
            unsafe { get_symbol_result_ref.unwrap_unchecked() }
                .ty
                .get_type()
        )
    }
}
