//! CLI `ironc build|run [--audit] <fichier.irn>`.

use ironc::{bind, driver, lsp, repl};
use std::path::Path;
use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let audit = args.iter().any(|a| a == "--audit");
    let debug = args.iter().any(|a| a == "--debug");
    let no_std = args.iter().any(|a| a == "--no-std"); // V12 [43000] : émission `#![no_std]` + alloc (embarqué)
    let strict = args.iter().any(|a| a == "--strict"); // V15 [50500] : `add` échoue si couverture < seuil
    let positional: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    // `ironc lsp` : serveur de langage (stdio, sans fichier).
    if positional.first().map(|s| s.as_str()) == Some("lsp") {
        exit(if lsp::lsp().is_ok() { 0 } else { 1 });
    }
    // `ironc update` : re-binde tous les `.iri` du projet (sans fichier).
    if positional.first().map(|s| s.as_str()) == Some("repl") {
        exit(if repl::repl().is_ok() { 0 } else { 1 });
    }
    if positional.first().map(|s| s.as_str()) == Some("update") {
        exit(if driver::update().is_ok() { 0 } else { 1 });
    }
    let instrumented = args.iter().any(|a| a == "--instrumented"); // `emit` : garder les marqueurs
    let cmd = positional.first().map(|s| s.as_str()).unwrap_or("");
    // Projet Iron : `ironc build|run|emit|check|test` **sans fichier** → utilise `[iron] entry` de ironc.toml.
    let usage = "usage: ironc <build|run|emit|test|check|fmt|new|add|bind> [--audit|--debug|--instrumented] <fichier.irn | nom>";
    let input_owned: String = if positional.len() >= 2 {
        positional[1].clone()
    } else if matches!(cmd, "build" | "run" | "emit" | "check" | "test") {
        match driver::manifest_entry() {
            Some(e) => e,
            None => {
                eprintln!("{usage}");
                exit(2);
            }
        }
    } else {
        eprintln!("{usage}");
        exit(2);
    };
    let input = Path::new(&input_owned);
    let res = match cmd {
        "build" if debug => driver::build_debug(input).map(|_| ()), // build + source-map I8 + launch.json (72300)
        "build" => driver::build(input, audit).map(|_| ()),
        "emit" => driver::emit(input, instrumented, no_std), // transpile → Rust lisible (sans compiler)
        "run" => driver::run(input, audit),
        "test" => driver::test(input),
        "new" => driver::new_project(positional[1].as_str()),
        "add" => driver::add(positional[1].as_str(), strict),
        "remove" => driver::remove(positional[1].as_str()),
        "check" => driver::check(input),
        "fmt" => driver::fmt(input),
        "doc" => driver::doc(input),
        "lib" => driver::lib(input),
        "lint" => driver::lint(input),
        "bind" => bind::bind(input, positional.get(2).map(|s| s.as_str())).map(|out| print!("{}", out.iri)),
        other => {
            eprintln!("ironc: commande inconnue : {other}");
            exit(2);
        }
    };
    if let Err(e) = res {
        eprintln!("ironc: erreur: {e}");
        exit(1);
    }
}
