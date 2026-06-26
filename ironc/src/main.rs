//! CLI `ironc build|run [--audit] <fichier.irn>`.

use ironc::{bind, driver, lsp};
use std::path::Path;
use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let audit = args.iter().any(|a| a == "--audit");
    let positional: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    // `ironc lsp` : serveur de langage (stdio, sans fichier).
    if positional.first().map(|s| s.as_str()) == Some("lsp") {
        exit(if lsp::lsp().is_ok() { 0 } else { 1 });
    }
    // `ironc update` : re-binde tous les `.iri` du projet (sans fichier).
    if positional.first().map(|s| s.as_str()) == Some("update") {
        exit(if driver::update().is_ok() { 0 } else { 1 });
    }
    if positional.len() < 2 {
        eprintln!("usage: ironc <build|run|test|bind|new> [--audit] <fichier.irn | nom>");
        exit(2);
    }
    let input = Path::new(positional[1].as_str());
    let res = match positional[0].as_str() {
        "build" => driver::build(input, audit).map(|_| ()),
        "run" => driver::run(input, audit),
        "test" => driver::test(input),
        "new" => driver::new_project(positional[1].as_str()),
        "add" => driver::add(positional[1].as_str()),
        "remove" => driver::remove(positional[1].as_str()),
        "check" => driver::check(input),
        "doc" => driver::doc(input),
        "lib" => driver::lib(input),
        "lint" => driver::lint(input),
        "bind" => bind::bind(input, positional.get(2).map(|s| s.as_str())).map(|iri| print!("{iri}")),
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
