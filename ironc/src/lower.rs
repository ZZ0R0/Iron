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
    _own: &OwnershipMap,
    param_modes: &ParamModes,
    bindings: &Bindings,
) -> (TokenStream, SourceMap, Vec<Span>) {
    let mut cx = Cx { sm: SourceMap::default(), types, param_modes, bindings, audit: Vec::new() };
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

struct Cx<'a> {
    sm: SourceMap,
    types: &'a TypeMap,
    param_modes: &'a ParamModes,
    bindings: &'a Bindings,
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
        match &it.kind {
            ItemKind::Fn { name, params, ret, body } => {
                let fid = format_ident!("{}", name);
                let ps: Vec<TokenStream> = params
                    .iter()
                    .enumerate()
                    .map(|(i, (n, t))| {
                        let pid = format_ident!("{}", n);
                        let ty = lower_type(t);
                        match mode_of(self.param_modes, name, i) {
                            ParamMode::ByRef => quote! { #pid: &#ty },
                            ParamMode::ByValue => quote! { #pid: #ty },
                        }
                    })
                    .collect();
                let param_names: HashSet<String> = params.iter().map(|(n, _)| n.clone()).collect();
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
                let ret_ts = match ret {
                    Type::Unit => quote! {},
                    t => {
                        let ty = lower_type(t);
                        quote! { -> #ty }
                    }
                };
                quote! { fn #fid( #(#ps),* ) #ret_ts { #(#decls)* #(#stmts)* } }
            }
            ItemKind::Struct { name, fields } => {
                let sid = format_ident!("{}", name);
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
                    struct #sid { #(#fs),* }
                }
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
                quote! { #id = #v; }
            }
            StmtKind::Assign { target, value } => {
                let v = self.value(value);
                if let ExprKind::Index { base, idx } = &target.kind {
                    let b = self.place(base);
                    let i = self.value(idx);
                    match self.type_of(base) {
                        Type::Dict(..) => quote! { #b.insert(#i, #v); },
                        _ => quote! { #b[(#i) as usize] = #v; },
                    }
                } else {
                    let t = self.place(target);
                    quote! { #t = #v; }
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
                Some(e) => {
                    let e = self.value(e);
                    quote! { return #e; }
                }
                None => quote! { return; },
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
                let id = format_ident!("{}", n);
                quote! { #id }
            }
            ExprKind::Field { base, name } => {
                let b = self.place(base);
                let f = format_ident!("{}", name);
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
                Lit::Int(n) => {
                    let n = proc_macro2::Literal::i64_unsuffixed(*n);
                    quote! { #n }
                }
                Lit::Float(f) => {
                    let f = proc_macro2::Literal::f64_unsuffixed(*f);
                    quote! { #f }
                }
                Lit::Bool(b) => quote! { #b },
                Lit::Str(s) => quote! { #s.to_string() },
            },
            ExprKind::Var(n) => {
                if n == "none" {
                    return quote! { None };
                }
                let id = format_ident!("{}", n);
                if is_copy(&self.type_of(e)) {
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
                let b = self.place(base);
                let f = format_ident!("{}", name);
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
        }
    }

    fn call(&mut self, callee: &Expr, args: &[Expr]) -> TokenStream {
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
                let byref = modes
                    .as_ref()
                    .and_then(|m| m.get(i))
                    .map_or(false, |m| *m == ParamMode::ByRef);
                if byref {
                    let p = self.place(x);
                    quote! { &#p }
                } else {
                    self.value(x)
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
            _ => {}
        }
    }
}

/// Annoter une déclaration hoistée ? Oui si le type est **concret** (sans `Unit` — sinon on laisse
/// l'inférence Rust trancher, p. ex. liste vide).
fn is_annotatable(t: &Type) -> bool {
    match t {
        Type::Unit => false,
        Type::List(e) | Type::Option(e) | Type::Result(e) => is_annotatable(e),
        Type::Dict(k, v) => is_annotatable(k) && is_annotatable(v),
        _ => true,
    }
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
