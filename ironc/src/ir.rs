//! IR partagé — types **GELÉS** (V0/README §3–4). Aucune vague 2 ne modifie ce fichier.
//!
//! Clé de jointure : chaque nœud porte `NodeId` (pré-ordre DFS, dense) + `Span` (octets).
//! Les passes communiquent par tables annexes indexées par `NodeId` ; l'AST est immuable.

use std::collections::HashMap;

pub type FileId = u32;

/// Sentinelle de type de paramètre **non annoté** (20500) : `Type::Generic(INFER_SENTINEL)`. Résolue
/// en pré-passe (`infer::params`) en `i64` ou générique implicite ; ne doit jamais atteindre `lower`.
pub(crate) const INFER_SENTINEL: &str = "__infer__";

/// Offsets **octets** demi-ouverts `[start, end)` (V0/README §3). Unité figée = octet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub file: FileId,
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub fn new(file: FileId, start: u32, end: u32) -> Self {
        Span { file, start, end }
    }
    /// Span neutre (stubs / nœuds synthétiques).
    pub const DUMMY: Span = Span { file: 0, start: 0, end: 0 };
}

/// `u32` dense, pré-ordre DFS dès 0 ; clé partagée AST ↔ TypeMap ↔ OwnershipMap ↔ SourceMap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

/// Nœud générique : `{ id, span, kind }` (V0/README §4.2).
#[derive(Debug, Clone)]
pub struct Node<K> {
    pub id: NodeId,
    pub span: Span,
    pub kind: K,
}

// ---------------------------------------------------------------- Tokens (10100, §4.1)

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // littéraux
    Int(i64),
    Float(f64),
    Str(String),
    /// f-string `f"…"` (V7, 10500) : contenu brut intérieur (avec `{`/`}` préservés).
    FStr(String),
    /// littéral caractère `'a'` (V7, 20800).
    Char(char),
    Bool(bool),
    Ident(String),
    // mots-clés
    Fn,
    Struct,
    Enum,
    Match,
    Trait,
    Impl,
    Inout,
    Lambda,
    If,
    Elif,
    Else,
    While,
    For,
    In,
    Break,
    Continue,
    Return,
    Pass,
    And,
    Or,
    Not,
    // ponctuation / opérateurs
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Semicolon,
    Comma,
    Dot,
    /// `..` (range exclusif) / `..=` (inclusif) / `|` (or-pattern) — V11, 11000.
    DotDot,
    DotDotEq,
    Pipe,
    Colon,
    Arrow,
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    /// V7 (10700) : augmentées + `**`/`//`.
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    SlashSlashEq,
    StarStar,
    SlashSlash,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    Question,
    // synthétiques (off-side rule, I9)
    Indent,
    Dedent,
    Newline,
    /// Contenu brut d'un bloc `rust { … }` (interop niveau 1, capté sans tokenisation Iron).
    RustVerbatim(String),
    /// Décorateur d'attribut Rust `@derive(...)`/`@repr(C)` (V9, 51900) : contenu brut après `@`.
    Decorator(String),
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

