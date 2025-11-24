pub mod scope;
pub mod test;
use std::{cell::RefCell, fmt::Display, rc::Rc};

use ast::{
    expr::{Expression, IntValue},
    node::Node,
    stmt::Statement,
};
use token::token::Token;

use crate::{
    error::{TypeCheckerError, TypeCheckerErrorKind},
    scope::{CheckScope, ScopeKind},
    table::{TypeTable, str_to_ty},
    typed_ast::{
        GetType, typed_expr::TypedExpression, typed_expressions::ident::Ident,
        typed_node::TypedNode, typed_stmt::TypedStatement,
    },
};

pub mod error;
pub mod table;
pub mod typed_ast;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntTy {
    ISize,
    I64,
    I32,
    I16,
    I8,
    USize,
    U64,
    U32,
    U16,
    U8,
}

impl Display for IntTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::I64 => "i64",
            Self::I32 => "i32",
            Self::I16 => "i16",
            Self::I8 => "i8",
            Self::U64 => "u64",
            Self::U32 => "u32",
            Self::U16 => "u16",
            Self::U8 => "u8",
            Self::ISize => "isize",
            Self::USize => "usize",
        };

        write!(f, "{s}")
    }
}

impl From<IntValue> for IntTy {
    fn from(value: IntValue) -> Self {
        match value {
            IntValue::ISize(_) => Self::ISize,
            IntValue::I64(_) => Self::I64,
            IntValue::I32(_) => Self::I32,
            IntValue::I16(_) => Self::I16,
            IntValue::I8(_) => Self::I8,
            IntValue::USize(_) => Self::USize,
            IntValue::U64(_) => Self::U64,
            IntValue::U32(_) => Self::U32,
            IntValue::U16(_) => Self::U16,
            IntValue::U8(_) => Self::U8,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ty {
    BigInt,
    Function {
        params_type: Vec<Ty>,
        ret_type: Box<Ty>,
    },
    IntTy(IntTy),
    Bool,
    Unit,
}

impl Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BigInt => write!(f, "BigInt"),
            Self::IntTy(it) => write!(f, "{it}"),
            Self::Bool => write!(f, "bool"),
            Self::Unit => write!(f, "Unit"),
            Self::Function {
                params_type,
                ret_type,
            } => write!(
                f,
                "Func({}) -> {}",
                params_type
                    .iter()
                    .map(|it| it.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
                ret_type
            ),
        }
    }
}

type CheckResult<T> = Result<T, TypeCheckerError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompileAs {
    AsValue,
    AsNone,
}

pub struct TypeChecker {
    table: Rc<RefCell<TypeTable>>,

    scopes: Vec<CheckScope>,
    scope_index: usize,

