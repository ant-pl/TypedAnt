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
    pub path: Vec<Arc<str>>,      // 从父路径一直到当前路径 (包含当前路径)
    pub ast: Option<NodeOrTyped>, // 模块 AST
    pub typed_module: Option<TypedModule<'a>>, // 类型检查后的结果
    pub exports: HashMap<Arc<str>, Symbol>, // 该模块导出的符号
    pub children: HashMap<Arc<str>, ModuleId>, // 子模块
}

#[derive(Debug)]
pub struct Crate<'a> {
    pub definitions: Vec<Def>,

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
