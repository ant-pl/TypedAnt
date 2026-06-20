pub mod definition;

use std::{collections::HashMap, sync::Arc};

use ast::node::Node;
use id::{DefId, ModuleId};
use indexmap::IndexMap;

use typed_ast::typed_node::TypedNode;
use typed_module::{module::TypedModule, type_table::Symbol};

use crate::definition::Def;

#[derive(Debug, Clone)]
pub enum NodeOrTyped {
    Node(Node),
    Typed(TypedNode),
}

#[derive(Debug, Default)]
pub struct ModuleNode<'a> {
    pub file: Arc<str>,
    pub parent: Option<ModuleId>,
    pub path: Vec<Arc<str>>,      // 从父路径一直到当前路径 (包含当前路径)
    pub ast: Option<NodeOrTyped>, // 模块 AST
    pub typed_module: Option<TypedModule<'a>>, // 类型检查后的结果
    pub exports: HashMap<Arc<str>, Symbol>, // 该模块导出的符号
    pub children: HashMap<Arc<str>, ModuleId>, // 子模块
}

#[derive(Debug)]
pub struct Crate<'a> {
    pub definitions: Vec<Def>,

    /// 被实现的 ADT 类型到所有实现的定义的映射
    pub impls: IndexMap<DefId, Vec<DefId>>,

    /// 从模块路径到模块定义的映射
    pub path_index: IndexMap<Vec<Arc<str>>, DefId>,

    pub modules: Vec<ModuleNode<'a>>,
    pub root_id: ModuleId,
}

impl<'a> Crate<'a> {
    pub fn get_def(&self, id: DefId) -> &Def {
        &self.definitions[id.0]
    }

    pub fn get_mut_def(&mut self, id: DefId) -> &mut Def {
        &mut self.definitions[id.0]
    }

    pub fn alloc_def(&mut self, def: Def) -> DefId {
        let id = DefId(self.definitions.len());

        self.definitions.push(def);

        id
    }
}

impl<'a> Crate<'a> {
    pub fn get_mod(&'_ self, id: ModuleId) -> &'_ ModuleNode<'_> {
        &self.modules[id.0]
    }

    pub fn get_mut_mod(&mut self, id: ModuleId) -> &mut ModuleNode<'a> {
        &mut self.modules[id.0]
    }

    pub fn alloc_mod(&mut self, module: ModuleNode<'a>) -> ModuleId {
        let id = ModuleId(self.modules.len());

        self.modules.push(module);

        id
    }
}

impl<'a> Crate<'a> {
    pub fn get_impls(&'_ self, id: DefId) -> &Vec<DefId> {
        self.impls.get(&id).unwrap()
    }

    pub fn get_mut_impls(&mut self, id: DefId) -> &mut Vec<DefId> {
        self.impls.get_mut(&id).unwrap()
    }

    pub fn alloc_impl_to_adt(&mut self, impl_def: Def, adt_def: DefId) -> DefId {
        let id = self.alloc_def(impl_def);

        self.impls.entry(adt_def).or_insert(Vec::new()).push(id);

        id
    }

    pub fn alloc_impl(&mut self, impl_def: Def) -> DefId {
        let Def::Impl(it) = &impl_def else {
            unreachable!()
        };

        let adt = it.target_def;

        self.alloc_impl_to_adt(impl_def, adt)
    }
}

impl<'a> Crate<'a> {
    /// 检查 a 是否为 b 的后代或者相等
    pub fn is_eq_or_succ_module(&self, mod_a_id: ModuleId, mod_b_id: ModuleId) -> bool {
        mod_a_id == mod_b_id || {
            let mut cur_mod_id = mod_a_id;

            while let Some(id) = self
                .modules
                .get(cur_mod_id.0)
                .and_then(|it| it.parent.clone())
            {
                if id == mod_b_id {
                    return true
                }

                cur_mod_id = id
            }

            return false
        }
    }
}
