macro_rules! derive_from_num {
    ($num_ty:ty, $impl_ty:ty) => {
        impl From<$num_ty> for $impl_ty {
            fn from(value: $num_ty) -> $impl_ty {
                Self(value as usize)
            }
        }
    };
}

macro_rules! derive_into_num {
    ($num_ty:ty, $impl_ty:ty) => {
        impl Into<$num_ty> for $impl_ty {
            fn into(self) -> $num_ty {
                self.0 as $num_ty
            }
        }
    };
}

macro_rules! derive_id_into_from {
    ($ty:ty) => {
        derive_from_num!(u64, $ty);
        derive_from_num!(u32, $ty);
        derive_from_num!(u16, $ty);
        derive_from_num!(u8, $ty);
        derive_from_num!(usize, $ty);
        derive_from_num!(i64, $ty);
        derive_from_num!(i32, $ty);
        derive_from_num!(i16, $ty);
        derive_from_num!(i8, $ty);
        derive_from_num!(isize, $ty);

        derive_into_num!(u64, $ty);
        derive_into_num!(u32, $ty);
        derive_into_num!(u16, $ty);
        derive_into_num!(u8, $ty);
        derive_into_num!(usize, $ty);
        derive_into_num!(i64, $ty);
        derive_into_num!(i32, $ty);
        derive_into_num!(i16, $ty);
        derive_into_num!(i8, $ty);
        derive_into_num!(isize, $ty);
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DefId(pub usize);
derive_id_into_from!(DefId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(pub usize);
derive_id_into_from!(ModuleId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExprId(pub usize);
derive_id_into_from!(ExprId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StmtId(pub usize);
derive_id_into_from!(StmtId);