// ---------------------------------------------------------------- Types (§4.3)

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    I64,
    F64,
    Bool,
    Str,
    Unit,
    Struct(String),
    Option(Box<Type>),
    Result(Box<Type>),
    List(Box<Type>),
    Dict(Box<Type>, Box<Type>),
    /// `Result[T, E]` à **erreur typée** (V9.1, 21500) : `E` concret (enum Iron, type externe…).
    /// `Result[T]` (sans `E`) reste `Type::Result(T)` = `Result<T, IronError>` (rétro-compat, D5).
    ResultE(Box<Type>, Box<Type>),
    /// Type Rust opaque (interop 50200) — stocke le chemin Rust.
    Extern(String),
    /// Type Rust externe **générique appliqué** (V12, [24000]) : `(rust_path, args)` → `rust_path::<args>` au
    /// lowering. `args` vide ⇒ équivalent `Extern`. Porte les paramètres de type jadis jetés (`Point<f64>`).
    ExternApp(String, Vec<Type>),
    /// Enum utilisateur (V3, 20300).
    Enum(String),
    /// Paramètre de type générique opaque (V3, 20300).
    Generic(String),
    /// Application de type générique : `Name[args]` → `Name<args>` (V3).
    App(String, Vec<Type>),
    /// `inout x: T` → `&mut T` (I3 — partage mutable explicite, V5).
    Ref(Box<Type>),
    /// Type fonction / closure (V6, 20700) : `fn(args) -> ret`. Lowering best-effort —
    /// les closures inline laissent `rustc` inférer (I2).
    Fn(Vec<Type>, Box<Type>),
    /// Partage explicite (V6, 30300, I5) : `shared[T]` → `Rc<RefCell<T>>`. Le clone est un
    /// `Rc::clone` (partage du pointeur), jamais une copie profonde.
    Shared(Box<Type>),
    /// Tuple anonyme `(A, B, …)` (V7, 10400).
    Tuple(Vec<Type>),
    /// Ensemble `set[T]` → `HashSet<T>` (V7, 20900).
    Set(Box<Type>),
    /// Numériques élargis hors `i64`/`f64` (V7, 20800) ; les défauts restent `I64`/`F64`.
    Num(NumKind),
    /// **Métavariable d'inférence** (V10, 21700) : trou de type à résoudre par le moteur de contraintes.
    /// Invariant de sortie (D2) : **aucune `Var` ne subsiste** après inférence (defaulting ou erreur).
    /// Les composants en aval (`lower`/`resolve`) n'en voient jamais.
    Var(u32),
    /// Caractère Unicode `char` (V7, 20800).
    Char,
    /// Référence faible `Weak` (V7, 30600).
    Weak(Box<Type>),
    /// Partage explicite **thread-safe** (V9, 30700) : `shared_sync[T]` → `Arc<Mutex<T>>`. Pendant `Send`
    /// de `Shared`/`Rc` (mono-thread) ; requis pour passer un état partagé à un `thread.spawn`.
    SharedSync(Box<Type>),
}

/// Variantes numériques élargies (V7, 20800). `i64`/`f64` restent `Type::I64`/`F64` (défauts).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumKind {
    I8,
    I16,
    I32,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    F32,
}

/// Nom de type numérique → `Type` (zoo + `i64`/`f64`), pour `parse_type` et les casts `to_<t>` (20800).
pub(crate) fn numeric_type_by_name(name: &str) -> Option<Type> {
    Some(match name {
        "i64" => Type::I64,
        "f64" => Type::F64,
        "i8" => Type::Num(NumKind::I8),
        "i16" => Type::Num(NumKind::I16),
        "i32" => Type::Num(NumKind::I32),
        "i128" => Type::Num(NumKind::I128),
        "isize" => Type::Num(NumKind::Isize),
        "u8" => Type::Num(NumKind::U8),
        "u16" => Type::Num(NumKind::U16),
        "u32" => Type::Num(NumKind::U32),
        "u64" => Type::Num(NumKind::U64),
        "u128" => Type::Num(NumKind::U128),
        "usize" => Type::Num(NumKind::Usize),
        "f32" => Type::Num(NumKind::F32),
        _ => return None,
    })
}

// ---------------------------------------------------------------- AST (10200, §4.2)

pub type Item = Node<ItemKind>;
pub type Stmt = Node<StmtKind>;
pub type Expr = Node<ExprKind>;

#[derive(Debug, Clone)]
pub enum ItemKind {
    Fn {
        name: String,
        generics: Generics,
        params: Vec<(String, Type)>,
        ret: Type,
        body: Vec<Stmt>,
        /// Valeurs par défaut par paramètre (surface Python) ; `None` = obligatoire. Même longueur que `params`.
        defaults: Vec<Option<Expr>>,
        /// `async def` (V9, 21200) → `async fn` + `await` ; runtime tokio injecté sur `main`.
        is_async: bool,
    },
    Struct {
        name: String,
        generics: Generics,
        fields: Vec<(String, Type)>,
    },
    /// `enum E:` variantes tuple (V3, 20300).
    Enum {
        name: String,
        generics: Generics,
        variants: Vec<(String, Vec<Type>)>,
    },
    /// `trait T:` — signatures de méthodes (V3, 20300).
    Trait { name: String, methods: Vec<Method> },
    /// `impl T for Ty:` — définitions de méthodes (V3, 20300).
    Impl { trait_name: String, type_name: String, methods: Vec<Method> },
    /// Constante globale immuable `NAME = <littéral>` (V11) → `const`/`static` Rust. Mutable global =
    /// hors-périmètre (I2). `ty` inféré du littéral.
    Const { name: String, ty: Type, value: Expr },
}

