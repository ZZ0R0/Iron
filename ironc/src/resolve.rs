//! [30100 + 30200] Résolveur ownership — escape analysis (planify-2.1).
//!
//! Produit `(OwnershipMap, ParamModes)`. **Conservateur** (I2) : un paramètre devient `ByRef` (`&T`)
//! seulement si **toutes** ses utilisations sont des emprunts (base de champ/index, opérande de
//! comparaison, argument de `print`/`len`, itérable de `for`) ; au moindre échappement (retour,
//! stockage, passage en valeur à un appel utilisateur) → `ByValue`. Les types `Copy` restent `ByValue`.

use crate::ir::*;
use std::collections::HashSet;

pub fn resolve(ast: &Ast, _types: &TypeMap) -> (OwnershipMap, ParamModes) {
    let mut modes = ParamModes::new();
    let mut own = OwnershipMap::new();
    for it in &ast.items {
        if let ItemKind::Fn { name, params, body, .. } = &it.kind {
            let mut escaped: HashSet<String> = HashSet::new();
            for s in body {
                visit_stmt(s, &mut escaped);
            }
            let pm = params
                .iter()
                .map(|(n, t)| {
                    if matches!(t, Type::Ref(_)) {
                        ParamMode::InOut
                    } else if is_copy(t) || escaped.contains(n) {
                        ParamMode::ByValue
                    } else {
                        ParamMode::ByRef
                    }
                })
                .collect();
            modes.insert(name.clone(), pm);
            compute_moves(body, &mut own);
        }
        if let ItemKind::Impl { methods, .. } = &it.kind {
            for m in methods {
                if let Some(body) = &m.body {
                    compute_moves(body, &mut own);
                }
            }
        }
    }
    (own, modes)
}

/// I1 — `move` à la dernière utilisation. Heuristique **single-use** saine : une variable lue **une seule
/// fois** en position valeur, **hors boucle**, est `Move` (sa lecture est forcément la dernière) ; sinon
/// clone-fallback. Conservateur (rate des moves multi-lectures), jamais incorrect.
fn compute_moves(body: &[Stmt], own: &mut OwnershipMap) {
    let mut reads: Vec<(String, NodeId, bool)> = Vec::new();
    for s in body {
        creads_stmt(s, false, &mut reads);
    }
    let mut count: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for (n, _, _) in &reads {
        *count.entry(n.as_str()).or_insert(0) += 1;
    }
    for (n, id, in_loop) in &reads {
        if !in_loop && count[n.as_str()] == 1 {
            own.insert(*id, Disposition::Move);
        }
    }
}

fn creads_stmt(s: &Stmt, in_loop: bool, out: &mut Vec<(String, NodeId, bool)>) {
    match &s.kind {
        StmtKind::Let { value, .. } => creads(value, Pos::Value, in_loop, out),
        StmtKind::Assign { target, value } => {
            creads(target, Pos::Borrow, in_loop, out);
            creads(value, Pos::Value, in_loop, out);
        }
        StmtKind::If { cond, then, elifs, els } => {
            creads(cond, Pos::Borrow, in_loop, out);
            then.iter().for_each(|s| creads_stmt(s, in_loop, out));
            for (c, b) in elifs {
                creads(c, Pos::Borrow, in_loop, out);
                b.iter().for_each(|s| creads_stmt(s, in_loop, out));
            }
            els.iter().for_each(|s| creads_stmt(s, in_loop, out));
        }
        StmtKind::While { cond, body } => {
            creads(cond, Pos::Borrow, in_loop, out);
            body.iter().for_each(|s| creads_stmt(s, true, out));
        }
        StmtKind::For { iter, body, .. } => {
            creads(iter, Pos::Borrow, in_loop, out);
            body.iter().for_each(|s| creads_stmt(s, true, out));
        }
        StmtKind::Return { value } => {
            if let Some(e) = value {
                creads(e, Pos::Value, in_loop, out);
            }
        }
        StmtKind::ExprStmt(e) => creads(e, Pos::Value, in_loop, out),
        StmtKind::Match { scrutinee, arms } => {
            creads(scrutinee, Pos::Value, in_loop, out);
            for arm in arms {
                arm.body.iter().for_each(|s| creads_stmt(s, in_loop, out));
            }
        }
        StmtKind::Break | StmtKind::Continue | StmtKind::Pass | StmtKind::RustBlock(_) => {}
    }
}

