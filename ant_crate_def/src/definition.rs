macro_rules! get_field {
    ($self:expr,$field:ident) => {
        match $self {
            Self::Module(data) => data.$field,
            Self::Struct(data) => data.$field,
            Self::Function(data) => data.$field,
            Self::Trait(data) => data.$field,
            Self::Constant(data) => data.$field,
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
use ty::TyId;

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
            Self::Struct(data) => Some(data.ty),
            Self::Function(data) => Some(data.ty),
            Self::Trait(data) => Some(data.ty),
            Self::Constant(data) => Some(data.ty),
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
    pub ty: TyId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionData {
    pub name: Arc<str>,
    pub visibility: Visibility,
    pub module_id: ModuleId,
    pub is_variadic: bool,
    /// 如果是函数声明，这里可能是 None
    pub body: Option<ExprId>,
    pub ast_index: StmtId,
    pub ty: TyId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitData {
    pub name: Arc<str>,
    pub visibility: Visibility,
    pub module_id: ModuleId,
    pub methods: IndexMap<Arc<str>, DefId>, // 指向 Function 定义
    pub ast_index: StmtId,
    pub ty: TyId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantData {
    pub name: Arc<str>,
    pub visibility: Visibility,
    pub module_id: ModuleId,
    pub ast_index: StmtId,
    pub ty: TyId,
}