/// Méthode de trait/impl (V3) : `body = None` = signature (trait), `Some` = définition (impl).
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub self_param: bool,
    pub params: Vec<(String, Type)>,
    pub ret: Type,
    pub body: Option<Vec<Stmt>>,
    /// Valeurs par défaut par param (surface Python) ; même longueur que `params`.
    pub defaults: Vec<Option<Expr>>,
}

/// Paramètres génériques (V3) : nom + bornes (`[T: Trait + …]`).
pub type Generics = Vec<(String, Vec<String>)>;

#[derive(Debug, Clone)]
pub enum StmtKind {
    Let { name: String, value: Expr },
    Assign { target: Expr, value: Expr },
    If {
        cond: Expr,
        then: Vec<Stmt>,
        elifs: Vec<(Expr, Vec<Stmt>)>,
        els: Vec<Stmt>,
    },
    While { cond: Expr, body: Vec<Stmt>, els: Vec<Stmt> },
    For { var: String, iter: Expr, body: Vec<Stmt>, els: Vec<Stmt> },
    Break,
    Continue,
    Return { value: Option<Expr> },
    ExprStmt(Expr),
    /// `yield e` (V9, 10900) : générateur → accumulé dans un Vec (eager), la fn renvoie `list[T]`.
    Yield(Expr),
    Pass,
    /// Séquence de statements traitée comme un seul (désucrage, V11 — ex. affectation chaînée). Les locaux
    /// sont hoistés au niveau fonction (comme partout) → pas de portée bloc.
    Seq(Vec<Stmt>),
    /// Bloc `rust { … }` : Rust verbatim (interop niveau 1, 50100).
    RustBlock(String),
    /// `match e:` + bras (V3, 20300).
    Match { scrutinee: Expr, arms: Vec<MatchArm> },
    /// `assert cond[, msg]` (V7, 10800).
    Assert { cond: Expr, msg: Option<Expr> },
    /// Affectation déstructurée `a, b = value` (V7, 10400).
    Destructure { targets: Vec<String>, value: Expr },
    /// `try: … except [e]: …` (surface Python) — capture les erreurs `Result`/`?` du corps.
    TryExcept { body: Vec<Stmt>, err_name: Option<String>, handler: Vec<Stmt> },
    /// `raise <expr>` (surface Python) → `return Err(IronError(...))`.
    Raise { value: Expr },
}

/// Un bras de `match` : motif + garde optionnelle + corps (V3 ; garde V6/20600).
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    /// Garde `if <cond>` (V6, 20600) : le bras ne filtre que si la condition est vraie.
    pub guard: Option<Expr>,
    pub body: Vec<Stmt>,
}

/// Motif de `match` (V3, 20300).
#[derive(Debug, Clone)]
pub enum Pattern {
    Wild,
    Var(String),
    Lit(Lit),
    /// Motif tuple `(a, b)` (V7, 10400).
    Tuple(Vec<Pattern>),
    /// `Some(x)`, `None`, `Shape.Circle(r)` — `path` joint en `::`, `sub` = sous-motifs.
    Variant { path: Vec<String>, sub: Vec<Pattern> },
    /// or-pattern `p1 | p2 | …` (V11, 11000).
    Or(Vec<Pattern>),
    /// range pattern `lo..hi` (exclusif) / `lo..=hi` (inclusif) (V11, 11000).
    Range(Box<Expr>, Box<Expr>, bool),
}

#[derive(Debug, Clone)]
pub enum Lit {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    /// Littéral caractère `'a'` (V7, 20800).
    Char(char),
}

