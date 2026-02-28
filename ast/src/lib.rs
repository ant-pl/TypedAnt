pub mod expressions;
pub mod expr;
pub mod stmt;
pub mod node;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExprId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StmtId(pub usize);

dervie_from_num!(u64, ExprId);
dervie_from_num!(u32, ExprId);
dervie_from_num!(u16, ExprId);
dervie_from_num!(u8, ExprId);
dervie_from_num!(usize, ExprId);
dervie_from_num!(i64, ExprId);
dervie_from_num!(i32, ExprId);
dervie_from_num!(i16, ExprId);
dervie_from_num!(i8, ExprId);
dervie_from_num!(isize, ExprId);

dervie_from_num!(u64, StmtId);
dervie_from_num!(u32, StmtId);
dervie_from_num!(u16, StmtId);
dervie_from_num!(u8, StmtId);
dervie_from_num!(usize, StmtId);
dervie_from_num!(i64, StmtId);
dervie_from_num!(i32, StmtId);
dervie_from_num!(i16, StmtId);
dervie_from_num!(i8, StmtId);
dervie_from_num!(isize, StmtId);

dervie_into_num!(u64, ExprId);
dervie_into_num!(u32, ExprId);
dervie_into_num!(u16, ExprId);
dervie_into_num!(u8, ExprId);
dervie_into_num!(usize, ExprId);
dervie_into_num!(i64, ExprId);
dervie_into_num!(i32, ExprId);
dervie_into_num!(i16, ExprId);
dervie_into_num!(i8, ExprId);
dervie_into_num!(isize, ExprId);

dervie_into_num!(u64, StmtId);
dervie_into_num!(u32, StmtId);
dervie_into_num!(u16, StmtId);
dervie_into_num!(u8, StmtId);
dervie_into_num!(usize, StmtId);
dervie_into_num!(i64, StmtId);
dervie_into_num!(i32, StmtId);
dervie_into_num!(i16, StmtId);
dervie_into_num!(i8, StmtId);
dervie_into_num!(isize, StmtId);