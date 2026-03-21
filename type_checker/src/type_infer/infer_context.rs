use std::collections::HashMap;

use crate::{module::TypedModule, ty::{Ty, TyId}};

pub struct InferContext<'a, 'b> {
    pub module: &'a mut TypedModule<'b>,
    
    /// Infer(id) -> TyId
    pub substitutions: HashMap<usize, TyId>,
    
    next_infer_id: usize,
}

impl InferContext<'_, '_> {
    /// 分配欲推导类型
    pub fn alloc_infer_ty(&mut self) -> TyId {
        let id = self.next_infer_id;
        self.next_infer_id += 1; // 每次使用后加 1，确保下次不重复
        
        // 将其包装成 Ty::Infer 存入类型列表并返回它的 TyId
        self.module.tcx_mut().alloc(Ty::Infer(id))
    }

    /// 分配欲推导整数类型 (与普通 Infer 类型共用替换)
    pub fn alloc_infer_int(&mut self) -> TyId {
        let id = self.next_infer_id;
        self.next_infer_id += 1; // 每次使用后加 1，确保下次不重复
        
        // 将其包装成 Ty::InferInt 存入类型列表并返回它的 TyId
        self.module.tcx_mut().alloc(Ty::InferInt(id))
    }
}

impl<'b, 'a> InferContext<'a, 'b> {
    pub fn new(module: &'a mut TypedModule<'b>) -> Self {
        Self {
            module,
            substitutions: HashMap::new(),
            next_infer_id: 0
        }
    }
}