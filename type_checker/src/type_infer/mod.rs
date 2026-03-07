pub mod constraint;
pub mod infer_context;

use std::cmp::min;
use std::collections::HashMap;

use ast::node::GetToken;
use ast::{ExprId, StmtId};
use token::token::Token;

use crate::CheckResult;
use crate::constants::BOOL_INFIX_OPERATORS;
use crate::error::{TypeCheckerError, TypeCheckerErrorKind};
use crate::module::TypedModule;
use crate::ty::{Ty, TyId};
use crate::ty_context::TypeContext;
use crate::type_infer::constraint::Constraint;
use crate::type_infer::infer_context::InferContext;
use crate::typed_ast::typed_expr::TypedExpression;
use crate::typed_ast::typed_stmt::TypedStatement;
use crate::typed_ast::{GetType, SetType};

pub struct TypeInfer<'a, 'b, 'c> {
    pub infer_ctx: &'a mut InferContext<'b, 'c>,

    current_expected_ret_ty: Option<TyId>,
}

impl<'c, 'b, 'a> TypeInfer<'a, 'b, 'c> {
    pub fn new(infer_ctx: &'a mut InferContext<'b, 'c>) -> Self {
        Self {
            infer_ctx,
            current_expected_ret_ty: None,
        }
    }

    #[inline(always)]
    fn tcx(&mut self) -> &mut TypeContext {
        self.infer_ctx.module.tcx_mut()
    }

    #[inline(always)]
    fn tcx_ref(&self) -> &TypeContext {
        self.infer_ctx.module.tcx_ref()
    }

    /// 除非你打算长期持有 Module 否则不推荐使用这个函数
    /// 使用伴随整个 TypeInfer 的生命周期 'a 总是有代价的
    #[inline(always)]
    fn module(&'c mut self) -> &'a mut TypedModule<'b> {
        self.infer_ctx.module
    }

