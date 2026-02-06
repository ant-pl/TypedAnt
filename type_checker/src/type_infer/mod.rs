pub mod constraint;
pub mod infer_context;
use token::token::Token;

use crate::CheckResult;
use crate::error::{TypeCheckerError, TypeCheckerErrorKind};
use crate::ty::{Ty, TyId};
use crate::ty_context::TypeContext;
use crate::type_infer::constraint::Constraint;
use crate::type_infer::infer_context::InferContext;

pub struct TypeInfer<'a> {
    infer_ctx: &'a mut InferContext<'a>,
}

impl<'a> TypeInfer<'a> {
    pub fn new(infer_ctx: &'a mut InferContext<'a>) -> Self {
        Self { infer_ctx }
    }

    fn tcx(&mut self) -> &mut TypeContext {
        self.infer_ctx.tcx
    }

    fn tcx_ref(&self) -> &TypeContext {
        self.infer_ctx.tcx
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
                    ..
                },
                Ty::Function {
                    params_type: p2,
                    ret_type: r2,
                    ..
                },
            ) => {
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
}

impl<'a> TypeInfer<'a> {
    /// 将最终结果注入 TypeContext，彻底抹除占位符
    pub fn finalize(&mut self) {
        // 拿一波新类型和原类型表
        let subs = &self.infer_ctx.substitutions;
        let tcx = &mut self.infer_ctx.tcx;

        for i in 0..tcx.types.len() {
            let mut current_id = i;
            
            // 追踪这个坑位最终指向谁
            // 这里复用 follow 逻辑 (为这里单独写个不依赖 self 的函数不值得也没必要)
            while let Ty::Infer(infer_id) = &tcx.types[current_id] {
                if let Some(target_id) = subs.get(infer_id) {
                    current_id = *target_id;
                } else {
                    break;
                }
            }

            // 3. 如果发现最终指向的不是自己，说明这是一个推导出来的变量
            if current_id != i {
                // 强制暴力修改
                tcx.types[i] = tcx.types[current_id].clone();
            }
        }
        
        // 执行到这里，tcx.types 里大部分的 Ty::Infer 余孽只要九族有记录，
        // 就全都被杀完了 (成了具体的类型 (如 i64, bool 等))
    }
}