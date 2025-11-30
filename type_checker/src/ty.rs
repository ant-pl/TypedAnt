use std::{fmt::Display, rc::Rc};

use ast::expr::IntValue;
use indexmap::IndexMap;

fn get_platform_width() -> usize {
    #[cfg(target_pointer_width = "64")]
    return 64;

    #[cfg(target_pointer_width = "32")]
    return 32;

    #[cfg(target_pointer_width = "16")]
    return 16;
}

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
        params_type: Vec<Ty>,
        ret_type: Box<Ty>,
    },
    Struct(Rc<str>, IndexMap<Rc<str>, Ty>),
    IntTy(IntTy),
    Bool,
    Unit,
    Str,
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BigInt => write!(f, "BigInt"),
            Self::Str => write!(f, "str"),
            Self::IntTy(it) => write!(f, "{it}"),
            Self::Bool => write!(f, "bool"),
            Self::Unit => write!(f, "Unit"),
            Self::Struct(_, it) => {
                if it.is_empty() {
                    write!(f, "struct {{}}")
                } else {
                    writeln!(f, "struct {{")?;
                    for (name, ty) in it {
                        writeln!(f, "\t{name}: {ty}")?;
                    }
                    write!(f, "}}")
                }
            }
            Self::Function {
                params_type,
                ret_type,
            } => write!(
                f,
                "Func({}) -> {}",
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

        _ => None
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
            (Ty::Struct("".into(), IndexMap::new()), "struct {}"),
            (
                Ty::Struct("".into(), {
                    let mut m = IndexMap::new();

                    m.insert("it".into(), Ty::IntTy(IntTy::U64));

                    m
                }),
                "struct {\n\tit: u64\n}",
            ),
        ];

        for (ty, expected) in cases {
            expected_ty_display(&ty, expected);
        }
    }
}