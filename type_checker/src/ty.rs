use std::{collections::HashMap, fmt::Display, rc::Rc};

use ast::expr::IntValue;

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
    Struct(Rc<str>, HashMap<Rc<str>, Ty>),
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

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
            (Ty::Struct("".into(), HashMap::new()), "struct {}"),
            (Ty::Struct("".into(), {
                let mut m = HashMap::new();

                m.insert("it".into(), Ty::IntTy(IntTy::U64));

                m
            }), "struct {\n\tit: u64\n}"),
        ];

        for (ty, expected) in cases {
            expected_ty_display(&ty, expected);
        }
    }
}