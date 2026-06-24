//! [60100] Source maps — squelette (planify-1.1). Étendu en planify-2.6 (⚠ risque n°1).
//!
//! Relocalise les marqueurs `__irn_mark!(IDX)` **après `prettyplease`** et les relie à
//! leur origine Iron via la `SourceMap`. Preuve que le mapping survit au reformatage (I8).

use crate::ir::*;

/// Un marqueur relocalisé : origine Iron ↔ ligne Rust générée.
#[derive(Debug, Clone, PartialEq)]
pub struct Mapping {
    pub node: NodeId,
    pub iron: Span,
    pub rust_line: u32,
}

/// Scanne le Rust **formaté** et résout chaque marqueur via ses `links`.
pub fn resolve(formatted_rust: &str, sm: &SourceMap) -> Vec<Mapping> {
    let mut out = Vec::new();
    for (lineno, line) in formatted_rust.lines().enumerate() {
        if let Some(idx) = parse_marker(line) {
            if let Some(link) = sm.links.get(idx) {
                out.push(Mapping {
                    node: link.node,
                    iron: link.iron,
                    rust_line: lineno as u32 + 1,
                });
            }
        }
    }
    out
}

/// Extrait l'index entre parenthèses de `__irn_mark!( IDX )` (tolère le formatage).
fn parse_marker(line: &str) -> Option<usize> {
    let pos = line.find("__irn_mark")?;
    let after = &line[pos..];
    let lp = after.find('(')?;
    let digits: String = after[lp + 1..]
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse::<usize>().ok()
}
