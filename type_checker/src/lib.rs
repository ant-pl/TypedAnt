pub mod constants;
pub mod error;
pub mod scope;
pub mod test;
pub mod type_infer;

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use ant_crate_def::{NodeOrTyped, definition::Def};
use ast::{
    expr::Expression,
    node::{GetToken, Node},
    stmt::Statement,
};

use id::{DefId, ExprId, ModuleId, StmtId};
use indexmap::IndexMap;
use name_resolver::NameResolver;
use token::token::Token;
use ty::{Ty, TyId};
use typed_ast::{
    GetType, typed_expr::TypedExpression, typed_expressions::ident::Ident, typed_node::TypedNode,
    typed_stmt::TypedStatement,
};
use typed_module::{
    display_ty, module::TypedModule, ty_context::TypeContext, type_table::TypeTable,
};

use crate::{
    constants::BOOL_INFIX_OPERATORS,
    error::{TypeCheckerError, TypeCheckerErrorKind},
    scope::{CheckScope, ScopeKind},
    type_infer::constraint::Constraint,
};

pub type CheckResult<T> = Result<T, TypeCheckerError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompileAs {
    AsValue,
    AsNone,
}

pub struct TypeChecker<'a, 'b> {
    name_resolver: &'a mut NameResolver<'b>,

    resolving_defs: HashSet<DefId>,

    module: &'a mut TypedModule<'b>,
    current_mod_id: ModuleId,

    constraints: Vec<Constraint>,

    scopes: Vec<CheckScope>,
    scope_index: usize,

    compile_as: CompileAs,

    builtins_table: Arc<Mutex<TypeTable>>,
}

impl<'a, 'b> TypeChecker<'a, 'b> {
    pub fn new(module: &'a mut TypedModule<'b>, name_resolver: &'a mut NameResolver<'b>) -> Self {
        let global_scope = CheckScope {
            kind: ScopeKind::Global,
            collect_return_types: vec![],
        };

        Self {
            builtins_table: Arc::new(Mutex::new(TypeTable::new().init(module.tcx_mut()))),

            resolving_defs: HashSet::new(),

            module,
            current_mod_id: name_resolver.krate.root_id,

            name_resolver,

            constraints: vec![],

            compile_as: CompileAs::AsNone,

            scope_index: 0,
            scopes: vec![global_scope],
        }
    }

    fn reset_to_builtins(&mut self) {
        // 以内置类型表为父级，创建一个新的空表
        self.module.tcx_mut().table = Arc::new(Mutex::new(TypeTable::with_outer(
            self.builtins_table.clone(),
        )));
    }

    pub fn check_all(&mut self, root_node: Node) -> CheckResult<TypedNode> {
        self.fill_defs_ty()?;

        self.check_all_modules()?;

        self.reset_to_builtins();

        let node = self.check_node(root_node)?;

        if let Some(it) = self
            .name_resolver
            .krate
            .modules
            .get_mut(self.name_resolver.krate.root_id.0)
        {
            it.ast = Some(NodeOrTyped::Typed(node.clone()))
        }

        Ok(node)
    }

    fn get_raw_stmt(&self, def_id: DefId) -> &Statement {
        let def = self.name_resolver.krate.get_def(def_id);
        let (mod_id, ast_idx) = (def.module_id(), def.ast_index());

        let mod_node = &self.name_resolver.krate.modules[mod_id.0];
        if let Some(NodeOrTyped::Node(Node::Program { statements, .. })) = &mod_node.ast {
            &statements[ast_idx.0]
        } else {
            panic!("module has not ast node or be typed")
        }
    }

    fn write_back_type(&mut self, def_id: DefId, ty: TyId) {
        self.name_resolver.krate.definitions[def_id.0].set_ty(ty)
    }

    pub fn fill_defs_ty(&mut self) -> CheckResult<()> {
        for def_id in 0..self.name_resolver.krate.definitions.len() {
            self.resolve_def_type(DefId(def_id))?;
        }

        Ok(())
    }

    pub fn check_all_modules(&mut self) -> CheckResult<()> {
        for i in 0..self.name_resolver.krate.modules.len() {
            if i == self.name_resolver.krate.root_id.0 {
                // 我们将在后面检查主模块
                continue;
            }

            let module = &self.name_resolver.krate.modules[i];
            let Some(NodeOrTyped::Node(node)) = module.ast.clone() else {
                continue;
            };

            let typed_node = self.check_module(node, ModuleId(i))?;

            let module = &mut self.name_resolver.krate.modules[i];
            module.ast = Some(NodeOrTyped::Typed(typed_node));
        }

        Ok(())
    }

    pub fn check_module(&mut self, node: Node, mod_id: ModuleId) -> CheckResult<TypedNode> {
        let old_mod = self.current_mod_id;
        self.current_mod_id = mod_id;

        // 重置符号表：每个模块都该有干净的开始
        self.reset_to_builtins();

        // 这一定是一个不同以往的浪漫古士
        let r = self.check_node(node);

        self.current_mod_id = old_mod;

        r
    }

    pub fn check_node(&mut self, node: Node) -> CheckResult<TypedNode> {
        match node {
            Node::Program { token, statements } => {
                let mut typed_statements = vec![];

                for stmt in statements {
                    let stmt = self.check_statement(stmt)?;
                    typed_statements.push(self.module.alloc_stmt(stmt));
                }

                let node = TypedNode::Program {
                    token,
                    statements: typed_statements,
                };

                Ok(node)
            }
        }
    }

    pub fn check_statements(
        &mut self,
        statements: Vec<Statement>,
        scope_kind: ScopeKind,
    ) -> CheckResult<(Vec<StmtId>, CheckScope)> {
        let mut stmt_ids = vec![];

        if statements.is_empty() {
            let unit_id = self.tcx().alloc(Ty::Unit);

            self.current_scope_mut()
                .collect_return_types
                .push((unit_id, Token::eof("unknown".into(), 0, 0)));
            return Ok((stmt_ids, self.leave_scope().0));
        }

        let statement_count = statements.len() - 1;

        for (i, stmt) in statements.into_iter().enumerate() {
            stmt_ids.push(
                if i == statement_count && scope_kind == ScopeKind::Function {
                    self.compile_as = CompileAs::AsValue;
                    let r = self.check_statement(stmt)?;
                    self.compile_as = CompileAs::AsNone;

                    self.current_scope_mut()
                        .collect_return_types
                        .push((r.get_type(), r.token()));

                    self.module.alloc_stmt(r)
                } else {
                    let typed_stmt = self.check_statement(stmt)?;
                    self.module.alloc_stmt(typed_stmt)
                },
            );
        }

        Ok((stmt_ids, self.leave_scope().0))
    }

    pub fn check_expr_as_val(&mut self, expr: Expression) -> CheckResult<TypedExpression> {
        let old = self.compile_as;
        self.compile_as = CompileAs::AsValue;

        let result = self.check_expr(expr);

        self.compile_as = old;

        result
    }

