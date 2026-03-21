macro_rules! dervie_from_num {
    ($num_ty:ty, $impl_ty:ty) => {
        impl From<$num_ty> for $impl_ty {
            fn from(value: $num_ty) -> $impl_ty {
                Self(value as usize)
            }
        }
    };
}

macro_rules! dervie_into_num {
    ($num_ty:ty, $impl_ty:ty) => {
        impl Into<$num_ty> for $impl_ty {
            fn into(self) -> $num_ty {
                self.0 as $num_ty
            }
        }
    };
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DefId(pub usize);

dervie_from_num!(u64, DefId);
dervie_from_num!(u32, DefId);
dervie_from_num!(u16, DefId);
dervie_from_num!(u8, DefId);
dervie_from_num!(usize, DefId);
dervie_from_num!(i64, DefId);
dervie_from_num!(i32, DefId);
dervie_from_num!(i16, DefId);
dervie_from_num!(i8, DefId);
dervie_from_num!(isize, DefId);

dervie_into_num!(u64, DefId);
dervie_into_num!(u32, DefId);
dervie_into_num!(u16, DefId);
dervie_into_num!(u8, DefId);
dervie_into_num!(usize, DefId);
dervie_into_num!(i64, DefId);
dervie_into_num!(i32, DefId);
dervie_into_num!(i16, DefId);
dervie_into_num!(i8, DefId);
dervie_into_num!(isize, DefId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
}

use std::sync::Arc;
use ast::ExprId;
use indexmap::IndexMap;
use crate::ty::TyId;
use crate::ant_crate::ModuleId;

#[derive(Debug, Clone)]
pub enum Def {
    Module(ModuleData),
    Struct(StructData),
    Function(FunctionData),
    Trait(TraitData),
    Constant(ConstantData),
}

#[derive(Debug, Clone)]
pub struct ModuleData {
    pub name: Arc<str>,
    pub parent: Option<ModuleId>,
    pub path: Vec<Arc<str>>, // 全路径 (包括本身, 例: ["std", "vec"])
}

#[derive(Debug, Clone)]
pub struct StructData {
    pub name: Arc<str>,
    pub vis: Visibility,
    pub module_id: ModuleId,
    pub generics: Vec<Arc<str>>, 
    pub fields: IndexMap<Arc<str>, TyId>, 
}

#[derive(Debug, Clone)]
pub struct FunctionData {
    pub name: Arc<str>,
    pub vis: Visibility,
    pub module_id: ModuleId,
    pub generics: Vec<Arc<str>>,
    pub params: Vec<TyId>,
    pub ret_ty: TyId,
    pub is_variadic: bool,
    /// 如果是 Extern 函数，这里可能是 None
    pub body: Option<ExprId>, 
}

#[derive(Debug, Clone)]
pub struct TraitData {
    pub name: Arc<str>,
    pub vis: Visibility,
    pub module_id: ModuleId,
    pub methods: IndexMap<Arc<str>, DefId>, // 指向 Function 定义
}

#[derive(Debug, Clone)]
pub struct ConstantData {
    pub name: Arc<str>,
    pub vis: Visibility,
    pub module_id: ModuleId,
    pub ty: TyId,
}