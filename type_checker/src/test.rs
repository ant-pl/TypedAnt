#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bigdecimal::BigDecimal;
    use name_resolver::NameResolver;
    use token::{token::Token, token_type::TokenType};
    use ty::Ty;
    use typed_ast::{GetType, typed_expr::TypedExpression, typed_stmt::TypedStatement};
    use typed_module::{module::TypedModule, ty_context::TypeContext};

    use crate::{
        TypeChecker,
        type_infer::{TypeInfer, infer_context::InferContext},
    };

    #[test]
    fn test_checker_var_get() {
        let file: Arc<str> = "__test_checker_var_get__".into();

        let mut tcx = TypeContext::new();

        let bigint_id = tcx.alloc(Ty::BigInt);

        tcx.table.lock().unwrap().define_var("a", bigint_id);

        let mut name_resolver = NameResolver::new(0.into(), &file);
        let mut module = TypedModule::new(&mut tcx);

        let checker = &mut TypeChecker::new(&mut module, &mut name_resolver);

        let ident_raw = ast::expressions::ident::Ident {
            token: Token::new("a".into(), TokenType::Ident, file.clone(), 1, 1),
            value: "a".into(),
        };

        let result = checker
            .check_expr(ast::expr::Expression::Ident(ident_raw))
            .unwrap();

        let constraints = checker.get_constraints().to_vec();

        let mut infer_ctx = InferContext::new(&mut module);

        let mut type_infer = TypeInfer::new(&mut infer_ctx);
        type_infer.unify_all(constraints).unwrap();
        type_infer.infer().unwrap();

        assert!(matches!(result, TypedExpression::Ident(..)));

        let (ident, ty) = match result {
            TypedExpression::Ident(it, ty) => (it, ty),
            _ => unreachable!(),
        };

        assert!(ident.value == "a".into());
        assert!(tcx.get(ty) == &Ty::BigInt);

        println!("ok! ident.value: {}, ty: {ty:#?}", ident.value)
    }

    #[test]
    fn test_checker_var_def() {
        let file: Arc<str> = "__test_checker_var_def__".into();

        let mut name_resolver = NameResolver::new(0.into(), &file);
        let mut tcx = TypeContext::new();
        let mut module = TypedModule::new(&mut tcx);

        let checker = &mut TypeChecker::new(&mut module, &mut name_resolver);

        let let_stmt_raw = ast::stmt::Statement::Let {
            name: ast::expressions::ident::Ident {
                token: Token::new("a".into(), TokenType::Ident, file.clone(), 1, 5),
                value: "a".into(),
            },
            value: ast::expr::Expression::UnknownTypeInt {
                token: Token::new("1".into(), TokenType::Integer, file.clone(), 1, 17),
                value: BigDecimal::from(1),
            },
            var_type: Some(ast::expressions::ident::Ident {
                token: Token::new("BigInt".into(), TokenType::Integer, file.clone(), 1, 8),
                value: "BigInt".into(),
            }),
            token: Token::new("let".into(), TokenType::Let, file.clone(), 1, 1),
        };

        let result = checker.check_statement(let_stmt_raw).unwrap();

        let constraints = checker.get_constraints().to_vec();

        let mut infer_ctx = InferContext::new(&mut module);

        let mut type_infer = TypeInfer::new(&mut infer_ctx);
        type_infer.unify_all(constraints).unwrap();
        type_infer.infer().unwrap();

        assert!(matches!(result, TypedStatement::Let { .. }));

        let (ident, ty) = match result {
            TypedStatement::Let { name, ty, .. } => (name, ty),
            _ => unreachable!(),
        };

        assert!(ident.value == "a".into());
        assert!(tcx.get(ty) == &Ty::BigInt);

        let get_symbol_result = tcx.table.lock().unwrap().get("a");
        let get_symbol_result_ref = get_symbol_result.as_ref();

        assert!(get_symbol_result.is_some());
        assert!(unsafe { get_symbol_result_ref.unwrap_unchecked() }.name == "a".into());
        assert!(
            tcx.get(
                unsafe { get_symbol_result_ref.unwrap_unchecked() }
                    .ty
                    .get_type()
            ) == &Ty::BigInt
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
