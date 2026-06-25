//! [20500] Inférence des types de paramètres non annotés (pré-passe, V6/3.1).
//!
//! Un param sans annotation porte la sentinelle `Type::Generic(INFER_SENTINEL)`. Cette passe la résout
//! AVANT le reste du pipeline : usage **numérique** (opérande d'une op arithmétique) → `i64` ; sinon →
//! **générique implicite** ajouté aux génériques de la fn (rustc tranche, I2). L'annotation explicite
//! prime (pas de sentinelle). Le retour non annoté (`Unit`) + `return <valeur>` est déduit du 1er return.

use crate::ir::*;

pub(crate) fn resolve_param_types(ast: &mut Ast) {
    for it in &mut ast.items {
        match &mut it.kind {
            ItemKind::Fn { generics, params, ret, body, defaults, .. } => {
                resolve_fn_params(generics, params, defaults, body);
                if matches!(ret, Type::Unit) {
                    if let Some(rt) = first_return_type(params, body) {
                        *ret = rt;
                    }
                }
            }
            // Méthodes (P.1 : params optionnels) : pas de liste de génériques → défaut littéral / numérique.
            ItemKind::Impl { methods, .. } | ItemKind::Trait { methods, .. } => {
                for m in methods.iter_mut() {
                    resolve_method_params(&mut m.params, &m.defaults, m.body.as_deref());
                    if matches!(m.ret, Type::Unit) {
                        if let Some(b) = &m.body {
                            if let Some(rt) = first_return_type(&m.params, b) {
                                m.ret = rt;
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Un param non annoté utilisé en **affichage** (f-string ou `print`) → `str` plutôt qu'un générique
/// non borné (un `T` libre n'implémente pas `Display`). Heuristique Python-friendly.
fn used_in_display(name: &str, body: &[Stmt]) -> bool {
    fn in_expr(name: &str, e: &Expr) -> bool {
        let direct = |x: &Expr| matches!(&x.kind, ExprKind::Var(n) if n == name);
        match &e.kind {
            ExprKind::FString(parts) => {
                parts.iter().any(|p| matches!(p, FStringPart::Expr(i) if direct(i) || in_expr(name, i)))
            }
            ExprKind::Call { callee, args } => {
                let is_print = matches!(&callee.kind, ExprKind::Var(n) if n == "print");
                (is_print && args.iter().any(direct)) || args.iter().any(|a| in_expr(name, a))
            }
            ExprKind::Binary { lhs, rhs, .. } => in_expr(name, lhs) || in_expr(name, rhs),
            ExprKind::Unary { expr, .. } => in_expr(name, expr),
            ExprKind::Field { base, .. } => in_expr(name, base),
            ExprKind::Index { base, idx } => in_expr(name, base) || in_expr(name, idx),
            _ => false,
        }
    }
    fn in_stmt(name: &str, s: &Stmt) -> bool {
        match &s.kind {
            StmtKind::Let { value, .. } | StmtKind::ExprStmt(value) | StmtKind::Assign { value, .. } => {
                in_expr(name, value)
            }
            StmtKind::Return { value: Some(v) } => in_expr(name, v),
            StmtKind::If { then, elifs, els, .. } => {
                used_in_display(name, then)
                    || elifs.iter().any(|(_, b)| used_in_display(name, b))
                    || used_in_display(name, els)
            }
            StmtKind::While { body, .. } | StmtKind::For { body, .. } => used_in_display(name, body),
            StmtKind::Match { arms, .. } => arms.iter().any(|a| used_in_display(name, &a.body)),
            _ => false,
        }
    }
    body.iter().any(|s| in_stmt(name, s))
}

/// Type d'un défaut littéral (`greeting="Hi"` → str, `by=1` → i64) — surface Python.
fn default_literal_type(default: Option<&Expr>) -> Option<Type> {
    match default.map(|e| &e.kind) {
        Some(ExprKind::Lit(Lit::Int(_))) => Some(Type::I64),
        Some(ExprKind::Lit(Lit::Float(_))) => Some(Type::F64),
        Some(ExprKind::Lit(Lit::Str(_))) => Some(Type::Str),
        Some(ExprKind::Lit(Lit::Bool(_))) => Some(Type::Bool),
        Some(ExprKind::Lit(Lit::Char(_))) => Some(Type::Char),
        _ => None,
    }
}

fn resolve_fn_params(generics: &mut Generics, params: &mut [(String, Type)], defaults: &[Option<Expr>], body: &[Stmt]) {
    for idx in 0..params.len() {
        let (is_sentinel, inout) = sentinel_of(&params[idx].1);
        if !is_sentinel {
            continue;
        }
        let pname = params[idx].0.clone();
        let resolved = default_literal_type(defaults.get(idx).and_then(|o| o.as_ref()))
            .or_else(|| used_numerically(&pname, body).then_some(Type::I64))
            .or_else(|| used_in_display(&pname, body).then_some(Type::Str))
            .unwrap_or_else(|| {
                let g = fresh_generic(generics);
                generics.push((g.clone(), Vec::new()));
                Type::Generic(g)
            });
        params[idx].1 = if inout { Type::Ref(Box::new(resolved)) } else { resolved };
    }
}

fn resolve_method_params(params: &mut [(String, Type)], defaults: &[Option<Expr>], body: Option<&[Stmt]>) {
    for idx in 0..params.len() {
        let (is_sentinel, inout) = sentinel_of(&params[idx].1);
        if !is_sentinel {
            continue;
        }
        let pname = params[idx].0.clone();
        // pas de génériques sur les méthodes → défaut littéral, sinon usage numérique → i64, sinon Unit (filet rustc).
        let resolved = default_literal_type(defaults.get(idx).and_then(|o| o.as_ref()))
            .or_else(|| body.map_or(false, |b| used_numerically(&pname, b)).then_some(Type::I64))
            .or_else(|| body.map_or(false, |b| used_in_display(&pname, b)).then_some(Type::Str))
            .unwrap_or(Type::Unit);
        params[idx].1 = if inout { Type::Ref(Box::new(resolved)) } else { resolved };
    }
}

/// `(sentinelle ?, inout ?)` pour un type de param.
fn sentinel_of(t: &Type) -> (bool, bool) {
    match t {
        Type::Generic(n) if n == INFER_SENTINEL => (true, false),
        Type::Ref(inner) => match &**inner {
            Type::Generic(n) if n == INFER_SENTINEL => (true, true),
            _ => (false, false),
        },
        _ => (false, false),
    }
}

/// Nom de générique frais, sans collision avec ceux déjà présents.
fn fresh_generic(generics: &Generics) -> String {
    for c in 'T'..='Z' {
        let cand = c.to_string();
        if !generics.iter().any(|(n, _)| n == &cand) {
            return cand;
        }
    }
    let mut k = 0;
    loop {
        let cand = format!("P{k}");
        if !generics.iter().any(|(n, _)| n == &cand) {
            return cand;
        }
        k += 1;
    }
}

// ---- détection d'usage numérique ----

fn used_numerically(name: &str, body: &[Stmt]) -> bool {
    body.iter().any(|s| stmt_num(name, s))
}

fn stmt_num(name: &str, s: &Stmt) -> bool {
    match &s.kind {
        StmtKind::Let { value, .. } | StmtKind::ExprStmt(value) => expr_num(name, value),
        StmtKind::Assign { target, value } => expr_num(name, target) || expr_num(name, value),
        StmtKind::Return { value } => value.as_ref().map_or(false, |v| expr_num(name, v)),
        StmtKind::If { cond, then, elifs, els } => {
            expr_num(name, cond)
                || used_numerically(name, then)
                || elifs.iter().any(|(c, b)| expr_num(name, c) || used_numerically(name, b))
                || used_numerically(name, els)
        }
        StmtKind::While { cond, body } => expr_num(name, cond) || used_numerically(name, body),
        StmtKind::For { iter, body, .. } => expr_num(name, iter) || used_numerically(name, body),
        StmtKind::Match { scrutinee, arms } => {
            expr_num(name, scrutinee) || arms.iter().any(|a| used_numerically(name, &a.body))
        }
        _ => false,
    }
}

/// `name` est-il opérande **direct** d'une opération arithmétique (ou comparé à un littéral entier) ?
fn expr_num(name: &str, e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Binary { op, lhs, rhs } => {
            let arith = matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem);
            let direct = |x: &Expr| matches!(&x.kind, ExprKind::Var(n) if n == name);
            (arith && (direct(lhs) || direct(rhs))) || expr_num(name, lhs) || expr_num(name, rhs)
        }
        ExprKind::Unary { expr, .. } => expr_num(name, expr),
        ExprKind::Call { args, .. } => args.iter().any(|a| expr_num(name, a)),
        ExprKind::Index { base, idx } => expr_num(name, base) || expr_num(name, idx),
        ExprKind::Field { base, .. } => expr_num(name, base),
        ExprKind::ListLit { items } => items.iter().any(|x| expr_num(name, x)),
        _ => false,
    }
}

// ---- inférence légère du type de retour ----

fn first_return_type(params: &[(String, Type)], body: &[Stmt]) -> Option<Type> {
    for s in body {
        if let Some(t) = stmt_ret(params, s) {
            return Some(t);
        }
    }
    None
}

fn stmt_ret(params: &[(String, Type)], s: &Stmt) -> Option<Type> {
    match &s.kind {
        StmtKind::Return { value } => value.as_ref().and_then(|v| ret_expr_type(params, v)),
        StmtKind::If { then, elifs, els, .. } => first_return_type(params, then)
            .or_else(|| elifs.iter().find_map(|(_, b)| first_return_type(params, b)))
            .or_else(|| first_return_type(params, els)),
        StmtKind::While { body, .. } | StmtKind::For { body, .. } => first_return_type(params, body),
        StmtKind::Match { arms, .. } => arms.iter().find_map(|a| first_return_type(params, &a.body)),
        _ => None,
    }
}

/// Type d'une expression de retour : param → son type ; arithmétique → `i64`.
fn ret_expr_type(params: &[(String, Type)], e: &Expr) -> Option<Type> {
    match &e.kind {
        ExprKind::Var(n) => params.iter().find(|(p, _)| p == n).map(|(_, t)| t.clone()),
        ExprKind::Binary { op, .. } if matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem) => {
            Some(Type::I64)
        }
        _ => None,
    }
}
