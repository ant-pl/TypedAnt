use std::collections::HashMap;

use crate::{ty::{Ty, TyId}, ty_context::TypeContext};

pub struct InferContext<'tcx> {
    pub tcx: &'tcx mut TypeContext,
    
    /// Infer(id) -> TyId
    pub substitutions: HashMap<usize, TyId>,
    
    next_infer_id: usize,
}

impl InferContext<'_> {
    /// 分配欲推导类型
    pub fn alloc_infer_ty(&mut self) -> TyId {
        let id = self.next_infer_id;
        self.next_infer_id += 1; // 每次使用后加 1，确保下次不重复
        
        // 将其包装成 Ty::Infer 存入类型列表并返回它的 TyId
        self.tcx.alloc(Ty::Infer(id))
    }
}

impl<'tcx> InferContext<'tcx> {
    pub fn new(tcx: &'tcx mut TypeContext) -> Self {
        Self {
            tcx,
            substitutions: HashMap::new(),
            next_infer_id: 0
        }
    }
}