//! [20100] Inférence locale : monotypes, structs, collections.

use super::{element_of, Inf};
use crate::ir::*;
use std::collections::HashMap;

type Scope = HashMap<String, Type>;

/// Type du 1ᵉʳ argument déjà inféré (pour `some`/`ok`).
fn arg0_ty(cx: &Inf, args: &[Expr]) -> Type {
    args.first().and_then(|a| cx.tm.get(&a.id).cloned()).unwrap_or(Type::Unit)
}

pub(crate) fn stmt(cx: &mut Inf, s: &Stmt, scope: &mut Scope) {
    match &s.kind {
        StmtKind::Let { name, value } => {
            let t = expr(cx, value, scope);
            scope.insert(name.clone(), t);
        }
        StmtKind::Assign { target, value } => {
            expr(cx, target, scope);
            expr(cx, value, scope);
        }
        StmtKind::If { cond, then, elifs, els } => {
            expr(cx, cond, scope);
            for s in then {
                stmt(cx, s, scope);
            }
            for (c, b) in elifs {
                expr(cx, c, scope);
                for s in b {
                    stmt(cx, s, scope);
                }
            }
            for s in els {
                stmt(cx, s, scope);
            }
        }
        StmtKind::While { cond, body } => {
            expr(cx, cond, scope);
            for s in body {
                stmt(cx, s, scope);
            }
        }
        StmtKind::For { var, iter, body } => {
            let it = expr(cx, iter, scope);
            scope.insert(var.clone(), element_of(&it));
            for s in body {
                stmt(cx, s, scope);
            }
        }
        StmtKind::Return { value } => {
            if let Some(e) = value {
                expr(cx, e, scope);
            }
        }
        StmtKind::ExprStmt(e) => {
            expr(cx, e, scope);
        }
        StmtKind::Match { scrutinee, arms } => {
            let sty = expr(cx, scrutinee, scope);
            for arm in arms {
                let mut s = scope.clone();
                cx.bind_pattern(&arm.pattern, &sty, &mut s);
                for st in &arm.body {
                    stmt(cx, st, &mut s);
                }
            }
        }
        StmtKind::Break | StmtKind::Continue | StmtKind::Pass | StmtKind::RustBlock(_) => {}
    }
}

pub(crate) fn expr(cx: &mut Inf, e: &Expr, scope: &Scope) -> Type {
    let ty = match &e.kind {
        ExprKind::Lit(l) => match l {
            Lit::Int(_) => Type::I64,
            Lit::Float(_) => Type::F64,
            Lit::Bool(_) => Type::Bool,
            Lit::Str(_) => Type::Str,
        },
        ExprKind::Var(n) => {
            if n == "none" {
                Type::Option(Box::new(Type::Unit))
            } else {
                scope.get(n).cloned().unwrap_or(Type::Unit)
            }
        }
        ExprKind::Unary { op, expr: inner } => {
            let t = expr(cx, inner, scope);
            match op {
                UnOp::Neg => t,
                UnOp::Not => Type::Bool,
            }
        }
        ExprKind::Binary { op, lhs, rhs } => {
            let lt = expr(cx, lhs, scope);
            let _rt = expr(cx, rhs, scope);
            match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => lt,
                _ => Type::Bool,
            }
        }
        ExprKind::Call { callee, args } => {
            for a in args {
                expr(cx, a, scope);
            }
            match &callee.kind {
                ExprKind::Var(name) => match name.as_str() {
                    "print" => Type::Unit,
                    "len" => Type::I64,
                    "range" => Type::List(Box::new(Type::I64)),
                    "some" => Type::Option(Box::new(arg0_ty(cx, args))),
                    "ok" => Type::Result(Box::new(arg0_ty(cx, args))),
                    "err" => Type::Result(Box::new(Type::Unit)),
                    _ => cx.sigs.get(name).map(|(_, r)| r.clone()).unwrap_or(Type::Unit),
                },
                _ => {
                    // construction de variante d'enum : `Enum.Variant(...)`
                    if let ExprKind::Field { base, .. } = &callee.kind {
                        if let ExprKind::Var(ename) = &base.kind {
                            if cx.enums.contains_key(ename) {
                                return cx.set(e.id, Type::Enum(ename.clone()));
                            }
                        }
                    }
                    expr(cx, callee, scope);
                    Type::Unit
                }
            }
        }
        ExprKind::StructLit { name, fields } => {
            for (_, v) in fields {
                expr(cx, v, scope);
            }
            Type::Struct(name.clone())
        }
        ExprKind::Field { base, name } => {
            let bt = expr(cx, base, scope);
            match bt {
                Type::Struct(s) => cx.field_type(&s, name),
                _ => Type::Unit,
            }
        }
        ExprKind::Index { base, idx } => {
            let bt = expr(cx, base, scope);
            expr(cx, idx, scope);
            match bt {
                Type::List(e) => *e,
                Type::Dict(_, v) => *v,
                Type::Str => Type::Str,
                _ => Type::Unit,
            }
        }
        ExprKind::ListLit { items } => {
            let mut elem = Type::Unit;
            for (i, it) in items.iter().enumerate() {
                let t = expr(cx, it, scope);
                if i == 0 {
                    elem = t;
                }
            }
            Type::List(Box::new(elem))
        }
        ExprKind::DictLit { pairs } => {
            let mut kt = Type::Unit;
            let mut vt = Type::Unit;
            for (i, (k, v)) in pairs.iter().enumerate() {
                let k = expr(cx, k, scope);
                let v = expr(cx, v, scope);
                if i == 0 {
                    kt = k;
                    vt = v;
                }
            }
            Type::Dict(Box::new(kt), Box::new(vt))
        }
        ExprKind::Try { inner } => {
            let t = expr(cx, inner, scope);
            match t {
                Type::Option(x) => *x,
                Type::Result(x) => *x,
                other => other,
            }
        }
    };
    cx.set(e.id, ty)
}
