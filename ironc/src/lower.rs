//! [40100 + 30200] Lowering AST → TokenStream — réel (planify-2.2 / 2.5).
//!
//! Émet le Rust via `proc_macro2::TokenStream` (I10) + la `SourceMap`. Respecte les `ParamModes`
//! (escape analysis V1) : param `ByRef` → `&T` en signature, `&x` au call site ; `print`/`len`
//! empruntent (pas de clone). Sinon clone-fallback (I1) collecté pour `--audit` (I4).
//! Crate généré : `forbid(unsafe_code)` (I2) + `IronError` (I13).

use crate::ir::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashSet;
use std::str::FromStr;

pub fn lower(
    ast: &Ast,
    types: &TypeMap,
    own: &OwnershipMap,
    param_modes: &ParamModes,
    bindings: &Bindings,
) -> (TokenStream, SourceMap, Vec<Span>) {
    let enums: HashSet<String> = ast
        .items
        .iter()
        .filter_map(|it| match &it.kind {
            ItemKind::Enum { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();
    let mut cx = Cx {
        sm: SourceMap::default(),
        types,
        param_modes,
        bindings,
        enums,
        own,
        inout_params: HashSet::new(),
        flow_wrap: None,
        audit: Vec::new(),
    };
    let items: Vec<TokenStream> = ast.items.iter().map(|it| cx.item(it)).collect();
    let body = quote! { #(#items)* };
    let file = quote! {
        #![forbid(unsafe_code)]
        #![allow(dead_code, unused_macros, unused_variables, unused_mut, unused_parens, unused_imports)]

        macro_rules! __irn_mark { ($n:literal) => {}; }

        #[derive(Debug)]
        pub struct IronError(pub String);

        #body
    };
    (file, cx.sm, cx.audit)
}

/// Emballage de retour induit par le flux (I11) : la fn a un type effectif `Option`/`Result` alors
/// que l'utilisateur n'a annoté que `T` → `return v` devient `Some(v)`/`Ok(v)`, la chute devient `None`.
#[derive(Clone, Copy, PartialEq)]
enum FlowWrap {
    Some_,
    Ok_,
}

struct Cx<'a> {
    sm: SourceMap,
    types: &'a TypeMap,
    param_modes: &'a ParamModes,
    bindings: &'a Bindings,
    enums: HashSet<String>,
    own: &'a OwnershipMap,
    inout_params: HashSet<String>,
    /// Emballage du retour pour la fn en cours de lowering (None hors fn flux-Option/Result).
    flow_wrap: Option<FlowWrap>,
    audit: Vec<Span>,
}

fn is_copy(t: &Type) -> bool {
    matches!(t, Type::I64 | Type::F64 | Type::Bool | Type::Unit)
}

fn mode_of(pm: &ParamModes, fn_name: &str, i: usize) -> ParamMode {
    pm.get(fn_name).and_then(|v| v.get(i)).copied().unwrap_or(ParamMode::ByValue)
}

impl Cx<'_> {
    fn type_of(&self, e: &Expr) -> Type {
        self.types.get(&e.id).cloned().unwrap_or(Type::Unit)
    }

    fn item(&mut self, it: &Item) -> TokenStream {
        self.flow_wrap = None; // (les méthodes d'impl ne font pas d'inférence flux dans cet incrément)
        match &it.kind {
            ItemKind::Fn { name, generics, params, ret, body } => {
                let fid = format_ident!("{}", name);
                let gens = lower_generics(generics);
                let ps: Vec<TokenStream> = params
                    .iter()
                    .enumerate()
                    .map(|(i, (n, t))| {
                        let pid = format_ident!("{}", n);
                        let ty = lower_type(t);
                        match mode_of(self.param_modes, name, i) {
                            ParamMode::ByRef => quote! { #pid: &#ty },
                            // InOut : `ty` est déjà `&mut T` (lower_type(Ref)).
                            ParamMode::ByValue | ParamMode::InOut => quote! { #pid: #ty },
                        }
                    })
                    .collect();
                let param_names: HashSet<String> = params.iter().map(|(n, _)| n.clone()).collect();
                self.inout_params =
                    params.iter().filter_map(|(n, t)| matches!(t, Type::Ref(_)).then(|| n.clone())).collect();
                // I11 par flux : type effectif (`tm[fn.id]`) vs déclaré (AST). S'ils divergent vers
                // Option/Result, on emballe les retours et (Option) on ajoute la chute `None`.
                let declared = ret;
                let effective = self.types.get(&it.id).cloned().unwrap_or_else(|| declared.clone());
                self.flow_wrap = match (&effective, declared) {
                    (Type::Option(_), d) if !matches!(d, Type::Option(_) | Type::Result(_)) => Some(FlowWrap::Some_),
                    (Type::Result(_), d) if !matches!(d, Type::Option(_) | Type::Result(_)) => Some(FlowWrap::Ok_),
                    _ => None,
                };
                let mut locals: Vec<(String, Type)> = Vec::new();
                collect_locals(body, &param_names, self.types, &mut locals);
                let decls: Vec<TokenStream> = locals
                    .iter()
                    .map(|(n, t)| {
                        let id = format_ident!("{}", n);
                        if is_annotatable(t) {
                            let ty = lower_type(t);
                            quote! { let mut #id: #ty; }
                        } else {
                            quote! { let mut #id; }
                        }
                    })
                    .collect();
                let stmts: Vec<TokenStream> = body.iter().map(|s| self.stmt(s)).collect();
                let ret_ts = match &effective {
                    Type::Unit => quote! {},
                    t => {
                        let ty = lower_type(t);
                        quote! { -> #ty }
                    }
                };
                // Chute → `None` pour une fn flux-Option qui ne retourne pas sur tous les chemins.
                let tail = if self.flow_wrap == Some(FlowWrap::Some_)
                    && !crate::infer::flow::definitely_returns(body)
                {
                    quote! { None }
                } else {
                    quote! {}
                };
                quote! { fn #fid #gens ( #(#ps),* ) #ret_ts { #(#decls)* #(#stmts)* #tail } }
            }
            ItemKind::Struct { name, generics, fields } => {
                let sid = format_ident!("{}", name);
                let gens = lower_generics(generics);
                let fs: Vec<TokenStream> = fields
                    .iter()
                    .map(|(n, t)| {
                        let fid = format_ident!("{}", n);
                        let ty = lower_type(t);
                        quote! { #fid: #ty }
                    })
                    .collect();
                quote! {
                    #[derive(Debug, Clone)]
                    struct #sid #gens { #(#fs),* }
                }
            }
            ItemKind::Enum { name, generics, variants } => {
                let eid = format_ident!("{}", name);
                let gens = lower_generics(generics);
                let vs: Vec<TokenStream> = variants
                    .iter()
                    .map(|(vn, fts)| {
                        let vid = format_ident!("{}", vn);
                        if fts.is_empty() {
                            quote! { #vid }
                        } else {
                            let tys: Vec<TokenStream> = fts.iter().map(lower_type).collect();
                            quote! { #vid( #(#tys),* ) }
                        }
                    })
                    .collect();
                quote! {
                    #[derive(Debug, Clone)]
                    enum #eid #gens { #(#vs),* }
                }
            }
            ItemKind::Trait { name, methods } => {
                let tid = format_ident!("{}", name);
                let ms: Vec<TokenStream> = methods.iter().map(|m| self.lower_method(m)).collect();
                quote! { trait #tid { #(#ms)* } }
            }
            ItemKind::Impl { trait_name, type_name, methods } => {
                let tid = format_ident!("{}", trait_name);
                let ty = format_ident!("{}", type_name);
                let ms: Vec<TokenStream> = methods.iter().map(|m| self.lower_method(m)).collect();
                quote! { impl #tid for #ty { #(#ms)* } }
            }
        }
    }

    fn lower_method(&mut self, m: &Method) -> TokenStream {
        let mid = format_ident!("{}", m.name);
        let ps: Vec<TokenStream> = m
            .params
            .iter()
            .map(|(n, t)| {
                let pid = format_ident!("{}", n);
                let ty = lower_type(t);
                quote! { #pid: #ty }
            })
            .collect();
        let sig_params = if m.self_param {
            if ps.is_empty() {
                quote! { &self }
            } else {
                quote! { &self, #(#ps),* }
            }
        } else {
            quote! { #(#ps),* }
        };
        let ret_ts = match &m.ret {
            Type::Unit => quote! {},
            t => {
                let ty = lower_type(t);
                quote! { -> #ty }
            }
        };
        match &m.body {
            None => quote! { fn #mid( #sig_params ) #ret_ts; },
            Some(body) => {
                let param_names: HashSet<String> = m.params.iter().map(|(n, _)| n.clone()).collect();
                self.inout_params =
                    m.params.iter().filter_map(|(n, t)| matches!(t, Type::Ref(_)).then(|| n.clone())).collect();
                let mut locals: Vec<(String, Type)> = Vec::new();
                collect_locals(body, &param_names, self.types, &mut locals);
                let decls: Vec<TokenStream> = locals
                    .iter()
                    .map(|(n, t)| {
                        let id = format_ident!("{}", n);
                        if is_annotatable(t) {
                            let ty = lower_type(t);
                            quote! { let mut #id: #ty; }
                        } else {
                            quote! { let mut #id; }
                        }
                    })
                    .collect();
                let stmts: Vec<TokenStream> = body.iter().map(|s| self.stmt(s)).collect();
                quote! { fn #mid( #sig_params ) #ret_ts { #(#decls)* #(#stmts)* } }
            }
        }
    }

    fn stmt(&mut self, s: &Stmt) -> TokenStream {
        let idx = self.sm.push(s.id, s.span);
        let mark = proc_macro2::Literal::usize_unsuffixed(idx);
        let body = match &s.kind {
            StmtKind::Let { name, value } => {
                let v = self.value(value);
                let id = format_ident!("{}", name);
                if self.inout_params.contains(name) {
                    quote! { (*#id) = #v; }
                } else {
                    quote! { #id = #v; }
                }
            }
            StmtKind::Assign { target, value } => {
                let v = self.value(value);
                match &target.kind {
                    ExprKind::Index { base, idx } => {
                        let b = self.place(base);
                        let i = self.value(idx);
                        match self.type_of(base) {
                            Type::Dict(..) => quote! { #b.insert(#i, #v); },
                            _ => quote! { #b[(#i) as usize] = #v; },
                        }
                    }
                    // écriture de champ à travers un shared[Struct] (30300) → borrow_mut().
                    ExprKind::Field { base, name } if matches!(self.type_of(base), Type::Shared(_)) => {
                        let b = self.place(base);
                        let f = format_ident!("{}", name);
                        // RHS évalué dans un temp D'ABORD : sinon `c.value = c.value + 1` tiendrait
                        // borrow() et borrow_mut() simultanément → panic RefCell (footgun I5, spec 🔶).
                        quote! { { let __v = #v; #b.borrow_mut().#f = __v; } }
                    }
                    _ => {
                        let t = self.place(target);
                        quote! { #t = #v; }
                    }
                }
            }
            StmtKind::If { cond, then, elifs, els } => {
                let c = self.value(cond);
                let th: Vec<TokenStream> = then.iter().map(|s| self.stmt(s)).collect();
                let mut chain = quote! { if #c { #(#th)* } };
                for (ec, eb) in elifs {
                    let ec = self.value(ec);
                    let eb: Vec<TokenStream> = eb.iter().map(|s| self.stmt(s)).collect();
                    chain = quote! { #chain else if #ec { #(#eb)* } };
                }
                if !els.is_empty() {
                    let eb: Vec<TokenStream> = els.iter().map(|s| self.stmt(s)).collect();
                    chain = quote! { #chain else { #(#eb)* } };
                }
                chain
            }
            StmtKind::While { cond, body } => {
                let c = self.value(cond);
                let b: Vec<TokenStream> = body.iter().map(|s| self.stmt(s)).collect();
                quote! { while #c { #(#b)* } }
            }
            StmtKind::For { var, iter, body } => {
                let vid = format_ident!("{}", var);
                let it = self.for_iter(iter);
                let b: Vec<TokenStream> = body.iter().map(|s| self.stmt(s)).collect();
                quote! { for #vid in #it { #(#b)* } }
            }
            StmtKind::Break => quote! { break; },
            StmtKind::Continue => quote! { continue; },
            StmtKind::Return { value } => match value {
                Some(e) if matches!(&e.kind, ExprKind::Var(n) if n == "none") => {
                    // `none` (constructeur Option) → toujours `None`, jamais emballé.
                    quote! { return None; }
                }
                Some(e) => {
                    // return d'une variable = dernière utilisation sur ce chemin → move (pas de clone).
                    let ev = match &e.kind {
                        ExprKind::Var(n) => {
                            let id = format_ident!("{}", n);
                            quote! { #id }
                        }
                        _ => self.value(e),
                    };
                    // I11 par flux : emballe le retour si la fn est flux-Option/Result.
                    match self.flow_wrap {
                        Some(FlowWrap::Some_) => quote! { return Some(#ev); },
                        Some(FlowWrap::Ok_) => quote! { return Ok(#ev); },
                        None => quote! { return #ev; },
                    }
                }
                None => match self.flow_wrap {
                    Some(FlowWrap::Some_) => quote! { return None; },
                    _ => quote! { return; },
                },
            },
            StmtKind::ExprStmt(e) => {
                let e = self.value(e);
                quote! { #e; }
            }
            StmtKind::Pass => quote! {},
            StmtKind::RustBlock(text) => match TokenStream::from_str(text) {
                Ok(ts) => ts,
                Err(_) => quote! { compile_error!("bloc rust: non tokenisable"); },
            },
            StmtKind::Match { scrutinee, arms } => {
                let s = self.value(scrutinee);
                let arm_ts: Vec<TokenStream> = arms
                    .iter()
                    .map(|arm| {
                        let pat = lower_pattern(&arm.pattern);
                        let guard = match &arm.guard {
                            Some(g) => {
                                let gv = self.value(g);
                                quote! { if #gv }
                            }
                            None => quote! {},
                        };
                        let body: Vec<TokenStream> = arm.body.iter().map(|st| self.stmt(st)).collect();
                        quote! { #pat #guard => { #(#body)* } }
                    })
                    .collect();
                quote! { match #s { #(#arm_ts)* } }
            }
        };
        quote! { __irn_mark!(#mark); #body }
    }

    fn for_iter(&mut self, iter: &Expr) -> TokenStream {
        if let ExprKind::Call { callee, args } = &iter.kind {
            if let ExprKind::Var(n) = &callee.kind {
                if n == "range" {
                    return self.range_bounds(args);
                }
            }
        }
        let it = self.place(iter);
        match self.type_of(iter) {
            Type::Dict(..) => quote! { #it.keys().cloned() },
            _ => quote! { #it.iter().cloned() },
        }
    }

    fn range_bounds(&mut self, args: &[Expr]) -> TokenStream {
        match args.len() {
            1 => {
                let hi = self.value(&args[0]);
                quote! { 0..#hi }
            }
            2 => {
                let lo = self.value(&args[0]);
                let hi = self.value(&args[1]);
                quote! { (#lo)..(#hi) }
            }
            _ => quote! { 0..0 },
        }
    }

    /// Lowering d'une **place** (lvalue / emprunt) : pas de clone.
    fn place(&mut self, e: &Expr) -> TokenStream {
        match &e.kind {
            ExprKind::Var(n) => {
                if n == "self" {
                    return quote! { self };
                }
                let id = format_ident!("{}", n);
                if matches!(self.type_of(e), Type::Ref(_)) {
                    return quote! { (*#id) };
                }
                quote! { #id }
            }
            ExprKind::Field { base, name } => {
                let f = format_ident!("{}", name);
                // place de champ à travers un shared[Struct] (30300) → borrow() (lecture empruntée).
                if matches!(self.type_of(base), Type::Shared(_)) {
                    let b = self.place(base);
                    return quote! { #b.borrow().#f };
                }
                let b = self.place(base);
                quote! { #b.#f }
            }
            ExprKind::Index { base, idx } => {
                let b = self.place(base);
                let i = self.value(idx);
                match self.type_of(base) {
                    Type::Dict(..) => quote! { #b[&#i] },
                    _ => quote! { #b[(#i) as usize] },
                }
            }
            _ => self.value(e),
        }
    }

    /// Lowering d'une **valeur** possédée : clone les places non-`Copy` (I1, collecté pour `--audit`).
    fn value(&mut self, e: &Expr) -> TokenStream {
        match &e.kind {
            ExprKind::Lit(l) => match l {
                // littéraux **suffixés** (Iron n'a qu'`i64`/`f64`) → type concret, indispensable pour
                // les appels de méthode (`abs`/`sqrt`/…) sans ambiguïté `{integer}`/`{float}`.
                Lit::Int(n) => {
                    let n = proc_macro2::Literal::i64_suffixed(*n);
                    quote! { #n }
                }
                Lit::Float(f) => {
                    let f = proc_macro2::Literal::f64_suffixed(*f);
                    quote! { #f }
                }
                Lit::Bool(b) => quote! { #b },
                Lit::Str(s) => quote! { #s.to_string() },
            },
            ExprKind::Var(n) => {
                if n == "none" {
                    return quote! { None };
                }
                if n == "self" {
                    return quote! { self };
                }
                let id = format_ident!("{}", n);
                // `inout` (Ref) : déréférencer (I3).
                if let Type::Ref(inner) = self.type_of(e) {
                    return if is_copy(&inner) {
                        quote! { (*#id) }
                    } else {
                        quote! { (*#id).clone() }
                    };
                }
                // Shared (Rc, 30300, I5) : TOUJOURS `Rc::clone` (partage explicite, cheap) — jamais un
                // move sur dernière utilisation, sinon le 2ᵉ propriétaire perdrait l'accès.
                if matches!(self.type_of(e), Type::Shared(_)) {
                    return quote! { #id.clone() };
                }
                // I1 : `Copy` ou `Move` (dernière utilisation, V5) → pas de clone.
                if is_copy(&self.type_of(e)) || self.own.get(&e.id) == Some(&Disposition::Move) {
                    quote! { #id }
                } else {
                    self.audit.push(e.span);
                    quote! { #id.clone() }
                }
            }
            ExprKind::Unary { op, expr } => {
                let x = self.value(expr);
                match op {
                    UnOp::Neg => quote! { (-(#x)) },
                    UnOp::Not => quote! { (!(#x)) },
                }
            }
            ExprKind::Binary { op, lhs, rhs } => {
                let l = self.value(lhs);
                let r = self.value(rhs);
                let o = lower_binop(*op);
                quote! { (#l #o #r) }
            }
            ExprKind::Call { callee, args } => self.call(callee, args),
            ExprKind::Field { base, name } => {
                // variante d'enum sans champ : `Enum.Variant`
                if let ExprKind::Var(ename) = &base.kind {
                    if self.enums.contains(ename) {
                        let eid = format_ident!("{}", ename);
                        let vid = format_ident!("{}", name);
                        return quote! { #eid::#vid };
                    }
                }
                let f = format_ident!("{}", name);
                // lecture de champ à travers un shared[Struct] (30300) → borrow().
                if matches!(self.type_of(base), Type::Shared(_)) {
                    let b = self.place(base);
                    return if is_copy(&self.type_of(e)) {
                        quote! { #b.borrow().#f }
                    } else {
                        quote! { #b.borrow().#f.clone() }
                    };
                }
                let b = self.place(base);
                if is_copy(&self.type_of(e)) {
                    quote! { #b.#f }
                } else {
                    self.audit.push(e.span);
                    quote! { #b.#f.clone() }
                }
            }
            ExprKind::Index { base, idx } => {
                let b = self.place(base);
                let i = self.value(idx);
                let access = match self.type_of(base) {
                    Type::Dict(..) => quote! { #b[&#i] },
                    _ => quote! { #b[(#i) as usize] },
                };
                if is_copy(&self.type_of(e)) {
                    quote! { #access }
                } else {
                    self.audit.push(e.span);
                    quote! { #access.clone() }
                }
            }
            ExprKind::ListLit { items } => {
                let xs: Vec<TokenStream> = items.iter().map(|x| self.value(x)).collect();
                quote! { vec![ #(#xs),* ] }
            }
            ExprKind::DictLit { pairs } => {
                let ins: Vec<TokenStream> = pairs
                    .iter()
                    .map(|(k, v)| {
                        let k = self.value(k);
                        let v = self.value(v);
                        quote! { __d.insert(#k, #v); }
                    })
                    .collect();
                quote! { { let mut __d = std::collections::HashMap::new(); #(#ins)* __d } }
            }
            ExprKind::StructLit { name, fields } => {
                let sid = format_ident!("{}", name);
                let fs: Vec<TokenStream> = fields
                    .iter()
                    .map(|(n, v)| {
                        let fid = format_ident!("{}", n);
                        let v = self.value(v);
                        quote! { #fid: #v }
                    })
                    .collect();
                quote! { #sid { #(#fs),* } }
            }
            ExprKind::Try { inner } => {
                let x = self.value(inner);
                quote! { (#x?) }
            }
            ExprKind::Lambda { params, body } => {
                // Closure Rust (20700) : types des params inférés par rustc (I2).
                let ps: Vec<proc_macro2::Ident> = params.iter().map(|p| format_ident!("{}", p)).collect();
                let b = self.value(body);
                quote! { | #(#ps),* | #b }
            }
        }
    }

    /// Argument-fonction de `map`/`filter` : nom nu (fn ou var-closure) ou closure inline.
    fn fn_arg(&mut self, e: &Expr) -> TokenStream {
        match &e.kind {
            ExprKind::Var(n) => {
                let id = format_ident!("{}", n);
                quote! { #id }
            }
            _ => self.value(e),
        }
    }

    fn call(&mut self, callee: &Expr, args: &[Expr]) -> TokenStream {
        // construction de variante d'enum : `Enum.Variant(args)`
        if let ExprKind::Field { base, name } = &callee.kind {
            if let ExprKind::Var(ename) = &base.kind {
                if self.enums.contains(ename) {
                    let eid = format_ident!("{}", ename);
                    let vid = format_ident!("{}", name);
                    let a: Vec<TokenStream> = args.iter().map(|x| self.value(x)).collect();
                    return quote! { #eid::#vid( #(#a),* ) };
                }
            }
        }
        if let ExprKind::Var(name) = &callee.kind {
            match name.as_str() {
                "print" if args.len() == 1 => {
                    let a = self.place(&args[0]); // println! emprunte → pas de clone
                    return match self.type_of(&args[0]) {
                        Type::List(_) | Type::Dict(..) | Type::Struct(_) | Type::Option(_) | Type::Result(_) => {
                            quote! { println!("{:?}", #a) }
                        }
                        _ => quote! { println!("{}", #a) },
                    };
                }
                "len" if args.len() == 1 => {
                    let a = self.place(&args[0]);
                    return quote! { (#a.len() as i64) };
                }
                "range" => {
                    let r = self.range_bounds(args);
                    return quote! { (#r).collect::<Vec<i64>>() };
                }
                "some" if args.len() == 1 => {
                    let a = self.value(&args[0]);
                    return quote! { Some(#a) };
                }
                "ok" if args.len() == 1 => {
                    let a = self.value(&args[0]);
                    return quote! { Ok(#a) };
                }
                "err" if args.len() == 1 => {
                    let a = self.value(&args[0]);
                    return quote! { Err(IronError(#a)) };
                }
                // stdlib V4 (80100) — wrappers fins sur std (I5)
                "str" if args.len() == 1 => {
                    let a = self.value(&args[0]);
                    return quote! { (#a).to_string() };
                }
                "abs" if args.len() == 1 => {
                    let a = self.value(&args[0]);
                    return quote! { (#a).abs() };
                }
                "sqrt" if args.len() == 1 => {
                    let a = self.value(&args[0]);
                    return quote! { (#a).sqrt() };
                }
                "min" if args.len() == 2 => {
                    let a = self.value(&args[0]);
                    let b = self.value(&args[1]);
                    return quote! { (#a).min(#b) };
                }
                "max" if args.len() == 2 => {
                    let a = self.value(&args[0]);
                    let b = self.value(&args[1]);
                    return quote! { (#a).max(#b) };
                }
                "concat" if args.len() == 2 => {
                    let a = self.value(&args[0]);
                    let b = self.value(&args[1]);
                    // concat sur listes (80200) → extension ; sinon concaténation de chaînes (80100).
                    if matches!(self.type_of(&args[0]), Type::List(_)) {
                        return quote! { { let mut __v = #a; __v.extend(#b); __v } };
                    }
                    return quote! { format!("{}{}", #a, #b) };
                }
                "push" if args.len() == 2 => {
                    let l = self.place(&args[0]);
                    let x = self.value(&args[1]);
                    return quote! { #l.push(#x) };
                }
                "sum" if args.len() == 1 => {
                    let l = self.place(&args[0]);
                    // somme générique (80200) : type d'élément déduit (i64 par défaut).
                    let et = match self.type_of(&args[0]) {
                        Type::List(e) => lower_type(&e),
                        _ => quote! { i64 },
                    };
                    return quote! { #l.iter().sum::<#et>() };
                }
                // stdlib élargie V6 (80200)
                "sort" if args.len() == 1 => {
                    let v = self.value(&args[0]);
                    return quote! { { let mut __v = #v; __v.sort_by(|a, b| a.partial_cmp(b).unwrap()); __v } };
                }
                "input" if args.is_empty() => {
                    return quote! {{
                        let mut __s = String::new();
                        std::io::stdin().read_line(&mut __s).ok();
                        __s.trim_end().to_string()
                    }};
                }
                "upper" if args.len() == 1 => {
                    let a = self.value(&args[0]);
                    return quote! { (#a).to_uppercase() };
                }
                "lower" if args.len() == 1 => {
                    let a = self.value(&args[0]);
                    return quote! { (#a).to_lowercase() };
                }
                "split" if args.len() == 2 => {
                    let a = self.value(&args[0]);
                    let b = self.value(&args[1]);
                    return quote! { (#a).split((#b).as_str()).map(|__p| __p.to_string()).collect::<Vec<_>>() };
                }
                "index_of" if args.len() == 2 => {
                    let a = self.value(&args[0]);
                    let b = self.value(&args[1]);
                    return quote! { (#a).find((#b).as_str()).map(|__i| __i as i64).unwrap_or(-1i64) };
                }
                // partage explicite (30300, I5) : Rc<RefCell<T>>.
                "ref" if args.len() == 1 => {
                    let a = self.value(&args[0]);
                    return quote! { std::rc::Rc::new(std::cell::RefCell::new(#a)) };
                }
                // map/filter (20700) : la fonction est une closure inline (`|x| …`) ou un nom (fn/var).
                "map" if args.len() == 2 => {
                    let f = self.fn_arg(&args[0]);
                    let xs = self.value(&args[1]);
                    return quote! { (#xs).into_iter().map(#f).collect::<Vec<_>>() };
                }
                "filter" if args.len() == 2 => {
                    let f = self.fn_arg(&args[0]);
                    let xs = self.value(&args[1]);
                    // prédicat sur une copie de l'élément (robuste pour tout type `Clone`).
                    return quote! { (#xs).into_iter().filter(|__e| (#f)(__e.clone())).collect::<Vec<_>>() };
                }
                _ => {}
            }
            // fonction externe (`.iri`) → chemin Rust (50200).
            if let Some(ef) = self.bindings.fns.get(name) {
                let path = TokenStream::from_str(&ef.path).unwrap_or_else(|_| quote! { () });
                let a: Vec<TokenStream> = args.iter().map(|x| self.value(x)).collect();
                let _ = ef;
                return quote! { #path( #(#a),* ) };
            }
        }
        // appel utilisateur : `&x` pour un param `ByRef`, valeur sinon.
        let modes = match &callee.kind {
            ExprKind::Var(name) => self.param_modes.get(name).cloned(),
            _ => None,
        };
        let c = self.place(callee);
        let a: Vec<TokenStream> = args
            .iter()
            .enumerate()
            .map(|(i, x)| {
                let mode = modes
                    .as_ref()
                    .and_then(|m| m.get(i))
                    .copied()
                    .unwrap_or(ParamMode::ByValue);
                match mode {
                    ParamMode::ByRef => {
                        let p = self.place(x);
                        quote! { &#p }
                    }
                    ParamMode::InOut => {
                        let p = self.place(x);
                        quote! { &mut #p }
                    }
                    ParamMode::ByValue => self.value(x),
                }
            })
            .collect();
        quote! { #c( #(#a),* ) }
    }
}

/// Collecte (ordonnée, dédupliquée) les variables locales liées par `Let`, avec leur type inféré,
/// récursivement, hors params. Pour le hoisting (scoping Python) + annotation de type.
fn collect_locals(body: &[Stmt], params: &HashSet<String>, types: &TypeMap, out: &mut Vec<(String, Type)>) {
    for s in body {
        match &s.kind {
            StmtKind::Let { name, value } => {
                if !params.contains(name) && !out.iter().any(|(n, _)| n == name) {
                    let t = types.get(&value.id).cloned().unwrap_or(Type::Unit);
                    out.push((name.clone(), t));
                }
            }
            StmtKind::If { then, elifs, els, .. } => {
                collect_locals(then, params, types, out);
                for (_, b) in elifs {
                    collect_locals(b, params, types, out);
                }
                collect_locals(els, params, types, out);
            }
            StmtKind::While { body, .. } => collect_locals(body, params, types, out),
            StmtKind::For { body, .. } => collect_locals(body, params, types, out),
            StmtKind::Match { arms, .. } => {
                for arm in arms {
                    collect_locals(&arm.body, params, types, out);
                }
            }
            _ => {}
        }
    }
}

/// Motif Iron → motif Rust (V3).
fn lower_pattern(p: &Pattern) -> TokenStream {
    match p {
        Pattern::Wild => quote! { _ },
        Pattern::Var(n) => {
            let id = format_ident!("{}", n);
            quote! { #id }
        }
        Pattern::Lit(l) => match l {
            Lit::Int(n) => {
                let n = proc_macro2::Literal::i64_unsuffixed(*n);
                quote! { #n }
            }
            Lit::Bool(b) => quote! { #b },
            Lit::Str(s) => quote! { #s },
            Lit::Float(_) => quote! { _ }, // motif flottant illégal en Rust
        },
        Pattern::Variant { path, sub } => {
            let p = lower_path(path);
            if sub.is_empty() {
                p
            } else {
                let subs: Vec<TokenStream> = sub.iter().map(lower_pattern).collect();
                quote! { #p( #(#subs),* ) }
            }
        }
    }
}

fn lower_path(path: &[String]) -> TokenStream {
    let ids: Vec<proc_macro2::Ident> = path.iter().map(|s| format_ident!("{}", s)).collect();
    quote! { #(#ids)::* }
}

/// Annoter une déclaration hoistée ? Oui si le type est **concret** (sans `Unit` — sinon on laisse
/// l'inférence Rust trancher, p. ex. liste vide).
fn is_annotatable(t: &Type) -> bool {
    // Seuls les primitifs (fixe l'ambiguïté `{float}`) ; le reste — structs (génériques !), collections,
    // enums, Option/Result — est laissé à l'inférence Rust (annoter casserait `let mut p: Pair;`).
    matches!(t, Type::I64 | Type::F64 | Type::Bool | Type::Str)
}

fn lower_generics(gens: &Generics) -> TokenStream {
    if gens.is_empty() {
        return quote! {};
    }
    let parts: Vec<TokenStream> = gens
        .iter()
        .map(|(n, bounds)| {
            let id = format_ident!("{}", n);
            if bounds.is_empty() {
                quote! { #id }
            } else {
                let bs: Vec<proc_macro2::Ident> = bounds.iter().map(|b| format_ident!("{}", b)).collect();
                quote! { #id: #(#bs)+* }
            }
        })
        .collect();
    quote! { < #(#parts),* > }
}

fn lower_type(t: &Type) -> TokenStream {
    match t {
        Type::I64 => quote! { i64 },
        Type::F64 => quote! { f64 },
        Type::Bool => quote! { bool },
        Type::Str => quote! { String },
        Type::Unit => quote! { () },
        Type::Struct(n) => {
            let id = format_ident!("{}", n);
            quote! { #id }
        }
        Type::Option(t) => {
            let i = lower_type(t);
            quote! { Option<#i> }
        }
        Type::Result(t) => {
            let i = lower_type(t);
            quote! { Result<#i, IronError> }
        }
        Type::List(t) => {
            let i = lower_type(t);
            quote! { Vec<#i> }
        }
        Type::Dict(k, v) => {
            let k = lower_type(k);
            let v = lower_type(v);
            quote! { std::collections::HashMap<#k, #v> }
        }
        Type::Extern(path) => TokenStream::from_str(path).unwrap_or_else(|_| quote! { () }),
        Type::Enum(n) => {
            let id = format_ident!("{}", n);
            quote! { #id }
        }
        Type::Generic(n) => {
            let id = format_ident!("{}", n);
            quote! { #id }
        }
        Type::App(name, args) => {
            let id = format_ident!("{}", name);
            let a: Vec<TokenStream> = args.iter().map(lower_type).collect();
            quote! { #id< #(#a),* > }
        }
        Type::Ref(t) => {
            let inner = lower_type(t);
            quote! { &mut #inner }
        }
        // Type fonction (20700) : rarement matérialisé (closures inline laissées à rustc) ; on émet
        // un `impl Fn` valide en position retour si jamais atteint.
        Type::Fn(ps, r) => {
            let pa: Vec<TokenStream> = ps.iter().map(lower_type).collect();
            let rt = lower_type(r);
            quote! { impl Fn( #(#pa),* ) -> #rt }
        }
        // Partage explicite (30300, I5) : Rc<RefCell<T>> ; chemins complets → aucun `use` à gérer.
        Type::Shared(t) => {
            let inner = lower_type(t);
            quote! { std::rc::Rc<std::cell::RefCell<#inner>> }
        }
    }
}

fn lower_binop(op: BinOp) -> TokenStream {
    match op {
        BinOp::Add => quote! { + },
        BinOp::Sub => quote! { - },
        BinOp::Mul => quote! { * },
        BinOp::Div => quote! { / },
        BinOp::Rem => quote! { % },
        BinOp::Eq => quote! { == },
        BinOp::Ne => quote! { != },
        BinOp::Lt => quote! { < },
        BinOp::Le => quote! { <= },
        BinOp::Gt => quote! { > },
        BinOp::Ge => quote! { >= },
        BinOp::And => quote! { && },
        BinOp::Or => quote! { || },
    }
}
