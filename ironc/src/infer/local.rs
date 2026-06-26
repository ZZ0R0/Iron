//! [20100] Inférence locale : monotypes, structs, collections.

use super::{element_of, Inf};
use crate::ir::*;
use std::collections::HashMap;

type Scope = HashMap<String, Type>;

/// Type du 1ᵉʳ argument déjà inféré (pour `some`/`ok`).
fn arg0_ty(cx: &Inf, args: &[Expr]) -> Type {
    args.first().and_then(|a| cx.tm.get(&a.id).cloned()).unwrap_or(Type::Unit)
}

/// Type de retour d'une méthode built-in (str/list/dict, 80500) ; `None` si inconnue.
fn builtin_method_ret(bt: &Type, name: &str) -> Option<Type> {
    Some(match (bt, name) {
        (Type::Str, "upper" | "lower" | "strip" | "replace") => Type::Str,
        (Type::Str, "split") => Type::List(Box::new(Type::Str)),
        (Type::Str, "startswith" | "endswith" | "contains") => Type::Bool,
        (Type::Str | Type::List(_), "len") => Type::I64,
        (Type::List(_), "append" | "sort" | "reverse") => Type::Unit,
        (Type::List(e), "pop") => Type::Option(e.clone()),
        (Type::List(_) | Type::Set(_), "contains") => Type::Bool,
        (Type::Dict(k, _), "keys") => Type::List(k.clone()),
        (Type::Dict(_, v), "values") => Type::List(v.clone()),
        (Type::Dict(_, v), "get") => Type::Option(v.clone()),
        (Type::Dict(..), "contains") => Type::Bool,
        _ => return None,
    })
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
                // La garde voit les liaisons du motif (20600).
                if let Some(g) = &arm.guard {
                    expr(cx, g, &mut s);
                }
                for st in &arm.body {
                    stmt(cx, st, &mut s);
                }
            }
        }
        StmtKind::Assert { cond, msg } => {
            expr(cx, cond, scope);
            if let Some(m) = msg {
                expr(cx, m, scope);
            }
        }
        StmtKind::Destructure { targets, value } => {
            let t = expr(cx, value, scope);
            for (i, name) in targets.iter().enumerate() {
                let et = match &t {
                    Type::Tuple(ts) => ts.get(i).cloned().unwrap_or(Type::Unit),
                    _ => Type::Unit,
                };
                scope.insert(name.clone(), et);
            }
        }
        StmtKind::TryExcept { body, err_name, handler } => {
            for s in body {
                stmt(cx, s, scope);
            }
            let mut h = scope.clone();
            if let Some(e) = err_name {
                h.insert(e.clone(), Type::Str); // l'erreur capturée = message (str)
            }
            for s in handler {
                stmt(cx, s, &mut h);
            }
        }
        StmtKind::Raise { value } => {
            expr(cx, value, scope);
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
            Lit::Char(_) => Type::Char,
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
                // les fns utilisateur/externes (dans `sigs`) priment sur les built-ins homonymes.
                ExprKind::Var(name) if cx.sigs.contains_key(name) => {
                    cx.sigs.get(name).map(|(_, r)| r.clone()).unwrap_or(Type::Unit)
                }
                ExprKind::Var(name) => match name.as_str() {
                    "print" => Type::Unit,
                    "len" => Type::I64,
                    "range" => Type::List(Box::new(Type::I64)),
                    "some" => Type::Option(Box::new(arg0_ty(cx, args))),
                    "ok" => Type::Result(Box::new(arg0_ty(cx, args))),
                    "err" => Type::Result(Box::new(Type::Unit)),
                    // stdlib V4 (80100)
                    "str" => Type::Str,
                    "sqrt" => Type::F64,
                    "abs" | "min" | "max" => arg0_ty(cx, args),
                    "push" => Type::Unit,
                    // stdlib élargie V6 (80200)
                    "sum" => match args.first().and_then(|a| cx.tm.get(&a.id)) {
                        Some(Type::List(e)) => (**e).clone(),
                        _ => Type::I64,
                    },
                    "concat" => match args.first().and_then(|a| cx.tm.get(&a.id)) {
                        Some(Type::List(e)) => Type::List(e.clone()),
                        _ => Type::Str,
                    },
                    "sort" => arg0_ty(cx, args),
                    "input" | "upper" | "lower" => Type::Str,
                    "split" => Type::List(Box::new(Type::Str)),
                    "index_of" => Type::I64,
                    // partage explicite (30300, I5)
                    "ref" => Type::Shared(Box::new(arg0_ty(cx, args))),
                    // ensembles (20900)
                    "set" => match args.first().and_then(|a| cx.tm.get(&a.id)) {
                        Some(Type::List(e)) => Type::Set(e.clone()),
                        _ => Type::Set(Box::new(Type::Unit)),
                    },
                    "add" => Type::Unit,
                    "union" | "inter" | "diff" => arg0_ty(cx, args),
                    // casts numériques `to_<t>(x)` (20800)
                    cast if cast.strip_prefix("to_").and_then(numeric_type_by_name).is_some() => {
                        numeric_type_by_name(&cast[3..]).unwrap()
                    }
                    // map/filter (20700) : best-effort = type de la liste source (args[1]).
                    "map" | "filter" => args.get(1).and_then(|a| cx.tm.get(&a.id)).cloned().unwrap_or(Type::Unit),
                    _ => cx.sigs.get(name).map(|(_, r)| r.clone()).unwrap_or(Type::Unit),
                },
                _ => {
                    if let ExprKind::Field { base, name } = &callee.kind {
                        // module stdlib `io` (80300) : types de retour (faillibles → Result).
                        if matches!(&base.kind, ExprKind::Var(m) if m == "io") {
                            let ret = match name.as_str() {
                                "read_file" => Type::Result(Box::new(Type::Str)),
                                "write_file" | "append_file" => Type::Result(Box::new(Type::Unit)),
                                "read_lines" => Type::Result(Box::new(Type::List(Box::new(Type::Str)))),
                                "exists" => Type::Bool,
                                _ => Type::Unit,
                            };
                            for a in args {
                                expr(cx, a, scope);
                            }
                            return cx.set(e.id, ret);
                        }
                        let bt = expr(cx, base, scope);
                        // construction de variante d'enum : `Enum.Variant(...)`
                        if let ExprKind::Var(ename) = &base.kind {
                            if cx.enums.contains_key(ename) {
                                return cx.set(e.id, Type::Enum(ename.clone()));
                            }
                        }
                        // méthodes des types built-in (str/list/dict, 80500)
                        if let Some(ret) = builtin_method_ret(&bt, name) {
                            return cx.set(e.id, ret);
                        }
                        // appel de méthode : type de retour de l'`impl` (V5/DX3)
                        if let Type::Struct(tn) = &bt {
                            if let Some(ret) = cx.methods.get(&(tn.clone(), name.clone())) {
                                return cx.set(e.id, ret.clone());
                            }
                        }
                        Type::Unit
                    } else {
                        expr(cx, callee, scope);
                        Type::Unit
                    }
                }
            }
        }
        ExprKind::StructLit { name, fields } => {
            for (_, v) in fields {
                expr(cx, v, scope);
            }
            // `Name(champ=val)` : appel à arguments nommés si `Name` est une fn → son type de retour.
            match cx.sigs.get(name) {
                Some((_, ret)) => ret.clone(),
                None => Type::Struct(name.clone()),
            }
        }
        ExprKind::Field { base, name } => {
            let bt = expr(cx, base, scope);
            match bt {
                Type::Struct(s) => cx.field_type(&s, name),
                // accès de champ à travers un `shared[Struct]` (30300) → borrow() au lowering.
                Type::Shared(inner) => match &*inner {
                    Type::Struct(s) => cx.field_type(s, name),
                    _ => Type::Unit,
                },
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
        ExprKind::Lambda { params, body } => {
            // Types des params laissés à l'inférence Rust (I2) ; on annote quand même le corps.
            let mut inner = scope.clone();
            for p in params {
                inner.insert(p.clone(), Type::Unit);
            }
            let bt = expr(cx, body, &mut inner);
            Type::Fn(params.iter().map(|_| Type::Unit).collect(), Box::new(bt))
        }
        ExprKind::Tuple(items) => {
            let ts: Vec<Type> = items.iter().map(|it| expr(cx, it, scope)).collect();
            Type::Tuple(ts)
        }
        ExprKind::FString(parts) => {
            for p in parts {
                if let FStringPart::Expr(e) = p {
                    expr(cx, e, scope);
                }
            }
            Type::Str
        }
        ExprKind::Comp { kind, body, var, iter, cond } => {
            // 2.7 : lier la variable de boucle depuis le type d'élément de l'itérable.
            let it = expr(cx, iter, scope);
            let elem = super::element_of(&it);
            let mut inner = scope.clone();
            cx.bind_pattern(var, &elem, &mut inner);
            let bt = expr(cx, body, &inner);
            if let Some(c) = cond {
                expr(cx, c, &inner);
            }
            match kind {
                CompKind::List => Type::List(Box::new(bt)),
                CompKind::Set => Type::Set(Box::new(bt)),
                CompKind::Dict => match bt {
                    Type::Tuple(ts) if ts.len() == 2 => {
                        Type::Dict(Box::new(ts[0].clone()), Box::new(ts[1].clone()))
                    }
                    _ => Type::Dict(Box::new(Type::Unit), Box::new(Type::Unit)),
                },
            }
        }
        ExprKind::Ternary { cond, then, els } => {
            expr(cx, cond, scope);
            let t = expr(cx, then, scope);
            expr(cx, els, scope);
            t
        }
        ExprKind::Slice { base, lo, hi } => {
            let b = expr(cx, base, scope);
            if let Some(l) = lo {
                expr(cx, l, scope);
            }
            if let Some(h) = hi {
                expr(cx, h, scope);
            }
            b
        }
    };
    cx.set(e.id, ty)
}
