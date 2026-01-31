pub mod error;
pub mod scope;
pub mod table;
pub mod test;
pub mod ty;
pub mod typed_ast;

use std::sync::{Arc, Mutex};

use ast::{
    expr::Expression,
    node::{GetToken, Node},
    stmt::Statement,
};
use indexmap::IndexMap;
use token::token::Token;

use crate::{
    error::{TypeCheckerError, TypeCheckerErrorKind},
    scope::{CheckScope, ScopeKind},
    table::{SymbolType, TypeTable},
    ty::Ty,
    typed_ast::{
        GetType, typed_expr::TypedExpression, typed_expressions::ident::Ident,
        typed_node::TypedNode, typed_stmt::TypedStatement,
    },
};

type CheckResult<T> = Result<T, TypeCheckerError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompileAs {
    AsValue,
    AsNone,
}

pub struct TypeChecker {
    table: Arc<Mutex<TypeTable>>,

    scopes: Vec<CheckScope>,
    scope_index: usize,

    compile_as: CompileAs,
}

impl TypeChecker {
    pub fn new(table: Arc<Mutex<TypeTable>>) -> Self {
        let global_scope = CheckScope {
            kind: ScopeKind::Global,
            collect_return_types: vec![],
        };

        Self {
            table,

            compile_as: CompileAs::AsNone,

            scope_index: 0,
            scopes: vec![global_scope],
        }
    }

    pub fn check_node(&mut self, node: Node) -> CheckResult<TypedNode> {
        match node {
            Node::Program { token, statements } => {
                let mut typed_statements = vec![];

                for stmt in statements {
                    typed_statements.push(self.check_statement(stmt)?);
                }

                Ok(TypedNode::Program {
                    token,
                    statements: typed_statements,
                })
            }
        }
    }

    pub fn check_statements(
        &mut self,
        statements: Vec<Statement>,
        scope_kind: ScopeKind,
    ) -> CheckResult<(Vec<TypedStatement>, CheckScope)> {
        let mut typed_statements = vec![];

        if statements.is_empty() {
            self.current_scope_mut()
                .collect_return_types
                .push((Ty::Unit, Token::eof("unknown".into(), 0, 0)));
            return Ok((typed_statements, self.leave_scope().0));
        }

        let statement_count = statements.len() - 1;

        for (i, stmt) in statements.into_iter().enumerate() {
            typed_statements.push(
                if i == statement_count && scope_kind == ScopeKind::Function {
                    self.compile_as = CompileAs::AsValue;
                    let r = self.check_statement(stmt)?;
                    self.compile_as = CompileAs::AsNone;

                    self.current_scope_mut()
                        .collect_return_types
                        .push((r.get_type(), r.token()));

                    r
                } else {
                    self.check_statement(stmt)?
                },
            );
        }

        Ok((typed_statements, self.leave_scope().0))
    }

    pub fn check_expr_as_val(&mut self, expr: Expression) -> CheckResult<TypedExpression> {
        self.compile_as = CompileAs::AsValue;

        let result = self.check_expr(expr);

        self.compile_as = CompileAs::AsNone;

        result
    }

