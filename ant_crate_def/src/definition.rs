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
use type_checker::ty::TyId;

use crate::ModuleId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Def {
    Module(ModuleData),
    Struct(StructData),
    Function(FunctionData),
    Trait(TraitData),
    Constant(ConstantData),
}

impl Def {
    pub fn visibility(&self) -> Visibility {
        match self {
            Def::Module(module_data) => module_data.visibility,
            Def::Struct(struct_data) => struct_data.visibility,
            Def::Function(function_data) => function_data.visibility,
            Def::Trait(trait_data) => trait_data.visibility,
            Def::Constant(constant_data) => constant_data.visibility,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleData {
    pub name: Arc<str>,
    pub parent: Option<ModuleId>,
    pub path: Vec<Arc<str>>, // 全路径 (包括本身, 例: ["std", "vec"])
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructData {
    pub name: Arc<str>,
    pub visibility: Visibility,
    pub module_id: ModuleId,
    pub generics: Vec<Arc<str>>, 
    pub fields: IndexMap<Arc<str>, TyId>, 
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionData {
    pub name: Arc<str>,
    pub visibility: Visibility,
    pub module_id: ModuleId,
    pub generics: Vec<Arc<str>>,
    pub params: Vec<TyId>,
    pub ret_ty: TyId,
    pub is_variadic: bool,
    /// 如果是函数声明，这里可能是 None
    pub body: Option<ExprId>, 
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitData {
    pub name: Arc<str>,
    pub visibility: Visibility,
    pub module_id: ModuleId,
    pub methods: IndexMap<Arc<str>, DefId>, // 指向 Function 定义
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantData {
    pub name: Arc<str>,
    pub visibility: Visibility,
    pub module_id: ModuleId,
    pub ty: TyId,
}