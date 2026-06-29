//! [30100 + 30200] Résolveur ownership — escape analysis (planify-2.1).
//!
//! Produit `(OwnershipMap, ParamModes)`. **Conservateur** (I2) : un paramètre devient `ByRef` (`&T`)
//! seulement si **toutes** ses utilisations sont des emprunts (base de champ/index, opérande de
//! comparaison, argument de `print`/`len`, itérable de `for`) ; au moindre échappement (retour,
//! stockage, passage en valeur à un appel utilisateur) → `ByValue`. Les types `Copy` restent `ByValue`.

use crate::ir::*;
use std::collections::HashSet;

/// [30500] Une fonction analysable (fn libre OU méthode d'impl) pour l'analyse interprocédurale.
struct FnDef<'a> {
    key: String, // nom (fn libre) ou `"Type::méthode"`
    params: &'a [(String, Type)],
    body: &'a [Stmt],
}

/// [30500] Contexte de l'escape analysis **interprocédurale** : les modes **courants** (état du point fixe)
/// + la `TypeMap` (pour résoudre un appel de méthode `x.m()` → `Type::m`). Le cœur de V16 : un argument passé
/// à une position **`ByRef` du callee** n'échappe PAS (alors qu'avant tout argument échappait, hors `print`/`len`).
struct EscapeCtx<'a> {
    modes: &'a ParamModes,
    types: &'a TypeMap,
}

/// Mode **initial optimiste** (I2) : non-`Copy` et non-`Ref` → `ByRef` (sera rétrogradé sur échappement réel).
fn init_mode(t: &Type) -> ParamMode {
    if matches!(t, Type::Ref(_)) {
        ParamMode::InOut
    } else if is_copy(t) {
        ParamMode::ByValue
    } else {
        ParamMode::ByRef
    }
}

/// Nom de type pour la clé de méthode (`obj.m()` où `obj: Sphere` → `"Sphere::m"`).
fn type_name(t: &Type) -> Option<String> {
    match t {
        Type::Struct(n) | Type::Enum(n) | Type::Extern(n) | Type::ExternApp(n, _) | Type::App(n, _) => {
            Some(n.rsplit("::").next().unwrap_or(n).to_string())
        }
        _ => None,
    }
}

