//! [20200 / 20400] Inférence Option/Result depuis le flux (I11, V6/2.1).
//!
//! `effective_ret` déduit le type de retour réel d'une fonction : une fn qui **chute** (ne retourne pas
//! sur tous les chemins) + a un `return <valeur>` → `Option<T>` ; une fn utilisant `?` → `Result<T>`.
//! L'annotation explicite `Option[T]`/`Result[T]` prime. `definitely_returns` doit classer un `match`
//! exhaustif tous-bras-`return` comme retournant (sinon régression du corpus).

use crate::ir::*;

/// Type de retour effectif (I11).
pub(crate) fn effective_ret(declared: &Type, body: &[Stmt]) -> Type {
    // Annotation explicite Option/Result (y compris `Result[T, E]` typé, 21500) : prime.
    if matches!(declared, Type::Option(_) | Type::Result(_) | Type::ResultE(..)) {
        return declared.clone();
    }
    // Sans type concret annoté, pas d'inférence (T inconnu) — `rustc` filet.
    if matches!(declared, Type::Unit) {
        return declared.clone();
    }
    if uses_question(body) {
        return Type::Result(Box::new(declared.clone()));
    }
    if !definitely_returns(body) && has_value_return(body) {
        return Type::Option(Box::new(declared.clone()));
    }
    declared.clone()
}

/// Le bloc retourne-t-il sur **tous** les chemins ?
pub(crate) fn definitely_returns(stmts: &[Stmt]) -> bool {
    stmts.iter().any(stmt_returns)
}

fn stmt_returns(s: &Stmt) -> bool {
    match &s.kind {
        StmtKind::Return { .. } => true,
        StmtKind::If { then, elifs, els, .. } => {
            !els.is_empty()
                && definitely_returns(then)
                && elifs.iter().all(|(_, b)| definitely_returns(b))
                && definitely_returns(els)
        }
        StmtKind::Match { arms, .. } => !arms.is_empty() && arms.iter().all(|a| definitely_returns(&a.body)),
        _ => false,
    }
}

fn has_value_return(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|s| match &s.kind {
        StmtKind::Return { value } => value.is_some(),
        StmtKind::If { then, elifs, els, .. } => {
            has_value_return(then) || elifs.iter().any(|(_, b)| has_value_return(b)) || has_value_return(els)
        }
        StmtKind::While { body, .. } | StmtKind::For { body, .. } => has_value_return(body),
        StmtKind::Match { arms, .. } => arms.iter().any(|a| has_value_return(&a.body)),
        _ => false,
    })
}

fn uses_question(stmts: &[Stmt]) -> bool {
    stmts.iter().any(stmt_uses_q)
}

fn stmt_uses_q(s: &Stmt) -> bool {
    match &s.kind {
        StmtKind::Let { value, .. } | StmtKind::ExprStmt(value) => expr_uses_q(value),
        StmtKind::Assign { target, value } => expr_uses_q(target) || expr_uses_q(value),
        StmtKind::Return { value } => value.as_ref().map_or(false, expr_uses_q),
        StmtKind::If { cond, then, elifs, els } => {
            expr_uses_q(cond)
                || uses_question(then)
                || elifs.iter().any(|(c, b)| expr_uses_q(c) || uses_question(b))
                || uses_question(els)
        }
        StmtKind::While { cond, body } => expr_uses_q(cond) || uses_question(body),
        StmtKind::For { iter, body, .. } => expr_uses_q(iter) || uses_question(body),
        StmtKind::Match { scrutinee, arms } => expr_uses_q(scrutinee) || arms.iter().any(|a| uses_question(&a.body)),
        _ => false,
    }
}

fn expr_uses_q(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Try { .. } => true,
        ExprKind::Unary { expr, .. } => expr_uses_q(expr),
        ExprKind::Binary { lhs, rhs, .. } => expr_uses_q(lhs) || expr_uses_q(rhs),
        ExprKind::Call { callee, args } => expr_uses_q(callee) || args.iter().any(expr_uses_q),
        ExprKind::Field { base, .. } => expr_uses_q(base),
        ExprKind::Index { base, idx } => expr_uses_q(base) || expr_uses_q(idx),
        ExprKind::StructLit { fields, .. } => fields.iter().any(|(_, v)| expr_uses_q(v)),
        ExprKind::ListLit { items } => items.iter().any(expr_uses_q),
        ExprKind::DictLit { pairs } => pairs.iter().any(|(k, v)| expr_uses_q(k) || expr_uses_q(v)),
        _ => false,
    }
}
