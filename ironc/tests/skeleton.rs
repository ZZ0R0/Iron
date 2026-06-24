//! Gate planify-1.1 : la colonne e2e boote + le marqueur source-map **round-trip**
//! (survit à `prettyplease`). N'invoque pas cargo (test rapide) ; l'e2e cargo est
//! vérifié via `ironc run`.

use ironc::driver;
use ironc::ir::*;

#[test]
fn squelette_compile_et_marqueur_roundtrip() {
    let compiled = driver::compile("fn main():\n    pass\n", 0).expect("compile ok");

    // 1. Le Rust généré porte l'invariant I2 (forbid unsafe) + le marqueur.
    assert!(
        compiled.rust_src.contains("forbid(unsafe_code)"),
        "forbid(unsafe_code) absent :\n{}",
        compiled.rust_src
    );
    assert!(
        compiled.rust_src.contains("__irn_mark"),
        "marqueur absent du Rust généré :\n{}",
        compiled.rust_src
    );

    // 2. Le marqueur a survécu au reformatage et a été relocalisé vers un Span Iron.
    assert!(!compiled.mappings.is_empty(), "aucun mapping source-map résolu");
    let m = &compiled.mappings[0];
    assert!(m.rust_line >= 1, "ligne Rust invalide");
    assert!(m.iron.end >= m.iron.start, "span Iron invalide");
}

#[test]
fn rust_genere_est_du_rust_valide() {
    // compile() échoue si lower produit du Rust que syn ne peut pas parser.
    let compiled = driver::compile("fn main():\n    pass\n", 0).expect("compile ok");
    assert!(compiled.rust_src.contains("fn main()"));
}

#[test]
fn fixtures_compilent_vers_du_rust_valide() {
    for name in ["factorial", "collections", "result", "structs", "braces", "gate"] {
        let src = std::fs::read_to_string(format!("tests/fixtures/{name}.irn")).unwrap();
        let c = driver::compile(&src, 0).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(c.rust_src.contains("forbid(unsafe_code)"), "{name}: forbid(unsafe_code) manquant");
        assert!(!c.mappings.is_empty(), "{name}: source map vide");
    }
}