    pub fn check_expr_with_mod_id(
        &mut self,
        expr: Expression,
        id: ModuleId,
    ) -> CheckResult<TypedExpression> {
        let old = self.current_mod_id;
        self.current_mod_id = id;

        let result = self.check_expr(expr);

        self.current_mod_id = old;

        result
    }

    pub fn check_expr(&mut self, expr: Expression) -> CheckResult<TypedExpression> {
        match expr {
            Expression::UnknownTypeInt { token, value } => Ok(TypedExpression::UnknownTypeInt {
                token,
                value,
                ty: self.tcx().alloc(Ty::Unknown),
            }),
            Expression::Bool { token, value } => Ok(TypedExpression::Bool {
                token,
                value,
                ty: self.tcx().alloc(Ty::Bool),
            }),
            Expression::Int { token, value } => Ok(TypedExpression::Int {
                token,
                value,
                ty: self.tcx().alloc(Ty::IntTy(value.into())),
            }),
            Expression::Float { token, value } => Ok(TypedExpression::Float {
                token,
                value: value.clone(),
                ty: self.tcx().alloc(Ty::FloatTy(value.into())),
            }),
            Expression::StrLiteral { token, value } => Ok(TypedExpression::StrLiteral {
                token,
                value,
                ty: self.tcx().alloc(Ty::Str),
            }),

            Expression::SizeOf(token, expr) => Ok(TypedExpression::SizeOf(
                token,
                {
                    let expr = self.check_expr_as_val(*expr)?;
                    self.module.alloc_expr(expr)
                },
                self.tcx().alloc(Ty::IntTy(ty::IntTy::USize)),
            )),

            Expression::Cast {
                token,
                val,
                cast_to,
            } => {
                let typed_val = self.check_expr_as_val(*val)?;
                let cast_to = self.check_type_expr(*cast_to)?;

                let cast_to_ty = cast_to.get_type();

                let val_id = self.module.alloc_expr(typed_val);
                let cast_to_id = self.module.alloc_expr(cast_to);

                Ok(TypedExpression::Cast {
                    token,
                    val: val_id,
                    cast_to: cast_to_id,
                    ty: cast_to_ty,
                })
            }

            Expression::FieldAccess(token, struct_expr, field) => {
                let new_field = Ident {
                    value: field.value,
                    token: field.token,
                };

                let typed_struct_expr = self.check_expr_as_val(*struct_expr)?;

                let struct_ty = self.tcx().get(typed_struct_expr.get_type()).clone();

                match struct_ty {
                    // 普通结构体（无泛型）
                    Ty::Struct { fields, name, .. } => {
                        let Some(field_ty) = fields.get(&new_field.value) else {
                            Err(Self::make_err(
                                Some(&format!(
                                    "field `{}` of struct {name} not found",
                                    &new_field.value
                                )),
                                TypeCheckerErrorKind::VariableNotFound,
                                new_field.token.clone(),
                            ))?
                        };

                        let typed_struct_expr_id = self.module.alloc_expr(typed_struct_expr);
                        Ok(TypedExpression::FieldAccess(
                            token,
                            typed_struct_expr_id,
                            new_field,
                            *field_ty,
                        ))
                    }

                    // 泛型实例
                    Ty::AppliedGeneric(base, args) => {
                        // 找回原始 Struct 定义
                        let base_ty = self
                            .tcx()
                            .table
                            .lock()
                            .unwrap()
                            .get(&base)
                            .unwrap()
                            .ty
                            .get_type();

                        let Ty::Struct {
                            generics, fields, ..
                        } = self.tcx().get(base_ty).clone()
                        else {
                            unreachable!()
                        };

                        let mut generic_map = HashMap::new();

                        for (i, g) in generics.iter().enumerate() {
                            generic_map.insert(g.clone(), args[i]);
                        }

                        let Some(field_ty) = fields.get(&new_field.value) else {
                            Err(Self::make_err(
                                Some(&format!("field `{}` not found", &new_field.value)),
                                TypeCheckerErrorKind::VariableNotFound,
                                new_field.token.clone(),
                            ))?
                        };

                        let mut concrete = *field_ty;

                        // 如果字段是泛型参数：用 args 替换
                        if let Ty::Generic(name, _) = self.tcx().get(*field_ty) {
                            if let Some(real) = generic_map.get(name) {
                                concrete = *real;
                            }
                        }

                        let typed_struct_expr_id = self.module.alloc_expr(typed_struct_expr);
                        Ok(TypedExpression::FieldAccess(
                            token,
                            typed_struct_expr_id,
                            new_field,
                            concrete,
                        ))
                    }

                    _ => Err(Self::make_err(
                        Some(&format!("not a struct")),
                        TypeCheckerErrorKind::TypeMismatch,
                        typed_struct_expr.token(),
                    ))?,
                }
            }

            Expression::BuildStruct(token, name, fields) => {
                // 查 struct 类型
                let struct_ty_id = self.lookup_type_by_name(&name.value, name.token.clone())?;

                // 取出 struct 的泛型参数和字段定义
                let def_generics = match self.tcx().get(struct_ty_id).clone() {
                    Ty::Struct { generics, .. } => generics,
                    it => {
                        return Err(Self::make_err(
                            Some(&format!(
                                "expected struct, got {}",
                                display_ty(&it, self.tcx_ref())
                            )),
                            TypeCheckerErrorKind::TypeMismatch,
                            name.token.clone(),
                        ));
                    }
                };

                // typecheck 每个字段
                let mut typed_field_ids = IndexMap::new();

                for (k, v) in fields {
                    let typed_val = self.check_expr_as_val(v)?;
                    typed_field_ids.insert(
                        Ident {
                            token: k.token,
                            value: k.value,
                        },
                        self.module.alloc_expr(typed_val),
                    );
                }

                if def_generics.is_empty() {
                    return Ok(TypedExpression::BuildStruct(
                        token,
                        Ident {
                            token: name.token,
                            value: name.value,
                        },
                        typed_field_ids,
                        struct_ty_id,
                    ));
                }

                Ok(TypedExpression::BuildStruct(
                    token,
                    Ident {
                        token: name.token,
                        value: name.value,
                    },
                    typed_field_ids,
                    struct_ty_id,
                ))
            }

            Expression::Assign { token, left, right } => {
                let typed_left = self.check_expr_as_val(*left)?;
                let typed_right = self.check_expr_as_val(*right)?;

                Ok(TypedExpression::Assign {
                    token,
                    ty: self.tcx().alloc(Ty::Unit),
                    left: self.module.alloc_expr(typed_left),
                    right: self.module.alloc_expr(typed_right),
                })
            }

            Expression::Ident(it) => {
                let ident_name = &it.value;

                if let Some(symbol) = self.tcx_ref().table.lock().unwrap().get(&ident_name) {
                    return Ok(TypedExpression::Ident(
                        Ident {
                            token: it.token,
                            value: it.value.clone(),
                        },
                        symbol.ty.get_type(),
                        self.name_resolver
                            .lookup_name(self.current_mod_id, &it.value),
                    ));
                } else if let Some(def_id) = self
                    .name_resolver
                    .lookup_name(self.current_mod_id, &it.value)
                {
                    let ty_id = self.resolve_def_type(def_id)?;

                    return Ok(TypedExpression::Ident(
                        Ident {
                            token: it.token,
                            value: it.value,
                        },
                        ty_id,
                        Some(def_id),
                    ));
                } else {
                    Err(Self::make_err(
                        None,
                        TypeCheckerErrorKind::VariableNotFound,
                        it.token,
                    ))
                }
            }

            Expression::Infix {
                token,
                left,
                right,
                op,
                ..
            } => {
                let left_t = self.check_expr_as_val(*left)?;
                let right_t = self.check_expr_as_val(*right)?;

                let lty = left_t.get_type();

                Ok(TypedExpression::Infix {
                    token,
                    ty: if BOOL_INFIX_OPERATORS.contains(&op.as_ref()) {
                        self.tcx().alloc(Ty::Bool)
                    } else {
                        lty
                    },
                    left: self.module.alloc_expr(left_t),
                    right: self.module.alloc_expr(right_t),
                    op,
                })
            }

            Expression::Prefix { token, op, right } => {
                let right_t = self.check_expr_as_val(*right)?;

                Ok(TypedExpression::Prefix {
                    ty: if op.as_ref() == "!" {
                        self.tcx().alloc(Ty::Bool)
                    } else if op.as_ref() == "-" || op.as_ref() == "+" {
                        right_t.get_type()
                    } else if op.as_ref() == "&" {
                        self.tcx().alloc(Ty::Ptr(right_t.get_type()))
                    } else if op.as_ref() == "*" {
                        let Ty::Ptr(inner_tyid) = self.tcx().get(right_t.get_type()) else {
                            return Err(Self::make_err(
                                Some(&format!(
                                    "type `{}` cannot be dereferenced",
                                    display_ty(
                                        self.tcx_ref().get(right_t.get_type()),
                                        self.tcx_ref()
                                    )
                                )),
                                TypeCheckerErrorKind::TypeMismatch,
                                right_t.token(),
                            ));
                        };

                        *inner_tyid
                    } else {
                        return Err(Self::make_err(
                            Some(&format!("unknown operator `{}`", &token.value)),
                            TypeCheckerErrorKind::Other,
                            token,
                        ));
                    },
                    token,
                    right: self.module.alloc_expr(right_t),
                    op,
                })
            }

            Expression::BoolAnd {
                token, left, right, ..
            } => {
                let left_expr = self.check_expr_as_val(*left)?;
                let right_expr = self.check_expr_as_val(*right)?;

                Ok(TypedExpression::BoolAnd {
                    token,
                    left: self.module.alloc_expr(left_expr),
                    right: self.module.alloc_expr(right_expr),
                    ty: self.tcx().alloc(Ty::Bool),
                })
            }

            Expression::BoolOr {
                token, left, right, ..
            } => {
                let left_expr = self.check_expr_as_val(*left)?;
                let right_expr = self.check_expr_as_val(*right)?;

                Ok(TypedExpression::BoolOr {
                    token,
                    left: self.module.alloc_expr(left_expr),
                    right: self.module.alloc_expr(right_expr),
                    ty: self.tcx().alloc(Ty::Bool),
                })
            }

            Expression::If {
                token,
                condition,
                consequence,
                else_block,
            } => {
                let typed_condition = self.check_expr_as_val(*condition)?;
                let typed_consequence = if self.compile_as == CompileAs::AsValue {
                    self.check_expr_as_val(*consequence)?
                } else {
                    self.check_expr(*consequence)?
                };

                let typed_else_block = match else_block {
                    Some(it) => Some({
                        let expr = if self.compile_as == CompileAs::AsValue {
                            self.check_expr_as_val(*it)
                        } else {
                            self.check_expr(*it)
                        }?;
                        self.module.alloc_expr(expr)
                    }),
                    None => {
                        let consequence_ty = typed_consequence.get_type();

                        if self.compile_as == CompileAs::AsValue
                            && self.tcx().get(consequence_ty) != &Ty::Unit
                        {
                            return Err(Self::make_err(
                                Some("`if` may be missing an `else` clause"),
                                TypeCheckerErrorKind::Other,
                                token,
                            ));
                        } else {
                            None
                        }
                    }
                };

                Ok(TypedExpression::If {
                    token,
                    ty: typed_consequence.get_type(),
                    condition: self.module.alloc_expr(typed_condition),
                    consequence: self.module.alloc_expr(typed_consequence),
                    else_block: typed_else_block,
                })
            }

            Expression::Function {
                token,
                name,
                params,
                block,
                ret_ty: ret_ty_expr,
                generics_params,
            } => {
                let mut generic_names = vec![];

                for generics_param in generics_params
                    .iter()
                    .filter(|it| matches!(&***it, Expression::Ident(_)))
                {
                    let Expression::Ident(it) = &**generics_param else {
                        unreachable!()
                    };

                    let ty_id = self.tcx().alloc(Ty::Generic(it.value.clone(), vec![]));

                    self.tcx()
                        .table
                        .lock()
                        .unwrap()
                        .define_var(&it.value, ty_id);

                    generic_names.push(it.value.clone());
                }

                let mut typed_param_ids: Vec<_> = vec![];

                for param in params {
                    typed_param_ids.push({
                        let expr = self.check_type_expr(*param)?;
                        self.module.alloc_expr(expr)
                    });
                }

                let mut params_type = vec![];

                for typed_param_id in &typed_param_ids {
                    let typed_param = self.module.get_expr(*typed_param_id).unwrap();
                    if let TypedExpression::TypeHint(_, _, ty) = typed_param {
                        params_type.push(ty.clone());
                    }
                }

                // 初步返回定义
                let ret_ty = ret_ty_expr
                    .as_ref()
                    .map_or_else(|| None, |it| Some(self.check_type_expr(*it.clone())))
                    .map_or(Ok(self.tcx().alloc(Ty::Unit)), |it| Ok(it?.get_type()))?;

                let func_ty = Ty::Function {
                    generics: generic_names.clone(),
                    params_type,
                    ret_type: ret_ty,
                    is_variadic: false,
                };

                if let Some(name) = &name {
                    let ty_id = self.tcx().alloc(func_ty.clone());

                    self.tcx()
                        .table
                        .lock()
                        .unwrap()
                        .define_var(&name.value, ty_id);
                }

                let typed_block = match *block {
                    Expression::Block(token, statements) => {
                        self.enter_scope(ScopeKind::Function);

                        for typed_param_id in &typed_param_ids {
                            let typed_param =
                                self.module.get_expr(*typed_param_id).unwrap().clone();

                            if let TypedExpression::TypeHint(name, _, ty) = typed_param {
                                self.tcx()
                                    .table
                                    .lock()
                                    .unwrap()
                                    .define_var(&name.value, ty.clone());
                            }
                        }

                        let (stmts, _) = self.check_statements(statements, ScopeKind::Function)?;

                        let ret_ty = ret_ty;

                        TypedExpression::Block(token, stmts, ret_ty)
                    }
                    other => self.check_type_expr(other)?,
                };

                let block_id = self.module.alloc_expr(typed_block);

                let checked_ret_ty_expr = ret_ty_expr.map(|it| self.check_type_expr(*it));

                let ty = Ty::Function {
                    generics: generic_names,
                    params_type: typed_param_ids
                        .iter()
                        .map(|p| self.get_expr_tyid(*p))
                        .collect(),
                    ret_type: checked_ret_ty_expr
                        .as_ref()
                        .map_or(Ok(self.tcx().alloc(Ty::Unit)), |ty_expr| {
                            Ok(ty_expr.clone()?.get_type())
                        })?,
                    is_variadic: false,
                };

                if let Some(name) = &name {
                    if let Some(def_id) = self
                        .name_resolver
                        .lookup_name(self.current_mod_id, &name.value)
                        && let Def::Function(func_data) =
                            self.name_resolver.krate.get_mut_def(def_id)
                    {
                        func_data.body = Some(block_id)
                    }
                }

                let typed_generic_params = generics_params
                    .clone()
                    .into_iter()
                    .map(|it| self.check_expr(*it))
                    .collect::<Result<Vec<TypedExpression>, _>>()?
                    .into_iter()
                    .map(|it| self.module.alloc_expr(it))
                    .collect();

                // 移除之前定义的泛型避免污染全局空间
                generics_params
                    .iter()
                    .filter(|it| matches!(&***it, Expression::Ident(_)))
                    .for_each(|it| {
                        let Expression::Ident(it) = &**it else {
                            unreachable!()
                        };

                        self.tcx().table.lock().unwrap().remove(&it.value);
                    });

                Ok(TypedExpression::Function {
                    token,
                    name,
                    params: typed_param_ids,
                    block: block_id,
                    ret_ty: if let Some(it) = checked_ret_ty_expr {
                        Some(self.module.alloc_expr(it?))
                    } else {
                        None
                    },
                    ty: self.tcx().alloc(ty),
                    generics_params: typed_generic_params,
                })
            }

            Expression::Call { token, func, args } => {
                let typed_func = self.check_expr(*func)?;

                let Ty::Function { ret_type, .. } = self.tcx().get(typed_func.get_type()).clone()
                else {
                    return Err(Self::make_err(
                        Some("not a function"),
                        TypeCheckerErrorKind::TypeMismatch,
                        typed_func.token(),
                    ));
                };

                let mut typed_arg_ids = vec![];

                for arg in args {
                    typed_arg_ids.push({
                        let expr = self.check_expr(*arg)?;
                        self.module.alloc_expr(expr)
                    });
                }

                Ok(TypedExpression::Call {
                    token,
                    func_ty: typed_func.get_type(),
                    args: typed_arg_ids,
                    func: self.module.alloc_expr(typed_func),
                    ret_ty: ret_type,
                })
            }

            Expression::Block(token, statements) => {
                let mut stmt_ids: Vec<StmtId> = vec![];

                for s in statements {
                    let typed_stmt = self.check_statement(s)?;
                    stmt_ids.push(self.module.alloc_stmt(typed_stmt));
                }

                let ty = stmt_ids
                    .last()
                    .map_or(self.tcx().alloc(Ty::Unit), |s| self.get_stmt_tyid(*s));

                Ok(TypedExpression::Block(token, stmt_ids, ty))
            }

            Expression::TypeHint(ident, ty_expr) => {
                let new_ident = Ident {
                    token: ident.token,
                    value: ident.value,
                };

                let new_ty_expr = self.check_type_expr(*ty_expr)?;

                let ty = new_ty_expr.get_type();

                Ok(TypedExpression::TypeHint(
                    new_ident,
                    self.module.alloc_expr(new_ty_expr),
                    ty,
                ))
            }

            Expression::GenericInstance { token, left, paths } => {
                let typed_left = self.check_expr_as_val(*left)?;

                // 将会在 TypeInfer 被替换
                let initial_ty = typed_left.get_type();

                let left_expr_id = self.module.alloc_expr(typed_left);

                let mut typed_paths = vec![];
                for path in paths {
                    let typed_path = self.check_type_expr(*path)?;
                    typed_paths.push(self.module.alloc_expr(typed_path));
                }

                Ok(TypedExpression::GenericInstance {
                    token,
                    left: left_expr_id,
                    paths: typed_paths,
                    ty: initial_ty,
                })
            }

            // 如果出现此表达式请考虑parser是否损坏
            Expression::ThreeDot(_) => unreachable!(),

            it => todo!("todo expr: {it}"),
        }
    }

