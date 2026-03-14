use std::{fmt::Display, sync::Arc};

use ast::expr::IntValue;
use indexmap::IndexMap;

use crate::ty_context::TypeContext;

fn get_platform_width() -> usize {
    #[cfg(target_pointer_width = "64")]
    return 64;

    #[cfg(target_pointer_width = "32")]
    return 32;

    #[cfg(target_pointer_width = "16")]
    return 16;
}

pub type TyId = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntTy {
    ISize,
    I64,
    I32,
    I16,
    I8,
    USize,
    U64,
    U32,
    U16,
    U8,
}

impl IntTy {
    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            Self::ISize | Self::I64 | Self::I32 | Self::I16 | Self::I8
        )
    }
}

impl IntTy {
    pub fn get_bytes_size(&self) -> usize {
        match *self {
            Self::I8 | Self::U8 => 1,
            Self::I16 | Self::U16 => 2,
            Self::I32 | Self::U32 => 4,
            Self::I64 | Self::U64 => 8,
            Self::ISize | Self::USize => get_platform_width() / 8,
        }
    }
}

impl Display for IntTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::I64 => "i64",
            Self::I32 => "i32",
            Self::I16 => "i16",
            Self::I8 => "i8",
            Self::U64 => "u64",
            Self::U32 => "u32",
            Self::U16 => "u16",
            Self::U8 => "u8",
            Self::ISize => "isize",
            Self::USize => "usize",
        };

        write!(f, "{s}")
    }
}

