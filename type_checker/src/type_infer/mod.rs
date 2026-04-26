pub mod constraint;
pub mod infer_context;

use std::cmp::min;
use std::collections::HashMap;
use std::sync::Arc;

use ant_crate_def::NodeOrTyped;
use ast::node::GetToken;
use id::{ExprId, ModuleId, StmtId};
use name_resolver::NameResolver;
use token::token::Token;
use ty::{IntTy, Ty, TyId};
use typed_ast::typed_expr::TypedExpression;
use typed_ast::typed_node::TypedNode;
use typed_ast::typed_stmt::TypedStatement;
use typed_ast::{GetType, SetType};
use typed_module::{display_ty, module::TypedModule, ty_context::TypeContext};

use crate::CheckResult;
use crate::constants::BOOL_INFIX_OPERATORS;
use crate::error::{TypeCheckerError, TypeCheckerErrorKind};
use crate::type_infer::constraint::Constraint;
use crate::type_infer::infer_context::InferContext;

pub struct TypeInfer<'a, 'b, 'c> {
    pub infer_ctx: &'a mut InferContext<'b, 'c>,
    pub name_resolver: NameResolver<'b>,

    locals_tyid: HashMap<Arc<str>, TyId>,

    current_expected_ret_ty: Option<TyId>,

    current_mod_id: ModuleId,
}