    /// 仅支持以下表达式以解析类型
    /// Ident, Prefix (*), TypeHint
    pub fn check_type_expr(&mut self, expr: Expression) -> CheckResult<TypedExpression> {
        match expr {
            Expression::Ident(it) => {
                let ident_name = &it.value;

                if let Some(symbol) = self.tcx().table.lock().unwrap().get(&ident_name) {
                    return Ok(TypedExpression::Ident(
                        Ident {
                            token: it.token,
                            value: it.value,
                        },
                        symbol.ty.get_type(),
                        None,
                    ));
                } else if let Some(def_id) = self
                    .name_resolver
                    .lookup_name(self.current_mod_id, &it.value)
                {
                    let ty_id = self.resolve_def_type(def_id)?;

                    return Ok(TypedExpression::Ident(
                        Ident {
                            token: it.token,
                            value: it.value,
                        },
                        ty_id,
                        Some(def_id),
                    ));
                } else {
                    Err(Self::make_err(
                        None,
                        TypeCheckerErrorKind::VariableNotFound,
                        it.token,
                    ))
                }
            }

            Expression::Prefix { token, op, right } => {
                let inner_t = self.check_type_expr(*right)?; // 递归解析

                Ok(TypedExpression::Prefix {
                    ty: if op.as_ref() == "*" {
                        let inner_ty = inner_t.get_type();
                        self.tcx().alloc(Ty::Ptr(inner_ty))
                    } else {
                        return Err(Self::make_err(
                            Some(&format!("unknown operator `{}`", &token.value)),
                            TypeCheckerErrorKind::Other,
                            token,
                        ));
                    },
                    token,
                    right: self.module.alloc_expr(inner_t),
                    op,
                })
            }

            Expression::TypeHint(ident, ty_expr) => {
                let new_ident = Ident {
                    token: ident.token,
                    value: ident.value,
                };

                let new_ty_expr = self.check_type_expr(*ty_expr)?;

                let ty = new_ty_expr.get_type();

                Ok(TypedExpression::TypeHint(
                    new_ident,
                    self.module.alloc_expr(new_ty_expr),
                    ty,
                ))
            }

            Expression::TypePath {
                token, left, paths, ..
            } => {
                let left = Ident {
                    token: left.token,
                    value: left.value,
                };

                let _base_ty = self.lookup_type_by_name(&left.value, left.token.clone())?;

                let typed_path_exprs = paths
                    .into_iter()
                    .map(|it| self.check_expr(*it))
                    .collect::<Result<Vec<_>, _>>()?;

                let paths_type = typed_path_exprs
                    .iter()
                    .map(|it| it.get_type())
                    .collect::<Vec<_>>();

                Ok(TypedExpression::TypePath {
                    token,
                    ty: self
                        .tcx()
                        .alloc(Ty::AppliedGeneric(left.value.clone(), paths_type)),
                    left,
                    paths: typed_path_exprs
                        .into_iter()
                        .map(|it| self.module.alloc_expr(it))
                        .collect(),
                })
            }

            _ => Err(Self::make_err(
                Some("not a type expr"),
                TypeCheckerErrorKind::Other,
                expr.token(),
            )),
        }
    }

