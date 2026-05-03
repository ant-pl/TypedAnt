macro_rules! get_field {
    ($self:expr,$field:ident) => {
        match $self {
            Self::Module(data) => data.$field,
            Self::Struct(data) => data.$field,
            Self::Function(data) => data.$field,
            Self::Trait(data) => data.$field,
            Self::Constant(data) => data.$field,
            Self::Impl(data) => data.$field,
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Visibility {
    Public,
    Private,
}

use id::{DefId, ModuleId};
use id::{ExprId, StmtId};
use std::sync::Arc;

use indexmap::IndexMap;
use ty::{TyCell, TyId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Def {
    Module(ModuleData),
    Struct(StructData),
    Function(FunctionData),
    Trait(TraitData),
    Constant(ConstantData),
    Impl(ImplData),
}

impl Def {
    pub fn visibility(&self) -> Visibility {
        get_field!(self, visibility)
    }

    pub fn ast_index(&self) -> StmtId {
        get_field!(self, ast_index)
    }

    pub fn module_id(&self) -> ModuleId {
        get_field!(self, module_id)
    }

    pub fn ty(&self) -> Option<TyId> {
        match self {
            Self::Module(_data) => None,
            Self::Struct(data) => Some(data.ty.get()),
            Self::Function(data) => Some(data.ty.get()),
            Self::Trait(data) => Some(data.ty.get()),
            Self::Constant(data) => Some(data.ty.get()),
            Self::Impl(data) => Some(data.ty.get()),
        }
    }

    pub fn set_ty(&self, new_ty: TyId) {
        match self {
            Self::Module(_data) => (),
            Self::Struct(data) => data.ty.set(new_ty),
            Self::Function(data) => data.ty.set(new_ty),
            Self::Trait(data) => data.ty.set(new_ty),
            Self::Constant(data) => data.ty.set(new_ty),
            Self::Impl(data) => data.ty.set(new_ty),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleData {
    pub name: Arc<str>,
    pub parent: Option<ModuleId>,
    pub path: Vec<Arc<str>>, // 全路径 (包括本身, 例: ["std", "vec"])
    pub visibility: Visibility,
    pub ast_index: StmtId,
    /// 模块本身的 id
    pub module_id: ModuleId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructData {
    pub name: Arc<str>,
    pub visibility: Visibility,
    pub module_id: ModuleId,
    pub generics: Vec<Arc<str>>,
    pub fields: IndexMap<Arc<str>, TyId>,
    pub ast_index: StmtId,
    pub ty: TyCell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionData {
    pub name: Arc<str>,
    pub visibility: Visibility,
    pub module_id: ModuleId,
    pub ast_index: StmtId,
    pub is_variadic: bool,
    pub params: IndexMap<Arc<str>, TyId>,
    /// 上层结构的 DefId
    pub parent: Option<DefId>,
    /// 如果是函数声明，这里可能是 None
    pub body: Option<ExprId>,
    pub ty: TyCell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitData {
    pub name: Arc<str>,
    pub visibility: Visibility,
    pub module_id: ModuleId,
    pub methods: IndexMap<Arc<str>, DefId>, // 指向 Function 定义
    pub ast_index: StmtId,
    pub ty: TyCell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantData {
    pub name: Arc<str>,
    pub visibility: Visibility,
    pub module_id: ModuleId,
    pub ast_index: StmtId,
    pub ty: TyCell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplData {
    pub visibility: Visibility,
    pub module_id: ModuleId,
    pub ast_index: StmtId,
    /// impl 块定义时的泛型 (即impl<T, ...>)
    pub generics: Vec<Arc<str>>,
    /// 在 impl 块中定义的所有方法
    pub methods: IndexMap<Arc<str>, DefId>,
    /// 实现目标的类型
    pub target_ty: TyCell,
    /// 实现目标的全局定义
    pub target_def: DefId,
    /// 即为 unit
    pub ty: TyCell
}