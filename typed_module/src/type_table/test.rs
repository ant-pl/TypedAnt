#[cfg(test)]
mod tests {
    use crate::ty_context::TypeContext;

    use crate::type_table::{Symbol, SymbolType, TypeTable};

    fn expected_symbol(expected: &Symbol, got: &Symbol) {
        if expected != got {
            eprintln!("expected: {expected:#?}, got: {got:#?}");
            assert_eq!(expected, got)
        }

        println!("ok! expected: {expected:#?}, got: {got:#?}")
    }

    #[test]
    fn test_define() {
        let mut tcx = TypeContext::new();

        let expected = Symbol {
            name: "a".into(),
            ty: SymbolType::Variable(tcx.alloc(ty::Ty::BigInt))
        };

        let mut table = TypeTable::new();

        expected_symbol(&expected, &table.define_var("a", tcx.alloc(ty::Ty::BigInt)));
    }

    #[test]
    fn test_table_get() {
        let mut tcx = TypeContext::new();
        
        let expected = Symbol {
            name: "a".into(),
            ty: SymbolType::Variable(tcx.alloc(ty::Ty::BigInt))
        };

        let mut table = TypeTable::new();

        table.define_var("a", tcx.alloc(ty::Ty::BigInt));

        let got = table.get("a");

        assert!(got.is_some());

        expected_symbol(&expected, &got.unwrap());
    }
}