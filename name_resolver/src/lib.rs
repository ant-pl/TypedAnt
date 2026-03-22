use ant_crate_def::definition::{Def, FunctionData, StructData, Visibility};
use ant_crate_def::{Crate, ModuleId, definition::DefId};
use ast::expr::Expression;
use ast::stmt::Statement;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ModuleScope {
    /// 记录该模块内有效的所有名字引用
    /// 例：use std::vec::Vec 之后，"Vec" -> DefId(101)
    pub bindings: HashMap<Arc<str>, DefId>,
}

pub struct Resolver<'a> {
    pub krate: Crate<'a>,

    /// ModuleId -> (LocalName -> DefId)
    pub local_maps: HashMap<ModuleId, HashMap<Arc<str>, DefId>>,
}

impl<'a> Resolver<'a> {
    pub fn new(root_module_id: ModuleId) -> Self {
        Self {
            krate: Crate {
                definitions: Vec::new(),
                path_index: IndexMap::new(),
                modules: Vec::new(),
                root_id: root_module_id,
            },
            local_maps: HashMap::new(),
        }
    }
}

impl<'a> Resolver<'a> {
    /// 遍历模块的 AST，收集顶层定义
    pub fn resolve_module_definitions(&mut self, module_id: ModuleId, stmts: &[Statement]) {
        let mut local_symbols = HashMap::new();

        for stmt in stmts {
            match stmt {
                Statement::Struct { name, generics, .. } => {
                    // 构造原始定义数据
                    let data = StructData {
                        name: name.value.clone(),
                        visibility: Visibility::Public, // 还没写访问控制语法先等着吧
                        module_id,
                        generics: generics.iter().map(|g| g.to_string().into()).collect(),
                        fields: IndexMap::new(), // TypeChecker 稍后填充
                    };

                    let def_id = self.krate.alloc_def(Def::Struct(data));
                    local_symbols.insert(name.value.clone(), def_id);
                }

                Statement::FuncDecl {
                    name,
                    generics_params,
                    ..
                } => {
                    let generic_names = generics_params
                        .iter()
                        .filter(|it| matches!(&***it, Expression::Ident(..)))
                        .map(|it| {
                            let Expression::Ident(it) = &**it else {
                                unreachable!()
                            };
                            it.value.clone()
                        });

                    let data = FunctionData {
                        name: name.value.clone(),
                        visibility: Visibility::Public, // 还没写访问控制语法先等着吧
                        module_id,
                        generics: generic_names.collect(),
                        body: None,
                        is_variadic: false, // 非外部函数一律不允许变长
                        params: vec![],     // 等到 TypeCheck 阶段填充，
                        ret_ty: 0usize,     // 同上
                    };

                    local_symbols.insert(
                        name.value.clone(),
                        self.krate.alloc_def(Def::Function(data)),
                    );
                }

                // 处理 Trait, Const, 子模块等...

                _ => {}
            }
        }

        self.local_maps.insert(module_id, local_symbols);
    }
}

impl<'a> Resolver<'a> {
    pub fn build_path_index(&mut self) {
        let mut path_index = IndexMap::new();

        for (mod_id, symbols) in &self.local_maps {
            let mod_path = &self.krate.modules[mod_id.0].path;

            for (name, def_id) in symbols {
                let mut full_path = mod_path.clone();
                full_path.push(name.clone());
                path_index.insert(full_path, *def_id);
            }
        }

        self.krate.path_index = path_index;
    }
}