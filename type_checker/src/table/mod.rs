pub mod test;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{Ty, typed_ast::GetType};

pub enum SymbolScope {
    Global,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolType {
    Variable(Ty),
    Function {
        params_type: Vec<Ty>,
        ret_type: Ty,
        is_variadic: bool,
    },
}

impl GetType for SymbolType {
    fn get_type(&self) -> Ty {
        match self {
            Self::Variable(ty) => ty.clone(),
            Self::Function {
                params_type,
                ret_type,
                is_variadic,
            } => Ty::Function {
                params_type: params_type.clone(),
                ret_type: Box::new(ret_type.clone()),
                is_variadic: *is_variadic
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: Rc<str>,
    pub ty: SymbolType,
}

pub struct TypeTable {
    pub outer: Option<Rc<RefCell<TypeTable>>>,

    var_map: HashMap<Rc<str>, Symbol>,
}

impl TypeTable {
    pub fn with_outer(outer: Rc<RefCell<TypeTable>>) -> Self {
        Self {
            outer: Some(outer),
            var_map: HashMap::new(),
        }
    }

    pub fn new() -> Self {
        Self {
            outer: None,
            var_map: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<Symbol> {
        let try_get = self.var_map.get(name.into());

        if let Some(it) = try_get {
            return Some(it.clone());
        }

        if let Some(outer) = &self.outer {
            return outer.borrow().get(name);
        }

        None
    }

    pub fn insert_var(&mut self, symbol: Symbol) {
        self.var_map.insert(symbol.name.clone(), symbol);
    }

    pub fn define_var(&mut self, name: &str, var_ty: Ty) -> Symbol {
        let sym = Symbol {
            name: name.into(),
            ty: SymbolType::Variable(var_ty),
        };

        self.insert_var(sym.clone());

        sym
    }
}