pub fn resolve(ast: &Ast, types: &TypeMap) -> OwnershipPlan {
    // 1. Recenser toutes les fonctions analysables : fns libres (clé = nom) + méthodes d'impl (clé = `Type::m`).
    let mut fns: Vec<FnDef> = Vec::new();
    for it in &ast.items {
        match &it.kind {
            ItemKind::Fn { name, params, body, .. } => {
                fns.push(FnDef { key: name.clone(), params, body });
            }
            // Méthodes **inhérentes** seulement (trait_name vide) : une méthode de **trait** doit garder la
            // signature du trait (non analysée) → reste **by-value**. On ne met QUE les inhérentes dans
            // l'analyse de modes → le site d'appel d'une méthode de trait ne trouve pas de clé → by-value (cohérent).
            ItemKind::Impl { type_name: tn, trait_name, methods } if trait_name.is_empty() => {
                for m in methods {
                    if let Some(body) = &m.body {
                        fns.push(FnDef { key: format!("{tn}::{}", m.name), params: &m.params, body });
                    }
                }
            }
            _ => {}
        }
    }

    // 2. Init optimiste : tout param non-`Copy`/non-`Ref` démarre `ByRef`.
    let mut modes: ParamModes = ParamModes::new();
    for f in &fns {
        modes.insert(f.key.clone(), f.params.iter().map(|(_, t)| init_mode(t)).collect());
    }

    // 3. Point fixe — **rétrogradation monotone** `ByRef → ByValue` (uniquement), donc **terminaison garantie**
    //    (même en récursion mutuelle `ray_color`↔`shade`). À chaque tour, on recalcule l'ensemble `escaped` de
    //    chaque fonction *avec les modes courants* : un argument à une position `ByRef` du callee n'échappe pas.
    loop {
        let mut changed = false;
        for f in &fns {
            let mut escaped: HashSet<String> = HashSet::new();
            let ctx = EscapeCtx { modes: &modes, types };
            for s in f.body {
                walk_stmt(s, &ctx, &mut escaped);
            }
            let pm: Vec<ParamMode> = f
                .params
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
            if modes.get(&f.key) != Some(&pm) {
                modes.insert(f.key.clone(), pm);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    // 4. Dispositions de move (OwnershipMap) — heuristique single-use **inchangée en vague 1** (améliorée en 2.1).
    let mut own = OwnershipMap::new();
    for it in &ast.items {
        match &it.kind {
            ItemKind::Fn { body, .. } => compute_moves(body, &mut own),
            ItemKind::Impl { methods, .. } => {
                for m in methods {
                    if let Some(body) = &m.body {
                        compute_moves(body, &mut own);
                    }
                }
            }
            _ => {}
        }
    }
    // [30800] V17 : la **seule** sortie du moteur = l'`OwnershipPlan` (sites = dispositions de move/clone,
    // params = modes interprocéduraux). Étape **neutre** : mêmes valeurs qu'avant, juste regroupées.
    OwnershipPlan { sites: own, params: modes }
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
        StmtKind::Let { value, .. } | StmtKind::Yield(value) => creads(value, Pos::Value, in_loop, out),
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
        StmtKind::While { cond, body, els } => {
            creads(cond, Pos::Borrow, in_loop, out);
            body.iter().for_each(|s| creads_stmt(s, true, out));
            els.iter().for_each(|s| creads_stmt(s, in_loop, out));
        }
        StmtKind::For { iter, body, els, .. } => {
            creads(iter, Pos::Borrow, in_loop, out);
            body.iter().for_each(|s| creads_stmt(s, true, out));
            els.iter().for_each(|s| creads_stmt(s, in_loop, out));
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
                if let Some(g) = &arm.guard {
                    creads(g, Pos::Borrow, in_loop, out);
                }
                arm.body.iter().for_each(|s| creads_stmt(s, in_loop, out));
            }
        }
        StmtKind::Assert { cond, msg } => {
            creads(cond, Pos::Borrow, in_loop, out);
            if let Some(m) = msg {
                creads(m, Pos::Borrow, in_loop, out);
            }
        }
        StmtKind::Destructure { value, .. } => creads(value, Pos::Value, in_loop, out),
        StmtKind::TryExcept { body, handler, .. } => {
            body.iter().for_each(|s| creads_stmt(s, in_loop, out));
            handler.iter().for_each(|s| creads_stmt(s, in_loop, out));
        }
        StmtKind::Raise { value } => creads(value, Pos::Value, in_loop, out),
        StmtKind::Seq(ss) => ss.iter().for_each(|s| creads_stmt(s, in_loop, out)),
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
        ExprKind::Await(inner) => creads(inner, Pos::Value, in_loop, out),
        // Closure (20700) : capture par référence → position Borrow (jamais un move de capture).
        ExprKind::Lambda { body, .. } => creads(body, Pos::Borrow, in_loop, out),
        ExprKind::Closure { body, .. } => body.iter().for_each(|s| creads_stmt(s, in_loop, out)),
        ExprKind::Tuple(items) => items.iter().for_each(|e| creads(e, Pos::Value, in_loop, out)),
        ExprKind::FString(parts) => parts.iter().for_each(|p| {
            if let FStringPart::Expr(e) = p {
                creads(e, Pos::Borrow, in_loop, out);
            }
        }),
        ExprKind::Comp { body, iter, cond, .. } => {
            creads(iter, Pos::Value, in_loop, out);
            creads(body, Pos::Value, in_loop, out);
            if let Some(c) = cond {
                creads(c, Pos::Borrow, in_loop, out);
            }
        }
        ExprKind::Ternary { cond, then, els } => {
            creads(cond, Pos::Borrow, in_loop, out);
            creads(then, Pos::Value, in_loop, out);
            creads(els, Pos::Value, in_loop, out);
        }
        ExprKind::Slice { base, lo, hi } => {
            creads(base, Pos::Borrow, in_loop, out);
            if let Some(l) = lo {
                creads(l, Pos::Borrow, in_loop, out);
            }
            if let Some(h) = hi {
                creads(h, Pos::Borrow, in_loop, out);
            }
        }
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

fn walk_stmt(s: &Stmt, ctx: &EscapeCtx, esc: &mut HashSet<String>) {
    match &s.kind {
        StmtKind::Let { value, .. } | StmtKind::Yield(value) => walk(value, Pos::Value, ctx, esc),
        StmtKind::Assign { target, value } => {
            walk(target, Pos::Borrow, ctx, esc);
            walk(value, Pos::Value, ctx, esc);
        }
        StmtKind::If { cond, then, elifs, els } => {
            walk(cond, Pos::Borrow, ctx, esc);
            then.iter().for_each(|s| walk_stmt(s, ctx, esc));
            for (c, b) in elifs {
                walk(c, Pos::Borrow, ctx, esc);
                b.iter().for_each(|s| walk_stmt(s, ctx, esc));
            }
            els.iter().for_each(|s| walk_stmt(s, ctx, esc));
        }
        StmtKind::While { cond, body, els } => {
            walk(cond, Pos::Borrow, ctx, esc);
            body.iter().chain(els).for_each(|s| walk_stmt(s, ctx, esc));
        }
        StmtKind::For { iter, body, els, .. } => {
            walk(iter, Pos::Borrow, ctx, esc); // `for x in p` → p.iter() emprunte
            body.iter().chain(els).for_each(|s| walk_stmt(s, ctx, esc));
        }
        StmtKind::Return { value } => {
            if let Some(e) = value {
                // DX2 (30400) : `return p.a` → `p` doit être **possédé** (ByValue) pour le move-hors-struct.
                if let ExprKind::Field { base, .. } = &e.kind {
                    if let ExprKind::Var(n) = &base.kind {
                        esc.insert(n.clone());
                    }
                }
                walk(e, Pos::Value, ctx, esc);
            }
        }
        StmtKind::ExprStmt(e) => walk(e, Pos::Value, ctx, esc),
        StmtKind::Match { scrutinee, arms } => {
            walk(scrutinee, Pos::Value, ctx, esc);
            for arm in arms {
                if let Some(g) = &arm.guard {
                    walk(g, Pos::Value, ctx, esc);
                }
                for s in &arm.body {
                    walk_stmt(s, ctx, esc);
                }
            }
        }
        StmtKind::Assert { cond, msg } => {
            walk(cond, Pos::Borrow, ctx, esc);
            if let Some(m) = msg {
                walk(m, Pos::Borrow, ctx, esc);
            }
        }
        StmtKind::Destructure { value, .. } => walk(value, Pos::Value, ctx, esc),
        StmtKind::TryExcept { body, handler, .. } => {
            body.iter().for_each(|s| walk_stmt(s, ctx, esc));
            handler.iter().for_each(|s| walk_stmt(s, ctx, esc));
        }
        StmtKind::Raise { value } => walk(value, Pos::Value, ctx, esc),
        StmtKind::Seq(ss) => ss.iter().for_each(|s| walk_stmt(s, ctx, esc)),
        StmtKind::Break | StmtKind::Continue | StmtKind::Pass | StmtKind::RustBlock(_) => {}
    }
}

fn walk(e: &Expr, pos: Pos, ctx: &EscapeCtx, esc: &mut HashSet<String>) {
    match &e.kind {
        ExprKind::Var(n) => {
            if pos == Pos::Value {
                esc.insert(n.clone()); // consommé entier → s'échappe
            }
        }
        ExprKind::Field { base, .. } => walk(base, Pos::Borrow, ctx, esc),
        ExprKind::Index { base, idx } => {
            walk(base, Pos::Borrow, ctx, esc);
            walk(idx, Pos::Value, ctx, esc);
        }
        ExprKind::Unary { expr, .. } => walk(expr, Pos::Value, ctx, esc),
        ExprKind::Binary { op, lhs, rhs } => {
            let p = match op {
                BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => Pos::Borrow,
                _ => Pos::Value,
            };
            walk(lhs, p, ctx, esc);
            walk(rhs, p, ctx, esc);
        }
        // [30500] LE cœur interprocédural : un argument à une position **`ByRef` du callee** n'échappe PAS
        // (avant : tout argument échappait sauf `print`/`len`). On consulte les modes **courants** du callee.
        ExprKind::Call { callee, args } => {
            let callee_modes: Option<Vec<ParamMode>> = match &callee.kind {
                ExprKind::Var(g) => ctx.modes.get(g).cloned(),
                ExprKind::Field { base, name } => ctx
                    .types
                    .get(&base.id)
                    .and_then(type_name)
                    .and_then(|tn| ctx.modes.get(&format!("{tn}::{name}")).cloned()),
                _ => None,
            };
            let borrow_builtin = matches!(&callee.kind, ExprKind::Var(g) if g == "print" || g == "len");
            for (j, a) in args.iter().enumerate() {
                let borrowed = borrow_builtin
                    || callee_modes
                        .as_ref()
                        .and_then(|m| m.get(j))
                        .map(|md| *md == ParamMode::ByRef)
                        .unwrap_or(false);
                walk(a, if borrowed { Pos::Borrow } else { Pos::Value }, ctx, esc);
            }
            // Receveur de méthode : emprunté (`scene.world.hit(r)` → `scene` ne s'échappe pas).
            if let ExprKind::Field { base, .. } = &callee.kind {
                walk(base, Pos::Borrow, ctx, esc);
            }
        }
        ExprKind::StructLit { fields, .. } => fields.iter().for_each(|(_, v)| walk(v, Pos::Value, ctx, esc)),
        ExprKind::ListLit { items } => items.iter().for_each(|x| walk(x, Pos::Value, ctx, esc)),
        ExprKind::DictLit { pairs } => pairs.iter().for_each(|(k, v)| {
            walk(k, Pos::Value, ctx, esc);
            walk(v, Pos::Value, ctx, esc);
        }),
        ExprKind::Try { inner } => walk(inner, Pos::Value, ctx, esc),
        ExprKind::Await(inner) => walk(inner, Pos::Value, ctx, esc),
        // Closure (20700) : un param capturé peut s'échapper → position Value (conservateur, sûr).
        ExprKind::Lambda { body, .. } => walk(body, Pos::Value, ctx, esc),
        ExprKind::Closure { body, .. } => body.iter().for_each(|s| walk_stmt(s, ctx, esc)),
        ExprKind::Tuple(items) => items.iter().for_each(|e| walk(e, Pos::Value, ctx, esc)),
        ExprKind::FString(parts) => parts.iter().for_each(|p| {
            if let FStringPart::Expr(e) = p {
                walk(e, Pos::Borrow, ctx, esc);
            }
        }),
        ExprKind::Comp { body, iter, cond, .. } => {
            walk(iter, Pos::Value, ctx, esc);
            walk(body, Pos::Value, ctx, esc);
            if let Some(c) = cond {
                walk(c, Pos::Borrow, ctx, esc);
            }
        }
        ExprKind::Ternary { cond, then, els } => {
            walk(cond, Pos::Borrow, ctx, esc);
            walk(then, Pos::Value, ctx, esc);
            walk(els, Pos::Value, ctx, esc);
        }
        ExprKind::Slice { base, lo, hi } => {
            walk(base, Pos::Borrow, ctx, esc);
            if let Some(l) = lo {
                walk(l, Pos::Borrow, ctx, esc);
            }
            if let Some(h) = hi {
                walk(h, Pos::Borrow, ctx, esc);
            }
        }
        ExprKind::Lit(_) => {}
    }
}