fn creads(e: &Expr, pos: Pos, in_loop: bool, out: &mut Vec<(String, NodeId, bool)>) {
    match &e.kind {
        ExprKind::Var(n) => {
            if pos == Pos::Value && n != "self" && n != "none" {
                out.push((n.clone(), e.id, in_loop));
            }
        }
        ExprKind::Field { base, .. } => creads(base, Pos::Borrow, in_loop, out),
        ExprKind::Index { base, idx } => {
            creads(base, Pos::Borrow, in_loop, out);
            creads(idx, Pos::Value, in_loop, out);
        }
        ExprKind::Unary { expr, .. } => creads(expr, Pos::Value, in_loop, out),
        ExprKind::Binary { op, lhs, rhs } => {
            let p = if matches!(op, BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge) {
                Pos::Borrow
            } else {
                Pos::Value
            };
            creads(lhs, p, in_loop, out);
            creads(rhs, p, in_loop, out);
        }
        ExprKind::Call { callee, args } => {
            for a in args {
                creads(a, Pos::Value, in_loop, out);
            }
            if let ExprKind::Field { base, .. } = &callee.kind {
                creads(base, Pos::Borrow, in_loop, out);
            }
        }
        ExprKind::StructLit { fields, .. } => fields.iter().for_each(|(_, v)| creads(v, Pos::Value, in_loop, out)),
        ExprKind::ListLit { items } => items.iter().for_each(|x| creads(x, Pos::Value, in_loop, out)),
        ExprKind::DictLit { pairs } => pairs.iter().for_each(|(k, v)| {
            creads(k, Pos::Value, in_loop, out);
            creads(v, Pos::Value, in_loop, out);
        }),
        ExprKind::Try { inner } => creads(inner, Pos::Value, in_loop, out),
        ExprKind::Lit(_) => {}
    }
}

fn is_copy(t: &Type) -> bool {
    matches!(t, Type::I64 | Type::F64 | Type::Bool | Type::Unit)
}

/// Marque les variables **consommées comme valeur entière** (donc qui « s'échappent » si ce sont des params).
#[derive(Clone, Copy, PartialEq)]
enum Pos {
    Value,
    Borrow,
}

fn visit_stmt(s: &Stmt, esc: &mut HashSet<String>) {
    match &s.kind {
        StmtKind::Let { value, .. } => visit(value, Pos::Value, esc),
        StmtKind::Assign { target, value } => {
            visit(target, Pos::Borrow, esc);
            visit(value, Pos::Value, esc);
        }
        StmtKind::If { cond, then, elifs, els } => {
            visit(cond, Pos::Borrow, esc);
            then.iter().for_each(|s| visit_stmt(s, esc));
            for (c, b) in elifs {
                visit(c, Pos::Borrow, esc);
                b.iter().for_each(|s| visit_stmt(s, esc));
            }
            els.iter().for_each(|s| visit_stmt(s, esc));
        }
        StmtKind::While { cond, body } => {
            visit(cond, Pos::Borrow, esc);
            body.iter().for_each(|s| visit_stmt(s, esc));
        }
        StmtKind::For { iter, body, .. } => {
            visit(iter, Pos::Borrow, esc); // `for x in p` → p.iter() emprunte
            body.iter().for_each(|s| visit_stmt(s, esc));
        }
        StmtKind::Return { value } => {
            if let Some(e) = value {
                visit(e, Pos::Value, esc);
            }
        }
        StmtKind::ExprStmt(e) => visit(e, Pos::Value, esc),
        StmtKind::Match { scrutinee, arms } => {
            visit(scrutinee, Pos::Value, esc);
            for arm in arms {
                for s in &arm.body {
                    visit_stmt(s, esc);
                }
            }
        }
        StmtKind::Break | StmtKind::Continue | StmtKind::Pass | StmtKind::RustBlock(_) => {}
    }
}

fn visit(e: &Expr, pos: Pos, esc: &mut HashSet<String>) {
    match &e.kind {
        ExprKind::Var(n) => {
            if pos == Pos::Value {
                esc.insert(n.clone()); // consommé entier → s'échappe
            }
        }
        ExprKind::Field { base, .. } => visit(base, Pos::Borrow, esc),
        ExprKind::Index { base, idx } => {
            visit(base, Pos::Borrow, esc);
            visit(idx, Pos::Value, esc);
        }
        ExprKind::Unary { expr, .. } => visit(expr, Pos::Value, esc),
        ExprKind::Binary { op, lhs, rhs } => {
            let p = match op {
                BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => Pos::Borrow,
                _ => Pos::Value,
            };
            visit(lhs, p, esc);
            visit(rhs, p, esc);
        }
        ExprKind::Call { callee, args } => {
            let borrow_arg = matches!(&callee.kind, ExprKind::Var(n) if n == "print" || n == "len");
            let ap = if borrow_arg { Pos::Borrow } else { Pos::Value };
            for a in args {
                visit(a, ap, esc);
            }
        }
        ExprKind::StructLit { fields, .. } => fields.iter().for_each(|(_, v)| visit(v, Pos::Value, esc)),
        ExprKind::ListLit { items } => items.iter().for_each(|x| visit(x, Pos::Value, esc)),
        ExprKind::DictLit { pairs } => pairs.iter().for_each(|(k, v)| {
            visit(k, Pos::Value, esc);
            visit(v, Pos::Value, esc);
        }),
        ExprKind::Try { inner } => visit(inner, Pos::Value, esc),
        ExprKind::Lit(_) => {}
    }
}