    pub fn check_stmt_with_mod_id(
        &mut self,
        stmt: Statement,
        id: ModuleId,
    ) -> CheckResult<TypedStatement> {
        let old = self.current_mod_id;
        self.current_mod_id = id;

        let result = self.check_statement(stmt);

        self.current_mod_id = old;

        result
    }

    pub fn check_statement(&mut self, stmt: Statement) -> CheckResult<TypedStatement> {
        match stmt {
            Statement::Use {
                token,
                full_path,
                alias,
            } => Ok(TypedStatement::Use {
                token,
                full_path,
                alias,
                ty: self.tcx().alloc(Ty::Unit),
            }),
            Statement::Impl {
                token,
                impl_,
                for_,
                block,
            } => {
                let new_impl_ = Ident {
                    token: impl_.token,
                    value: impl_.value,
                };

                if self
                    .tcx()
                    .table
                    .lock()
                    .unwrap()
                    .get(&new_impl_.value)
                    .is_none()
                {
                    return Err(Self::make_err(
                        Some(&format!("cannot find type '{new_impl_}' in this scope")),
                        TypeCheckerErrorKind::TypeNotFound,
                        new_impl_.token,
                    ));
                }

                let new_for_ = for_.map_or(None, |it| {
                    Some(Ident {
                        token: it.token,
                        value: it.value,
                    })
                });

                if let Some(ref new_for_) = new_for_
                    && self
                        .tcx()
                        .table
                        .lock()
                        .unwrap()
                        .get(&new_for_.value)
                        .is_none()
                {
                    return Err(Self::make_err(
                        Some(&format!("cannot find type '{new_for_}' in this scope")),
                        TypeCheckerErrorKind::TypeNotFound,
                        new_for_.token.clone(),
                    ));
                }

                self.enter_scope(ScopeKind::Class);

                let typed_block = self.check_statement(*block)?;

                let table = self.leave_scope().1;

                let mut new_fields = IndexMap::new();

                let symbols = self.tcx().table.lock().unwrap().var_map.clone();

                for (_, sym) in symbols {
                    let Ty::Struct {
                        name: struct_name,
                        fields,
                        ..
                    } = self.tcx().get_mut(sym.ty.get_type())
                    else {
                        continue;
                    };

                    table.lock().unwrap().var_map.iter().for_each(|(k, v)| {
                        new_fields.insert(k.clone(), v.clone().ty.get_type());
                    });

                    // impl XXXX {}
                    if new_for_.is_none() && new_impl_.value == *struct_name {
                        fields.append(&mut new_fields);
                    }

                    continue;
                }

                Ok(TypedStatement::Impl {
                    token,
                    new_fields,
                    impl_: new_impl_,
                    for_: new_for_,
                    block: self.module.alloc_stmt(typed_block),
                    ty: self.tcx().alloc(Ty::Unit),
                })
            }

            Statement::FuncDecl {
                token,
                name,
                params,
                ret_ty: ret_ty_expr,
                generics_params,
            } => {
                let mut generic_names = vec![];

                for generics_param in generics_params
                    .iter()
                    .filter(|it| matches!(&***it, Expression::Ident(_)))
                {
                    let Expression::Ident(it) = &**generics_param else {
                        unreachable!()
                    };

                    let ty_id = self.tcx().alloc(Ty::Generic(it.value.clone(), vec![]));

                    self.tcx()
                        .table
                        .lock()
                        .unwrap()
                        .define_var(&it.value, ty_id);

                    generic_names.push(it.value.clone());
                }

                let mut typed_param_ids: Vec<_> = vec![];

                for param in params {
                    let typed_param = self.check_expr(*param)?;
                    typed_param_ids.push(self.module.alloc_expr(typed_param));
                }

                let mut params_type = vec![];

                for typed_param_id in &typed_param_ids {
                    let typed_param = self.module.get_expr(*typed_param_id).unwrap();
                    if let TypedExpression::TypeHint(_, _, ty) = typed_param {
                        params_type.push(ty.clone());
                    }
                }

                // 初步返回定义
                let ret_ty = ret_ty_expr
                    .as_ref()
                    .map_or_else(|| None, |it| Some(self.check_type_expr(*it.clone())))
                    .map_or(Ok(self.tcx().alloc(Ty::Unit)), |it| Ok(it?.get_type()))?;

                let func_ty = Ty::Function {
                    generics: generic_names.clone(),
                    params_type,
                    ret_type: *&ret_ty,
                    is_variadic: false,
                };

                let func_ty_id = self.tcx().alloc(func_ty.clone());

                self.tcx()
                    .table
                    .lock()
                    .unwrap()
                    .define_var(&name.value, func_ty_id);

                let checked_ret_ty_expr = ret_ty_expr.map(|it| self.check_type_expr(*it));

                let ty = Ty::Function {
                    generics: generic_names,
                    params_type: typed_param_ids
                        .iter()
                        .map(|p| self.get_expr_tyid(*p))
                        .collect(),
                    ret_type: checked_ret_ty_expr
                        .as_ref()
                        .map_or(Ok(self.tcx().alloc(Ty::Unit)), |ty_expr| {
                            Ok(ty_expr.clone()?.get_type())
                        })?,
                    is_variadic: false,
                };

                let ty_id = self.tcx().alloc(ty.clone());

                self.tcx()
                    .table
                    .lock()
                    .unwrap()
                    .define_var(&name.value, ty_id);

                let typed_generic_params = generics_params
                    .clone()
                    .into_iter()
                    .map(|it| self.check_expr(*it))
                    .collect::<Result<Vec<TypedExpression>, _>>()?
                    .into_iter()
                    .map(|it| self.module.alloc_expr(it))
                    .collect();

                // 移除之前定义的泛型避免污染全局空间
                generics_params
                    .iter()
                    .filter(|it| matches!(&***it, Expression::Ident(_)))
                    .for_each(|it| {
                        let Expression::Ident(it) = &**it else {
                            unreachable!()
                        };

                        self.tcx().table.lock().unwrap().remove(&it.value);
                    });

                Ok(TypedStatement::FuncDecl {
                    token,
                    name,
                    params: typed_param_ids,
                    ret_ty: if let Some(it) = checked_ret_ty_expr {
                        Some(self.module.alloc_expr(it?))
                    } else {
                        None
                    },
                    ty: self.tcx().alloc(ty),
                    generics_params: typed_generic_params,
                })
            }

            Statement::Extern {
                token,
                abi,
                extern_func_name,
                alias,
                params,
                ret_ty,
                vararg,
            } => {
                let mut typed_param_ids = vec![];
                let mut params_type = vec![];
                let mut param_mapping = IndexMap::new();

                for param in params {
                    let typed_param = self.check_type_expr(*param)?;
                    if let TypedExpression::TypeHint(ident, _, ty) = &typed_param {
                        param_mapping.insert(ident.value.clone(), *ty);
                    }

                    params_type.push(typed_param.get_type());
                    typed_param_ids.push(self.module.alloc_expr(typed_param));
                }

                let ret_ty_expr_typed = if let Some(ret_ty) = ret_ty {
                    Some(self.check_type_expr(*ret_ty)?)
                } else {
                    None
                };

                let func_ty = Ty::Function {
                    generics: vec![],
                    params_type,
                    ret_type: ret_ty_expr_typed
                        .as_ref()
                        .map_or_else(|| self.tcx().alloc(Ty::Unit), |it| it.get_type()),
                    is_variadic: vararg,
                };

                let func_ty_id = self.tcx().alloc(func_ty.clone());

                self.tcx()
                    .table
                    .lock()
                    .unwrap()
                    .define_var(&alias.value, func_ty_id);

                // 在此填充 Def
                if let Some(def_id) = self
                    .name_resolver
                    .lookup_name(self.current_mod_id, &alias.value)
                    && let Def::Function(func_data) = self.name_resolver.krate.get_mut_def(def_id)
                {
                    func_data.ty = func_ty_id.into();
                    func_data.params = param_mapping;
                    func_data.is_variadic = vararg;
                }

                Ok(TypedStatement::Extern {
                    ty: func_ty_id,
                    token,
                    abi,
                    alias,
                    extern_func_name,
                    params: typed_param_ids,
                    ret_ty: ret_ty_expr_typed.map(|it| self.module.alloc_expr(it)),
                    vararg,
                })
            }

            Statement::Struct {
                token,
                name,
                fields,
                generics,
            } => {
                let typed_name = Ident {
                    token: name.token,
                    value: name.value.clone(),
                };

                for generic in generics
                    .iter()
                    .filter(|it| matches!(&***it, Expression::Ident(_)))
                {
                    let Expression::Ident(it) = &**generic else {
                        unreachable!()
                    };

                    let ty_id = self.tcx().alloc(Ty::Generic(it.value.clone(), vec![]));

                    self.tcx()
                        .table
                        .lock()
                        .unwrap()
                        .define_var(&it.value, ty_id);
                }

                let mut typed_field_ids = vec![];

                for field in fields {
                    if !matches!(*field, Expression::TypeHint(_, _)) {
                        return Err(Self::make_err(
                            Some(&format!("not a type hint: {field}")),
                            TypeCheckerErrorKind::Other,
                            field.token(),
                        ));
                    }

                    let typed_field = self.check_expr(*field)?;
                    typed_field_ids.push(self.module.alloc_expr(typed_field));
                }

                let generic_names = generics
                    .iter()
                    .map(|g| match &**g {
                        Expression::Ident(ident) => ident.value.clone(),
                        _ => todo!("generic param must be ident"),
                    })
                    .collect::<Vec<_>>();

                let fields = {
                    let mut m = IndexMap::new();

                    for field_id in &typed_field_ids {
                        let field = self.module.get_expr(*field_id).unwrap();
                        if let TypedExpression::TypeHint(name, _, ty) = field {
                            m.insert(name.value.clone(), ty.clone());
                        } else {
                            return Err(Self::make_err(
                                Some(&format!("not a type hint")),
                                TypeCheckerErrorKind::Other,
                                field.token(),
                            ));
                        }
                    }

                    m
                };

                // 在此填充 Def
                if let Some(def_id) = self
                    .name_resolver
                    .lookup_name(self.current_mod_id, &name.value)
                    && let Def::Struct(struct_data) = self.name_resolver.krate.get_mut_def(def_id)
                {
                    struct_data.fields = fields.clone();
                    struct_data.generics = generic_names.clone();
                }

                let ty = Ty::Struct {
                    name: name.value.clone(),
                    generics: generic_names,
                    fields,
                    impl_traits: IndexMap::new(),
                };

                let ty_id = self.tcx().alloc(ty.clone());

                self.tcx()
                    .table
                    .lock()
                    .unwrap()
                    .define_var(&typed_name.value, ty_id);

                let typed_generics = generics
                    .clone()
                    .into_iter()
                    .map(|it| self.check_expr(*it))
                    .collect::<CheckResult<Vec<TypedExpression>>>()?
                    .into_iter()
                    .map(|it| self.module.alloc_expr(it))
                    .collect();

                // 移除之前定义的泛型避免污染全局空间
                generics
                    .iter()
                    .filter(|it| matches!(&***it, Expression::Ident(_)))
                    .for_each(|it| {
                        let Expression::Ident(it) = &**it else {
                            unreachable!()
                        };

                        self.tcx().table.lock().unwrap().remove(&it.value);
                    });

                Ok(TypedStatement::Struct {
                    ty: ty_id,
                    token,
                    name: typed_name,
                    fields: typed_field_ids,
                    generics: typed_generics,
                })
            }
            Statement::Trait { token, name, block } => {
                let typed_name = Ident {
                    token: name.token,
                    value: name.value.clone(),
                };

                let mut stmt_ids = vec![];

                let Statement::Block {
                    token: block_token,
                    statements,
                    ..
                } = *block
                else {
                    return Err(Self::make_err(
                        Some(&format!("not a block: {block}")),
                        TypeCheckerErrorKind::Other,
                        block.token(),
                    ));
                };

                self.enter_scope(ScopeKind::Trait);

                for stmt in statements {
                    if !matches!(stmt, Statement::FuncDecl { .. }) {
                        return Err(Self::make_err(
                            Some(&format!("not a func decl: {stmt}")),
                            TypeCheckerErrorKind::Other,
                            stmt.token(),
                        ));
                    }

                    let typed_stmt = self.check_statement(stmt)?;
                    stmt_ids.push(self.module.alloc_stmt(typed_stmt));
                }

                self.leave_scope();

                let ty = Ty::Trait {
                    name: name.value.clone(),
                    functions: {
                        let mut m = IndexMap::new();

                        for funcid in &stmt_ids {
                            let func = self.module.get_stmt(*funcid).unwrap();

                            if let TypedStatement::FuncDecl { ty, .. } = func {
                                m.insert(name.value.clone(), ty.clone());
                            } else {
                                return Err(Self::make_err(
                                    Some(&format!("not a func decl")),
                                    TypeCheckerErrorKind::Other,
                                    func.token(),
                                ));
                            }
                        }

                        m
                    },
                };

                let ty_id = self.tcx().alloc(ty.clone());

                self.tcx()
                    .table
                    .lock()
                    .unwrap()
                    .define_var(&typed_name.value, ty_id);

                let block_ty = self.tcx().alloc(Ty::Unit);

                Ok(TypedStatement::Trait {
                    ty: self.tcx().alloc(ty),
                    token,
                    name: typed_name,
                    block: self.module.alloc_stmt(TypedStatement::Block {
                        token: block_token,
                        statements: stmt_ids,
                        ty: block_ty,
                    }),
                })
            }
            Statement::While {
                token,
                condition,
                block,
            } => {
                let typed_condition = self.check_expr(condition)?;

                let typed_block = self.check_statement(*block)?;

                Ok(TypedStatement::While {
                    token,
                    condition: self.module.alloc_expr(typed_condition),
                    block: self.module.alloc_stmt(typed_block),
                    ty: self.tcx().alloc(Ty::Unit),
                })
            }

            Statement::ExpressionStatement(expr) => {
                let token = expr.token();

                let typed_expr = self.check_expr(expr)?;

                let ty = typed_expr.get_type();

                Ok(TypedStatement::ExpressionStatement(
                    token,
                    self.module.alloc_expr(typed_expr),
                    ty,
                ))
            }

            Statement::Let {
                token,
                name,
                var_type,
                value,
            } => {
                // 检查表达式的类型
                let typed_val = self.check_expr_as_val(value)?;

                // 如果有类型标注尝试获取类型 否则直接获取表达式的值
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
                    typed_val.get_type()
                };

                self.tcx()
                    .table
                    .lock()
                    .unwrap()
                    .define_var(&name.value, ty.clone());

                Ok(TypedStatement::Let {
                    token: token.clone(),
                    name: Ident {
                        token,
                        value: name.value,
                    },
                    var_type: match var_type {
                        Some(it) => Some(Ident {
                            token: it.token,
                            value: it.value,
                        }),

                        None => None,
                    },
                    ty,
                    value: self.module.alloc_expr(typed_val),
                })
            }

            Statement::Const {
                token,
                name,
                var_type,
                value,
            } => {
                // 检查表达式是否为字面量
                if !value.is_literal() {
                    return Err(Self::make_err(
                        Some(&format!("expression `{value}` is not a constant")),
                        TypeCheckerErrorKind::NotAConstant,
                        value.token(),
                    ));
                }

                // 检查表达式的类型
                let typed_val = self.check_expr_as_val(value)?;

                // 如果有类型标注尝试获取类型 否则直接获取表达式的值
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
                    typed_val.get_type()
                };

                self.tcx()
                    .table
                    .lock()
                    .unwrap()
                    .define_var(&name.value, ty.clone());

                Ok(TypedStatement::Const {
                    token: token.clone(),
                    name: Ident {
                        token,
                        value: name.value,
                    },
                    var_type: match var_type {
                        Some(it) => Some(Ident {
                            token: it.token,
                            value: it.value,
                        }),

                        None => None,
                    },
                    ty,
                    value: self.module.alloc_expr(typed_val),
                })
            }

