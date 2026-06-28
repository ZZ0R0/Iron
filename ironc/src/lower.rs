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
    clean: bool,
) -> (TokenStream, SourceMap, Vec<Span>) {
    let enums: HashSet<String> = ast
        .items
        .iter()
        .filter_map(|it| match &it.kind {
            ItemKind::Enum { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();
    let structs: HashSet<String> = ast
        .items
        .iter()
        .filter_map(|it| match &it.kind {
            ItemKind::Struct { name, .. } => Some(name.clone()),
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
        item_attrs: &ast.item_attrs,
        clean,
        enums,
        structs,
        user_fns,
        fn_params,
        fn_defaults,
        method_defaults,
        own,
        inout_params: HashSet::new(),
        byref_params: HashSet::new(),
        flow_wrap: None,
        cur_err: None,
        try_break: None,
        try_counter: 0,
        loop_else: None,
        loop_else_counter: 0,
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
    // Mode propre (`ironc emit`) : pas de marqueurs de source-map → Rust lisible, reprenable à la main.
    let (allow, mark_macro) = if clean {
        (
            quote! { #![allow(dead_code, unused_variables, unused_mut, unused_parens, unused_imports)] },
            quote! {},
        )
    } else {
        (
            quote! { #![allow(dead_code, unused_macros, unused_variables, unused_mut, unused_parens, unused_imports)] },
            quote! { macro_rules! __irn_mark { ($n:literal) => {}; } },
        )
    };
    let file = quote! {
        #![forbid(unsafe_code)]
        #allow

        #mark_macro

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
    /// Attributs Rust pass-through par item (V9, 51900) : `NodeId` → `derive(Serialize)`/`repr(C)`…
    item_attrs: &'a HashMap<NodeId, Vec<String>>,
    clean: bool,
    enums: HashSet<String>,
    structs: HashSet<String>,
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
    /// Type d'erreur `E` de la fonction courante (V9.1, 21500) : `Some(E)` si `Result[T, E]` explicite,
    /// `None` = défaut `IronError`. Gouverne `raise`/`err(...)`/`try`-`?` (typage de la propagation).
    cur_err: Option<Type>,
    /// Id du bloc `try` en cours (V9, 40300) : un `?` y devient `match { Ok=>v, Err=>{…; break 'label} }`
    /// au lieu de `?` — les assignations du `try` touchent les locaux hoistés (pas de capture non-init).
    try_break: Option<u32>,
    try_counter: u32,
    /// Boucle `for/while … else` en cours (V11, DD-2) : un `break` y met le drapeau `__brokeN` à vrai (et
    /// le `else` ne s'exécute qu'à `false`). `None` hors d'une telle boucle ; sauvé/restauré sur imbrication.
    loop_else: Option<u32>,
    loop_else_counter: u32,
    audit: Vec<Span>,
}

fn is_copy(t: &Type) -> bool {
    matches!(t, Type::I64 | Type::F64 | Type::Bool | Type::Unit | Type::Num(_) | Type::Char)
}

/// Le corps contient-il un `yield` → générateur (V9, 10900) : accumulation eager dans `__gen`.
fn body_has_yield(body: &[Stmt]) -> bool {
    body.iter().any(|s| match &s.kind {
        StmtKind::Yield(_) => true,
        StmtKind::If { then, elifs, els, .. } => {
            body_has_yield(then) || elifs.iter().any(|(_, b)| body_has_yield(b)) || body_has_yield(els)
        }
        StmtKind::While { body, .. } | StmtKind::For { body, .. } => body_has_yield(body),
        StmtKind::Match { arms, .. } => arms.iter().any(|a| body_has_yield(&a.body)),
        StmtKind::TryExcept { body, handler, .. } => body_has_yield(body) || body_has_yield(handler),
        _ => false,
    })
}

/// Un paramètre est-il **réassigné** dans le corps (`n = …`) → `mut` dans la signature Rust (pattern Python).
fn param_reassigned(name: &str, body: &[Stmt]) -> bool {
    body.iter().any(|s| match &s.kind {
        StmtKind::Let { name: n, .. } => n == name,
        // réassignation OU mutation d'élément/champ : `p = …`, `p[i] = …`, `p.f = …` → le param doit être `mut`.
        StmtKind::Assign { target, .. } => target_base_var(target) == Some(name),
        // mutation par méthode : `p.append(x)`, `p.sort()`, … → idem `mut`.
        StmtKind::ExprStmt(e) => expr_mutates_var(e, name),
        StmtKind::If { then, elifs, els, .. } => {
            param_reassigned(name, then)
                || elifs.iter().any(|(_, b)| param_reassigned(name, b))
                || param_reassigned(name, els)
        }
        StmtKind::While { body, .. } | StmtKind::For { body, .. } => param_reassigned(name, body),
        StmtKind::Match { arms, .. } => arms.iter().any(|a| param_reassigned(name, &a.body)),
        StmtKind::TryExcept { body, handler, .. } => {
            param_reassigned(name, body) || param_reassigned(name, handler)
        }
        _ => false,
    })
}

/// Variable de **base** d'une lvalue (`p` dans `p`, `p[i]`, `p.f`, `p[i].f`…).
fn target_base_var(e: &Expr) -> Option<&str> {
    match &e.kind {
        ExprKind::Var(n) => Some(n),
        ExprKind::Index { base, .. } | ExprKind::Field { base, .. } => target_base_var(base),
        _ => None,
    }
}

/// `name.append(x)` / `.sort()` / … (méthode mutante sur la variable `name`).
fn expr_mutates_var(e: &Expr, name: &str) -> bool {
    if let ExprKind::Call { callee, .. } = &e.kind {
        if let ExprKind::Field { base, name: m } = &callee.kind {
            if is_mutating_method(m) {
                return target_base_var(base) == Some(name);
            }
        }
    }
    false
}

/// Une méthode mute-t-elle `self` → `&mut self` : affectation à `self.champ`/`self[i]` OU appel d'une
/// méthode mutante sur un champ de `self` (`self.items.append(x)`). (surface Python)
fn stmts_mutate_self(body: &[Stmt]) -> bool {
    body.iter().any(|s| match &s.kind {
        StmtKind::Assign { target, .. } => expr_is_self_place(target),
        StmtKind::ExprStmt(e) | StmtKind::Let { value: e, .. } => expr_mutates_self(e),
        StmtKind::If { then, elifs, els, .. } => {
            stmts_mutate_self(then) || elifs.iter().any(|(_, b)| stmts_mutate_self(b)) || stmts_mutate_self(els)
        }
        StmtKind::While { body, .. } | StmtKind::For { body, .. } => stmts_mutate_self(body),
        StmtKind::Match { arms, .. } => arms.iter().any(|a| stmts_mutate_self(&a.body)),
        _ => false,
    })
}

/// Place dérivée de `self` (`self.champ`, `self[i]`, `self.a.b`).
fn expr_is_self_place(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Var(n) => n == "self",
        ExprKind::Field { base, .. } | ExprKind::Index { base, .. } => expr_is_self_place(base),
        _ => false,
    }
}

/// Méthode built-in qui mute son receveur (`xs.append(x)`, `s.insert`, …).
fn is_mutating_method(name: &str) -> bool {
    matches!(
        name,
        "append" | "push" | "pop" | "sort" | "reverse" | "insert" | "remove" | "add" | "clear" | "extend"
    )
}

fn expr_mutates_self(e: &Expr) -> bool {
    match &e.kind {
        ExprKind::Call { callee, args } => {
            let direct = matches!(&callee.kind,
                ExprKind::Field { base, name } if expr_is_self_place(base) && is_mutating_method(name));
            direct || args.iter().any(expr_mutates_self)
        }
        ExprKind::Binary { lhs, rhs, .. } => expr_mutates_self(lhs) || expr_mutates_self(rhs),
        ExprKind::Unary { expr, .. } | ExprKind::Field { base: expr, .. } => expr_mutates_self(expr),
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
            | Type::ResultE(..)
    )
}

fn mode_of(pm: &ParamModes, fn_name: &str, i: usize) -> ParamMode {
    pm.get(fn_name).and_then(|v| v.get(i)).copied().unwrap_or(ParamMode::ByValue)
}

impl Cx<'_> {
    fn type_of(&self, e: &Expr) -> Type {
        self.types.get(&e.id).cloned().unwrap_or(Type::Unit)
    }

    /// Erreur typée capturée par un `try` (21500) : type `E` du premier `?` du corps dont l'expression est
    /// un `Result[_, E]` typé ; `None` = `IronError` (défaut). Indépendant du retour de la fn englobante —
    /// un `try/except` qui récupère et renvoie une valeur ordinaire doit quand même typer son `e: E`.
    fn caught_err_ty(&self, stmts: &[Stmt]) -> Option<Type> {
        for s in stmts {
            let found = match &s.kind {
                StmtKind::Let { value, .. } | StmtKind::ExprStmt(value) | StmtKind::Yield(value) => self.q_err(value),
                StmtKind::Assign { target, value } => self.q_err(target).or_else(|| self.q_err(value)),
                StmtKind::Return { value } => value.as_ref().and_then(|v| self.q_err(v)),
                StmtKind::If { cond, then, elifs, els } => self
                    .q_err(cond)
                    .or_else(|| self.caught_err_ty(then))
                    .or_else(|| elifs.iter().find_map(|(c, b)| self.q_err(c).or_else(|| self.caught_err_ty(b))))
                    .or_else(|| self.caught_err_ty(els)),
                StmtKind::While { cond, body, .. } => self.q_err(cond).or_else(|| self.caught_err_ty(body)),
                StmtKind::For { iter, body, .. } => self.q_err(iter).or_else(|| self.caught_err_ty(body)),
                StmtKind::Match { scrutinee, arms } => {
                    self.q_err(scrutinee).or_else(|| arms.iter().find_map(|a| self.caught_err_ty(&a.body)))
                }
                _ => None,
            };
            if found.is_some() {
                return found;
            }
        }
        None
    }

    /// `E` typé d'un `?` (`Try`) dans une expression, en profondeur ; `None` si aucun `?` typé.
    fn q_err(&self, e: &Expr) -> Option<Type> {
        match &e.kind {
            ExprKind::Try { inner } => {
                if let Type::ResultE(_, et) = self.type_of(inner) {
                    return Some(*et);
                }
                self.q_err(inner)
            }
            ExprKind::Unary { expr, .. } => self.q_err(expr),
            ExprKind::Binary { lhs, rhs, .. } => self.q_err(lhs).or_else(|| self.q_err(rhs)),
            ExprKind::Call { callee, args } => self.q_err(callee).or_else(|| args.iter().find_map(|a| self.q_err(a))),
            ExprKind::Field { base, .. } => self.q_err(base),
            ExprKind::Index { base, idx } => self.q_err(base).or_else(|| self.q_err(idx)),
            ExprKind::StructLit { fields, .. } => fields.iter().find_map(|(_, v)| self.q_err(v)),
            ExprKind::ListLit { items } => items.iter().find_map(|i| self.q_err(i)),
            ExprKind::DictLit { pairs } => pairs.iter().find_map(|(k, v)| self.q_err(k).or_else(|| self.q_err(v))),
            _ => None,
        }
    }

    /// Attributs d'un item (V9, 51900) : fusionne les `@derive(...)` user avec les derives auto, émet les
    /// autres attributs (`repr(C)`/`inline`…) en `#[…]` verbatim.
    fn item_attrs_ts(&self, id: NodeId, auto: &[&str]) -> TokenStream {
        let mut derives: Vec<String> = auto.iter().map(|s| s.to_string()).collect();
        let mut others: Vec<TokenStream> = Vec::new();
        if let Some(list) = self.item_attrs.get(&id) {
            for a in list {
                let a = a.trim();
                if let Some(inner) = a.strip_prefix("derive(").and_then(|s| s.strip_suffix(')')) {
                    for d in inner.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                        if !derives.iter().any(|x| x == d) {
                            derives.push(d.to_string());
                        }
                    }
                } else if let Ok(ts) = TokenStream::from_str(a) {
                    others.push(quote! { #[#ts] }); // `repr(C)`, `inline(never)`, …
                }
            }
        }
        let derive_ts = TokenStream::from_str(&derives.join(", ")).unwrap_or_default();
        quote! { #[derive(#derive_ts)] #(#others)* }
    }

    fn item(&mut self, it: &Item) -> TokenStream {
        self.flow_wrap = None; // (les méthodes d'impl ne font pas d'inférence flux dans cet incrément)
        match &it.kind {
            ItemKind::Fn { name, generics, params, ret, body, is_async, .. } => {
                let fid = format_ident!("{}", name);
                let gens = lower_generics(generics);
                let ps: Vec<TokenStream> = params
                    .iter()
                    .enumerate()
                    .map(|(i, (n, t))| {
                        let pid = format_ident!("{}", n);
                        let ty = lower_type(t);
                        // `mut` si le param est réassigné dans le corps (`n = …`) — pattern Python.
                        let mut_kw = if param_reassigned(n, body) {
                            quote! { mut }
                        } else {
                            quote! {}
                        };
                        match mode_of(self.param_modes, name, i) {
                            ParamMode::ByRef => quote! { #pid: &#ty },
                            // InOut : `ty` est déjà `&mut T` (lower_type(Ref)).
                            ParamMode::ByValue | ParamMode::InOut => quote! { #mut_kw #pid: #ty },
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
                // I11 complet (V9, 40400) : on emballe dès que la fn est flux-Option/Result (annotée OU
                // inférée) ; le wrap au `return` est **conscient du type** (n'emballe pas un expr déjà
                // Option/Result), ce qui supprime le `ok(...)`/`some(...)` manuel.
                self.flow_wrap = match &effective {
                    Type::Option(_) => Some(FlowWrap::Some_),
                    Type::Result(_) | Type::ResultE(..) => Some(FlowWrap::Ok_),
                    _ => None,
                };
                // Erreur typée (21500) : `E` de la fn courante pour `raise`/`err`/`try` ; `None` = IronError.
                self.cur_err = match &effective {
                    Type::ResultE(_, e) => Some((**e).clone()),
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
                    && !crate::infer::engine::flux::definitely_returns(body)
                {
                    quote! { None }
                } else {
                    quote! {}
                };
                // Générateur (V9, 10900) : `yield` → `__gen` accumulé puis renvoyé.
                let is_gen = body_has_yield(body);
                let gen_init = if is_gen { quote! { let mut __gen = Vec::new(); } } else { quote! {} };
                let tail = if is_gen { quote! { __gen } } else { tail };
                // `def test_*()` → `#[test]` (ironc test) ; ignoré au build/run normal (cfg(test)).
                let test_attr = if name.starts_with("test_") {
                    quote! { #[test] }
                } else {
                    quote! {}
                };
                // `pub` : items réutilisables si exportés en crate (`ironc lib`, 51800) ; inoffensif en binaire.
                // async (V9, 21200) : `async fn` ; `main` async → `#[tokio::main]` (transforme + lance le runtime).
                let header = if *is_async && name == "main" {
                    quote! { #[tokio::main] async }
                } else if *is_async {
                    quote! { #test_attr pub async }
                } else {
                    quote! { #test_attr pub }
                };
                quote! { #header fn #fid #gens ( #(#ps),* ) #ret_ts { #gen_init #(#decls)* #(#stmts)* #tail } }
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
                let attrs = self.item_attrs_ts(it.id, &["Debug", "Clone"]);
                quote! {
                    #attrs
                    pub struct #sid #gens { #(pub #fs),* }
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
                let attrs = self.item_attrs_ts(it.id, &["Debug", "Clone"]);
                quote! {
                    #attrs
                    pub enum #eid #gens { #(#vs),* }
                }
            }
            ItemKind::Trait { name, methods } => {
                let tid = format_ident!("{}", name);
                let ms: Vec<TokenStream> = methods.iter().map(|m| self.lower_method(m)).collect();
                // `pub` : un trait privé borné sur une `pub fn` déclenche `private_bounds` (cohérent avec
                // struct/fn publics).
                quote! { pub trait #tid { #(#ms)* } }
            }
            ItemKind::Impl { trait_name, type_name, methods } => {
                let ty = format_ident!("{}", type_name);
                if trait_name.is_empty() {
                    // impl inhérent : les dunders bénis (V11, DC-4) sont extraits en `impl std::ops::*`
                    // (+ un éventuel helper inhérent) ; le reste forme l'impl inhérent normal.
                    let mut normal: Vec<TokenStream> = Vec::new();
                    let mut dunders: Vec<TokenStream> = Vec::new();
                    for m in methods {
                        if is_blessed_dunder(&m.name) {
                            let (helper, trait_impl) = self.dunder_impl(type_name, m);
                            if let Some(h) = helper {
                                normal.push(h);
                            }
                            dunders.push(trait_impl);
                        } else {
                            normal.push(self.lower_method(m));
                        }
                    }
                    let inherent =
                        if normal.is_empty() { quote! {} } else { quote! { impl #ty { #(#normal)* } } };
                    quote! { #inherent #(#dunders)* }
                } else {
                    let ms: Vec<TokenStream> = methods.iter().map(|m| self.lower_method(m)).collect();
                    let tid = format_ident!("{}", trait_name);
                    quote! { impl #tid for #ty { #(#ms)* } }
                }
            }
            // Constante globale immuable (V11, DD-bonus) → `const NAME: T = <littéral>;`. Les chaînes →
            // `&str` (une `String` n'est pas constructible en contexte `const`).
            ItemKind::Const { name, ty, value } => {
                let id = format_ident!("{}", name);
                match &value.kind {
                    ExprKind::Lit(Lit::Str(s)) => quote! { const #id: &str = #s; },
                    ExprKind::Lit(Lit::Int(n)) => {
                        let n = proc_macro2::Literal::i64_unsuffixed(*n);
                        quote! { const #id: i64 = #n; }
                    }
                    ExprKind::Lit(Lit::Float(f)) => {
                        let f = proc_macro2::Literal::f64_unsuffixed(*f);
                        quote! { const #id: f64 = #f; }
                    }
                    ExprKind::Lit(Lit::Bool(b)) => quote! { const #id: bool = #b; },
                    ExprKind::Lit(Lit::Char(c)) => quote! { const #id: char = #c; },
                    _ => {
                        let t = lower_type(ty);
                        let v = self.value(value);
                        quote! { const #id: #t = #v; }
                    }
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
            Some(_) => {
                let block = self.method_body_block(m);
                quote! { fn #mid( #sig_params ) #ret_ts #block }
            }
        }
    }

    /// Bloc `{ decls; stmts }` d'un corps de méthode (locaux hoistés + statements). Partagé par
    /// `lower_method` et le désucrage des dunders (V11, DC-4), qui forgent leur propre signature.
    fn method_body_block(&mut self, m: &Method) -> TokenStream {
        let body = m.body.as_ref().expect("method_body_block sur méthode sans corps");
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
        quote! { { #(#decls)* #(#stmts)* } }
    }

    /// Désucre un dunder béni (V11, DC-4) en `(helper inhérent optionnel, impl de trait std)`.
    /// Convention v0 : opérateurs **homogènes** (Rhs = Self). `__getitem__` est différé (retour `&Output`,
    /// incompatible avec la sémantique par valeur sans analyse de durée de vie).
    fn dunder_impl(&mut self, type_name: &str, m: &Method) -> (Option<TokenStream>, TokenStream) {
        let ty = format_ident!("{}", type_name);
        let block = self.method_body_block(m);
        let rhs_name = m.params.first().map(|(n, _)| n.clone()).unwrap_or_else(|| "rhs".to_string());
        let rhs = format_ident!("{}", rhs_name);
        match m.name.as_str() {
            // Arithmétiques : `self`/`rhs` par valeur, `type Output` = retour annoté (sinon Self).
            "__add__" | "__sub__" | "__mul__" | "__div__" => {
                let (trait_id, meth) = match m.name.as_str() {
                    "__add__" => ("Add", "add"),
                    "__sub__" => ("Sub", "sub"),
                    "__mul__" => ("Mul", "mul"),
                    _ => ("Div", "div"),
                };
                let trait_id = format_ident!("{}", trait_id);
                let meth = format_ident!("{}", meth);
                let out = match &m.ret {
                    Type::Unit => quote! { #ty },
                    t => lower_type(t),
                };
                let trait_impl = quote! {
                    impl std::ops::#trait_id for #ty {
                        type Output = #out;
                        fn #meth(self, #rhs: #ty) -> #out #block
                    }
                };
                (None, trait_impl)
            }
            // Égalité : `fn eq(&self, o: &Self) -> bool`.
            "__eq__" => {
                let trait_impl = quote! {
                    impl PartialEq for #ty {
                        fn eq(&self, #rhs: &#ty) -> bool #block
                    }
                };
                (None, trait_impl)
            }
            // Ordre : helper inhérent `__lt` + `partial_cmp` dérivé (nécessite `__eq__` pour le sur-trait
            // PartialEq). `__lt` appelé dans les deux sens → Less/Greater/Equal.
            "__lt__" => {
                let helper = quote! {
                    fn __lt(&self, #rhs: &#ty) -> bool #block
                };
                let trait_impl = quote! {
                    impl PartialOrd for #ty {
                        fn partial_cmp(&self, other: &#ty) -> Option<std::cmp::Ordering> {
                            if self.__lt(other) {
                                Some(std::cmp::Ordering::Less)
                            } else if other.__lt(self) {
                                Some(std::cmp::Ordering::Greater)
                            } else {
                                Some(std::cmp::Ordering::Equal)
                            }
                        }
                    }
                };
                (Some(helper), trait_impl)
            }
            // Affichage : helper inhérent `__str` (→ String) + `Display` qui délègue.
            "__str__" => {
                let helper = quote! {
                    fn __str(&self) -> String #block
                };
                let trait_impl = quote! {
                    impl std::fmt::Display for #ty {
                        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                            write!(f, "{}", self.__str())
                        }
                    }
                };
                (Some(helper), trait_impl)
            }
            // tout autre dunder béni non encore désucré → méthode inhérente normale (filet : pas d'opérateur).
            _ => (Some(self.lower_method(m)), quote! {}),
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
            StmtKind::Yield(e) => {
                // V9 (10900) : générateur → accumulation eager dans `__gen` (Vec), la fn renvoie `__gen`.
                let v = self.value(e);
                quote! { __gen.push(#v); }
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
            StmtKind::While { cond, body, els } => {
                let c = self.value(cond);
                let (b, els_ts, flag) = self.lower_loop_body(body, els);
                match flag {
                    Some(fl) => quote! { let mut #fl = false; while #c { #(#b)* } if !#fl { #(#els_ts)* } },
                    None => quote! { while #c { #(#b)* } },
                }
            }
            StmtKind::For { var, iter, body, els } => {
                let vid = format_ident!("{}", var);
                let it = self.for_iter(iter);
                let (b, els_ts, flag) = self.lower_loop_body(body, els);
                match flag {
                    Some(fl) => quote! { let mut #fl = false; for #vid in #it { #(#b)* } if !#fl { #(#els_ts)* } },
                    None => quote! { for #vid in #it { #(#b)* } },
                }
            }
            // `Seq` (désucrage, V11) : statements en séquence, pas de portée bloc (locaux hoistés).
            StmtKind::Seq(stmts) => {
                let ss: Vec<TokenStream> = stmts.iter().map(|s| self.stmt(s)).collect();
                quote! { #(#ss)* }
            }
            StmtKind::Break => {
                // dans une boucle `… else`, un `break` lève le drapeau (et saute le `else`).
                match self.loop_else {
                    Some(n) => {
                        let fl = format_ident!("__broke{}", n);
                        quote! { { #fl = true; break; } }
                    }
                    None => quote! { break; },
                }
            }
            StmtKind::Continue => quote! { continue; },
            StmtKind::Return { value } => match value {
                Some(e) if matches!(&e.kind, ExprKind::Var(n) if n == "none" || n == "None") => {
                    // `none`/`None` (constructeur Option) → toujours `None`, jamais emballé.
                    quote! { return None; }
                }
                Some(e) => {
                    let et = self.type_of(e);
                    let ev = self.return_value(e);
                    // I11 conscient du type : n'emballe que si l'expr n'est pas DÉJÀ Option/Result
                    // (`return ok(x)`/`return compile(p)?` ... → laissés tels quels ; `return bool` → `Ok(bool)`).
                    match self.flow_wrap {
                        Some(FlowWrap::Some_) if !matches!(et, Type::Option(_)) => quote! { return Some(#ev); },
                        Some(FlowWrap::Ok_) if !matches!(et, Type::Result(_) | Type::ResultE(..)) => quote! { return Ok(#ev); },
                        _ => quote! { return #ev; },
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
                // V9 (40300) : bloc labellisé + `?` réécrit en break — les assignations du `try` touchent
                // les locaux hoistés directement (plus de capture non-initialisée par une IIFE-closure).
                let id = self.try_counter;
                self.try_counter += 1;
                let prev = self.try_break;
                self.try_break = Some(id);
                let body_ts: Vec<TokenStream> = body.iter().map(|s| self.stmt(s)).collect();
                self.try_break = prev;
                let handler_ts: Vec<TokenStream> = handler.iter().map(|s| self.stmt(s)).collect();
                let lbl = syn::Lifetime::new(&format!("'__try{id}"), proc_macro2::Span::call_site());
                // Erreur typée (21500) : le réceptacle `__try_err` porte le `E` **capturé par le corps** (le
                // `?` typé), sinon `IronError`. Le `except e:` lie `e = __err` (valeur `E` entière) en typé,
                // ou `__err.0` (le message) pour `IronError` (rétro-compat).
                let caught = self.caught_err_ty(body);
                let errty = match &caught {
                    Some(e) => lower_type(e),
                    None => quote! { IronError },
                };
                let err_bind = match err_name {
                    Some(e) => {
                        let eid = format_ident!("{}", e);
                        if caught.is_some() {
                            quote! { let #eid = __err; }
                        } else {
                            quote! { let #eid = __err.0; }
                        }
                    }
                    None => quote! {},
                };
                // Totalité (V9.1) : si le corps **diverge sur le chemin de succès** (return/raise sur tous
                // les chemins), le post-bloc n'est atteint **que** via le `break` du `?` ⇒ `__try_err` est
                // forcément `Some`. On rend alors le `if let` total (`else unreachable!`) pour qu'une fn qui
                // renvoie une valeur (try/except au tail) compile — sinon `rustc` voit une chute non typée.
                let total = crate::infer::engine::flux::definitely_returns(body);
                let else_arm = if total {
                    quote! { else { unreachable!("le corps du `try` retourne sur le chemin sans erreur") } }
                } else {
                    quote! {}
                };
                quote! {
                    let mut __try_err: Option<#errty> = None;
                    #lbl: { #(#body_ts)* }
                    if let Some(__err) = __try_err {
                        #err_bind
                        #(#handler_ts)*
                    } #else_arm
                }
            }
            StmtKind::Raise { value } => {
                let v = self.value(value);
                // Erreur typée (21500) : `raise e` dans une fn `Result[T, E]` → `Err(e.into())` (identité si
                // `e: E`, conversion si `e: E1` avec `E1: Into<E>` — filet rustc). Défaut = `IronError` (texte).
                if self.cur_err.is_some() {
                    quote! { return Err((#v).into()); }
                } else if matches!(self.type_of(value), Type::Str) {
                    quote! { return Err(IronError(#v)); }
                } else {
                    quote! { return Err(IronError(format!("{}", #v))); }
                }
            }
        };
        // Mode propre : pas de marqueur (Rust lisible). Sinon : marqueur de source-map (debug I8).
        if self.clean {
            body
        } else {
            quote! { __irn_mark!(#mark); #body }
        }
    }

    /// Lowering du corps d'une boucle + son `else` (V11, DD-2). Sauve/restaure `loop_else` → les `break` du
    /// corps lèvent le drapeau de CETTE boucle (si `else`), pas celui d'une boucle englobante ; une boucle
    /// imbriquée sans `else` remet `loop_else = None` (ses `break` sont de simples `break`).
    fn lower_loop_body(
        &mut self,
        body: &[Stmt],
        els: &[Stmt],
    ) -> (Vec<TokenStream>, Vec<TokenStream>, Option<proc_macro2::Ident>) {
        let prev = self.loop_else;
        let flag = if els.is_empty() {
            self.loop_else = None;
            None
        } else {
            let n = self.loop_else_counter;
            self.loop_else_counter += 1;
            self.loop_else = Some(n);
            Some(format_ident!("__broke{}", n))
        };
        let b: Vec<TokenStream> = body.iter().map(|s| self.stmt(s)).collect();
        self.loop_else = prev; // l'`else` s'exécute hors de la boucle → contexte englobant
        let els_ts: Vec<TokenStream> = els.iter().map(|s| self.stmt(s)).collect();
        (b, els_ts, flag)
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
            // `for c in s` (Python) → caractères Rust `char` (rapide, sans allocation). `ord(c)`/`c == "x"`/
            // `s + c` gérés pour les `char` (cf. ord/Binary). Python n'a pas de type char : `c` se comporte
            // comme une chaîne d'un caractère.
            Type::Str => quote! { #it.chars() },
            // type externe → itération **par valeur** : la boucle `for` Rust appelle `into_iter`, ce qui
            // couvre à la fois les `impl IntoIterator` (`WalkDir`) ET les `impl Iterator` (`StringRecordsIter`,
            // via le blanket `IntoIterator for I: Iterator`). Un externe non itérable échouera proprement.
            Type::Extern(_) => quote! { #it },
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
                    // str n'est jamais une lvalue d'index → accès caractère par valeur (indices négatifs gérés).
                    Type::Str => quote! {{
                        let __s = &#b;
                        let __i = (#i) as i64;
                        let __n = __s.chars().count() as i64;
                        let __k = (if __i < 0 { __n + __i } else { __i }) as usize;
                        __s.chars().nth(__k).unwrap().to_string()
                    }},
                    _ => quote! { #b[(#i) as usize] },
                }
            }
            _ => self.value(e),
        }
    }

    /// Argument d'affichage (`print`) : emprunte une place simple (Var/Field — pas de consommation), mais
    /// **calcule** un Index/Slice/etc. via `value` (indices négatifs, chars de str gérés correctement).
    fn display_arg(&mut self, e: &Expr) -> TokenStream {
        match &e.kind {
            ExprKind::Var(_) | ExprKind::Field { .. } => self.place(e),
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
                    // `+` sur chaînes/char (Python) : concaténation. Couvre `s + c`/`c + s`/`c + c` (char
                    // d'une boucle `for c in s`) → `format!` (Rust `String + String`/`char + char` échouent).
                    BinOp::Add
                        if matches!(self.type_of(lhs), Type::Str | Type::Char)
                            || matches!(self.type_of(rhs), Type::Str | Type::Char) =>
                    {
                        let r = self.value(rhs);
                        quote! { format!("{}{}", #l, #r) }
                    }
                    // `a + b` sur listes (Python) : concaténation → nouvelle liste.
                    BinOp::Add if matches!(self.type_of(lhs), Type::List(_)) => {
                        let r = self.value(rhs);
                        quote! { { let mut __v = #l; __v.extend(#r); __v } }
                    }
                    // comparaisons : `char` vs littéral chaîne d'1 caractère → comparaison de `char` (rapide).
                    BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                        let o = lower_binop(*op);
                        let lc = matches!(self.type_of(lhs), Type::Char);
                        let rc = matches!(self.type_of(rhs), Type::Char);
                        if lc && !rc {
                            if let Some(ch) = str_lit_char(rhs) {
                                let cl = proc_macro2::Literal::character(ch);
                                return quote! { (#l #o #cl) };
                            }
                        }
                        if rc && !lc {
                            if let Some(ch) = str_lit_char(lhs) {
                                let cl = proc_macro2::Literal::character(ch);
                                let r = self.value(rhs);
                                return quote! { (#cl #o #r) };
                            }
                        }
                        let r = self.value(rhs);
                        quote! { (#l #o #r) }
                    }
                    // `[x] * n` / `n * [x]` (Python) : répétition de liste. Idem `"ab" * n` pour les chaînes.
                    BinOp::Mul if matches!(self.type_of(lhs), Type::List(_) | Type::Str) => {
                        let r = self.value(rhs);
                        quote! { (#l).repeat((#r) as usize) }
                    }
                    BinOp::Mul if matches!(self.type_of(rhs), Type::List(_) | Type::Str) => {
                        let r = self.value(rhs);
                        quote! { (#r).repeat((#l) as usize) }
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
                    // variante d'enum **externe** (`SampleFormat.Int`, 52400) → `crate::path::Variant`.
                    if let Some(et) = self.bindings.etypes.get(ename) {
                        if et.variants.iter().any(|v| v == name) {
                            let path = TokenStream::from_str(&et.rust_path).unwrap_or_else(|_| quote! { () });
                            let vid = format_ident!("{}", name);
                            return quote! { #path::#vid };
                        }
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
                match self.type_of(base) {
                    Type::Dict(..) => {
                        if is_copy(&self.type_of(e)) {
                            quote! { #b[&#i] }
                        } else {
                            self.audit.push(e.span);
                            quote! { #b[&#i].clone() }
                        }
                    }
                    // `s[i]` (Python) → str d'un caractère ; indices négatifs depuis la fin.
                    Type::Str => quote! {{
                        let __s = &#b;
                        let __i = (#i) as i64;
                        let __n = __s.chars().count() as i64;
                        let __k = (if __i < 0 { __n + __i } else { __i }) as usize;
                        __s.chars().nth(__k).unwrap().to_string()
                    }},
                    // liste : index négatif depuis la fin ; clone si non-Copy.
                    _ => {
                        let non_copy = !is_copy(&self.type_of(e));
                        if non_copy {
                            self.audit.push(e.span);
                        }
                        let tail = if non_copy { quote! { .clone() } } else { quote! {} };
                        quote! {{
                            let __b = &#b;
                            let __i = (#i) as i64;
                            let __n = __b.len() as i64;
                            let __k = (if __i < 0 { __n + __i } else { __i }) as usize;
                            __b[__k] #tail
                        }}
                    }
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
                // struct **externe** à champs publics (52400) → `crate::path::S { champ: (val) as T, … }`
                // (chemin Rust complet ; coercition numérique par champ, **sans** `&` — champs par valeur).
                if let Some(et) = self.bindings.etypes.get(name).filter(|e| !e.fields.is_empty()).cloned() {
                    let path = TokenStream::from_str(&et.rust_path).unwrap_or_else(|_| quote! { () });
                    let fs: Vec<TokenStream> = fields
                        .iter()
                        .map(|(fname, fexpr)| {
                            let fid = format_ident!("{}", fname);
                            let declared = et.fields.iter().find(|(n, _)| n == fname).map(|(_, t)| t);
                            let v = self.coerce_to(fexpr, declared);
                            quote! { #fid: #v }
                        })
                        .collect();
                    return quote! { #path { #(#fs),* } };
                }
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
                match self.try_break {
                    Some(id) => {
                        let lbl = syn::Lifetime::new(&format!("'__try{id}"), proc_macro2::Span::call_site());
                        // `.into()` : identité pour `IronError`, conversion `E1→E2` en erreur typée (21500).
                        quote! { match #x { Ok(__v) => __v, Err(__e) => { __try_err = Some(__e.into()); break #lbl; } } }
                    }
                    None => quote! { (#x?) },
                }
            }
            // `await e` (V9, 21200) → `(e).await`.
            ExprKind::Await(inner) => {
                let v = self.value(inner);
                quote! { (#v).await }
            }
            ExprKind::Lambda { params, body } => {
                // Closure Rust (20700) : types des params inférés par rustc (I2). `move` (V9.1, 21600) →
                // capture par valeur : nécessaire pour une closure **retournée**/**stockée**, sûr pour map/filter.
                let ps: Vec<proc_macro2::Ident> = params.iter().map(|p| format_ident!("{}", p)).collect();
                let b = self.value(body);
                quote! { move | #(#ps),* | #b }
            }
            // Closure à corps de bloc (V11, DD-1) : fonction imbriquée → `move |params| { stmts }`.
            ExprKind::Closure { params, body } => {
                let ps: Vec<proc_macro2::Ident> = params.iter().map(|p| format_ident!("{}", p)).collect();
                let b: Vec<TokenStream> = body.iter().map(|s| self.stmt(s)).collect();
                quote! { move | #(#ps),* | { #(#b)* } }
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
                // Indices Python : bornes négatives = depuis la fin (`len + i`), clampées dans [0, len].
                let b = self.place(base);
                let is_str = matches!(self.type_of(base), Type::Str);
                let lo_raw = match lo {
                    Some(l) => self.value(l),
                    None => quote! { 0i64 },
                };
                let hi_raw = match hi {
                    Some(h) => self.value(h),
                    None => quote! { __n },
                };
                // __n = longueur (chars pour str, len pour list) ; normalise une borne i64 → usize clampé.
                let len_ts =
                    if is_str { quote! { (__base.chars().count() as i64) } } else { quote! { (__base.len() as i64) } };
                let body = if is_str {
                    quote! { __base.chars().skip(__lo).take(__hi.saturating_sub(__lo)).collect::<String>() }
                } else {
                    quote! { __base[__lo..__hi.max(__lo)].to_vec() }
                };
                quote! {{
                    let __base = &#b;
                    let __n = #len_ts;
                    let __lo = { let __x = (#lo_raw) as i64; (if __x < 0 { (__n + __x).max(0) } else { __x.min(__n) }) as usize };
                    let __hi = { let __x = (#hi_raw) as i64; (if __x < 0 { (__n + __x).max(0) } else { __x.min(__n) }) as usize };
                    #body
                }}
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
                // `self` est toujours emprunté (`&self`/`&mut self`, jamais possédé) → on ne peut pas
                // move-hors-`self` : `return self.f` non-Copy ⇒ clone-fallback (via `value`). Sinon, base
                // possédée (param ByValue) ⇒ move partiel sûr (le `return` quitte la fn).
                let owned_base =
                    matches!(&base.kind, ExprKind::Var(bn) if bn != "self" && !self.byref_params.contains(bn));
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
        // modules stdlib namespacés (surface Python, D2) : `io`/`os`/`time` → wrappers `std`.
        if let ExprKind::Field { base, name } = &callee.kind {
            if let ExprKind::Var(m) = &base.kind {
                match m.as_str() {
                    // V9.1 (52000) : io/os/time RETIRÉS — passer par `import std` (zéro lib Iron, D1).
                    "thread" => return self.lower_thread(name, args), // primitive concurrence (V9, 70900)
                    _ => {}
                }
            }
        }
        // handle de thread (V9, 70900) : `h.join()` → `h.join().unwrap()`.
        if let ExprKind::Field { base, name } = &callee.kind {
            if name == "join" {
                if let Type::App(n, _) = self.type_of(base) {
                    if n == "__Handle" {
                        let recv = self.place(base);
                        return quote! { #recv.join().unwrap() };
                    }
                }
            }
        }
        // construction d'une classe **sans champ** : `Name()` où Name est un struct connu → `Name {}`
        // (une classe avec champs se construit en `Name(champ=...)` → StructLit, géré ailleurs).
        if let ExprKind::Var(name) = &callee.kind {
            if args.is_empty() && self.structs.contains(name) {
                let id = format_ident!("{}", name);
                return quote! { #id {} };
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
        // constructeur namespacé `Type.ctor(...)` (V8, 40200) : `Regex.new(...)` → `crate::Regex::new(...)`.
        if let ExprKind::Field { base, name } = &callee.kind {
            if let Some(ts) = self.try_extern_ctor(base, name, args) {
                return ts;
            }
        }
        // méthode externe (V8, 40200) : `obj.m(...)` où `obj : Extern(nom Iron)` — prime sur les built-ins.
        if let ExprKind::Field { base, name } = &callee.kind {
            if let Some(ts) = self.try_extern_method(base, name, args) {
                return ts;
            }
        }
        // méthodes des types built-in (surface Python) : `xs.append(x)`, `s.upper()`, `d.keys()`.
        if let ExprKind::Field { base, name } = &callee.kind {
            if let Some(ts) = self.try_builtin_method(base, name, args) {
                return ts;
            }
        }
        if let ExprKind::Var(name) = &callee.kind {
            // les fns utilisateur priment sur les built-ins homonymes (ex. un `fn add` perso).
            if !self.user_fns.contains(name) {
            match name.as_str() {
                "print" if args.len() == 1 => {
                    let a = self.display_arg(&args[0]);
                    return if is_debug_type(&self.type_of(&args[0])) {
                        quote! { println!("{:?}", #a) }
                    } else {
                        quote! { println!("{}", #a) }
                    };
                }
                "len" if args.len() == 1 => {
                    let a = self.place(&args[0]);
                    // Sémantique Python : `len(str)` = nombre de **caractères** (Unicode), pas d'octets.
                    // list/dict/set → `.len()` (nombre d'éléments).
                    return if matches!(self.type_of(&args[0]), Type::Str) {
                        quote! { (#a.chars().count() as i64) }
                    } else {
                        quote! { (#a.len() as i64) }
                    };
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
                    // Erreur typée (21500) : `err(e)` dans `Result[T, E]` → `Err(e.into())` ; défaut `IronError`.
                    return if self.cur_err.is_some() {
                        quote! { Err((#a).into()) }
                    } else {
                        quote! { Err(IronError(#a)) }
                    };
                }
                // stdlib V4 (80100) — wrappers fins sur std (I5)
                "str" if args.len() == 1 => {
                    let a = self.value(&args[0]);
                    return quote! { (#a).to_string() };
                }
                // ord(c)/chr(n) (V11-test) : code Unicode ↔ caractère. `c` = char (boucle `for c in s`) ou
                // str d'un caractère (indexation/littéral).
                "ord" if args.len() == 1 => {
                    if matches!(self.type_of(&args[0]), Type::Char) {
                        let a = self.value(&args[0]);
                        return quote! { ((#a) as i64) };
                    }
                    let a = self.place(&args[0]);
                    return quote! { (#a.chars().next().unwrap() as i64) };
                }
                "chr" if args.len() == 1 => {
                    let a = self.value(&args[0]);
                    return quote! { (char::from_u32((#a) as u32).unwrap().to_string()) };
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
                // itertools (80700)
                "enumerate" if args.len() == 1 => {
                    let xs = self.value(&args[0]);
                    return quote! { (#xs).into_iter().enumerate().map(|(__i, __x)| (__i as i64, __x)).collect::<Vec<_>>() };
                }
                "zip" if args.len() == 2 => {
                    let xs = self.value(&args[0]);
                    let ys = self.value(&args[1]);
                    return quote! { (#xs).into_iter().zip(#ys).collect::<Vec<_>>() };
                }
                "reversed" if args.len() == 1 => {
                    let xs = self.value(&args[0]);
                    return quote! { { let mut __v = #xs; __v.reverse(); __v } };
                }
                "sorted" if args.len() == 1 => {
                    let xs = self.value(&args[0]);
                    return quote! { { let mut __v = #xs; __v.sort_by(|a, b| a.partial_cmp(b).unwrap()); __v } };
                }
                "any" if args.len() == 1 => {
                    let xs = self.place(&args[0]);
                    return quote! { #xs.iter().any(|__b| *__b) };
                }
                "all" if args.len() == 1 => {
                    let xs = self.place(&args[0]);
                    return quote! { #xs.iter().all(|__b| *__b) };
                }
                "reduce" if args.len() == 3 => {
                    let f = self.fn_arg(&args[0]);
                    let xs = self.value(&args[1]);
                    let init = self.value(&args[2]);
                    return quote! { (#xs).into_iter().fold(#init, #f) };
                }
                // min/max sur une LISTE (Python) : `min(xs)` ; les formes 2-arg restent plus bas.
                "min" if args.len() == 1 => {
                    let xs = self.place(&args[0]);
                    return quote! { #xs.iter().cloned().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() };
                }
                "max" if args.len() == 1 => {
                    let xs = self.place(&args[0]);
                    return quote! { #xs.iter().cloned().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap() };
                }
                // conversions / chaînes
                "join" if args.len() == 2 => {
                    let xs = self.place(&args[0]);
                    let sep = self.value(&args[1]);
                    return quote! { #xs.join((#sep).as_str()) };
                }
                "int" if args.len() == 1 => {
                    let s = self.value(&args[0]);
                    return quote! { (#s).trim().parse::<i64>().unwrap_or(0) };
                }
                "float" if args.len() == 1 => {
                    let s = self.value(&args[0]);
                    return quote! { (#s).trim().parse::<f64>().unwrap_or(0.0) };
                }
                "round" if args.len() == 1 => {
                    let x = self.value(&args[0]);
                    return quote! { ((#x).round() as i64) };
                }
                // concurrence (V9, 30700/70900) : `Arc<Mutex>` partagé entre threads.
                "ref_sync" if args.len() == 1 => {
                    let v = self.value(&args[0]);
                    return quote! { std::sync::Arc::new(std::sync::Mutex::new(#v)) };
                }
                "clone_sync" if args.len() == 1 => {
                    let s = self.place(&args[0]);
                    return quote! { std::sync::Arc::clone(&#s) };
                }
                "lock_get" if args.len() == 1 => {
                    let s = self.place(&args[0]);
                    return quote! { (*#s.lock().unwrap()).clone() };
                }
                "lock_set" if args.len() == 2 => {
                    let s = self.place(&args[0]);
                    let v = self.value(&args[1]);
                    return quote! { { *#s.lock().unwrap() = #v; } };
                }
                "lock_add" if args.len() == 2 => {
                    let s = self.place(&args[0]);
                    let v = self.value(&args[1]);
                    return quote! { { *#s.lock().unwrap() += #v; } };
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
            // fonction externe (`.iri`) → chemin Rust (50200/40200).
            if self.bindings.fns.contains_key(name) {
                let path = TokenStream::from_str(&self.bindings.fns[name].path).unwrap_or_else(|_| quote! { () });
                let is_result = matches!(self.bindings.fns[name].ret, Type::Result(_));
                let a: Vec<TokenStream> = args.iter().map(|x| self.extern_arg(x)).collect();
                let call = quote! { #path( #(#a),* ) };
                // erreur du crate → `IronError` unifié (I13) pour que `?`/`try` propagent.
                return if is_result {
                    quote! { #call.map_err(|e| IronError(format!("{:?}", e))) }
                } else {
                    call
                };
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

    /// Méthodes des types built-in (surface Python, 80500) : `xs.append(x)`, `s.upper()`, `d.keys()`…
    /// `None` = pas une méthode built-in connue → l'appel retombe sur le chemin normal.
    /// Argument d'un appel externe (40200) : réinjecte `&` pour `str`/collections (Rust attend `&str`/
    /// `&[T]` ; Iron a `String`/`Vec` → `&String`/`&Vec` coercent par Deref). Scalaires par valeur.
    fn extern_arg(&mut self, x: &Expr) -> TokenStream {
        let ty = self.type_of(x);
        let v = self.value(x);
        match ty {
            // `str`/collections/struct/extern → `&` (les API Rust prennent surtout des emprunts ; coerce par Deref).
            Type::Str | Type::List(_) | Type::Dict(_, _) | Type::Set(_) | Type::Struct(_) | Type::Extern(_) => {
                quote! { &#v }
            }
            _ => v,
        }
    }

    /// Comme `extern_arg`, mais **coerce** un argument numérique au type de param **dimensionné** du
    /// binding (`u32`/`i32`/`usize`/`u64`/`f64`…). Iron défaute les entiers à `i64` ; sans cast, un appel
    /// `from_ymd_opt(2026, 6, 28)` (i64) vers `(i32, u32, u32)` casse (E0308). On émet `(v) as T` seulement
    /// si le type diffère (pas de cast inutile → pas de bruit clippy).
    fn extern_arg_to(&mut self, x: &Expr, expected: Option<&Type>) -> TokenStream {
        if let Some(exp) = expected {
            let target: Option<&'static str> = match exp {
                Type::Num(k) => Some(num_kind_str(*k)),
                Type::F64 => Some("f64"),
                _ => None,
            };
            if let Some(t) = target {
                let at = self.type_of(x);
                if at != *exp && matches!(at, Type::I64 | Type::Num(_) | Type::F64) {
                    let v = self.value(x);
                    let id = format_ident!("{}", t);
                    return quote! { (#v) as #id };
                }
            }
        }
        self.extern_arg(x)
    }

    /// Valeur coercée au type numérique **dimensionné** attendu (`(v) as u16`), **sans** `&` (champ de
    /// struct par valeur — 52400). Non numérique → valeur telle quelle.
    fn coerce_to(&mut self, x: &Expr, expected: Option<&Type>) -> TokenStream {
        let v = self.value(x);
        match expected {
            Some(Type::Num(k)) if matches!(self.type_of(x), Type::I64 | Type::Num(_) | Type::F64) && self.type_of(x) != Type::Num(*k) => {
                let id = format_ident!("{}", num_kind_str(*k));
                quote! { (#v) as #id }
            }
            Some(Type::F64) if matches!(self.type_of(x), Type::I64 | Type::Num(_)) => quote! { (#v) as f64 },
            _ => v,
        }
    }

    /// Constructeur externe namespacé (V8, 40200) : `Type.ctor(args)` → `crate::Type::ctor(args)`.
    fn try_extern_ctor(&mut self, base: &Expr, name: &str, args: &[Expr]) -> Option<TokenStream> {
        let tname = match &base.kind {
            ExprKind::Var(n) => n.clone(),
            _ => return None,
        };
        let ctor = self
            .bindings
            .etypes
            .get(&tname)
            .and_then(|et| et.ctors.iter().find(|(n, _)| n == name).map(|(_, f)| f.clone()))?;
        let path = TokenStream::from_str(&ctor.path).unwrap_or_else(|_| quote! { () });
        let a: Vec<TokenStream> =
            args.iter().enumerate().map(|(i, x)| self.extern_arg_to(x, ctor.params.get(i))).collect();
        let call = quote! { #path( #(#a),* ) };
        Some(if matches!(ctor.ret, Type::Result(_)) {
            quote! { #call.map_err(|e| IronError(format!("{:?}", e))) }
        } else {
            call
        })
    }

    /// Méthode externe (V8, 40200) : `obj.m(args)` → `RustPath::m(<recv> obj, args)` + glue d'idiomes.
    /// `recv` : `&self` (ByRef) → `&obj`, `&mut self` (ByRefMut) → `&mut obj`, `self` (ByValue) → `obj`.
    fn try_extern_method(&mut self, base: &Expr, name: &str, args: &[Expr]) -> Option<TokenStream> {
        let iron_name = match self.type_of(base) {
            Type::Extern(n) => n,
            _ => return None,
        };
        let ef = match self.bindings.etypes.get(&iron_name) {
            Some(et) => match et.methods.iter().find(|(n, _)| n == name) {
                Some((_, f)) => f.clone(),
                None => return None,
            },
            None => return None,
        };
        let recv = self.place(base);
        let a: Vec<TokenStream> =
            args.iter().enumerate().map(|(i, x)| self.extern_arg_to(x, ef.params.get(i))).collect();
        // Méthode de **trait** (chemin `crate::Trait::method`) → UFCS `Trait::method(<recv>, args)` (évite
        // `use Trait`). Méthode **inhérente** (nom court) → syntaxe-méthode `recv.m(args)` (auto-ref/deref,
        // gère les méthodes via `Deref` comme `str` sur `String`).
        let mut call = if ef.path.contains("::") {
            let path = TokenStream::from_str(&ef.path).unwrap_or_else(|_| quote! { () });
            let recv_ref = match ef.recv {
                Some(Recv::ByRefMut) => quote! { &mut #recv },
                Some(Recv::ByValue) => quote! { #recv },
                _ => quote! { &#recv },
            };
            quote! { #path( #recv_ref #(, #a)* ) }
        } else {
            let mid = format_ident!("{}", ef.path.rsplit("::").next().unwrap_or(&ef.path));
            quote! { #recv.#mid( #(#a),* ) }
        };
        for idiom in &ef.idioms {
            match idiom {
                // getter (D4, 51700) : retour emprunté → valeur possédée, **typé** :
                // `&str`→`.to_string()`, `&T`→`.clone()`, `Option<&str>`→`.map(to_string)`, `Option<&T>`→`.cloned()`.
                Idiom::Clone => {
                    call = match &ef.ret {
                        Type::Str => quote! { (#call).to_string() },
                        Type::Option(inner) if matches!(**inner, Type::Str) => {
                            quote! { (#call).map(|__s| __s.to_string()) }
                        }
                        Type::Option(_) => quote! { (#call).cloned() },
                        _ => quote! { (#call).clone() },
                    };
                    self.audit.push(base.span); // warning perf I4 (--audit)
                }
                Idiom::Collect => call = quote! { #call.collect::<Vec<_>>() },
                Idiom::Builder => {} // `&mut Self` jeté ; la mutation en place a eu lieu
                Idiom::Closure(_) => {}
            }
        }
        // méthode renvoyant un Result du crate → `IronError` unifié (I13).
        if matches!(ef.ret, Type::Result(_)) {
            call = quote! { #call.map_err(|e| IronError(format!("{:?}", e))) };
        }
        Some(call)
    }

    fn try_builtin_method(&mut self, base: &Expr, name: &str, args: &[Expr]) -> Option<TokenStream> {
        let bt = self.type_of(base);
        let arg = |s: &mut Self, i: usize| s.value(&args[i]);
        Some(match (&bt, name) {
            // --- str ---
            (Type::Str, "upper") => { let b = self.place(base); quote! { #b.to_uppercase() } }
            (Type::Str, "lower") => { let b = self.place(base); quote! { #b.to_lowercase() } }
            (Type::Str, "strip") => { let b = self.place(base); quote! { #b.trim().to_string() } }
            (Type::Str, "split") if args.len() == 1 => {
                let b = self.place(base);
                let s = arg(self, 0);
                quote! { #b.split((#s).as_str()).map(|x| x.to_string()).collect::<Vec<_>>() }
            }
            // `s.split()` sans argument (Python) → découpe sur les blancs (runs d'espaces, trim).
            (Type::Str, "split") if args.is_empty() => {
                let b = self.place(base);
                quote! { #b.split_whitespace().map(|x| x.to_string()).collect::<Vec<_>>() }
            }
            // `s.splitlines()` → lignes (sans le `\n`), gère un `\n` final.
            (Type::Str, "splitlines") => {
                let b = self.place(base);
                quote! { #b.lines().map(|x| x.to_string()).collect::<Vec<_>>() }
            }
            // `s.count(sub)` → nombre d'occurrences (non chevauchantes), comme Python.
            (Type::Str, "count") if args.len() == 1 => {
                let b = self.place(base);
                let s = arg(self, 0);
                quote! { (#b.matches((#s).as_str()).count() as i64) }
            }
            // `sep.join(list)` (forme méthode Python) → `list.join(sep)`.
            (Type::Str, "join") if args.len() == 1 => {
                let b = self.place(base);
                let xs = self.place(&args[0]);
                quote! { #xs.join(#b.as_str()) }
            }
            (Type::Str, "replace") if args.len() == 2 => {
                let b = self.place(base);
                let a = arg(self, 0);
                let c = arg(self, 1);
                quote! { #b.replace((#a).as_str(), (#c).as_str()) }
            }
            (Type::Str, "startswith") if args.len() == 1 => {
                let b = self.place(base);
                let s = arg(self, 0);
                quote! { #b.starts_with((#s).as_str()) }
            }
            (Type::Str, "endswith") if args.len() == 1 => {
                let b = self.place(base);
                let s = arg(self, 0);
                quote! { #b.ends_with((#s).as_str()) }
            }
            (Type::Str, "contains") if args.len() == 1 => {
                let b = self.place(base);
                let s = arg(self, 0);
                quote! { #b.contains((#s).as_str()) }
            }
            (Type::Str, "len") => {
                let b = self.place(base);
                quote! { (#b.chars().count() as i64) } // Python : len(str) = nb de caractères Unicode
            }
            (Type::List(_), "len") => {
                let b = self.place(base);
                quote! { (#b.len() as i64) }
            }
            // --- list ---
            (Type::List(_), "append") if args.len() == 1 => {
                let b = self.place(base);
                let x = arg(self, 0);
                quote! { #b.push(#x) }
            }
            (Type::List(_), "pop") => {
                let b = self.place(base);
                quote! { #b.pop() }
            }
            (Type::List(_), "sort") => {
                let b = self.place(base);
                quote! { #b.sort_by(|a, b| a.partial_cmp(b).unwrap()) }
            }
            (Type::List(_), "reverse") => {
                let b = self.place(base);
                quote! { #b.reverse() }
            }
            (Type::List(_) | Type::Set(_), "contains") if args.len() == 1 => {
                let b = self.place(base);
                let x = arg(self, 0);
                quote! { #b.contains(&#x) }
            }
            // --- dict ---
            (Type::Dict(..), "keys") => {
                let b = self.place(base);
                quote! { #b.keys().cloned().collect::<Vec<_>>() }
            }
            (Type::Dict(..), "values") => {
                let b = self.place(base);
                quote! { #b.values().cloned().collect::<Vec<_>>() }
            }
            (Type::Dict(..), "items") => {
                let b = self.place(base);
                quote! { #b.iter().map(|(__k, __v)| (__k.clone(), __v.clone())).collect::<Vec<_>>() }
            }
            (Type::Dict(..), "get") if args.len() == 1 => {
                let b = self.place(base);
                let k = arg(self, 0);
                quote! { #b.get(&#k).cloned() }
            }
            (Type::Dict(..), "contains") if args.len() == 1 => {
                let b = self.place(base);
                let k = arg(self, 0);
                quote! { #b.contains_key(&#k) }
            }
            _ => return None,
        })
    }

    /// Module stdlib `os` (80800) : env/args/exit — wrappers `std::env`/`std::process`.
    /// Module `thread` (V9, 70900) : `spawn(lambda)` → `std::thread::spawn(move || …)`. Le `move` rend la
    /// closure `Send` ; l'état partagé entre threads exige `shared_sync` (Arc<Mutex>, 30700) — un `shared`
    /// (Rc, !Send) est rejeté par `rustc` (filet I2).
    fn lower_thread(&mut self, name: &str, args: &[Expr]) -> TokenStream {
        match name {
            "spawn" if args.len() == 1 => {
                let body = match &args[0].kind {
                    ExprKind::Lambda { params, body } if params.is_empty() => self.value(body),
                    _ => {
                        let f = self.value(&args[0]);
                        quote! { (#f)() }
                    }
                };
                quote! { std::thread::spawn(move || #body) }
            }
            other => {
                let msg = format!("thread.{other} non supporté (spawn uniquement au v0)");
                quote! { compile_error!(#msg) }
            }
        }
    }

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
            StmtKind::While { body, els, .. } => {
                collect_locals(body, params, types, out);
                collect_locals(els, params, types, out);
            }
            StmtKind::For { body, els, .. } => {
                collect_locals(body, params, types, out);
                collect_locals(els, params, types, out);
            }
            StmtKind::Match { arms, .. } => {
                for arm in arms {
                    collect_locals(&arm.body, params, types, out);
                }
            }
            StmtKind::TryExcept { body, handler, .. } => {
                collect_locals(body, params, types, out);
                collect_locals(handler, params, types, out);
            }
            // séquence désucrée (affectation chaînée…) : ses Let sont des locaux de la fonction (hoistés).
            StmtKind::Seq(ss) => collect_locals(ss, params, types, out),
            // PAS de descente dans une `Closure` : ses locaux sont à elle (déclarés dans son corps Rust).
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
        // or-pattern `p1 | p2 | …` (V11, 11000).
        Pattern::Or(alts) => {
            let ps: Vec<TokenStream> = alts.iter().map(lower_pattern).collect();
            quote! { #(#ps)|* }
        }
        // range pattern (V11, 11000). Les ranges **exclusifs** étant instables en Rust stable, on émet
        // toujours de l'**inclusif** : `lo..=hi` (inclusif) ; `lo..hi` (exclusif, entier) → `lo..=(hi-1)`.
        Pattern::Range(lo, hi, inclusive) => {
            let l = lower_range_bound(lo);
            if *inclusive {
                let h = lower_range_bound(hi);
                quote! { #l..=#h }
            } else if let ExprKind::Lit(Lit::Int(n)) = &hi.kind {
                let h = proc_macro2::Literal::i64_unsuffixed(*n - 1);
                quote! { #l..=#h }
            } else {
                let h = lower_range_bound(hi); // char/const exclusif (rare) → best-effort inclusif
                quote! { #l..=#h }
            }
        }
    }
}

/// Borne d'un range pattern → token Rust (littéral int/char ou nom de const).
fn lower_range_bound(e: &Expr) -> TokenStream {
    match &e.kind {
        ExprKind::Lit(Lit::Int(n)) => {
            let n = proc_macro2::Literal::i64_unsuffixed(*n);
            quote! { #n }
        }
        ExprKind::Lit(Lit::Char(c)) => quote! { #c },
        ExprKind::Var(n) => {
            let id = format_ident!("{}", n);
            quote! { #id }
        }
        _ => quote! { _ },
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

/// Dunders bénis surchargeables (V11, DC-4) — désucrés en `impl std::ops`/`PartialEq`/`PartialOrd`/`Display`.
/// `__getitem__` (→ `Index`) différé (retour `&Output`). Hors de cet ensemble : méthode inhérente normale.
/// Caractère d'un littéral chaîne d'**un seul** caractère (sinon `None`) — pour comparer un `char` à `"x"`.
fn str_lit_char(e: &Expr) -> Option<char> {
    if let ExprKind::Lit(Lit::Str(s)) = &e.kind {
        let mut it = s.chars();
        let c = it.next()?;
        if it.next().is_none() {
            return Some(c);
        }
    }
    None
}

fn is_blessed_dunder(name: &str) -> bool {
    matches!(name, "__add__" | "__sub__" | "__mul__" | "__div__" | "__eq__" | "__lt__" | "__str__")
}

fn lower_generics(gens: &Generics) -> TokenStream {
    if gens.is_empty() {
        return quote! {};
    }
    let parts: Vec<TokenStream> = gens
        .iter()
        .map(|(n, bounds)| {
            let id = format_ident!("{}", n);
            // Borne `Clone` par défaut (V9.1, 21400) : la sémantique-valeur d'Iron (I1) clone → `T: Clone`.
            // Les bornes sont des **chemins** (`std::fmt::Display`…) → parsées comme TokenStream (V10.1, R2).
            let bs: Vec<TokenStream> = bounds
                .iter()
                .map(|b| TokenStream::from_str(b).unwrap_or_else(|_| quote! { Clone }))
                .collect();
            if bs.is_empty() {
                quote! { #id: Clone }
            } else {
                quote! { #id: Clone + #(#bs)+* }
            }
        })
        .collect();
    quote! { < #(#parts),* > }
}

fn lower_type(t: &Type) -> TokenStream {
    match t {
        // D2 (V10) : aucune métavariable ne doit atteindre le lowering. Garde-fou : si une `Var` arrive,
        // l'inférence n'a pas clos (defaulting/erreur) — bug du moteur, pas du programme utilisateur.
        Type::Var(_) => unreachable!("Type::Var atteint le lowering — invariant D2 violé (inférence non close)"),
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
        // Erreur typée (21500) : `Result[T, E]` → `Result<T, E>`.
        Type::ResultE(t, e) => {
            let ti = lower_type(t);
            let ei = lower_type(e);
            quote! { Result<#ti, #ei> }
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
        // Partage thread-safe (V9, 30700) : Arc<Mutex<T>>.
        Type::SharedSync(t) => {
            let inner = lower_type(t);
            quote! { std::sync::Arc<std::sync::Mutex<#inner>> }
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
