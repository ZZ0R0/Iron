//! [60500] Passe de lint réelle (V9, I4). Travaille sur l'AST Iron ; chaque lint porte un `Span` Iron + un
//! code `IRN-Lxxx`. Émis par `ironc lint` ; jamais bloquant (warning), conforme I4.

use crate::ir::*;
use std::collections::{HashMap, HashSet};

pub struct Lint {
    pub span: Span,
    pub code: &'static str,
    pub msg: String,
}

/// Lints sur tout le programme. V0 : variables locales **inutilisées** (assignées, jamais lues) — `IRN-L001`.
pub fn lint(ast: &Ast) -> Vec<Lint> {
    let mut out = Vec::new();
    for it in &ast.items {
        match &it.kind {
            ItemKind::Fn { params, body, .. } => unused_in_body(params, body, &mut out),
            ItemKind::Impl { methods, .. } => {
                for m in methods {
                    if let Some(b) = &m.body {
                        unused_in_body(&m.params, b, &mut out);
                    }
                }
            }
            _ => {}
        }
    }
    out
}

fn unused_in_body(params: &[(String, Type)], body: &[Stmt], out: &mut Vec<Lint>) {
    // 1ʳᵉ assignation (span) par nom ; 2) noms lus (hors cible d'assignation).
    let mut assigned: HashMap<String, Span> = HashMap::new();
    let mut read: HashSet<String> = HashSet::new();
    let pnames: HashSet<String> = params.iter().map(|(n, _)| n.clone()).collect();
    collect(body, &mut assigned, &mut read);
    let mut unused: Vec<(&String, &Span)> = assigned
        .iter()
        .filter(|(n, _)| !read.contains(*n) && !pnames.contains(*n) && !n.starts_with('_'))
        .collect();
    unused.sort_by_key(|(_, s)| s.start);
    for (n, s) in unused {
        out.push(Lint { span: *s, code: "IRN-L001", msg: format!("variable `{n}` inutilisée (assignée, jamais lue)") });
    }
}

fn collect(stmts: &[Stmt], assigned: &mut HashMap<String, Span>, read: &mut HashSet<String>) {
    for s in stmts {
        match &s.kind {
            StmtKind::Let { name, value } => {
                reads(value, read);
                assigned.entry(name.clone()).or_insert(s.span);
            }
            StmtKind::Assign { target, value } => {
                reads(value, read);
                match &target.kind {
                    ExprKind::Var(n) => {
                        assigned.entry(n.clone()).or_insert(s.span);
                    }
                    other => reads_kind(other, read), // `d[k] = v`, `obj.f = v` → la base est lue
                }
            }
            StmtKind::ExprStmt(e) | StmtKind::Return { value: Some(e) } => reads(e, read),
            StmtKind::If { cond, then, elifs, els } => {
                reads(cond, read);
                collect(then, assigned, read);
                for (c, b) in elifs {
                    reads(c, read);
                    collect(b, assigned, read);
                }
                collect(els, assigned, read);
            }
            StmtKind::While { cond, body, els } => {
                reads(cond, read);
                collect(body, assigned, read);
                collect(els, assigned, read);
            }
            StmtKind::For { iter, body, els, .. } => {
                reads(iter, read);
                collect(body, assigned, read);
                collect(els, assigned, read);
            }
            StmtKind::Match { scrutinee, arms } => {
                reads(scrutinee, read);
                for a in arms {
                    collect(&a.body, assigned, read);
                }
            }
            StmtKind::TryExcept { body, handler, .. } => {
                collect(body, assigned, read);
                collect(handler, assigned, read);
            }
            StmtKind::Raise { value } => reads(value, read),
            StmtKind::Assert { cond, msg } => {
                reads(cond, read);
                if let Some(m) = msg {
                    reads(m, read);
                }
            }
            _ => {}
        }
    }
}

fn reads(e: &Expr, read: &mut HashSet<String>) {
    reads_kind(&e.kind, read);
}

fn reads_kind(k: &ExprKind, read: &mut HashSet<String>) {
    match k {
        ExprKind::Var(n) => {
            read.insert(n.clone());
        }
        ExprKind::Call { callee, args } => {
            reads(callee, read);
            args.iter().for_each(|a| reads(a, read));
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            reads(lhs, read);
            reads(rhs, read);
        }
        ExprKind::Unary { expr, .. } | ExprKind::Field { base: expr, .. } | ExprKind::Try { inner: expr } => {
            reads(expr, read)
        }
        ExprKind::Index { base, idx } => {
            reads(base, read);
            reads(idx, read);
        }
        ExprKind::Ternary { cond, then, els } => {
            reads(cond, read);
            reads(then, read);
            reads(els, read);
        }
        ExprKind::Comp { body, iter, cond, .. } => {
            reads(body, read);
            reads(iter, read);
            if let Some(c) = cond {
                reads(c, read);
            }
        }
        ExprKind::Tuple(xs) | ExprKind::ListLit { items: xs } => xs.iter().for_each(|x| reads(x, read)),
        ExprKind::Slice { base, lo, hi } => {
            reads(base, read);
            lo.iter().for_each(|e| reads(e, read));
            hi.iter().for_each(|e| reads(e, read));
        }
        ExprKind::Lambda { body, .. } => reads(body, read),
        ExprKind::FString(parts) => {
            for p in parts {
                if let FStringPart::Expr(e) = p {
                    reads(e, read);
                }
            }
        }
        ExprKind::StructLit { fields, .. } => fields.iter().for_each(|(_, v)| reads(v, read)),
        ExprKind::DictLit { pairs } => pairs.iter().for_each(|(k, v)| {
            reads(k, read);
            reads(v, read);
        }),
        _ => {}
    }
}
