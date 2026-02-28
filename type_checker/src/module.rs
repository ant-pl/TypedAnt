use ast::{ExprId, StmtId};

use crate::{
    ty_context::TypeContext,
    typed_ast::{typed_expr::TypedExpression, typed_stmt::TypedStatement},
};

#[derive(Debug, Clone)]
pub struct TypedModuleCloned {
    pub tcx: TypeContext,

    pub typed_exprs: Vec<TypedExpression>,
    pub typed_stmts: Vec<TypedStatement>,
}

#[derive(Debug)]
pub struct TypedModule<'a> {
    tcx: &'a mut TypeContext,

    pub typed_exprs: Vec<TypedExpression>,
    pub typed_stmts: Vec<TypedStatement>,
}

impl TypedModule<'_> {
    pub fn alloc_expr(&mut self, expr: TypedExpression) -> ExprId {
        let id = self.typed_exprs.len();

        self.typed_exprs.push(expr);

        id.into()
    }

    pub fn alloc_stmt(&mut self, stmt: TypedStatement) -> StmtId {
        let id = self.typed_stmts.len();

        self.typed_stmts.push(stmt);

        id.into()
    }
}

impl TypedModule<'_> {
    #[inline(always)]
    pub fn get_expr(&self, id: ExprId) -> Option<&TypedExpression> {
        self.typed_exprs.get(id.0)
    }

    #[inline(always)]
    pub fn get_mut_expr(&mut self, id: ExprId) -> Option<&mut TypedExpression> {
        self.typed_exprs.get_mut(id.0)
    }

    pub fn take_expr(&mut self, id: ExprId) -> Option<TypedExpression> {
        if id.0 >= self.typed_exprs.len() {
            return None;
        }

        Some(self.typed_exprs.remove(id.0))
    }

    #[inline(always)]
    pub fn get_stmt(&self, id: StmtId) -> Option<&TypedStatement> {
        self.typed_stmts.get(id.0)
    }

    #[inline(always)]
    pub fn get_mut_stmt(&mut self, id: StmtId) -> Option<&mut TypedStatement> {
        self.typed_stmts.get_mut(id.0)
    }

    pub fn take_stmt(&mut self, id: StmtId) -> Option<TypedStatement> {
        if id.0 >= self.typed_stmts.len() {
            return None;
        }

        Some(self.typed_stmts.remove(id.0))
    }
}

impl TypedModule<'_> {
    #[inline(always)]
    pub fn tcx_ref(&self) -> &TypeContext {
        self.tcx
    }

    #[inline(always)]
    pub fn tcx_mut(&mut self) -> &mut TypeContext {
        self.tcx
    }
}

impl<'a> TypedModule<'a> {
    pub fn new(tcx: &'a mut TypeContext) -> Self {
        Self {
            tcx,
            typed_exprs: vec![],
            typed_stmts: vec![],
        }
    }
}

impl TypedModule<'_> {
    pub fn cloned(&self) -> TypedModuleCloned {
        TypedModuleCloned {
            tcx: self.tcx.clone(),
            typed_exprs: self.typed_exprs.clone(),
            typed_stmts: self.typed_stmts.clone(),
        }
    }
}

impl TypedModuleCloned {
    pub fn alloc_expr(&mut self, expr: TypedExpression) -> ExprId {
        let id = self.typed_exprs.len();

        self.typed_exprs.push(expr);

        id.into()
    }

    pub fn alloc_stmt(&mut self, stmt: TypedStatement) -> StmtId {
        let id = self.typed_stmts.len();

        self.typed_stmts.push(stmt);

        id.into()
    }
}

impl TypedModuleCloned {
    #[inline(always)]
    pub fn get_expr(&self, id: ExprId) -> Option<&TypedExpression> {
        self.typed_exprs.get(id.0)
    }

    #[inline(always)]
    pub fn get_mut_expr(&mut self, id: ExprId) -> Option<&mut TypedExpression> {
        self.typed_exprs.get_mut(id.0)
    }

    pub fn take_expr(&mut self, id: ExprId) -> Option<TypedExpression> {
        if id.0 >= self.typed_exprs.len() {
            return None;
        }

        Some(self.typed_exprs.remove(id.0))
    }

    #[inline(always)]
    pub fn get_stmt(&self, id: StmtId) -> Option<&TypedStatement> {
        self.typed_stmts.get(id.0)
    }

    #[inline(always)]
    pub fn get_mut_stmt(&mut self, id: StmtId) -> Option<&mut TypedStatement> {
        self.typed_stmts.get_mut(id.0)
    }

    pub fn take_stmt(&mut self, id: StmtId) -> Option<TypedStatement> {
        if id.0 >= self.typed_stmts.len() {
            return None;
        }

        Some(self.typed_stmts.remove(id.0))
    }
}

impl TypedModuleCloned {
    #[inline(always)]
    pub fn tcx_ref(&self) -> &TypeContext {
        &self.tcx
    }

    #[inline(always)]
    pub fn tcx_mut(&mut self) -> &mut TypeContext {
        &mut self.tcx
    }
}