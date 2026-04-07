#[cfg(test)]
pub mod tests {
    use std::collections::HashMap;

    use ant_crate_def::{
        Crate, ModuleNode,
        definition::{ConstantData, Def, Visibility},
    };
    use id::StmtId;
    use indexmap::IndexMap;
    use ty::{IntTy, Ty};
    use typed_module::{
        ty_context::TypeContext,
        type_table::{Symbol, SymbolType},
    };

    use crate::{ModuleScope, NameResolver};

    #[test]
    fn test_name_resolver_lookup_name_local() {
        let file: std::sync::Arc<str> = "__test_name_resolver_lookup_name_local__".into();

        let mut tcx = TypeContext::new();

        let root_mod = ModuleNode {
            file: file.clone(),
            ast: None,
            path: vec![file.clone()],
            typed_module: None,
            exports: HashMap::new(),
            children: HashMap::new(),
        };

        let mut krate = Crate {
            definitions: vec![],
            modules: vec![],
            path_index: IndexMap::new(),
            root_id: 0.into(),
        };

        let mod_id = krate.alloc_mod(root_mod);
        krate.root_id = mod_id;

        let test_const_stack_size = Def::Constant(ConstantData {
            name: "STACK_SIZE".into(),
            visibility: Visibility::Private,
            module_id: mod_id,
            ty: tcx.alloc(Ty::IntTy(IntTy::I32)),
            ast_index: StmtId(0),
        });

        let expected_const_stack_size_id = krate.alloc_def(test_const_stack_size.clone());

        let mut local_maps = HashMap::new();
        local_maps.insert("STACK_SIZE".into(), expected_const_stack_size_id);

        let mut name_resolver = NameResolver::from_crate(krate, file, vec![]);
        name_resolver.local_maps.insert(mod_id, local_maps);

        let got_const_stack_size_id = name_resolver.lookup_name(mod_id, "STACK_SIZE");

        assert!(got_const_stack_size_id.is_some());

        let Some(got_const_stack_size_id) = got_const_stack_size_id else {
            unreachable!()
        };

        let got_const_stack_size = name_resolver.krate.get_def(got_const_stack_size_id);
        let expected_const_stack_size = name_resolver.krate.get_def(expected_const_stack_size_id);

        assert_eq!(expected_const_stack_size, got_const_stack_size);

        println!(
            "ok! expected_const_stack_size: {expected_const_stack_size:#?}, got_const_stack_size: {got_const_stack_size:#?}"
        )
    }

    #[test]
    fn test_name_resolver_lookup_name_outer() {
        let file: std::sync::Arc<str> = "__test_name_resolver_lookup_name_outer__".into();

        let mut tcx = TypeContext::new();

        let root_mod = ModuleNode {
            file: file.clone(),
            ast: None,
            path: vec![file.clone()],
            typed_module: None,
            exports: HashMap::new(),
            children: HashMap::new(),
        };

        let stack_size_ty = tcx.alloc(Ty::IntTy(IntTy::I32));

        let mod_constants = ModuleNode {
            file: "constants".into(),
            ast: None,
            path: vec!["constants".into()],
            typed_module: None,
            exports: {
                let mut map = HashMap::new();
                map.insert(
                    "STACK_SIZE".into(),
                    Symbol {
                        name: "STACK_SIZE".into(),
                        ty: SymbolType::Variable(stack_size_ty),
                    },
                );

                map
            },
            children: HashMap::new(),
        };

        let mut krate = Crate {
            definitions: vec![],
            modules: vec![],
            path_index: IndexMap::new(),
            root_id: 0.into(),
        };

        let root_mod_id = krate.alloc_mod(root_mod);
        let mod_constants_id = krate.alloc_mod(mod_constants);
        krate.root_id = root_mod_id;

        let test_const_stack_size = Def::Constant(ConstantData {
            name: "STACK_SIZE".into(),
            visibility: Visibility::Public,
            module_id: mod_constants_id,
            ty: stack_size_ty,
            ast_index: StmtId(0),
        });

        let expected_const_stack_size_id = krate.alloc_def(test_const_stack_size.clone());

        let mut local_maps = HashMap::new();
        local_maps.insert("STACK_SIZE".into(), expected_const_stack_size_id);

        let mut resolved_imports = HashMap::new();
        resolved_imports.insert(
            root_mod_id,
            ModuleScope {
                bindings: {
                    let mut map = HashMap::new();
                    map.insert("STACK_SIZE".into(), expected_const_stack_size_id);

                    map
                },
            },
        );

        let mut name_resolver = NameResolver::from_crate(krate, file, vec![]);
        name_resolver.resolved_imports = resolved_imports;
        name_resolver
            .local_maps
            .insert(mod_constants_id, local_maps);

        let got_const_stack_size_id = name_resolver.lookup_name(root_mod_id, "STACK_SIZE");

        let Some(got_const_stack_size_id) = got_const_stack_size_id else {
            unreachable!()
        };

        let got_const_stack_size = name_resolver.krate.get_def(got_const_stack_size_id);
        let expected_const_stack_size = name_resolver.krate.get_def(expected_const_stack_size_id);

        assert_eq!(expected_const_stack_size, got_const_stack_size);

        println!(
            "ok! expected_const_stack_size: {expected_const_stack_size:#?}, got_const_stack_size: {got_const_stack_size:#?}"
        )
    }
}
