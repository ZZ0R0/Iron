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
    for it in &ast.items {
        if let ItemKind::Fn { name, params, body, .. } = &it.kind {
            let mut escaped: HashSet<String> = HashSet::new();
            for s in body {
                visit_stmt(s, &mut escaped);
            }
            let pm = params
                .iter()
                .map(|(n, t)| {
                    if is_copy(t) || escaped.contains(n) {
                        ParamMode::ByValue
                    } else {
                        ParamMode::ByRef
                    }
                })
                .collect();
            modes.insert(name.clone(), pm);
        }
    }
    // OwnershipMap (Move/Borrow locaux) : raffinement ultérieur — vide ⇒ clone-default (I1).
    (OwnershipMap::new(), modes)
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
