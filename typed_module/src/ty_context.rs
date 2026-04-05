use std::sync::{Arc, Mutex};

use ty::{Ty, TyId};

use crate::type_table::TypeTable;

#[derive(Clone)]
pub struct TypeContext {
    pub types: Vec<Ty>,
    pub table: Arc<Mutex<TypeTable>>,
}

impl std::fmt::Debug for TypeContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_table = self.table.fmt(f);

        f.debug_struct("TypeContext")
            .field(
                "types",
                &self.types.iter().enumerate().collect::<Vec<(usize, &Ty)>>(),
            )
            .field("table", &fmt_table?)
            .finish()
    }
}

impl TypeContext {
    pub fn alloc(&mut self, ty: Ty) -> TyId {
        for (i, old) in self.types.iter().enumerate() {
            if *old == ty {
                return i;
            }
        }

        let id = self.types.len();
        self.types.push(ty);
        id
    }
}

impl TypeContext {
    pub fn get(&self, id: TyId) -> &Ty {
        &self.types[id]
    }

    pub fn get_mut(&mut self, id: TyId) -> &mut Ty {
        &mut self.types[id]
    }
}

impl TypeContext {
    pub fn new() -> Self {
        let mut me = Self {
            types: vec![],
            table: Arc::new(Mutex::new(TypeTable::new())),
        };

        let new_table = TypeTable::new().init(&mut me);

        me.table = Arc::new(Mutex::new(new_table));

        me
    }
}