#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    /// V7 (10700) : puissance `**`, division entière `//`, bit-à-bit.
    Pow,
    FloorDiv,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    /// V7 (10800) : appartenance `in` / `not in`.
    In,
    NotIn,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Lit(Lit),
    Var(String),
    Unary { op: UnOp, expr: Box<Expr> },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr> },
    Call { callee: Box<Expr>, args: Vec<Expr> },
    StructLit { name: String, fields: Vec<(String, Expr)> },
    Field { base: Box<Expr>, name: String },
    Index { base: Box<Expr>, idx: Box<Expr> },
    ListLit { items: Vec<Expr> },
    DictLit { pairs: Vec<(Expr, Expr)> },
    Try { inner: Box<Expr> },
    /// `await e` (V9, 21200) → `(e).await`.
    Await(Box<Expr>),
    /// Closure (V6, 20700) : `lambda x, y: expr`. Corps mono-expression ; types des params
    /// laissés à l'inférence Rust (I2).
    Lambda { params: Vec<String>, body: Box<Expr> },
    /// Closure à **corps de bloc** (V11) : fonction imbriquée désucrée → `|params| { stmts }`. Types des
    /// params inférés par rustc (comme une closure).
    Closure { params: Vec<String>, body: Vec<Stmt> },
    /// Tuple `(a, b, …)` (V7, 10400).
    Tuple(Vec<Expr>),
    /// f-string `f"…{e}…"` (V7, 10500).
    FString(Vec<FStringPart>),
    /// Compréhension liste/dict/set (V7, 10600). Pour `Dict`, `body` = `Tuple([clé, valeur])`.
    Comp { kind: CompKind, body: Box<Expr>, var: Pattern, iter: Box<Expr>, cond: Option<Box<Expr>> },
    /// Ternaire `a if c else b` (V7, 10800).
    Ternary { cond: Box<Expr>, then: Box<Expr>, els: Box<Expr> },
    /// Slicing `xs[lo:hi]` (V7, 10800) ; bornes optionnelles.
    Slice { base: Box<Expr>, lo: Option<Box<Expr>>, hi: Option<Box<Expr>> },
}

/// Fragment de f-string (V7, 10500) : texte littéral ou sous-expression interpolée.
#[derive(Debug, Clone)]
pub enum FStringPart {
    Lit(String),
    Expr(Box<Expr>),
}

/// Sorte de compréhension (V7, 10600).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompKind {
    List,
    Dict,
    Set,
}

#[derive(Debug, Clone, Default)]
pub struct Ast {
    pub items: Vec<Item>,
    /// Chemins `.iri` importés via `use "…"` (interop 50200).
    pub imports: Vec<String>,
    /// Modules Iron importés via `import foo` (→ `foo.irn`) (70200). Fusionnés à plat par le driver.
    pub modules: Vec<String>,
    /// Types opaques `extern type Name = "rust::path"` (50400) : nom → chemin Rust ; émis en alias.
    pub extern_types: Vec<(String, String)>,
    /// Attributs Rust pass-through (V9, 51900) : `NodeId` de l'item → attributs bruts (`derive(Serialize)`,
    /// `repr(C)`…), émis en `#[…]`. Side-map pour éviter de toucher chaque variante d'`ItemKind`.
    pub item_attrs: std::collections::HashMap<NodeId, Vec<String>>,
    /// Ascription de type (V12 [52900]) : `NodeId` d'un `Let` → type **annoté** (`cfg: Config = …`). Le
    /// lowering émet `let cfg: Config = …` → `rustc` infère les génériques de l'appel (ex. `from_str::<Config>`).
    /// Side-map pour ne pas toucher `StmtKind::Let`.
    pub ascriptions: std::collections::HashMap<NodeId, Type>,
}

// ---------------------------------------------------------------- Tables annexes

/// `NodeId → Type` (20100/20200).
pub type TypeMap = HashMap<NodeId, Type>;

/// Disposition d'une lecture de variable (30100 / 30200).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disposition {
    Clone,
    Move,
    Borrow,
}

/// `NodeId(lecture de Var) → Disposition`.
pub type OwnershipMap = HashMap<NodeId, Disposition>;

/// Mode de passage d'un paramètre (escape analysis, V1/30200) : `&T` (ByRef) ou `T` (ByValue).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamMode {
    ByValue,
    ByRef,
    InOut,
}

/// `fn_name → modes de ses paramètres`.
pub type ParamModes = HashMap<String, Vec<ParamMode>>;

/// Mode de réception d'une méthode externe (V8, 50900) : `self`/`&self`/`&mut self`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recv {
    ByValue,
    ByRef,
    ByRefMut,
}

/// Idiome Rust reconnu au binding (V8, 51300) ; la glue est émise par le lowering (40200).
/// - `Builder` : recv `ByRefMut`, retour `&mut Self` jeté (mutation en place).
/// - `Clone`   : retour emprunté `&T` → `.clone()`/`.cloned()` (getter, D4 ; warning perf I4).
/// - `Collect` : retour itérateur → `.collect::<Vec<_>>()` → `list[T]`.
/// - `Closure(idx)` : le paramètre `idx` est une closure (`Fn`-borné) → lambda Iron.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Idiom {
    Builder,
    Clone,
    Collect,
    Closure(usize),
}

