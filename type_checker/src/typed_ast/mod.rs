use crate::ty::TyId;

pub mod typed_expressions;
pub mod typed_expr;
pub mod typed_stmt;
pub mod typed_node;

pub trait GetType {
    fn get_type(&self) -> TyId;
}

pub trait SetType {
    fn set_type(&mut self, new_ty: TyId);
}