    compile_as: CompileAs,
}

impl TypeChecker {
    pub fn new(table: Rc<RefCell<TypeTable>>) -> Self {
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
            self.current_scope_mut().collect_return_types.push(Ty::Unit);
            return Ok((typed_statements, self.leave_scope()));
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
                        .push(r.get_type());

                    r
                } else {
                    self.check_statement(stmt)?
                },
            );
        }

        Ok((typed_statements, self.leave_scope()))
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

            Expression::Ident(it) => {
                let ident_name = &it.value;

                match self.table.borrow().get(&ident_name) {
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
                        Some(it.token),
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
                        Some(token),
                    ));
                }

                Ok(TypedExpression::Infix {
                    token,
                    left: Box::new(left_t),
                    right: Box::new(right_t),
                    op,
                    ty: lty,
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
                                Some(token),
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
            } => {
                let mut typed_params: Vec<Box<TypedExpression>> = vec![];

                for expr in params {
                    typed_params.push(Box::new(self.check_expr(*expr)?))
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
                    .map_or_else(|| None, |it| str_to_ty(&it.value))
                    .map_or(Ty::Unit, |it| it);

                let func_ty = Ty::Function {
                    params_type,
                    ret_type: Box::new(ret_ty.clone()),
                };

                if let Some(name) = &name {
                    self.table.borrow_mut().define_var(&name.value, func_ty.clone());
                }

                let typed_block = match *block {
                    Statement::Block {
                        token: block_token,
                        statements,
                    } => {
                        self.enter_scope(ScopeKind::Function);

                        for typed_param in &typed_params {
                            if let TypedExpression::TypeHint(name, _, ty) = &**typed_param {
                                self.table.borrow_mut().define_var(&name.value, ty.clone());
                            }
                        }

                        let (stmts, scope) =
                            self.check_statements(statements, ScopeKind::Function)?;

                        for cur_ret_ty in &scope.collect_return_types {
                            if cur_ret_ty == &ret_ty {
                                continue;
                            }

                            return Err(Self::make_err(
                                Some(&format!("expected: {ret_ty}, got: {cur_ret_ty}",)),
                                TypeCheckerErrorKind::TypeMismatch,
                                None,
                            ));
                        }

                        let ret_ty = ret_ty;

                        TypedStatement::Block {
                            token: block_token,
                            statements: stmts,
                            ty: ret_ty,
                        }
                    }
                    other => self.check_statement(other)?,
                };

                let ret_ident = ret_ty_ident.map(|it| Ident {
                    token: it.token,
                    value: it.value,
                });

                let ty = Ty::Function {
                    params_type: typed_params.iter().map(|p| p.get_type()).collect(),
                    ret_type: Box::new(
                        ret_ident
                            .as_ref()
                            .map_or(Ty::Unit, |id| str_to_ty(&id.value).unwrap_or(Ty::BigInt)),
                    ),
                };

                if let Some(name) = &name {
                    self.table.borrow_mut().define_var(&name.value, ty.clone());
                }

                Ok(TypedExpression::Function {
                    token,
                    name,
                    params: typed_params,
                    block: Box::new(typed_block),
                    ret_ty: ret_ident,
                    ty,
                })
            }

            Expression::Call { token, func, args } => {
                let typed_func = self.check_expr(*func)?;
                if !matches!(typed_func.get_type(), Ty::Function { .. }) {
                    return Err(Self::make_err(
                        Some("not a function"),
                        TypeCheckerErrorKind::TypeMismatch,
                        None,
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

            Expression::Block(statements) => {
                let mut typed_statements: Vec<TypedStatement> = vec![];

                for s in statements {
                    typed_statements.push(self.check_statement(s)?);
                }

                let ty = typed_statements.last().map_or(Ty::Unit, |s| s.get_type());

                Ok(TypedExpression::Block(typed_statements, ty))
            }

            Expression::TypeHint(ident, ty_ident) => {
                let ty = str_to_ty(&ty_ident.value).map_or_else(
                    || {
                        Err(Self::make_err(
                            None,
                            TypeCheckerErrorKind::TypeNotFound,
                            Some(ty_ident.token.clone()),
                        ))
                    },
                    |it| Ok(it),
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
        }
    }

    pub fn check_statement(&mut self, stmt: Statement) -> CheckResult<TypedStatement> {
        match stmt {
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
                    match str_to_ty(&ty_ident.value) {
                        Some(it) => it,
                        None => {
                            return Err(Self::make_err(
                                None,
                                TypeCheckerErrorKind::TypeNotFound,
                                Some(ty_ident.token.clone()),
                            ));
                        }
                    }
                } else {
                    typed_val.get_type()
                };

                self.table.borrow_mut().define_var(&name.value, ty.clone());

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

            Statement::Return { token, expr } => {
                let typed_expr = self.check_expr(expr)?;

                let rty = typed_expr.get_type();

                self.current_scope_mut()
                    .collect_return_types
                    .push(rty.clone());

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
        self.table = Rc::new(RefCell::new(TypeTable::with_outer(self.table.clone())));

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

    pub fn leave_scope(&mut self) -> CheckScope {
        let outer = self
            .table
            .borrow()
            .outer
            .clone()
            .expect("expected an outer");

        self.table = outer;

        self.scope_index -= 1;

        self.scopes.pop().unwrap()
    }

    pub fn make_err(
        message: Option<&str>,
        kind: TypeCheckerErrorKind,
        token: Option<Token>,
    ) -> TypeCheckerError {
        TypeCheckerError {
            kind,
            token,
            message: message.map_or_else(|| None, |it| Some(it.into())),
        }
    }
}