    #[inline(always)]
    fn module_ref(&self) -> &TypedModule<'_> {
        self.infer_ctx.module
    }

    pub fn infer(&mut self) -> CheckResult<()> {
        let stmt_len = self.module_ref().typed_stmts.len();

        for i in 0..stmt_len {
            self.infer_stmt(StmtId(i))?;
        }

        self.finalize();

        Ok(())
    }

    fn infer_stmt(&mut self, stmt_id: StmtId) -> CheckResult<()> {
        let stmt = self.module_ref().typed_stmts[stmt_id.0].clone();

        let ty = match stmt {
            TypedStatement::ExpressionStatement(_, id, _) => Some(self.infer_expr(id)?),

            TypedStatement::Let {
                value: id,
                var_type,
                ..
            } => {
                let expr = self.module_ref().get_expr(id).unwrap().clone();
                let expr_ty = self.infer_expr(id)?;

                let ty = if let Some(ref ty_ident) = var_type {
                    match self.tcx().table.lock().unwrap().get(&ty_ident.value) {
                        Some(it) => it.ty.get_type(),
                        None => {
                            return Err(Self::make_err(
                                None,
                                TypeCheckerErrorKind::TypeNotFound,
                                ty_ident.token.clone(),
                            ));
                        }
                    }
                } else {
                    expr_ty
                };

                self.unify(ty, expr_ty, expr.token())?;

                Some(ty)
            }

            TypedStatement::Extern { params, ret_ty, .. } => {
                for param in params {
                    self.infer_type_expr(param)?;
                }

                self.infer_type_expr(ret_ty)?;

                None
            }

            TypedStatement::Return { expr: id, .. } => {
                let ty = self.infer_expr(id)?;

                if let Some(expected) = self.current_expected_ret_ty {
                    self.unify(
                        expected,
                        ty,
                        self.module_ref().get_expr(id).unwrap().token(),
                    )?;
                }

                Some(ty)
            }

            _ => None,
        };

        if let Some(it) = ty {
            self.infer_ctx.module.typed_stmts[stmt_id.0].set_type(it);
        }

        Ok(())
    }

    fn infer_type_expr(&mut self, expr_id: ExprId) -> CheckResult<TyId> {
        let expr = self.module_ref().get_expr(expr_id).cloned().unwrap();

        let ty = match expr {
            TypedExpression::TypeHint(_, _, ty) => ty,

            TypedExpression::Prefix { right, token, .. } => {
                let op = token.value.clone();

                let right_ty = self.infer_expr(right)?;
                let right_token = self.module_ref().get_expr(right).unwrap().token();

                if op.as_ref() == "!" {
                    let bool_ty = self.tcx().alloc(Ty::Bool);
                    self.unify(bool_ty, right_ty, right_token.clone())?;
                } else if op.as_ref() == "-" || op.as_ref() == "+" {
                    if !matches!(self.tcx_ref().get(right_ty), Ty::IntTy(_)) {
                        return Err(TypeCheckerError {
                            kind: TypeCheckerErrorKind::TypeMismatch,
                            token,
                            message: Some(
                                format!("expected `integer`, got {}", self.tcx_ref().get(right_ty))
                                    .into(),
                            ),
                        });
                    }
                } else if op.as_ref() == "*" {
                    return Ok(self.tcx().alloc(Ty::Ptr(right_ty)));
                }

                right_ty
            }

            TypedExpression::Ident(_, ty) => {
                if let Ty::Function {
                    params_type,
                    ret_type,
                    ..
                } = self.tcx_ref().get(ty)
                {
                    let mut generic_params = params_type
                        .iter()
                        .map(|it| self.tcx_ref().get(*it))
                        .filter(|it| matches!(it, Ty::Generic(..)))
                        .cloned()
                        .map(|it| {
                            if let Ty::Generic(name, _impl_traits) = it {
                                name.to_string()
                            } else {
                                unreachable!()
                            }
                        })
                        .collect::<Vec<_>>();

                    if let Ty::Generic(name, _impl_traits) = self.tcx_ref().get(*ret_type) {
                        generic_params.push(name.to_string());
                    }

                    let new_ty = self.instantiate(ty, generic_params.as_slice());
                    new_ty
                } else {
                    ty
                }
            }

            _ => panic!("not a type expr"),
        };

        self.infer_ctx.module.typed_exprs[expr_id.0].set_type(ty);

        return Ok(ty);
    }

    fn infer_expr(&mut self, expr_id: ExprId) -> CheckResult<TyId> {
        let expr = self.module_ref().get_expr(expr_id).cloned().unwrap();

        let ty = match expr {
            TypedExpression::Int { ty, .. } => ty,
            TypedExpression::StrLiteral { ty, .. } => ty,
            TypedExpression::Bool { ty, .. } => ty,
            TypedExpression::BigInt { ty, .. } => ty,
            TypedExpression::TypeHint(_, _, ty) => ty,

            TypedExpression::If {
                consequence,
                else_block,
                condition,
                ..
            } => {
                let bool_ty = self.tcx().alloc(Ty::Bool);
                let condition_ty = self.infer_expr(condition)?;
                let condition_token = self.module_ref().get_expr(condition).unwrap().token();

                self.unify(bool_ty, condition_ty, condition_token)?;

                let then_block_ty = self.infer_expr(consequence)?;

                if let Some(it) = else_block.and_then(|it| {
                    Some((
                        self.infer_expr(it),
                        self.module_ref().get_expr(it).unwrap().token(),
                    ))
                }) {
                    let else_block_ty = it.0?;
                    let token = it.1;
                    self.unify(then_block_ty, else_block_ty, token)?;
                }

                then_block_ty
            }

            TypedExpression::Infix {
                left: left_id,
                right: right_id,
                ty,
                op,
                ..
            } => {
                let right = self.module_ref().get_expr(right_id).unwrap().clone();

                let left_t = self.infer_expr(left_id)?;
                let right_t = self.infer_expr(right_id)?;

                self.unify(left_t, right_t, right.token())?;

                if !BOOL_INFIX_OPERATORS.contains(&op.as_ref()) {
                    left_t
                } else {
                    ty
                }
            }

            TypedExpression::Prefix {
                right,
                ty: result_ty,
                token,
                ..
            } => {
                let op = token.value.clone();

                let right_ty = self.infer_expr(right)?;
                let right_token = self.module_ref().get_expr(right).unwrap().token();

                if op.as_ref() == "!" {
                    let bool_ty = self.tcx().alloc(Ty::Bool);
                    self.unify(bool_ty, right_ty, right_token.clone())?;
                } else if op.as_ref() == "-" || op.as_ref() == "+" {
                    if !matches!(self.tcx_ref().get(right_ty), Ty::IntTy(_)) {
                        return Err(TypeCheckerError {
                            kind: TypeCheckerErrorKind::TypeMismatch,
                            token,
                            message: Some(
                                format!("expected `integer`, got {}", self.tcx_ref().get(right_ty))
                                    .into(),
                            ),
                        });
                    }
                } else if op.as_ref() == "*" {
                    let expected_ptr_ty = self.tcx().alloc(Ty::Ptr(result_ty));

                    self.unify(expected_ptr_ty, right_ty, right_token)?;
                }

                result_ty
            }

            TypedExpression::SizeOf(_, val, ty) => {
                self.infer_expr(val)?;

                ty
            }

            TypedExpression::BuildStruct(_, _, fields, ty) => {
                for (_, val) in fields {
                    self.infer_expr(val)?;
                }

                ty
            }

            TypedExpression::FieldAccess(_, obj, _, ty) => {
                self.infer_expr(obj)?;

                ty
            }

            TypedExpression::Block(_, stmts, ty) => {
                for stmt in stmts {
                    self.infer_stmt(stmt)?;
                }

                ty
            }

            TypedExpression::Assign {
                left: left_id,
                right: right_id,
                ty,
                ..
            } => {
                let right = self.module_ref().get_expr(right_id).unwrap().clone();

                let left_t = self.infer_expr(left_id)?;
                let right_t = self.infer_expr(right_id)?;

                self.unify(left_t, right_t, right.token())?;

                ty
            }

            TypedExpression::BoolAnd {
                left: left_id,
                right: right_id,
                ty,
                ..
            } => {
                let left = self.module_ref().get_expr(left_id).unwrap().clone();
                let right = self.module_ref().get_expr(right_id).unwrap().clone();

                let bool_ty = self.tcx().alloc(Ty::Bool);

                let left_t = self.infer_expr(left_id)?;
                let right_t = self.infer_expr(right_id)?;

                self.unify(bool_ty, left_t, left.token())?;
                self.unify(bool_ty, right_t, right.token())?;

                ty
            }

            TypedExpression::BoolOr {
                left: left_id,
                right: right_id,
                ty,
                ..
            } => {
                let left = self.module_ref().get_expr(left_id).unwrap().clone();
                let right = self.module_ref().get_expr(right_id).unwrap().clone();

                let bool_ty = self.tcx().alloc(Ty::Bool);

                let left_t = self.infer_expr(left_id)?;
                let right_t = self.infer_expr(right_id)?;

                self.unify(bool_ty, left_t, left.token())?;
                self.unify(bool_ty, right_t, right.token())?;

                ty
            }

            TypedExpression::Function { block, ty, .. } => {
                let expected_ret_ty = match self.tcx_ref().get(ty) {
                    Ty::Function { ret_type, .. } => *ret_type,
                    _ => unreachable!(),
                };

                // 保存旧的状态 (为了支持嵌套函数)
                let old_ret_ty = self.current_expected_ret_ty;
                self.current_expected_ret_ty = Some(expected_ret_ty);

                self.infer_expr(block)?;

                self.current_expected_ret_ty = old_ret_ty;

                ty
            }

            TypedExpression::Call { func, args, .. } => {
                let callee_ty = self.infer_expr(func)?;

                let arg_types = args
                    .iter()
                    .map(|it| self.infer_expr(*it))
                    .collect::<Result<Vec<_>, _>>()?;

                let new_ret_ty = self.infer_ctx.alloc_infer_ty();

                let new_func_ty = self.tcx().alloc(Ty::Function {
                    params_type: arg_types,
                    ret_type: new_ret_ty,
                    is_variadic: false,
                });

                self.unify(
                    callee_ty,
                    new_func_ty,
                    self.module_ref().get_expr(func).unwrap().token(),
                )?;

                new_ret_ty
            }

            TypedExpression::Ident(_, ty) => {
                if let Ty::Function {
                    params_type,
                    ret_type,
                    ..
                } = self.tcx_ref().get(ty)
                {
                    let mut generic_params = params_type
                        .iter()
                        .map(|it| self.tcx_ref().get(*it))
                        .filter(|it| matches!(it, Ty::Generic(..)))
                        .cloned()
                        .map(|it| {
                            if let Ty::Generic(name, _impl_traits) = it {
                                name.to_string()
                            } else {
                                unreachable!()
                            }
                        })
                        .collect::<Vec<_>>();

                    if let Ty::Generic(name, _impl_traits) = self.tcx_ref().get(*ret_type) {
                        generic_params.push(name.to_string());
                    }

                    let new_ty = self.instantiate(ty, generic_params.as_slice());
                    new_ty
                } else {
                    ty
                }
            }
        };

        self.infer_ctx.module.typed_exprs[expr_id.0].set_type(ty);

        return Ok(ty);
    }

    pub fn unify_all(&mut self, constraints: Vec<Constraint>) -> CheckResult<()> {
        for c in constraints {
            self.unify(c.expected, c.got, c.token)?;
        }

        self.finalize();

        Ok(())
    }

    /// 核心：统一两个类型。如果失败，利用 Token 抛出 TypeChecker 错误
    pub fn unify(&mut self, t1: TyId, t2: TyId, token: Token) -> CheckResult<()> {
        let t1 = self.follow(t1);
        let t2 = self.follow(t2);

        if t1 == t2 {
            return Ok(());
        }

        let ty1 = self.tcx().get(t1).clone();
        let ty2 = self.tcx().get(t2).clone();

        match (ty1, ty2) {
            // 如果其中一个是推导变量，记录替换关系
            (Ty::Infer(id), _) => {
                self.infer_ctx.substitutions.insert(id, t2);
                Ok(())
            }
            (_, Ty::Infer(id)) => {
                self.infer_ctx.substitutions.insert(id, t1);
                Ok(())
            }

            (Ty::Ptr(id1), Ty::Ptr(id2)) => self.unify(id1, id2, token),

            // 泛型结构体的递归统一
            (Ty::AppliedGeneric(name1, args1), Ty::AppliedGeneric(name2, args2)) => {
                if name1 != name2 || args1.len() != args2.len() {
                    return Err(self.make_mismatch_error(t1, t2, token));
                }
                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    self.unify(*a1, *a2, token.clone())?;
                }
                Ok(())
            }

            // 函数类型的统一
            (
                Ty::Function {
                    params_type: p1,
                    ret_type: r1,
                    is_variadic: v1,
                    ..
                },
                Ty::Function {
                    params_type: p2,
                    ret_type: r2,
                    is_variadic: v2,
                    ..
                },
            ) => {
                if v1 || v2 {
                    let min_len = min(p1.len(), p2.len());
                    for i in 0..min_len {
                        self.unify(p1[i], p2[i], token.clone())?;
                    }

                    return self.unify(r1, r2, token);
                }

                if p1.len() != p2.len() {
                    return Err(self.make_mismatch_error(t1, t2, token));
                }

                for (a, b) in p1.iter().zip(p2.iter()) {
                    self.unify(*a, *b, token.clone())?;
                }

                self.unify(r1, r2, token)
            }

            // 如果确实不匹配，抛出错误
            (ty1, ty2) => {
                if ty1 != ty2 {
                    return Err(self.make_mismatch_error(t1, t2, token));
                }
                Ok(())
            }
        }
    }

    // 辅助方法：生成符合你定义的错误结构
    fn make_mismatch_error(&mut self, t1: TyId, t2: TyId, token: Token) -> TypeCheckerError {
        TypeCheckerError {
            kind: TypeCheckerErrorKind::TypeMismatch,
            token,
            message: Some(
                format!(
                    "expected `{}`, got {}",
                    self.tcx_ref().get(t1),
                    self.tcx_ref().get(t2)
                )
                .into(),
            ),
        }
    }

    pub fn make_err(
        message: Option<&str>,
        kind: TypeCheckerErrorKind,
        token: Token,
    ) -> TypeCheckerError {
        TypeCheckerError {
            kind,
            token,
            message: message.map_or_else(|| None, |it| Some(it.into())),
        }
    }

    /// 沿着替换链找到最终的真实类型
    pub fn follow(&self, mut id: TyId) -> TyId {
        while let Ty::Infer(infer_id) = &self.tcx_ref().get(id) {
            if let Some(target) = self.infer_ctx.substitutions.get(infer_id) {
                id = *target;
            } else {
                break;
            }
        }
        id
    }

    /// 核心：把一个可能是 Infer 的 TyId 彻底转正
    pub fn resolve_real_ty(&self, id: TyId) -> TyId {
        let real_id = self.follow(id);
        // 如果 real_id 指向的依然是 Ty::Infer，说明这个变量到最后也没推导出来（报错点）
        real_id
    }

    fn get_expr_tyid(&self, exprid: ExprId) -> TyId {
        self.infer_ctx.module.get_expr(exprid).unwrap().get_type()
    }

    fn get_stmt_tyid(&self, stmtid: StmtId) -> TyId {
        self.infer_ctx.module.get_stmt(stmtid).unwrap().get_type()
    }

    /// 替换泛型到推导类型
    fn substitute(&mut self, ty_id: TyId, mapping: &HashMap<String, TyId>) -> TyId {
        let ty = self.tcx_ref().get(ty_id).clone();

        match ty {
            Ty::Generic(name, _) => mapping.get(name.as_ref()).copied().unwrap_or(ty_id),

            Ty::Ptr(inner) => {
                let new_inner = self.substitute(inner, mapping);
                self.tcx().alloc(Ty::Ptr(new_inner))
            }

            Ty::Function {
                params_type,
                ret_type,
                is_variadic,
            } => {
                let new_params = params_type
                    .iter()
                    .map(|p| self.substitute(*p, mapping))
                    .collect();
                let new_ret = self.substitute(ret_type, mapping);

                // 重新打包成一个新的函数 TyId 返回
                self.tcx().alloc(Ty::Function {
                    params_type: new_params,
                    ret_type: new_ret,
                    is_variadic,
                })
            }

            _ => ty_id,
        }
    }

    fn instantiate(&mut self, ty_id: TyId, generic_params: &[String]) -> TyId {
        if generic_params.is_empty() {
            return ty_id;
        }

        let ty = self.tcx_ref().get(ty_id).clone();

        let mut mapping = HashMap::new();

        for param in generic_params {
            mapping.insert(param.clone(), self.infer_ctx.alloc_infer_ty());
        }

        match ty {
            _ => self.substitute(ty_id, &mapping),
        }
    }
}

impl<'c, 'b, 'a> TypeInfer<'a, 'b, 'c> {
    /// 将最终结果注入 TypeContext，彻底抹除占位符
    pub fn finalize(&mut self) {
        // 替换推导类型
        for expr_idx in 0..self.infer_ctx.module.typed_exprs.len() {
            let ty = self.infer_ctx.module.typed_exprs[expr_idx].get_type();
            let real_ty = self.follow(ty);

            let expr = &mut self.infer_ctx.module.typed_exprs[expr_idx];
            expr.set_type(real_ty);
        }

        for stmt_idx in 0..self.infer_ctx.module.typed_stmts.len() {
            let ty = self.infer_ctx.module.typed_stmts[stmt_idx].get_type();
            let real_ty = self.follow(ty);

            let stmt = &mut self.infer_ctx.module.typed_stmts[stmt_idx];
            stmt.set_type(real_ty);
        }
    }
}