            Statement::Return { token, expr } => {
                let expr = if let Some(it) = expr {
                    Some(self.check_expr(it)?)
                } else {
                    None
                };

                let (ret_ty, ret_token) = if let Some(ref it) = expr {
                    (it.get_type(), it.token())
                } else {
                    (self.tcx().alloc(Ty::Unit), token.clone())
                };

                self.current_scope_mut()
                    .collect_return_types
                    .push((ret_ty, ret_token));

                Ok(TypedStatement::Return {
                    token,
                    expr: expr.and_then(|it| Some(self.module.alloc_expr(it))),
                    ty: ret_ty,
                })
            }

            Statement::Block { token, statements } => {
                let mut stmt_ids = vec![];

                for s in statements {
                    let typed_stmt = self.check_statement(s)?;
                    stmt_ids.push(self.module.alloc_stmt(typed_stmt));
                }

                let ty = stmt_ids
                    .last()
                    .map_or(self.tcx().alloc(Ty::Unit), |s| self.get_stmt_tyid(*s));

                Ok(TypedStatement::Block {
                    token,
                    statements: stmt_ids,
                    ty,
                })
            }
        }
    }

    fn get_def_token(&'a self, id: DefId) -> Token {
        let stmt = self.get_raw_stmt(id);
        stmt.token()
    }

    fn get_expr_tyid(&self, exprid: ExprId) -> TyId {
        self.module.get_expr(exprid).unwrap().get_type()
    }

    fn get_stmt_tyid(&self, stmtid: StmtId) -> TyId {
        self.module.get_stmt(stmtid).unwrap().get_type()
    }

    #[allow(unused)]
    fn require_eq(&mut self, expected: TyId, got: TyId, token: Token) {
        self.constraints.push(Constraint::new(expected, got, token));
    }

    pub fn enter_scope(&mut self, kind: ScopeKind) {
        self.tcx().table = Arc::new(Mutex::new(TypeTable::with_outer(self.tcx().table.clone())));

        self.scope_index += 1;

        self.scopes.push(CheckScope {
            kind,
            collect_return_types: vec![],
        });
    }

    pub fn current_scope(&self) -> &CheckScope {
        &self.scopes[self.scope_index]
    }

    pub fn current_scope_mut(&mut self) -> &mut CheckScope {
        &mut self.scopes[self.scope_index]
    }

    pub fn tcx(&mut self) -> &mut TypeContext {
        self.module.tcx_mut()
    }

    pub fn tcx_ref(&self) -> &TypeContext {
        self.module.tcx_ref()
    }

    pub fn leave_scope(&mut self) -> (CheckScope, Arc<Mutex<TypeTable>>) {
        let before_enter_scope_table = self
            .tcx()
            .table
            .lock()
            .unwrap()
            .outer
            .clone()
            .expect("expected an outer");

        let cur_scope_table = self.tcx().table.clone();

        self.tcx().table = before_enter_scope_table;

        self.scope_index -= 1;

        (self.scopes.pop().unwrap(), cur_scope_table)
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
}