impl<'c, 'b, 'a> TypeInfer<'a, 'b, 'c> {
    pub fn new(
        infer_ctx: &'a mut InferContext<'b, 'c>,
        name_resolver: NameResolver<'b>,
    ) -> Self {
        Self {
            infer_ctx,
            current_mod_id: name_resolver.krate.root_id,
            name_resolver,
            current_expected_ret_ty: None,
            locals_tyid: HashMap::new(),
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
    #[allow(unused)]
    fn module(&'c mut self) -> &'a mut TypedModule<'b> {
        self.infer_ctx.module
    }

    #[inline(always)]
    fn module_ref(&self) -> &TypedModule<'_> {
        self.infer_ctx.module
    }

    pub fn infer(&mut self) -> CheckResult<()> {
        let mod_count = self.name_resolver.krate.modules.len();

        for i in 0..mod_count {
            let mod_id = ModuleId(i);

            self.current_mod_id = mod_id;

            let module_node = &self.name_resolver.krate.modules[i];

            if let Some(NodeOrTyped::Typed(typed_node)) = &module_node.ast {
                let TypedNode::Program { statements, .. } = typed_node.clone();

                for stmt_id in statements {
                    self.infer_stmt(stmt_id)?;
                }
            }
        }

        self.finalize();

        Ok(())
    }

    fn infer_stmt(&mut self, stmt_id: StmtId) -> CheckResult<Option<usize>> {
        let stmt = self.module_ref().typed_stmts[stmt_id.0].clone();

        let ty = match stmt {
            TypedStatement::ExpressionStatement(_, id, _) => Some(self.infer_expr(id)?),

            TypedStatement::Const {
                value: id,
                var_type,
                name,
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

                let followed = self.follow_all(ty);

                self.locals_tyid.insert(name.value.clone(), followed);

                Some(ty)
            }

            TypedStatement::Let {
                value: id,
                var_type,
                name,
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

                let followed = self.follow_all(ty);

                self.locals_tyid.insert(name.value.clone(), followed);

                Some(ty)
            }

            TypedStatement::Extern { params, ret_ty, .. } => {
                for param in params {
                    self.infer_type_expr(param)?;
                }

                if let Some(ret_ty) = ret_ty {
                    self.infer_type_expr(ret_ty)?;
                }

                None
            }

            TypedStatement::Return {
                expr: id, token, ..
            } => {
                let ty = if let Some(it) = id {
                    self.infer_expr(it)?
                } else {
                    self.tcx().alloc(Ty::Unit)
                };

                if let Some(expected) = self.current_expected_ret_ty {
                    self.unify(
                        expected,
                        ty,
                        if let Some(it) = id {
                            self.module_ref().get_expr(it).unwrap().token()
                        } else {
                            token
                        },
                    )?;
                }

                Some(ty)
            }

            TypedStatement::Struct { name, ty, .. } => {
                self.tcx().table.lock().unwrap().define_var(&name.value, ty);
                None
            }

            TypedStatement::FuncDecl { name, ty, .. } => {
                self.tcx().table.lock().unwrap().define_var(&name.value, ty);
                None
            }

            TypedStatement::Block { statements, .. } => {
                statements
                    .iter()
                    .map(|id| self.infer_stmt(*id))
                    .collect::<CheckResult<Vec<_>>>()?;
                None
            }

            _ => None,
        };

        if let Some(it) = ty {
            self.infer_ctx.module.typed_stmts[stmt_id.0].set_type(it);
        }

        Ok(ty)
    }

    fn infer_type_expr(&mut self, expr_id: ExprId) -> CheckResult<TyId> {
        let expr = self.module_ref().get_expr(expr_id).cloned().unwrap();

        let ty = match expr {
            TypedExpression::TypeHint(_, expr, _) => self.infer_type_expr(expr)?,

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
                                format!(
                                    "expected `integer`, got {}",
                                    display_ty(self.tcx_ref().get(right_ty), self.tcx_ref())
                                )
                                .into(),
                            ),
                        });
                    }
                } else if op.as_ref() == "*" {
                    return Ok(self.tcx().alloc(Ty::Ptr(right_ty)));
                }

                right_ty
            }

            TypedExpression::Ident(name, ty, _) => {
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
                } else if let Some(&current_ty) = self.locals_tyid.get(&name.value) {
                    current_ty
                } else {
                    ty
                }
            }

            TypedExpression::TypePath {
                left: name,
                paths: path_ids,
                ty,
                ..
            } => {
                let path_types = path_ids
                    .iter()
                    .map(|it| self.infer_type_expr(*it))
                    .collect::<Result<Vec<_>, _>>()?;

                let new_ty = self
                    .tcx()
                    .alloc(Ty::AppliedGeneric(name.value.clone(), path_types));

                self.unify(new_ty, ty, name.token.clone())?;

                new_ty
            }

            _ => {
                return Err(Self::make_err(
                    Some("not a type expr"),
                    TypeCheckerErrorKind::Other,
                    self.module_ref().get_expr(expr_id).unwrap().token(),
                ));
            }
        };

        self.infer_ctx.module.typed_exprs[expr_id.0].set_type(ty);

        return Ok(ty);
    }

    fn infer_expr(&mut self, expr_id: ExprId) -> CheckResult<TyId> {
        let expr = self.module_ref().get_expr(expr_id).cloned().unwrap();

        let ty = match expr {
            TypedExpression::Float { ty, .. } => ty,
            TypedExpression::Int { ty, .. } => ty,
            TypedExpression::StrLiteral { ty, .. } => ty,
            TypedExpression::Bool { ty, .. } => ty,
            TypedExpression::UnknownTypeInt { .. } => self.infer_ctx.alloc_infer_int(),
            TypedExpression::TypeHint(_, expr, _) => self.infer_expr(expr)?,

            TypedExpression::Cast {
                val, cast_to, ty, ..
            } => {
                let val_ty = self.infer_expr(val)?;
                let val_ty = self.follow_all(val_ty);

                let new_ty = self.infer_type_expr(cast_to)?;
                let new_ty = self.follow_all(new_ty);

                self.unify(
                    ty,
                    new_ty,
                    self.module_ref().get_expr(cast_to).unwrap().token(),
                )?;

                let target_ty = self.tcx_ref().get(new_ty);
                let value_ty = self.tcx_ref().get(val_ty);
                if matches!(value_ty, Ty::InferInt(_))
                    && matches!(target_ty, Ty::IntTy(_) | Ty::Ptr(_))
                {
                    // 这会让 InferInt 真正变成目标类型
                    self.unify(
                        val_ty,
                        new_ty,
                        self.module_ref().get_expr(val).unwrap().token(),
                    )?;
                }

                let val_ty = self.follow_all(val_ty);
                let new_ty = self.follow_all(new_ty);

                self.check_cast_valid(
                    val_ty,
                    new_ty,
                    self.module_ref().get_expr(val).unwrap().token(),
                )?;

                new_ty
            }

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

                let ltyid = self.follow_all(left_t);
                let rtyid = self.follow_all(right_t);

                let lty = self.tcx_ref().get(ltyid);
                let rty = self.tcx_ref().get(rtyid);

                match (lty, rty, op.as_ref()) {
                    (Ty::Ptr(_), Ty::IntTy(IntTy::USize), "+") => ltyid, // 指针加法，结果是左边的指针类型
                    (Ty::IntTy(IntTy::USize), Ty::Ptr(_), "+") => rtyid, // 整数+指针，结果是右边的指针类型
                    (Ty::Ptr(_), Ty::IntTy(IntTy::USize), "-") => ltyid, // 指针减法

                    _ => {
                        self.unify(left_t, right_t, right.token())?;

                        if !BOOL_INFIX_OPERATORS.contains(&op.as_ref()) {
                            left_t
                        } else {
                            ty
                        }
                    }
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

            TypedExpression::BuildStruct(_, ident, fields, ty) => {
                // 拿到原始定义
                let (def_generics, def_fields) = match self.tcx_ref().get(ty).clone() {
                    Ty::Struct {
                        generics, fields, ..
                    } => (generics, fields),

                    Ty::AppliedGeneric(name, _) => {
                        let ty = self.resolve_type_by_name(&name, &ident.token)?;

                        match self.tcx_ref().get(ty).clone() {
                            Ty::Struct {
                                generics, fields, ..
                            } => (generics, fields),
                            it => {
                                return Err(Self::make_err(
                                    Some(&format!(
                                        "expected struct, got {}",
                                        display_ty(&it, self.tcx_ref())
                                    )),
                                    TypeCheckerErrorKind::TypeMismatch,
                                    ident.token,
                                ));
                            }
                        }
                    }

                    it => Err(Self::make_err(
                        Some(&format!(
                            "expected struct, got {}",
                            display_ty(&it, self.tcx_ref())
                        )),
                        TypeCheckerErrorKind::TypeMismatch,
                        ident.token,
                    ))?,
                };

                let mut mapping = HashMap::new();

                // 在 TypeInfer 里分配 Infer 变量
                let mut arg_infer_ids = vec![];
                for gen_name in &def_generics {
                    let infer_id = self.infer_ctx.alloc_infer_ty();

                    mapping.insert(gen_name.to_string(), infer_id);

                    arg_infer_ids.push(infer_id);
                }

                // 构造这一次实例化的真正类型 (比如 Vec<?0>)
                let instantiated_struct_ty = self.tcx().alloc(Ty::AppliedGeneric(
                    ident.value.clone().into(),
                    arg_infer_ids,
                ));

                // 递归推导并统一字段类型
                for (field_ident, val_id) in fields {
                    let val_ty = self.infer_expr(val_id)?; // 实际传入的类型

                    // 字段声明时的类型
                    let field_decl_ty = def_fields.get(&field_ident.value).unwrap();

                    // 替换 (T -> mapping type)
                    let expected_field_ty = self.substitute(*field_decl_ty, &mapping);

                    // 统一
                    self.unify(expected_field_ty, val_ty, field_ident.token.clone())?;
                }

                instantiated_struct_ty
            }

            TypedExpression::FieldAccess(_, obj, field_ident, ty) => {
                let new_obj_ty = self.infer_expr(obj)?;
                let obj_ty_followed = self.follow_all(new_obj_ty);

                match self.tcx_ref().get(obj_ty_followed).clone() {
                    // 访问的是泛型实例
                    Ty::AppliedGeneric(struct_name, args) => {
                        // 找回原始 Struct 定义
                        let base_id = self.resolve_type_by_name(
                            &struct_name,
                            &self.module_ref().get_expr(obj).unwrap().token(),
                        )?;

                        let (generics_defs, fields_defs) = match self.tcx_ref().get(base_id) {
                            Ty::Struct {
                                generics, fields, ..
                            } => (generics, fields),
                            _ => unreachable!(),
                        };

                        // 建立映射 T -> real_ty
                        let mut mapping = HashMap::new();
                        for (i, name) in generics_defs.iter().enumerate() {
                            mapping.insert(name.to_string(), args[i]);
                        }

                        // 拿到字段, 并替换
                        let field_ty = fields_defs.get(&field_ident.value).unwrap();
                        self.substitute(*field_ty, &mapping)
                    }

                    // 普通结构体
                    Ty::Struct { .. } => {
                        // 使用原来的类型
                        ty
                    }

                    _ => Err(Self::make_err(
                        Some(&format!("not a struct")),
                        TypeCheckerErrorKind::TypeMismatch,
                        self.module_ref().get_expr(obj).unwrap().token(),
                    ))?,
                }
            }

            TypedExpression::Block(_, stmts, ty) => {
                let mut stmt_types = vec![];
                for stmt in &stmts {
                    stmt_types.push(self.infer_stmt(*stmt)?);
                }

                let new_ty = stmt_types.last().map_or(ty, |s| s.map_or(ty, |it| it));

                new_ty
            }

            TypedExpression::Assign {
                left: left_id,
                right: right_id,
                ty,
                ..
            } => {
                let right = self.module_ref().get_expr(right_id).unwrap().clone();

                let left_t = self.infer_expr(left_id)?;
                let left_t = self.follow_all(left_t);

                let right_t = self.infer_expr(right_id)?;
                let right_t = self.follow_all(right_t);

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

                let block_ty = self.infer_expr(block)?;
                self.unify(
                    expected_ret_ty,
                    block_ty,
                    self.module_ref().get_expr(block).unwrap().token(),
                )?;

                self.current_expected_ret_ty = old_ret_ty;

                ty
            }

            TypedExpression::Call { func, args, .. } => {
                let mut callee_ty = self.infer_expr(func)?;

                let Ty::Function { generics, .. } = self.tcx_ref().get(callee_ty) else {
                    return Err(self.unexpected_error(
                        "function",
                        &display_ty(self.tcx_ref().get(callee_ty), self.tcx_ref()),
                        self.module_ref().get_expr(func).unwrap().token(),
                    ));
                };

                let generics = generics.clone();

                if !generics.is_empty() {
                    callee_ty = self.instantiate(
                        callee_ty,
                        &generics
                            .iter()
                            .map(|it| it.to_string())
                            .collect::<Vec<String>>(),
                    )
                }

                let arg_types = args
                    .iter()
                    .map(|it| self.infer_expr(*it))
                    .collect::<Result<Vec<_>, _>>()?;

                let new_ret_ty = self.infer_ctx.alloc_infer_ty();

                let new_func_ty = self.tcx().alloc(Ty::Function {
                    generics,
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

            TypedExpression::Ident(name, ty, _) => {
                if let Some(&current_ty) = self.locals_tyid.get(&name.value) {
                    current_ty
                } else {
                    ty
                }
            }

            TypedExpression::GenericInstance { left, paths, .. } => {
                let base_ty_id = self.infer_expr(left)?;
                let base_ty_id = self.follow_all(base_ty_id);
                let base_ty = self.tcx_ref().get(base_ty_id).clone();

                let Ty::Function { generics, .. } = &base_ty else {
                    return Err(self.unexpected_error(
                        "function",
                        &display_ty(&base_ty, self.tcx_ref()),
                        self.module_ref().get_expr(left).unwrap().token(),
                    ));
                };

                let path_types = paths
                    .iter()
                    .map(|&g| self.infer_type_expr(g))
                    .collect::<Result<Vec<_>, _>>()?;

                let mut mapping = HashMap::new();
                for (name, concrete_id) in generics.iter().zip(path_types) {
                    mapping.insert(name.to_string(), concrete_id);
                }

                let specialized_ty = self.substitute(base_ty_id, &mapping);

                specialized_ty
            }

            _ => todo!("todo expr"),
        };

        self.infer_ctx.module.typed_exprs[expr_id.0].set_type(ty);

        return Ok(ty);
    }

    fn resolve_type_by_name(&self, name: &str, token: &Token) -> CheckResult<TyId> {
        //  先查局部符号表
        if let Some(symbol) = self.tcx_ref().table.lock().unwrap().get(name) {
            return Ok(symbol.ty.get_type());
        }

        // 使用 NameResolver 查找
        if let Some(def_id) = self.name_resolver.lookup_name(self.current_mod_id, name) {
            let def = self.name_resolver.krate.get_def(def_id);
            if let Some(ty) = def.ty() {
                return Ok(ty);
            }
        }

        Err(TypeCheckerError {
            kind: TypeCheckerErrorKind::TypeNotFound,
            token: token.clone(),
            message: Some(format!("type `{}` not found during inference", name).into()),
        })
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
        let t1 = self.follow_all(t1);
        let t2 = self.follow_all(t2);

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

            // 如果其中一个是整数推导变量，记录替换关系
            (Ty::InferInt(id), Ty::IntTy(_)) => {
                self.infer_ctx.substitutions.insert(id, t2);
                Ok(())
            }
            (Ty::IntTy(_), Ty::InferInt(id)) => {
                self.infer_ctx.substitutions.insert(id, t1);
                Ok(())
            }

            (Ty::InferInt(id), Ty::Ptr(_)) | (Ty::Ptr(_), Ty::InferInt(id)) => {
                let usize_ty = self.tcx().alloc(Ty::IntTy(IntTy::USize));
                self.infer_ctx.substitutions.insert(id, usize_ty);
                Ok(())
            }

            (Ty::InferInt(id1), Ty::InferInt(id2)) => {
                if id1 != id2 {
                    self.infer_ctx.substitutions.insert(id1, t2);
                }
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
                    display_ty(self.tcx_ref().get(t1), self.tcx_ref()),
                    display_ty(self.tcx_ref().get(t2), self.tcx_ref()),
                )
                .into(),
            ),
        }
    }

    fn unexpected_error(&mut self, expected: &str, got: &str, token: Token) -> TypeCheckerError {
        TypeCheckerError {
            kind: TypeCheckerErrorKind::TypeMismatch,
            token,
            message: Some(format!("expected `{expected}`, got {got}",).into()),
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

    pub fn follow_all(&mut self, id: TyId) -> TyId {
        let id = self.follow(id);
        self.follow_int(id)
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

    pub fn follow_int(&mut self, mut id: TyId) -> TyId {
        while let Ty::InferInt(infer_id) = &self.tcx_ref().get(id) {
            if let Some(target) = self.infer_ctx.substitutions.get(infer_id) {
                id = *target;
            } else {
                break;
            }
        }

        id
    }

    /// 核心：把一个可能是 Infer 的 TyId 彻底转正
    pub fn resolve_real_ty(&mut self, id: TyId) -> TyId {
        let real_id = self.follow_all(id);
        // 如果 real_id 指向的依然是 Ty::Infer，说明这个变量到最后也没推导出来（报错点）
        real_id
    }

    #[allow(unused)]
    fn get_expr_tyid(&self, exprid: ExprId) -> TyId {
        self.infer_ctx.module.get_expr(exprid).unwrap().get_type()
    }

    #[allow(unused)]
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

            Ty::AppliedGeneric(name, args) => {
                let new_args = args
                    .iter()
                    .map(|arg| self.substitute(*arg, mapping))
                    .collect();
                self.tcx().alloc(Ty::AppliedGeneric(name, new_args))
            }

            Ty::Function {
                generics,
                params_type,
                ret_type,
                is_variadic,
            } => {
                let new_params = params_type
                    .iter()
                    .map(|p| self.substitute(*p, mapping))
                    .collect();
                let new_ret = self.substitute(ret_type, mapping);

                let new_generics: Vec<_> = generics
                    .into_iter()
                    .filter(|g| !mapping.contains_key(g.as_ref()))
                    .collect();

                // 重新打包成一个新的函数 TyId 返回
                self.tcx().alloc(Ty::Function {
                    generics: new_generics,
                    params_type: new_params,
                    ret_type: new_ret,
                    is_variadic,
                })
            }

            Ty::InferInt(_) => self.tcx().alloc(Ty::IntTy(IntTy::I32)),

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
    /// 检查类型转换是否合法
    fn check_cast_valid(&self, from: TyId, to: TyId, token: Token) -> CheckResult<()> {
        let from_ty = self.tcx_ref().get(from);
        let to_ty = self.tcx_ref().get(to);

        // 相同类型, 允许
        if from == to {
            return Ok(());
        }

        // 数值类型之间允许转换（i32 as i64, u8 as i32 等）
        if Self::is_numeric(from_ty) && Self::is_numeric(to_ty) {
            return Ok(());
        }

        // 指针转换 (*T as *U, *T as usize 等)
        if Self::is_ptr(from_ty) {
            // *T as *U
            if Self::is_ptr(to_ty) {
                return Ok(());
            }
            // *T as usize/isize
            if Self::is_integer(to_ty, Some(IntTy::USize))
                || Self::is_integer(to_ty, Some(IntTy::ISize))
            {
                return Ok(());
            }
        }

        // usize/isize 转指针
        if (Self::is_integer(from_ty, Some(IntTy::USize))
            || Self::is_integer(from_ty, Some(IntTy::ISize)))
            && Self::is_ptr(to_ty)
        {
            return Ok(());
        }

        // bool -> int / int -> bool
        match (from_ty, to_ty) {
            (Ty::Bool, Ty::IntTy(_)) => return Ok(()),
            (Ty::IntTy(_), Ty::Bool) => return Ok(()),
            _ => (),
        }

        // 不合法的转换
        Err(Self::make_err(
            Some(&format!(
                "cannot cast `{}` as `{}`",
                display_ty(self.tcx_ref().get(from), self.tcx_ref()),
                display_ty(self.tcx_ref().get(to), self.tcx_ref()),
            )),
            TypeCheckerErrorKind::InvalidCast,
            token,
        ))
    }

    fn is_numeric(ty: &Ty) -> bool {
        matches!(ty, Ty::IntTy(_) | Ty::FloatTy(_))
    }

    fn is_ptr(ty: &Ty) -> bool {
        matches!(ty, Ty::Ptr(_))
    }

    fn is_integer(ty: &Ty, specific: Option<IntTy>) -> bool {
        match (ty, specific) {
            (Ty::IntTy(_), None) => true,
            (Ty::IntTy(it), Some(want)) => *it == want,
            _ => false,
        }
    }
}

impl<'c, 'b, 'a> TypeInfer<'a, 'b, 'c> {
    /// 将最终结果注入 TypeContext，彻底抹除占位符
    pub fn finalize(&mut self) {
        // 替换推导类型
        for expr_idx in 0..self.infer_ctx.module.typed_exprs.len() {
            let ty = self.infer_ctx.module.typed_exprs[expr_idx].get_type();
            let mut real_ty_id = self.deep_resolve(ty);

            let is_infer_int = matches!(self.tcx_ref().get(real_ty_id), Ty::InferInt(_));
            if is_infer_int {
                real_ty_id = self.follow_int(real_ty_id)
            }

            let expr = &mut self.infer_ctx.module.typed_exprs[expr_idx];
            expr.set_type(real_ty_id);
        }

        for stmt_idx in 0..self.infer_ctx.module.typed_stmts.len() {
            let ty = self.infer_ctx.module.typed_stmts[stmt_idx].get_type();
            let mut real_ty_id = self.deep_resolve(ty);

            let is_infer_int = matches!(self.tcx_ref().get(real_ty_id), Ty::InferInt(_));
            if is_infer_int {
                real_ty_id = self.follow_int(real_ty_id)
            }

            let stmt = &mut self.infer_ctx.module.typed_stmts[stmt_idx];
            stmt.set_type(real_ty_id);
        }

        for (local_name, local_ty) in self.locals_tyid.clone() {
            let mut real_ty_id = self.deep_resolve(local_ty);

            let is_infer_int = matches!(self.tcx_ref().get(real_ty_id), Ty::InferInt(_));
            if is_infer_int {
                real_ty_id = self.follow_int(real_ty_id)
            }

            self.locals_tyid.insert(local_name, real_ty_id);
        }

        let def_count = self.name_resolver.krate.definitions.len();
        for i in 0..def_count {
            let def_id = id::DefId(i);
            let old_ty = self.name_resolver.krate.get_def(def_id).ty();
            
            if let Some(old_ty) = old_ty {
                let real_ty = self.deep_resolve(old_ty);
                self.name_resolver.krate.get_mut_def(def_id).set_ty(real_ty);
            }
        }
    }

    fn deep_resolve(&mut self, id: TyId) -> TyId {
        let id = self.follow_all(id); // 先解开最外层
        let ty = self.tcx_ref().get(id).clone();

        match ty {
            Ty::Ptr(inner) => {
                let resolved = self.deep_resolve(inner);
                self.tcx().alloc(Ty::Ptr(resolved))
            }

            Ty::AppliedGeneric(name, args) => {
                let resolved_args = args.iter().map(|a| self.deep_resolve(*a)).collect();
                self.tcx().alloc(Ty::AppliedGeneric(name, resolved_args))
            }

            Ty::Function {
                generics,
                params_type,
                ret_type,
                is_variadic,
            } => {
                let ps = params_type.iter().map(|p| self.deep_resolve(*p)).collect();
                let rs = self.deep_resolve(ret_type);
                self.tcx().alloc(Ty::Function {
                    generics,
                    params_type: ps,
                    ret_type: rs,
                    is_variadic,
                })
            }

            Ty::InferInt(_) => self.tcx().alloc(Ty::IntTy(IntTy::I32)),

            _ => id,
        }
    }
}
