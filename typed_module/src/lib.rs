use ty::Ty;

use crate::ty_context::TypeContext;

pub mod module;
pub mod ty_context;
pub mod type_table;

pub fn display_ty(ty: &Ty, tcx: &TypeContext) -> String {
    match ty {
        Ty::Unknown => "unknown".to_owned(),
        Ty::BigInt => "BigInt".to_owned(),
        Ty::Str => "str".to_owned(),
        Ty::FloatTy(it) => it.to_string(),
        Ty::IntTy(it) => it.to_string(),
        Ty::Bool => "bool".to_owned(),
        Ty::Unit => "Unit".to_owned(),
        Ty::Ptr(it) => format!("*{}", display_ty(tcx.get(*it), tcx)),
        Ty::Infer(it) => format!("Infer({it})"),
        Ty::InferInt(it) => format!("InferInt({it})"),
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