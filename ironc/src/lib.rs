//! `ironc` — compilateur Iron → Rust (V0).
//!
//! Squelette intégré (planify-1.1) : pipeline figé (I9) câblé bout en bout,
//! types IR **gelés** dans `ir`, passes en stubs, preuve du mécanisme source-map.
//! Les vagues 2.x remplissent chaque module sans toucher à `ir`.

pub mod bind;
pub mod ir;
pub mod iri;
pub mod lex;
pub mod parse;
pub mod infer;
pub mod resolve;
pub mod lower;
pub mod sourcemap;
pub mod driver;
pub mod lint;
pub mod lsp;
pub mod repl;