/// Fonction Rust externe importée via `.iri` (50200, étendu V8 50900) : signature Iron + chemin Rust.
/// `recv` = `None` pour une fn libre/constructeur, `Some(_)` pour une méthode.
#[derive(Debug, Clone)]
pub struct ExternFn {
    pub params: Vec<Type>,
    /// Param passé **par valeur** (`own T` dans le `.iri`) plutôt que par emprunt → pas de `&` au lowering
    /// (ex. `WavWriter::create(spec: WavSpec)` consomme `spec`). Parallèle à `params` ; vide ⇒ tous par réf.
    pub param_owned: Vec<bool>,
    pub ret: Type,
    pub path: String,
    pub recv: Option<Recv>,
    pub idioms: Vec<Idiom>,
}

/// Type Rust externe à API (V8, 50900) : handle opaque + constructeurs + méthodes (D2).
/// V12 (52400) : `fields` (struct à champs publics → construction par littéral `WavSpec(channels=1, …)`)
/// et `variants` (enum à variantes → `SampleFormat.Int`).
#[derive(Debug, Clone, Default)]
pub struct ExternType {
    pub rust_path: String,
    pub ctors: Vec<(String, ExternFn)>,
    pub methods: Vec<(String, ExternFn)>,
    /// Champs publics d'une struct externe : `(nom, type)` → `path { nom: val, … }` au lowering.
    pub fields: Vec<(String, Type)>,
    /// Variantes d'un enum externe : `path::Variant` au lowering.
    pub variants: Vec<String>,
    /// Paramètres de type du type externe (V12, [24000]) : `extern type Point[T] = …` → `["T"]`. Base de la
    /// **substitution** en inférence (`Point[f64]`.méthode rendant `T` → `f64`). Vide = type non générique.
    pub params: Vec<String>,
}

/// Bindings chargés depuis les `.iri` importés (50200, étendu V8 50900).
#[derive(Debug, Clone, Default)]
pub struct Bindings {
    pub fns: HashMap<String, ExternFn>,
    /// Types opaques sans API (`extern type Name = "path"`, rétro-compat 50400) : nom → chemin Rust.
    pub types: HashMap<String, String>,
    /// Types externes à API (`extern type Name = "path":` + bloc de méthodes/ctors, V8 50900).
    pub etypes: HashMap<String, ExternType>,
    /// Dépendances Cargo (V9, 70800/51900) : `(nom, version, features)` → `name = { version, features }`.
    pub crates: Vec<(String, String, Vec<String>)>,
    /// Namespacing (V8, 51500) : nom de module → noms d'items qu'il regroupe.
    pub mods: HashMap<String, Vec<String>>,
    /// Constantes publiques d'un crate (V12 [50500]) : `NOM` → (type, chemin Rust). `BASE64_STANDARD` →
    /// (`Extern("…GeneralPurpose")`, `"base64::engine::general_purpose::STANDARD"`). Référencé en `Var`.
    pub consts: HashMap<String, (Type, String)>,
    /// Traits externes **implémentables** par l'utilisateur (V12 [24300]) : nom court → chemin Rust complet.
    /// `impl App for MyApp` émet `impl eframe::App for MyApp { … }` (chemin résolu ici).
    pub traits: HashMap<String, String>,
}

// ---------------------------------------------------------------- SourceMap (§4.6)

/// Lien marqueur ↔ origine Iron. Le marqueur émis dans le Rust encode l'**index** de ce lien.
#[derive(Debug, Clone, PartialEq)]
pub struct SpanLink {
    pub node: NodeId,
    pub iron: Span,
}

/// Produite par `lower` (40100), consommée par `sourcemap` (60100).
#[derive(Debug, Clone, Default)]
pub struct SourceMap {
    pub links: Vec<SpanLink>,
}

impl SourceMap {
    /// Enregistre un lien et renvoie son index (= valeur du marqueur émis).
    pub fn push(&mut self, node: NodeId, iron: Span) -> usize {
        let idx = self.links.len();
        self.links.push(SpanLink { node, iron });
        idx
    }
}

// ---------------------------------------------------------------- Erreur interne

/// Erreur **du compilateur** `ironc`. À ne pas confondre avec le type `IronError`
/// émis *dans* le crate généré (I13).
#[derive(Debug)]
pub struct CompileError {
    pub span: Option<Span>,
    pub msg: String,
}

impl CompileError {
    pub fn msg(msg: impl Into<String>) -> Self {
        CompileError { span: None, msg: msg.into() }
    }
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for CompileError {}

pub type CResult<T> = Result<T, CompileError>;