impl From<IntValue> for IntTy {
    fn from(value: IntValue) -> Self {
        match value {
            IntValue::ISize(_) => Self::ISize,
            IntValue::I64(_) => Self::I64,
            IntValue::I32(_) => Self::I32,
            IntValue::I16(_) => Self::I16,
            IntValue::I8(_) => Self::I8,
            IntValue::USize(_) => Self::USize,
            IntValue::U64(_) => Self::U64,
            IntValue::U32(_) => Self::U32,
            IntValue::U16(_) => Self::U16,
            IntValue::U8(_) => Self::U8,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ty {
    BigInt,
    Function {
        params_type: Vec<TyId>,
        ret_type: TyId,
        is_variadic: bool,
    },
    Struct {
        name: Arc<str>,
        generics: Vec<Arc<str>>,
        fields: IndexMap<Arc<str>, TyId>,
        impl_traits: IndexMap<Arc<str>, TyId>,
    },
    Trait {
        name: Arc<str>,
        functions: IndexMap<Arc<str>, TyId>,
    },

    // T, K: Eq, V: Eq + Clone ...
    Generic(Arc<str>, Vec<TyId>),

    /// StructName<AppliedType>
    AppliedGeneric(Arc<str>, Vec<TyId>),

    Infer(usize),
    IntTy(IntTy),
    Ptr(TyId),
    Bool,
    Unit,
    Str,
    Unknown,
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "unknown"),
            Self::Ptr(it) => write!(f, "*{it}"),
            Self::Infer(it) => write!(f, "Infer({it})"),
            Self::Generic(it, _) => write!(f, "{it}"),
            Self::AppliedGeneric(it, _) => write!(f, "{it}"),
            Self::BigInt => write!(f, "BigInt"),
            Self::Str => write!(f, "str"),
            Self::IntTy(it) => write!(f, "{it}"),
            Self::Bool => write!(f, "bool"),
            Self::Unit => write!(f, "Unit"),
            Self::Struct { name, .. } => write!(f, "{name}"),
            Self::Trait { name, .. } => write!(f, "{name}"),
            Self::Function {
                params_type,
                ret_type,
                ..
            } => write!(
                f,
                "Fn({}) -> {}",
                params_type
                    .iter()
                    .map(|it| it.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
                ret_type
            ),
        }
    }
}

pub fn display_ty(ty: &Ty, tcx: &TypeContext) -> String {
    match ty {
        Ty::Unknown => "unknown".to_owned(),
        Ty::BigInt => "BigInt".to_owned(),
        Ty::Str => "str".to_owned(),
        Ty::IntTy(it) => it.to_string(),
        Ty::Bool => "bool".to_owned(),
        Ty::Unit => "Unit".to_owned(),
        Ty::Ptr(it) => format!("*{}", display_ty(tcx.get(*it), tcx)),
        Ty::Infer(it) => format!("Infer({it})"),
        Ty::Generic(it, _impl_traits) => it.to_string(),
        Ty::AppliedGeneric(it, args) => format!(
            "{it}{}",
            if !args.is_empty() {
                "<".to_owned()
                    + &args
                        .iter()
                        .map(|it| display_ty(tcx.get(*it), tcx))
                        .collect::<Vec<_>>()
                        .join(", ")
                    + ">"
            } else {
                String::new()
            }
        ),
        Ty::Struct { name, generics, .. } => format!(
            "{name}{}",
            if !generics.is_empty() {
                format!("<{}>", generics.join(", "))
            } else {
                String::new()
            }
        ),
        Ty::Trait { name, .. } => name.to_string(),
        Ty::Function {
            params_type,
            ret_type,
            ..
        } => format!(
            "Fn({}) -> {}",
            params_type
                .iter()
                .map(|it| display_ty(tcx.get(*it), tcx))
                .collect::<Vec<String>>()
                .join(", "),
            display_ty(tcx.get(*ret_type), tcx)
        ),
    }
}

pub fn str_to_ty(ty_str: &str) -> Option<Ty> {
    match ty_str {
        "str" => Some(Ty::Str),
        "i64" => Some(Ty::IntTy(crate::ty::IntTy::I64)),
        "i32" => Some(Ty::IntTy(crate::ty::IntTy::I32)),
        "i16" => Some(Ty::IntTy(crate::ty::IntTy::I16)),
        "i8" => Some(Ty::IntTy(crate::ty::IntTy::I8)),
        "u64" => Some(Ty::IntTy(crate::ty::IntTy::U64)),
        "u32" => Some(Ty::IntTy(crate::ty::IntTy::U32)),
        "u16" => Some(Ty::IntTy(crate::ty::IntTy::U16)),
        "u8" => Some(Ty::IntTy(crate::ty::IntTy::U8)),
        "usize" => Some(Ty::IntTy(crate::ty::IntTy::USize)),
        "isize" => Some(Ty::IntTy(crate::ty::IntTy::ISize)),
        "BigInt" => Some(Ty::BigInt),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use crate::ty::{IntTy, Ty};

    fn expected_ty_display(ty: &Ty, expected: &str) {
        assert_eq!(&ty.to_string(), expected);

        println!("ok! ty: {ty}, expected: {expected}")
    }

    #[test]
    fn expected_ty_displays() {
        let cases = vec![
            (Ty::BigInt, "BigInt"),
            (Ty::IntTy(IntTy::I64), "i64"),
            (Ty::IntTy(IntTy::I32), "i32"),
            (Ty::IntTy(IntTy::I16), "i16"),
            (Ty::IntTy(IntTy::I8), "i8"),
            (Ty::IntTy(IntTy::U64), "u64"),
            (Ty::IntTy(IntTy::U32), "u32"),
            (Ty::IntTy(IntTy::U16), "u16"),
            (Ty::IntTy(IntTy::U8), "u8"),
            (Ty::IntTy(IntTy::USize), "usize"),
            (Ty::IntTy(IntTy::ISize), "isize"),
            (Ty::BigInt, "BigInt"),
            (
                Ty::Struct {
                    name: "CialloInfo".into(),
                    generics: Vec::new(),
                    fields: IndexMap::new(),
                    impl_traits: IndexMap::new(),
                },
                "CialloInfo",
            ),
        ];

        for (ty, expected) in cases {
            expected_ty_display(&ty, expected);
        }
    }
}