    pub fn check_expr(&mut self, expr: Expression) -> CheckResult<TypedExpression> {
        match expr {
            Expression::BigInt { token, value } => Ok(TypedExpression::BigInt {
                token,
                value,
                ty: Ty::BigInt,
            }),
            Expression::Bool { token, value } => Ok(TypedExpression::Bool {
                token,
                value,
                ty: Ty::Bool,
            }),
            Expression::Int { token, value } => Ok(TypedExpression::Int {
                token,
                value,
                ty: Ty::IntTy(value.into()),
            }),
            Expression::StrLiteral { token, value } => Ok(TypedExpression::StrLiteral {
                token,
                value,
                ty: Ty::Str,
            }),

            Expression::FieldAccess(struct_expr, field) => {
                let new_field = Ident {
                    value: field.value,
                    token: field.token,
                };

                let typed_struct_expr = Box::new(self.check_expr(*struct_expr)?);

                let Ty::Struct {
                    name: struct_name,
                    fields,
                    impl_traits: _impl_traits,
                } = typed_struct_expr.get_type()
                else {
                    Err(Self::make_err(
                        Some("not a struct: {typed_struct_expr}"),
                        TypeCheckerErrorKind::TypeMismatch,
                        typed_struct_expr.token(),
                    ))?
                };

                let Some(field_ty) = fields.get(&new_field.value) else {
                    Err(Self::make_err(
                        Some(&format!(
                            "field {} of struct {struct_name} not found",
                            &new_field.value
                        )),
                        TypeCheckerErrorKind::VariableNotFound,
                        new_field.token.clone(),
                    ))?
                };

                Ok(TypedExpression::FieldAccess(
                    typed_struct_expr,
                    new_field,
                    field_ty.clone(),
                ))
            }

            Expression::BuildStruct(token, struct_name, fields) => {
                let struct_name = Ident {
                    value: struct_name.value,
                    token: struct_name.token,
                };

                let struct_ty = self
                    .table
                    .lock()
                    .unwrap()
                    .get(&struct_name.value)
                    .map_or_else(
                        || {
                            Err(Self::make_err(
                                Some(&format!("type not found: {}", &struct_name.value)),
                                TypeCheckerErrorKind::VariableNotFound,
                                struct_name.token.clone(),
                            ))
                        },
                        |it| Ok(it.ty.get_type()),
                    )?;

                let mut typed_fields = IndexMap::new();

                for (field_name, field_val) in fields {
                    typed_fields.insert(
                        Ident {
                            value: field_name.value.clone(),
                            token: field_name.token,
                        },
                        self.check_expr(field_val)?,
                    );
                }

                Ok(TypedExpression::BuildStruct(
                    token,
                    struct_name,
                    typed_fields,
                    struct_ty,
                ))
            }

            Expression::Assign { token, left, right } => {
                let typed_left = self.check_expr(*left)?;
                let typed_right = self.check_expr(*right)?;

                if typed_left.get_type() != typed_right.get_type() {
                    // 赋值给变量的类型不符合
                    return Err(Self::make_err(
                        Some(&format!(
                            "expected: {}, got: {}",
                            typed_left.get_type(),
                            typed_right.get_type()
                        )),
                        TypeCheckerErrorKind::TypeMismatch,
                        typed_right.token(),
                    ));
                }

                Ok(TypedExpression::Assign {
                    token,
                    left: Box::new(typed_left),
                    right: Box::new(typed_right),
                })
            }

            Expression::Ident(it) => {
                let ident_name = &it.value;

                match self.table.lock().unwrap().get(&ident_name) {
                    Some(symbol) => Ok(TypedExpression::Ident(
                        Ident {
                            token: it.token,
                            value: it.value,
                        },
                        symbol.ty.get_type(),
                    )),
                    None => Err(Self::make_err(
                        None,
                        TypeCheckerErrorKind::VariableNotFound,
                        it.token,
                    )),
                }
            }

            Expression::Infix {
                token,
                left,
                right,
                op,
                ..
            } => {
                let left_t = self.check_expr(*left)?;
                let right_t = self.check_expr(*right)?;

                let lty = left_t.get_type();
                let rty = right_t.get_type();

                if lty != rty {
                    return Err(Self::make_err(
                        None,
                        TypeCheckerErrorKind::TypeMismatch,
                        token,
                    ));
                }

                Ok(TypedExpression::Infix {
                    token,
                    ty: if op.as_ref() == "==" || op.as_ref() == "!=" {
                        Ty::Bool
                    } else {
                        lty
                    },
                    left: Box::new(left_t),
                    right: Box::new(right_t),
                    op,
                })
            }

            Expression::BoolAnd {
                token, left, right, ..
            } => {
                let left_t = self.check_expr(*left)?;
                let right_t = self.check_expr(*right)?;

                let lty = left_t.get_type();
                let rty = right_t.get_type();

                if lty != Ty::Bool {
                    return Err(Self::make_err(
                        Some(&format!("expected `bool` got {lty}")),
                        TypeCheckerErrorKind::TypeMismatch,
                        token,
                    ));
                }

                if rty != Ty::Bool {
                    return Err(Self::make_err(
                        Some(&format!("expected `bool` got {rty}")),
                        TypeCheckerErrorKind::TypeMismatch,
                        token,
                    ));
                }

                Ok(TypedExpression::BoolAnd {
                    token,
                    left: Box::new(left_t),
                    right: Box::new(right_t),
                })
            }

            Expression::BoolOr {
                token, left, right, ..
            } => {
                let left_t = self.check_expr(*left)?;
                let right_t = self.check_expr(*right)?;

                let lty = left_t.get_type();
                let rty = right_t.get_type();

                if lty != Ty::Bool {
                    return Err(Self::make_err(
                        Some(&format!("expected `bool` got {lty}")),
                        TypeCheckerErrorKind::TypeMismatch,
                        token,
                    ));
                }

                if rty != Ty::Bool {
                    return Err(Self::make_err(
                        Some(&format!("expected `bool` got {rty}")),
                        TypeCheckerErrorKind::TypeMismatch,
                        token,
                    ));
                }

                Ok(TypedExpression::BoolOr {
                    token,
                    left: Box::new(left_t),
                    right: Box::new(right_t),
                })
            }

            Expression::If {
                token,
                condition,
                consequence,
                else_block,
            } => {
                let typed_condition = self.check_expr(*condition)?;
                let typed_consequence = self.check_expr(*consequence)?;

                let typed_else_block = match else_block {
                    Some(it) => Some(Box::new(self.check_expr(*it)?)),
                    None => {
                        if self.compile_as == CompileAs::AsValue {
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
                    condition: Box::new(typed_condition),
                    consequence: Box::new(typed_consequence),
                    else_block: typed_else_block,
                })
            }

            Expression::Function {
                token,
                name,
                params,
                block,
                ret_ty: ret_ty_ident,
                generics_params,
            } => {
                for generics_param in generics_params
                    .iter()
                    .filter(|it| matches!(&***it, Expression::Ident(_)))
                {
                    let Expression::Ident(it) = &**generics_param else {
                        unreachable!()
                    };

                    self.table
                        .lock()
                        .unwrap()
                        .define_var(&it.value, Ty::Generic(it.value.clone(), vec![]));
                }

                let mut typed_params: Vec<Box<TypedExpression>> = vec![];

                for param in params {
                    typed_params.push(Box::new(self.check_expr(*param)?))
                }

                let mut params_type = vec![];

                for typed_param in &typed_params {
                    if let TypedExpression::TypeHint(_, _, ty) = &**typed_param {
                        params_type.push(ty.clone());
                    }
                }

                // 初步返回定义
                let ret_ty = ret_ty_ident
                    .as_ref()
                    .map_or_else(
                        || None,
                        |it| {
                            self.table
                                .lock()
                                .unwrap()
                                .get(&it.value)
                                .map_or(None, |it| Some(it.ty.get_type()))
                        },
                    )
                    .map_or(Ty::Unit, |it| it);

                let func_ty = Ty::Function {
                    params_type,
                    ret_type: Box::new(ret_ty.clone()),
                    is_variadic: false,
                };

                if let Some(name) = &name {
                    self.table
                        .lock()
                        .unwrap()
                        .define_var(&name.value, func_ty.clone());
                }

                let typed_block = match *block {
                    Expression::Block(token, statements) => {
                        self.enter_scope(ScopeKind::Function);

                        for typed_param in &typed_params {
                            if let TypedExpression::TypeHint(name, _, ty) = &**typed_param {
                                self.table
                                    .lock()
                                    .unwrap()
                                    .define_var(&name.value, ty.clone());
                            }
                        }

                        let (stmts, scope) =
                            self.check_statements(statements, ScopeKind::Function)?;

                        for (cur_ret_ty, token) in &scope.collect_return_types {
                            if cur_ret_ty == &ret_ty {
                                continue;
                            }

                            return Err(Self::make_err(
                                Some(&format!("expected: {ret_ty}, got: {cur_ret_ty}",)),
                                TypeCheckerErrorKind::TypeMismatch,
                                token.clone(),
                            ));
                        }

                        let ret_ty = ret_ty;

                        TypedExpression::Block(token, stmts, ret_ty)
                    }
                    other => self.check_expr(other)?,
                };

                let ret_ident = ret_ty_ident.map(|it| Ident {
                    token: it.token,
                    value: it.value,
                });

                let ty = Ty::Function {
                    params_type: typed_params.iter().map(|p| p.get_type()).collect(),
                    ret_type: Box::new(ret_ident.as_ref().map_or(Ok(Ty::Unit), |id| {
                        self.table.lock().unwrap().get(&id.value).map_or_else(
                            || {
                                Err(TypeChecker::make_err(
                                    Some(&format!("unknown type: {}", &id.value)),
                                    TypeCheckerErrorKind::TypeNotFound,
                                    id.token.clone(),
                                ))
                            },
                            |it| Ok(it.ty.get_type()),
                        )
                    })?),
                    is_variadic: false,
                };

                if let Some(name) = &name {
                    self.table
                        .lock()
                        .unwrap()
                        .define_var(&name.value, ty.clone());
                }

                let typed_generic_params = generics_params
                    .clone()
                    .into_iter()
                    .map(|it| self.check_expr(*it))
                    .collect::<Result<Vec<TypedExpression>, _>>()?
                    .into_iter()
                    .map(|it| Box::new(it))
                    .collect();

                // 移除之前定义的泛型避免污染全局空间
                generics_params
                    .iter()
                    .filter(|it| matches!(&***it, Expression::Ident(_)))
                    .for_each(|it| {
                        let Expression::Ident(it) = &**it else {
                            unreachable!()
                        };

                        self.table.lock().unwrap().remove(&it.value);
                    });

                Ok(TypedExpression::Function {
                    token,
                    name,
                    params: typed_params,
                    block: Box::new(typed_block),
                    ret_ty: ret_ident,
                    ty,
                    generics_params: typed_generic_params,
                })
            }

            Expression::Call { token, func, args } => {
                let typed_func = self.check_expr(*func)?;
                if !matches!(typed_func.get_type(), Ty::Function { .. }) {
                    return Err(Self::make_err(
                        Some("not a function"),
                        TypeCheckerErrorKind::TypeMismatch,
                        typed_func.token(),
                    ));
                }

                let mut typed_args = vec![];

                for arg in args {
                    typed_args.push(Box::new(self.check_expr(*arg)?));
                }

                Ok(TypedExpression::Call {
                    token,
                    func_ty: typed_func.get_type(),
                    args: typed_args,
                    func: Box::new(typed_func),
                })
            }

            Expression::Block(token, statements) => {
                let mut typed_statements: Vec<TypedStatement> = vec![];

                for s in statements {
                    typed_statements.push(self.check_statement(s)?);
                }

                let ty = typed_statements.last().map_or(Ty::Unit, |s| s.get_type());

                Ok(TypedExpression::Block(token, typed_statements, ty))
            }

            Expression::TypeHint(ident, ty_ident) => {
                let ty = self
                    .table
                    .lock()
                    .unwrap()
                    .get(&ty_ident.value)
                    .map_or_else(
                        || {
                            Err(Self::make_err(
                                None,
                                TypeCheckerErrorKind::TypeNotFound,
                                ty_ident.token.clone(),
                            ))
                        },
                        |it| Ok(it.ty.get_type()),
                    )?;

                let new_ident = Ident {
                    token: ident.token,
                    value: ident.value,
                };

                let new_ty_ident = Ident {
                    token: ty_ident.token,
                    value: ty_ident.value,
                };

                Ok(TypedExpression::TypeHint(new_ident, new_ty_ident, ty))
            }

            // 如果出现此表达式请考虑parser是否损坏
            Expression::ThreeDot(_) => unreachable!(),
        }
    }

    pub fn check_statement(&mut self, stmt: Statement) -> CheckResult<TypedStatement> {
        match stmt {
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

                if self.table.lock().unwrap().get(&new_impl_.value).is_none() {
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
                    && self.table.lock().unwrap().get(&new_for_.value).is_none()
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

                for (_, sym) in &mut self.table.lock().unwrap().var_map {
                    let SymbolType::Variable(Ty::Struct {
                        name: struct_name,
                        fields,
                        ..
                    }) = &mut sym.ty
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
                    block: Box::new(typed_block),
                })
            }
            Statement::FuncDecl {
                token,
                name,
                params,
                ret_ty: ret_ty_ident,
                generics_params,
            } => {
                for generics_param in generics_params
                    .iter()
                    .filter(|it| matches!(&***it, Expression::Ident(_)))
                {
                    let Expression::Ident(it) = &**generics_param else {
                        unreachable!()
                    };

                    self.table
                        .lock()
                        .unwrap()
                        .define_var(&it.value, Ty::Generic(it.value.clone(), vec![]));
                }

                let mut typed_params: Vec<Box<TypedExpression>> = vec![];

                for param in params {
                    typed_params.push(Box::new(self.check_expr(*param)?))
                }

                let mut params_type = vec![];

                for typed_param in &typed_params {
                    if let TypedExpression::TypeHint(_, _, ty) = &**typed_param {
                        params_type.push(ty.clone());
                    }
                }

                // 初步返回定义
                let ret_ty = ret_ty_ident
                    .as_ref()
                    .map_or_else(
                        || None,
                        |it| {
                            self.table
                                .lock()
                                .unwrap()
                                .get(&it.value)
                                .map_or(None, |it| Some(it.ty.get_type()))
                        },
                    )
                    .map_or(Ty::Unit, |it| it);

                let func_ty = Ty::Function {
                    params_type,
                    ret_type: Box::new(ret_ty.clone()),
                    is_variadic: false,
                };

                self.table
                    .lock()
                    .unwrap()
                    .define_var(&name.value, func_ty.clone());

                let ret_ident = ret_ty_ident.map(|it| Ident {
                    token: it.token,
                    value: it.value,
                });

                let ty = Ty::Function {
                    params_type: typed_params.iter().map(|p| p.get_type()).collect(),
                    ret_type: Box::new(ret_ident.as_ref().map_or(Ok(Ty::Unit), |id| {
                        self.table.lock().unwrap().get(&id.value).map_or_else(
                            || {
                                Err(TypeChecker::make_err(
                                    Some(&format!("unknown type: {}", &id.value)),
                                    TypeCheckerErrorKind::TypeNotFound,
                                    id.token.clone(),
                                ))
                            },
                            |it| Ok(it.ty.get_type()),
                        )
                    })?),
                    is_variadic: false,
                };

                self.table
                    .lock()
                    .unwrap()
                    .define_var(&name.value, ty.clone());

                let typed_generic_params = generics_params
                    .clone()
                    .into_iter()
                    .map(|it| self.check_expr(*it))
                    .collect::<Result<Vec<TypedExpression>, _>>()?
                    .into_iter()
                    .map(|it| Box::new(it))
                    .collect();

                // 移除之前定义的泛型避免污染全局空间
                generics_params
                    .iter()
                    .filter(|it| matches!(&***it, Expression::Ident(_)))
                    .for_each(|it| {
                        let Expression::Ident(it) = &**it else {
                            unreachable!()
                        };

                        self.table.lock().unwrap().remove(&it.value);
                    });

                Ok(TypedStatement::FuncDecl {
                    token,
                    name,
                    params: typed_params,
                    ret_ty: ret_ident,
                    ty,
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
                let mut typed_params = vec![];
                let mut params_type = vec![];

                for param in params {
                    let typed_param = self.check_expr(*param)?;

                    params_type.push(typed_param.get_type());
                    typed_params.push(Box::new(typed_param));
                }

                let ret_ty_ident = Ident {
                    value: ret_ty.value,
                    token: ret_ty.token,
                };

                let func_ty = Ty::Function {
                    params_type,
                    ret_type: Box::new(
                        self.table
                            .lock()
                            .unwrap()
                            .get(&ret_ty_ident.value)
                            .map_or_else(
                                || {
                                    Err(TypeChecker::make_err(
                                        Some(&format!("unknown type: {}", &ret_ty_ident.value)),
                                        TypeCheckerErrorKind::TypeNotFound,
                                        ret_ty_ident.token.clone(),
                                    ))
                                },
                                |it| Ok(it.ty.get_type()),
                            )?,
                    ),
                    is_variadic: true,
                };

                self.table
                    .lock()
                    .unwrap()
                    .define_var(&alias.value, func_ty.clone());

                Ok(TypedStatement::Extern {
                    ty: func_ty,
                    token,
                    abi,
                    alias,
                    extern_func_name,
                    params: typed_params,
                    ret_ty: ret_ty_ident,
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

                    self.table
                        .lock()
                        .unwrap()
                        .define_var(&it.value, Ty::Generic(it.value.clone(), vec![]));
                }

                let mut typed_fields = vec![];

                for field in fields {
                    if !matches!(*field, Expression::TypeHint(_, _)) {
                        return Err(Self::make_err(
                            Some(&format!("not a type hint: {field}")),
                            TypeCheckerErrorKind::Other,
                            field.token(),
                        ));
                    }
                    typed_fields.push(self.check_expr(*field)?);
                }

                let ty = Ty::Struct {
                    name: name.value.clone(),
                    fields: {
                        let mut m = IndexMap::new();

                        for field in &typed_fields {
                            if let TypedExpression::TypeHint(name, _, ty) = field {
                                m.insert(name.value.clone(), ty.clone());
                            } else {
                                return Err(Self::make_err(
                                    Some(&format!("not a type hint: {field}")),
                                    TypeCheckerErrorKind::Other,
                                    field.token(),
                                ));
                            }
                        }

                        m
                    },
                    impl_traits: IndexMap::new(),
                };

                self.table
                    .lock()
                    .unwrap()
                    .define_var(&typed_name.value, ty.clone());

                let typed_generics = generics
                    .clone()
                    .into_iter()
                    .map(|it| self.check_expr(*it))
                    .collect::<CheckResult<Vec<TypedExpression>>>()?
                    .into_iter()
                    .map(Box::new)
                    .collect();

                // 移除之前定义的泛型避免污染全局空间
                generics
                    .iter()
                    .filter(|it| matches!(&***it, Expression::Ident(_)))
                    .for_each(|it| {
                        let Expression::Ident(it) = &**it else {
                            unreachable!()
                        };

                        self.table.lock().unwrap().remove(&it.value);
                    });

                Ok(TypedStatement::Struct {
                    ty,
                    token,
                    name: typed_name,
                    fields: typed_fields,
                    generics: typed_generics
                })
            }
            Statement::Trait { token, name, block } => {
                let typed_name = Ident {
                    token: name.token,
                    value: name.value.clone(),
                };

                let mut typed_statements = vec![];

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

                    typed_statements.push(self.check_statement(stmt)?);
                }

                self.leave_scope();

                let ty = Ty::Trait {
                    name: name.value.clone(),
                    functions: {
                        let mut m = IndexMap::new();

                        for func in &typed_statements {
                            if let TypedStatement::FuncDecl { ty, .. } = func {
                                m.insert(name.value.clone(), ty.clone());
                            } else {
                                return Err(Self::make_err(
                                    Some(&format!("not a func decl: {func}")),
                                    TypeCheckerErrorKind::Other,
                                    func.token(),
                                ));
                            }
                        }

                        m
                    },
                };

                self.table
                    .lock()
                    .unwrap()
                    .define_var(&typed_name.value, ty.clone());

                Ok(TypedStatement::Trait {
                    ty,
                    token,
                    name: typed_name,
                    block: Box::new(TypedStatement::Block {
                        token: block_token,
                        statements: typed_statements,
                        ty: Ty::Unit,
                    }),
                })
            }
            Statement::While {
                token,
                condition,
                block,
            } => {
                let typed_condition = self.check_expr(condition)?;

                let typed_block = Box::new(self.check_statement(*block)?);

                Ok(TypedStatement::While {
                    token,
                    condition: typed_condition,
                    block: typed_block,
                })
            }

            Statement::ExpressionStatement(expr) => {
                Ok(TypedStatement::ExpressionStatement(self.check_expr(expr)?))
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
                    match self.table.lock().unwrap().get(&ty_ident.value) {
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

                self.table
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
                    value: typed_val,
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
                    match self.table.lock().unwrap().get(&ty_ident.value) {
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

                self.table
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
                    value: typed_val,
                })
            }

            Statement::Return { token, expr } => {
                let typed_expr = self.check_expr(expr)?;

                let rty = typed_expr.get_type();

                self.current_scope_mut()
                    .collect_return_types
                    .push((rty.clone(), typed_expr.token()));

                Ok(TypedStatement::Return {
                    token,
                    expr: typed_expr,
                    ty: rty,
                })
            }

            Statement::Block { token, statements } => {
                let mut typed_statements = vec![];

                for s in statements {
                    typed_statements.push(self.check_statement(s)?);
                }

                let ty = typed_statements.last().map_or(Ty::Unit, |s| s.get_type());

                Ok(TypedStatement::Block {
                    token,
                    statements: typed_statements,
                    ty,
                })
            }
        }
    }

    pub fn enter_scope(&mut self, kind: ScopeKind) {
        self.table = Arc::new(Mutex::new(TypeTable::with_outer(self.table.clone())));

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

    pub fn leave_scope(&mut self) -> (CheckScope, Arc<Mutex<TypeTable>>) {
        let before_enter_scope_table = self
            .table
            .lock()
            .unwrap()
            .outer
            .clone()
            .expect("expected an outer");

        let cur_scope_table = self.table.clone();

        self.table = before_enter_scope_table;

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
