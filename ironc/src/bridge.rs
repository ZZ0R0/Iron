//! Le **pont** de matérialisation Iron↔Rust (V12, [24000] — RACINE 1).
//!
//! Source **unique** de la correspondance Iron↔Rust, pour mettre fin à son encodage en triple (projection
//! `bind` / inférence `infer` / coercition `lower`) qui a engendré la dette B5→B33. Construit en vagues :
//! - **Vague 1 (ici)** : `rust_num_name` (table numérique canonique) + l'ABI `Type::ExternApp` (dans `ir`).
//! - **Vagues 2-3** : projection/coercition/inférence des **génériques** délèguent au pont (stratégie n°1).
//! - **Vagues 4-6/8** : ajout des **stratégies** « borne → matérialisation » (str, octets, io, closures, serde).
//!
//! Principe directeur : ajouter le support d'un motif de crate = **une entrée de donnée** consommée par les
//! trois couches, jamais un correctif dispersé.

use crate::ir::NumKind;

/// Nom Rust **canonique** d'un `NumKind` (V7, 20800 ; centralisé V12). Seule définition — `lower::num_kind_str`
/// et toute coercition numérique délèguent ici (R1, vague 2). `i64`/`f64` restent `Type::I64`/`F64` (défauts).
pub fn rust_num_name(k: NumKind) -> &'static str {
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
