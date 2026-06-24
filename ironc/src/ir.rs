//! IR partagé — types **GELÉS** (V0/README §3–4). Aucune vague 2 ne modifie ce fichier.
//!
//! Clé de jointure : chaque nœud porte `NodeId` (pré-ordre DFS, dense) + `Span` (octets).
//! Les passes communiquent par tables annexes indexées par `NodeId` ; l'AST est immuable.

use std::collections::HashMap;

pub type FileId = u32;

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
    Colon,
    Arrow,
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
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
    /// Type Rust opaque (interop 50200) — stocke le chemin Rust.
    Extern(String),
    /// Enum utilisateur (V3, 20300).
    Enum(String),
    /// Paramètre de type générique opaque (V3, 20300).
    Generic(String),
    /// Application de type générique : `Name[args]` → `Name<args>` (V3).
    App(String, Vec<Type>),
    /// `inout x: T` → `&mut T` (I3 — partage mutable explicite, V5).
    Ref(Box<Type>),
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
}

/// Méthode de trait/impl (V3) : `body = None` = signature (trait), `Some` = définition (impl).
#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub self_param: bool,
    pub params: Vec<(String, Type)>,
    pub ret: Type,
    pub body: Option<Vec<Stmt>>,
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
    While { cond: Expr, body: Vec<Stmt> },
    For { var: String, iter: Expr, body: Vec<Stmt> },
    Break,
    Continue,
    Return { value: Option<Expr> },
    ExprStmt(Expr),
    Pass,
    /// Bloc `rust { … }` : Rust verbatim (interop niveau 1, 50100).
    RustBlock(String),
    /// `match e:` + bras (V3, 20300).
    Match { scrutinee: Expr, arms: Vec<MatchArm> },
}

/// Un bras de `match` : motif + corps (V3).
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Vec<Stmt>,
}

/// Motif de `match` (V3, 20300).
#[derive(Debug, Clone)]
pub enum Pattern {
    Wild,
    Var(String),
    Lit(Lit),
    /// `Some(x)`, `None`, `Shape.Circle(r)` — `path` joint en `::`, `sub` = sous-motifs.
    Variant { path: Vec<String>, sub: Vec<Pattern> },
}

#[derive(Debug, Clone)]
pub enum Lit {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
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
}

#[derive(Debug, Clone, Default)]
pub struct Ast {
    pub items: Vec<Item>,
    /// Chemins `.iri` importés via `use "…"` (interop 50200).
    pub imports: Vec<String>,
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

/// Fonction Rust externe importée via `.iri` (50200) : signature Iron + chemin Rust.
#[derive(Debug, Clone)]
pub struct ExternFn {
    pub params: Vec<Type>,
    pub ret: Type,
    pub path: String,
}

/// Bindings chargés depuis les `.iri` importés (50200).
#[derive(Debug, Clone, Default)]
pub struct Bindings {
    pub fns: HashMap<String, ExternFn>,
    pub types: HashMap<String, String>,
    pub crates: Vec<(String, String)>,
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
