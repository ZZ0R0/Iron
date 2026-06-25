//! [40100 + 30200] Lowering AST → TokenStream — réel (planify-2.2 / 2.5).
//!
//! Émet le Rust via `proc_macro2::TokenStream` (I10) + la `SourceMap`. Respecte les `ParamModes`
//! (escape analysis V1) : param `ByRef` → `&T` en signature, `&x` au call site ; `print`/`len`
//! empruntent (pas de clone). Sinon clone-fallback (I1) collecté pour `--audit` (I4).
//! Crate généré : `forbid(unsafe_code)` (I2) + `IronError` (I13).

use crate::ir::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
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
    let user_fns: HashSet<String> = ast
        .items
        .iter()
        .filter_map(|it| match &it.kind {
            ItemKind::Fn { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();
    let mut fn_params: HashMap<String, Vec<String>> = HashMap::new();
    let mut fn_defaults: HashMap<String, Vec<Option<Expr>>> = HashMap::new();
    let mut method_defaults: HashMap<(String, String), Vec<Option<Expr>>> = HashMap::new();
    for it in &ast.items {
        match &it.kind {
            ItemKind::Fn { name, params, defaults, .. } => {
                fn_params.insert(name.clone(), params.iter().map(|(n, _)| n.clone()).collect());
                if defaults.iter().any(Option::is_some) {
                    fn_defaults.insert(name.clone(), defaults.clone());
                }
            }
            ItemKind::Impl { type_name, methods, .. } => {
                for m in methods {
                    if m.defaults.iter().any(Option::is_some) {
                        method_defaults.insert((type_name.clone(), m.name.clone()), m.defaults.clone());
                    }
                }
            }
            _ => {}
        }
    }
    let mut cx = Cx {
        sm: SourceMap::default(),
        types,
        param_modes,
        bindings,
        enums,
        user_fns,
        fn_params,
        fn_defaults,
        method_defaults,
        own,
        inout_params: HashSet::new(),
        byref_params: HashSet::new(),
        flow_wrap: None,
        audit: Vec::new(),
    };
    let items: Vec<TokenStream> = ast.items.iter().map(|it| cx.item(it)).collect();
    // Alias des types opaques `extern type Name = "path"` (50400) → `type Name = path;`.
    let aliases: Vec<TokenStream> = ast
        .extern_types
        .iter()
        .map(|(name, path)| {
            let id = format_ident!("{}", name);
            let ty = TokenStream::from_str(path).unwrap_or_else(|_| quote! { () });
            quote! { type #id = #ty; }
        })
        .collect();
    let body = quote! { #(#aliases)* #(#items)* };
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
    /// Fns définies par l'utilisateur — priment sur les built-ins homonymes (ex. un `fn add` perso).
    user_fns: HashSet<String>,
    /// Noms des params par fn (surface Python) : pour réordonner les arguments nommés `f(x=1, y=2)`.
    fn_params: HashMap<String, Vec<String>>,
    /// Valeurs par défaut des params (surface Python) : fn → defaults ; (type, méthode) → defaults.
    fn_defaults: HashMap<String, Vec<Option<Expr>>>,
    method_defaults: HashMap<(String, String), Vec<Option<Expr>>>,
    own: &'a OwnershipMap,
    inout_params: HashSet<String>,
    /// Params passés en `&T` (ByRef) pour la fn en cours — base non possédée → pas de move-hors-struct.
    byref_params: HashSet<String>,
    /// Emballage du retour pour la fn en cours de lowering (None hors fn flux-Option/Result).
    flow_wrap: Option<FlowWrap>,
    audit: Vec<Span>,
}

fn is_copy(t: &Type) -> bool {
    matches!(t, Type::I64 | Type::F64 | Type::Bool | Type::Unit | Type::Num(_) | Type::Char)
}

/// Une méthode mute-t-elle `self` (affectation à `self.champ`/`self[i]`) → `&mut self` (surface Python).
fn stmts_mutate_self(body: &[Stmt]) -> bool {
    body.iter().any(|s| match &s.kind {
        StmtKind::Assign { target, .. } => expr_is_self_place(target),
        StmtKind::If { then, elifs, els, .. } => {
            stmts_mutate_self(then) || elifs.iter().any(|(_, b)| stmts_mutate_self(b)) || stmts_mutate_self(els)
        }
        StmtKind::While { body, .. } | StmtKind::For { body, .. } => stmts_mutate_self(body),
        StmtKind::Match { arms, .. } => arms.iter().any(|a| stmts_mutate_self(&a.body)),
        _ => false,
    })
}

fn expr_is_self_place(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => {
            matches!(&base.kind, ExprKind::Var(n) if n == "self") || expr_is_self_place(base)
        }
        _ => false,
    }
}

/// Type affiché via `{:?}` (Debug) plutôt que `{}` (Display) — collections, structs, Option/Result.
/// Utilisé par `print` ET les f-strings pour choisir le spécificateur de format.
fn is_debug_type(t: &Type) -> bool {
    matches!(
        t,
        Type::List(_)
            | Type::Dict(..)
            | Type::Set(_)
            | Type::Tuple(_)
            | Type::Struct(_)
            | Type::Option(_)
            | Type::Result(_)
    )
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
            ItemKind::Fn { name, generics, params, ret, body, .. } => {
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
                self.byref_params = params
                    .iter()
                    .enumerate()
                    .filter_map(|(i, (n, _))| {
                        matches!(mode_of(self.param_modes, name, i), ParamMode::ByRef).then(|| n.clone())
                    })
                    .collect();
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
                let ty = format_ident!("{}", type_name);
                let ms: Vec<TokenStream> = methods.iter().map(|m| self.lower_method(m)).collect();
                if trait_name.is_empty() {
                    quote! { impl #ty { #(#ms)* } } // impl inhérent (méthodes sans trait)
                } else {
                    let tid = format_ident!("{}", trait_name);
                    quote! { impl #tid for #ty { #(#ms)* } }
                }
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
        // `&mut self` si la méthode mute `self` (ex. `self.value += by`), sinon `&self` (surface Python).
        let self_ts = if m.self_param {
            let mutates = m.body.as_ref().map_or(false, |b| stmts_mutate_self(b));
            if mutates {
                quote! { &mut self }
            } else {
                quote! { &self }
            }
        } else {
            quote! {}
        };
        let sig_params = if m.self_param {
            if ps.is_empty() {
                quote! { #self_ts }
            } else {
                quote! { #self_ts, #(#ps),* }
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
                Some(e) if matches!(&e.kind, ExprKind::Var(n) if n == "none" || n == "None") => {
                    // `none`/`None` (constructeur Option) → toujours `None`, jamais emballé.
                    quote! { return None; }
                }
                Some(e) => {
                    let ev = self.return_value(e);
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
            // V7 — assert (10800) + destructuring (10400) : lowering réel.
            StmtKind::Assert { cond, msg } => {
                let c = self.value(cond);
                match msg {
                    Some(m) => {
                        let mv = self.value(m);
                        quote! { assert!(#c, "{}", #mv); }
                    }
                    None => quote! { assert!(#c); },
                }
            }
            StmtKind::Destructure { targets, value } => {
                let v = self.value(value);
                let ids: Vec<proc_macro2::Ident> = targets.iter().map(|t| format_ident!("{}", t)).collect();
                quote! { let ( #(mut #ids),* ) = #v; }
            }
            // `try: … except [e]: …` (surface Python) : IIFE renvoyant Result, capture les `?`/raise.
            // Les locaux du corps sont hoistés (fn-level) → le closure les assigne par &mut.
            StmtKind::TryExcept { body, err_name, handler } => {
                let body_ts: Vec<TokenStream> = body.iter().map(|s| self.stmt(s)).collect();
                let handler_ts: Vec<TokenStream> = handler.iter().map(|s| self.stmt(s)).collect();
                let err_arm = match err_name {
                    Some(e) => {
                        let id = format_ident!("{}", e);
                        quote! { Err(__err) => { let #id = __err.0; #(#handler_ts)* } }
                    }
                    None => quote! { Err(_) => { #(#handler_ts)* } },
                };
                quote! {
                    match (|| -> Result<(), IronError> { #(#body_ts)* Ok(()) })() {
                        Ok(()) => {}
                        #err_arm
                    }
                }
            }
            StmtKind::Raise { value } => {
                let v = self.value(value);
                if matches!(self.type_of(value), Type::Str) {
                    quote! { return Err(IronError(#v)); }
                } else {
                    quote! { return Err(IronError(format!("{}", #v))); }
                }
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
                // index de tuple `t.0` (10400) → `syn::Index`.
                if let Ok(n) = name.parse::<usize>() {
                    let b = self.place(base);
                    let idx = syn::Index::from(n);
                    return quote! { #b.#idx };
                }
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
                Lit::Char(c) => quote! { #c },
            },
            ExprKind::Var(n) => {
                if n == "none" || n == "None" {
                    return quote! { None }; // `None` (Python) ≡ `none`
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
                match op {
                    // `**` (10700) : flottant → powf ; entier → pow(exposant en u32).
                    BinOp::Pow => {
                        let r = self.value(rhs);
                        if matches!(self.type_of(lhs), Type::F64 | Type::Num(NumKind::F32)) {
                            quote! { (#l).powf(#r) }
                        } else {
                            quote! { (#l).pow((#r) as u32) }
                        }
                    }
                    // `//` (10700) : division entière/plancher (Euclidienne).
                    BinOp::FloorDiv => {
                        let r = self.value(rhs);
                        quote! { (#l).div_euclid(#r) }
                    }
                    // `in`/`not in` (10800) : contains / contains_key.
                    BinOp::In | BinOp::NotIn => {
                        let coll = self.place(rhs);
                        let contains = match self.type_of(rhs) {
                            Type::Dict(..) => quote! { #coll.contains_key(&#l) },
                            _ => quote! { #coll.contains(&#l) }, // list, set
                        };
                        if matches!(op, BinOp::NotIn) {
                            quote! { (!#contains) }
                        } else {
                            contains
                        }
                    }
                    _ => {
                        let r = self.value(rhs);
                        let o = lower_binop(*op);
                        quote! { (#l #o #r) }
                    }
                }
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
                // index de tuple `t.0` (10400) → `syn::Index` (un ident ne peut pas être un chiffre).
                if let Ok(n) = name.parse::<usize>() {
                    let b = self.place(base);
                    let idx = syn::Index::from(n);
                    return if is_copy(&self.type_of(e)) {
                        quote! { #b.#idx }
                    } else {
                        self.audit.push(e.span);
                        quote! { #b.#idx.clone() }
                    };
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
                // `Name(champ=val)` : appel à **arguments nommés** si `Name` est une fn, sinon construction.
                if self.user_fns.contains(name) {
                    let fid = format_ident!("{}", name);
                    let params = self.fn_params.get(name).cloned().unwrap_or_default();
                    let defaults = self.fn_defaults.get(name).cloned().unwrap_or_default();
                    let modes = self.param_modes.get(name).cloned();
                    let args: Vec<TokenStream> = params
                        .iter()
                        .enumerate()
                        .map(|(i, p)| {
                            let arg = fields
                                .iter()
                                .find(|(fname, _)| fname == p)
                                .map(|(_, e)| e.clone())
                                .or_else(|| defaults.get(i).and_then(|o| o.clone()));
                            let mode = modes.as_ref().and_then(|m| m.get(i)).copied().unwrap_or(ParamMode::ByValue);
                            match arg {
                                Some(e) => match mode {
                                    ParamMode::ByRef => {
                                        let pl = self.place(&e);
                                        quote! { &#pl }
                                    }
                                    ParamMode::InOut => {
                                        let pl = self.place(&e);
                                        quote! { &mut #pl }
                                    }
                                    ParamMode::ByValue => self.value(&e),
                                },
                                None => quote! { Default::default() },
                            }
                        })
                        .collect();
                    return quote! { #fid( #(#args),* ) };
                }
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
            // V7 — tuple/f-string/ternaire : lowering réel ; comp/slice : raffinés en 2.7/2.5.
            ExprKind::Tuple(items) => {
                let xs: Vec<TokenStream> = items.iter().map(|e| self.value(e)).collect();
                quote! { ( #(#xs),* ) }
            }
            ExprKind::FString(parts) => {
                let mut fmt = String::new();
                let mut args: Vec<TokenStream> = Vec::new();
                for p in parts {
                    match p {
                        FStringPart::Lit(s) => fmt.push_str(&s.replace('{', "{{").replace('}', "}}")),
                        FStringPart::Expr(e) => {
                            // Debug pour les collections/structs, Display sinon (comme `print`).
                            fmt.push_str(if is_debug_type(&self.type_of(e)) { "{:?}" } else { "{}" });
                            args.push(self.place(e)); // format! emprunte → pas de clone
                        }
                    }
                }
                quote! { format!(#fmt #(, #args)*) }
            }
            ExprKind::Ternary { cond, then, els } => {
                let c = self.value(cond);
                let t = self.value(then);
                let e = self.value(els);
                quote! { if #c { #t } else { #e } }
            }
            ExprKind::Comp { kind, body, var, iter, cond } => {
                // 2.7 : chaîne d'itérateurs into_iter().map/filter_map(...).collect (10600).
                let it = self.value(iter);
                let pat = lower_pattern(var);
                let body_ts = self.value(body);
                let mapper = match cond {
                    Some(c) => {
                        let cv = self.value(c);
                        quote! { filter_map(|#pat| if #cv { Some(#body_ts) } else { None }) }
                    }
                    None => quote! { map(|#pat| #body_ts) },
                };
                let collect = match kind {
                    CompKind::List => quote! { collect::<Vec<_>>() },
                    CompKind::Set => quote! { collect::<std::collections::HashSet<_>>() },
                    CompKind::Dict => quote! { collect::<std::collections::HashMap<_, _>>() },
                };
                quote! { (#it).into_iter().#mapper.#collect }
            }
            ExprKind::Slice { base, lo, hi } => {
                let b = self.place(base);
                let lo_ts = match lo {
                    Some(l) => {
                        let l = self.value(l);
                        quote! { (#l) as usize }
                    }
                    None => quote! { 0usize },
                };
                let range = match hi {
                    Some(h) => {
                        let h = self.value(h);
                        quote! { #lo_ts..((#h) as usize) }
                    }
                    None => quote! { #lo_ts.. },
                };
                if matches!(self.type_of(base), Type::Str) {
                    quote! { #b[#range].to_string() }
                } else {
                    quote! { #b[#range].to_vec() }
                }
            }
        }
    }

    /// Valeur retournée : un `return` est terminal, donc on **move** plutôt que cloner quand c'est sûr.
    fn return_value(&mut self, e: &Expr) -> TokenStream {
        match &e.kind {
            // return d'une variable = dernière utilisation → move (pas de clone).
            ExprKind::Var(n) => {
                let id = format_ident!("{}", n);
                quote! { #id }
            }
            // DX2 (30400) : `return p.a` où `p` est possédé (pas ByRef ni Shared) et `a` non-Copy →
            // move-hors-struct (move partiel sûr car le `return` quitte la fn → pas de borne Clone).
            ExprKind::Field { base, name } => {
                let owned_base = matches!(&base.kind, ExprKind::Var(bn) if !self.byref_params.contains(bn));
                if owned_base && !is_copy(&self.type_of(e)) && !matches!(self.type_of(base), Type::Shared(_)) {
                    let b = self.place(base);
                    let f = format_ident!("{}", name);
                    return quote! { #b.#f };
                }
                self.value(e)
            }
            _ => self.value(e),
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
        // modules stdlib namespacés (surface Python, D2) : `io.read_file(...)` → wrappers `std::fs`.
        if let ExprKind::Field { base, name } = &callee.kind {
            if let ExprKind::Var(m) = &base.kind {
                if m == "io" && !self.user_fns.contains("io") {
                    return self.lower_io(name, args);
                }
            }
        }
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
            // les fns utilisateur priment sur les built-ins homonymes (ex. un `fn add` perso).
            if !self.user_fns.contains(name) {
            match name.as_str() {
                "print" if args.len() == 1 => {
                    let a = self.place(&args[0]); // println! emprunte → pas de clone
                    return if is_debug_type(&self.type_of(&args[0])) {
                        quote! { println!("{:?}", #a) }
                    } else {
                        quote! { println!("{}", #a) }
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
                // ensembles (20900)
                "set" if args.len() == 1 => {
                    let l = self.value(&args[0]);
                    return quote! { (#l).into_iter().collect::<std::collections::HashSet<_>>() };
                }
                "add" if args.len() == 2 => {
                    let s = self.place(&args[0]);
                    let x = self.value(&args[1]);
                    return quote! { #s.insert(#x) };
                }
                "union" if args.len() == 2 => {
                    let a = self.place(&args[0]);
                    let b = self.place(&args[1]);
                    return quote! { (&#a | &#b) };
                }
                "inter" if args.len() == 2 => {
                    let a = self.place(&args[0]);
                    let b = self.place(&args[1]);
                    return quote! { (&#a & &#b) };
                }
                "diff" if args.len() == 2 => {
                    let a = self.place(&args[0]);
                    let b = self.place(&args[1]);
                    return quote! { (&#a - &#b) };
                }
                // casts numériques `to_<t>(x)` → `(x as t)` (20800).
                cast if args.len() == 1
                    && cast.strip_prefix("to_").map_or(false, |t| numeric_type_by_name(t).is_some()) =>
                {
                    let a = self.value(&args[0]);
                    let ty = format_ident!("{}", &cast[3..]);
                    return quote! { (#a as #ty) };
                }
                _ => {}
            }
            }
            // fonction externe (`.iri`) → chemin Rust (50200).
            if let Some(ef) = self.bindings.fns.get(name) {
                let path = TokenStream::from_str(&ef.path).unwrap_or_else(|_| quote! { () });
                let a: Vec<TokenStream> = args.iter().map(|x| self.value(x)).collect();
                let _ = ef;
                return quote! { #path( #(#a),* ) };
            }
        }
        // appel utilisateur : `&x` ByRef, `&mut x` InOut, valeur sinon ; + args par défaut (surface Python).
        let modes = match &callee.kind {
            ExprKind::Var(name) => self.param_modes.get(name).cloned(),
            _ => None,
        };
        let pad = self.call_padding(callee, args.len());
        let c = self.place(callee);
        let mut a: Vec<TokenStream> = Vec::new();
        for (i, x) in args.iter().chain(pad.iter()).enumerate() {
            let mode = modes.as_ref().and_then(|m| m.get(i)).copied().unwrap_or(ParamMode::ByValue);
            let ts = match mode {
                ParamMode::ByRef => {
                    let p = self.place(x);
                    quote! { &#p }
                }
                ParamMode::InOut => {
                    let p = self.place(x);
                    quote! { &mut #p }
                }
                ParamMode::ByValue => self.value(x),
            };
            a.push(ts);
        }
        quote! { #c( #(#a),* ) }
    }

    /// Module stdlib `io` (80300, D2) : wrappers minces sur `std::fs`, faillibles → `Result`/`IronError`.
    fn lower_io(&mut self, name: &str, args: &[Expr]) -> TokenStream {
        match name {
            "read_file" if args.len() == 1 => {
                let p = self.value(&args[0]);
                quote! { std::fs::read_to_string(#p).map_err(|e| IronError(format!("{}", e))) }
            }
            "write_file" if args.len() == 2 => {
                let p = self.value(&args[0]);
                let c = self.value(&args[1]);
                quote! { std::fs::write(#p, #c).map_err(|e| IronError(format!("{}", e))) }
            }
            "read_lines" if args.len() == 1 => {
                let p = self.value(&args[0]);
                quote! {
                    std::fs::read_to_string(#p)
                        .map(|s| s.lines().map(|l| l.to_string()).collect::<Vec<_>>())
                        .map_err(|e| IronError(format!("{}", e)))
                }
            }
            "append_file" if args.len() == 2 => {
                let p = self.value(&args[0]);
                let c = self.value(&args[1]);
                quote! {{
                    use std::io::Write;
                    std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(#p)
                        .and_then(|mut f| f.write_all((#c).as_bytes()))
                        .map_err(|e| IronError(format!("{}", e)))
                }}
            }
            "exists" if args.len() == 1 => {
                let p = self.value(&args[0]);
                quote! { std::path::Path::new(&#p).exists() }
            }
            other => {
                let msg = format!("io.{other} : fonction inconnue ou mauvaise arité");
                quote! { compile_error!(#msg) }
            }
        }
    }

    /// Valeurs par défaut à ajouter pour un appel partiel `f(args)` / `c.m(args)` (surface Python) :
    /// les défauts des params en trop par rapport aux args fournis.
    fn call_padding(&self, callee: &Expr, n_args: usize) -> Vec<Expr> {
        let defaults = match &callee.kind {
            ExprKind::Var(name) => self.fn_defaults.get(name),
            ExprKind::Field { base, name } => match self.type_of(base) {
                Type::Struct(tn) => self.method_defaults.get(&(tn, name.clone())),
                _ => None,
            },
            _ => None,
        };
        match defaults {
            Some(d) if d.len() > n_args => d[n_args..].iter().filter_map(|o| o.clone()).collect(),
            _ => Vec::new(),
        }
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
            StmtKind::TryExcept { body, handler, .. } => {
                collect_locals(body, params, types, out);
                collect_locals(handler, params, types, out);
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
            Lit::Char(c) => quote! { #c },
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
        Pattern::Tuple(subs) => {
            let ps: Vec<TokenStream> = subs.iter().map(lower_pattern).collect();
            quote! { ( #(#ps),* ) }
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
    matches!(t, Type::I64 | Type::F64 | Type::Bool | Type::Str | Type::Num(_) | Type::Char)
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
        // V7 (10400/20900/20800/30600)
        Type::Tuple(ts) => {
            let xs: Vec<TokenStream> = ts.iter().map(lower_type).collect();
            quote! { ( #(#xs),* ) }
        }
        Type::Set(t) => {
            let i = lower_type(t);
            quote! { std::collections::HashSet<#i> }
        }
        Type::Num(k) => {
            let id = format_ident!("{}", num_kind_str(*k));
            quote! { #id }
        }
        Type::Char => quote! { char },
        Type::Weak(t) => {
            let i = lower_type(t);
            quote! { std::rc::Weak<std::cell::RefCell<#i>> }
        }
    }
}

/// Nom Rust d'un `NumKind` (V7, 20800).
fn num_kind_str(k: NumKind) -> &'static str {
    match k {
        NumKind::I8 => "i8",
        NumKind::I16 => "i16",
        NumKind::I32 => "i32",
        NumKind::I128 => "i128",
        NumKind::Isize => "isize",
        NumKind::U8 => "u8",
        NumKind::U16 => "u16",
        NumKind::U32 => "u32",
        NumKind::U64 => "u64",
        NumKind::U128 => "u128",
        NumKind::Usize => "usize",
        NumKind::F32 => "f32",
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
        BinOp::BitAnd => quote! { & },
        BinOp::BitOr => quote! { | },
        BinOp::BitXor => quote! { ^ },
        BinOp::Shl => quote! { << },
        BinOp::Shr => quote! { >> },
        // Pow/FloorDiv/In/NotIn ne passent jamais par lower_binop (value(Binary) les spécial-case, 2.4/2.5).
        BinOp::Pow | BinOp::FloorDiv | BinOp::In | BinOp::NotIn => quote! { + },
    }
}