impl<'a, 'b> TypeChecker<'a, 'b> {
    pub fn resolve_def_type(&mut self, def_id: DefId) -> CheckResult<TyId> {
        let existing_ty = self.name_resolver.krate.get_def(def_id).ty().unwrap_or(0);

        /*
        考虑到目前零号类型是 BigInt，该类型很少见。所以
        即使重新检查一遍所耗费时间也可忽略不计
        */
        if existing_ty != 0 {
            return Ok(existing_ty);
        }

        // 循环检测 防止 const A = B; const B = A; 这种死循环
        if !self.resolving_defs.insert(def_id) {
            return Err(Self::make_err(
                Some("circular dependency detected"),
                TypeCheckerErrorKind::Other,
                self.get_def_token(def_id),
            ));
        }

        let mod_id_of_def = self.name_resolver.krate.get_def(def_id).module_id();
        let raw_stmt = self.get_raw_stmt(def_id).clone();

        // 上下文切换
        let old_mod_id = self.current_mod_id;
        self.current_mod_id = mod_id_of_def;

        let result = match raw_stmt {
            Statement::Const { value, .. } => {
                let typed_expr = self.check_expr(value)?;
                let ty = typed_expr.get_type();
                self.write_back_type(def_id, ty);
                Ok(ty)
            }

            Statement::FuncDecl {
                params,
                ret_ty,
                generics_params,
                ..
            } => {
                // 这里只解析函数头
                let mut generics = vec![];

                for generics_param in generics_params
                    .iter()
                    .filter(|it| matches!(&***it, Expression::Ident(_)))
                {
                    let Expression::Ident(it) = &**generics_param else {
                        unreachable!()
                    };

                    let ty_id = self.tcx().alloc(Ty::Generic(it.value.clone(), vec![]));

                    generics.push(it.value.clone());

                    self.tcx()
                        .table
                        .lock()
                        .unwrap()
                        .define_var(&it.value, ty_id);
                }

                let mut params_type = vec![];
                for param in params {
                    params_type.push(self.check_type_expr(*param)?.get_type());
                }

                let ret_ty_id = ret_ty.map_or(Ok(self.tcx().alloc(Ty::Unit)), |ret| {
                    self.check_type_expr(*ret).map(|it| it.get_type())
                })?;

                // 清理泛型
                generics.iter().for_each(|it| {
                    self.tcx().table.lock().unwrap().var_map.remove(it);
                });

                let func_ty = self.tcx().alloc(Ty::Function {
                    generics,
                    params_type,
                    ret_type: ret_ty_id,
                    is_variadic: false,
                });

                self.write_back_type(def_id, func_ty);
                Ok(func_ty)
            }

            Statement::ExpressionStatement(Expression::Function {
                params,
                ret_ty,
                generics_params,
                ..
            }) => {
                // 这里只解析函数头
                let mut generics = vec![];

                for generics_param in generics_params
                    .iter()
                    .filter(|it| matches!(&***it, Expression::Ident(_)))
                {
                    let Expression::Ident(it) = &**generics_param else {
                        unreachable!()
                    };

                    let ty_id = self.tcx().alloc(Ty::Generic(it.value.clone(), vec![]));

                    generics.push(it.value.clone());

                    self.tcx()
                        .table
                        .lock()
                        .unwrap()
                        .define_var(&it.value, ty_id);
                }

                let mut params_type = vec![];
                let mut param_mapping = IndexMap::new();

                for param in params {
                    let typed_param = self.check_type_expr(*param)?;

                    if let TypedExpression::TypeHint(name, ..) = &typed_param {
                        param_mapping.insert(name.value.clone(), typed_param.get_type());
                    }

                    params_type.push(typed_param.get_type());
                }

                let ret_ty_id = ret_ty.map_or(Ok(self.tcx().alloc(Ty::Unit)), |ret| {
                    self.check_type_expr(*ret).map(|it| it.get_type())
                })?;

                // 清理泛型
                generics.iter().for_each(|it| {
                    self.tcx().table.lock().unwrap().var_map.remove(it);
                });

                let func_ty = self.tcx().alloc(Ty::Function {
                    generics,
                    params_type,
                    ret_type: ret_ty_id,
                    is_variadic: false,
                });

                self.write_back_type(def_id, func_ty);

                if let Def::Function(func_data) = self.name_resolver.krate.get_mut_def(def_id) {
                    func_data.params = param_mapping
                }

                Ok(func_ty)
            }

            Statement::Struct {
                name: struct_name,
                fields: raw_fields,
                generics: raw_generics,
                ..
            } => {
                for generic in raw_generics
                    .iter()
                    .filter(|it| matches!(&***it, Expression::Ident(_)))
                {
                    let Expression::Ident(it) = &**generic else {
                        unreachable!()
                    };

                    let ty_id = self.tcx().alloc(Ty::Generic(it.value.clone(), vec![]));

                    self.tcx()
                        .table
                        .lock()
                        .unwrap()
                        .define_var(&it.value, ty_id);
                }

                let mut fields = IndexMap::new();

                for field_expr in raw_fields {
                    let typed_field = self.check_type_expr(*field_expr.clone())?;

                    if let TypedExpression::TypeHint(field_ident, _, field_ty) = typed_field {
                        fields.insert(field_ident.value.clone(), field_ty);
                    }
                }

                // 移除之前定义的泛型避免污染全局空间
                raw_generics
                    .iter()
                    .filter(|it| matches!(&***it, Expression::Ident(_)))
                    .for_each(|it| {
                        let Expression::Ident(it) = &**it else {
                            unreachable!()
                        };

                        self.tcx().table.lock().unwrap().remove(&it.value);
                    });

                // 处理泛型名
                let generics: Vec<Arc<str>> =
                    raw_generics.iter().map(|g| g.to_string().into()).collect();

                let struct_ty = self.tcx().alloc(Ty::Struct {
                    name: struct_name.value.clone(),
                    generics,
                    fields,
                    impl_traits: IndexMap::new(),
                });

                self.write_back_type(def_id, struct_ty);
                Ok(struct_ty)
            }

            Statement::Extern {
                vararg,
                alias,
                params,
                ret_ty,
                ..
            } => {
                let mut typed_param_ids = vec![];
                let mut params_type = vec![];
                let mut param_mapping = IndexMap::new();

                for param in params {
                    let typed_param = self.check_type_expr(*param)?;
                    if let TypedExpression::TypeHint(ident, _, ty) = &typed_param {
                        param_mapping.insert(ident.value.clone(), *ty);
                    }

                    params_type.push(typed_param.get_type());
                    typed_param_ids.push(self.module.alloc_expr(typed_param));
                }

                let ret_ty_expr_typed = if let Some(ret_ty) = ret_ty {
                    Some(self.check_type_expr(*ret_ty)?)
                } else {
                    None
                };

                let func_ty = Ty::Function {
                    generics: vec![],
                    params_type,
                    ret_type: ret_ty_expr_typed
                        .as_ref()
                        .map_or_else(|| self.tcx().alloc(Ty::Unit), |it| it.get_type()),
                    is_variadic: vararg,
                };

                let func_ty_id = self.tcx().alloc(func_ty.clone());

                self.tcx()
                    .table
                    .lock()
                    .unwrap()
                    .define_var(&alias.value, func_ty_id);

                self.write_back_type(def_id, func_ty_id);
                Ok(func_ty_id)
            }

            it => todo!("todo def {it}"),
        };

        self.current_mod_id = old_mod_id;
        self.resolving_defs.remove(&def_id);
        result
    }
}

impl<'a, 'b> TypeChecker<'a, 'b> {
    pub fn lookup_type_by_name(&mut self, name: &str, token: Token) -> CheckResult<TyId> {
        // 查局部符号表
        if let Some(symbol) = self.tcx().table.lock().unwrap().get(name) {
            return Ok(symbol.ty.get_type());
        }

        // 全局符号表
        if let Some(def_id) = self.name_resolver.lookup_name(self.current_mod_id, name) {
            // 触发拉取式推导，确保那个 Def 已经被填坑了
            return self.resolve_def_type(def_id);
        }

        Err(Self::make_err(
            Some(&format!("type `{}` not found in this scope", name)),
            TypeCheckerErrorKind::TypeNotFound,
            token,
        ))
    }
}

impl TypeChecker<'_, '_> {
    pub fn get_type_context(&self) -> &TypeContext {
        self.module.tcx_ref()
    }

    pub fn get_constraints(&self) -> &Vec<Constraint> {
        &self.constraints
    }
}
