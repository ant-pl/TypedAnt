pub mod test;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    Ty, ty::{IntTy, TyId}, ty_context::TypeContext, typed_ast::GetType
};

pub enum SymbolScope {
    Global,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolType {
    Variable(TyId),
    Function(TyId),
}

impl GetType for SymbolType {
    fn get_type(&self) -> TyId {
        match self {
            Self::Variable(ty) => *ty,
            Self::Function(ty) => *ty,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub name: Arc<str>,
    pub ty: SymbolType,
}

#[derive(Debug, Clone)]
pub struct TypeTable {
    pub outer: Option<Arc<Mutex<TypeTable>>>,

    pub var_map: HashMap<Arc<str>, Symbol>,
}

impl TypeTable {
    pub fn init_table(&mut self, tcx: &mut TypeContext) {
        self.define_var("str", tcx.alloc(Ty::Str));
        self.define_var("BigInt", tcx.alloc(Ty::BigInt));
        self.define_var("bool", tcx.alloc(Ty::Bool));
        self.define_var("unit", tcx.alloc(Ty::Unit));

        // signed
        self.define_var("i64", tcx.alloc(Ty::IntTy(IntTy::I64)));
        self.define_var("i32", tcx.alloc(Ty::IntTy(IntTy::I32)));
        self.define_var("i16", tcx.alloc(Ty::IntTy(IntTy::I16)));
        self.define_var("i8", tcx.alloc(Ty::IntTy(IntTy::I8)));
        self.define_var("isize", tcx.alloc(Ty::IntTy(IntTy::ISize)));

        // unsigned
        self.define_var("u64", tcx.alloc(Ty::IntTy(IntTy::U64)));
        self.define_var("u32", tcx.alloc(Ty::IntTy(IntTy::U32)));
        self.define_var("u16", tcx.alloc(Ty::IntTy(IntTy::U16)));
        self.define_var("u8", tcx.alloc(Ty::IntTy(IntTy::U8)));
        self.define_var("usize", tcx.alloc(Ty::IntTy(IntTy::USize)));
    }

    pub fn init(mut self, tcx: &mut TypeContext) -> Self {
        self.init_table(tcx);
        self
    }

    pub fn with_outer(outer: Arc<Mutex<TypeTable>>) -> Self {
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
            return outer.lock().unwrap().get(name);
        }

        None
    }

    pub fn insert_var(&mut self, symbol: Symbol) {
        self.var_map.insert(symbol.name.clone(), symbol);
    }

    pub fn define_var(&mut self, name: &str, var_ty: TyId) -> Symbol {
        let sym = Symbol {
            name: name.into(),
            ty: SymbolType::Variable(var_ty),
        };

        self.insert_var(sym.clone());

        sym
    }

    pub fn remove(&mut self, name: &str) {
        self.var_map.remove(name);
    }
}
