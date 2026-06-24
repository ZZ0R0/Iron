//! CLI `ironc build|run [--audit] <fichier.irn>`.

use ironc::driver;
use std::path::Path;
use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let audit = args.iter().any(|a| a == "--audit");
    let positional: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    if positional.len() < 2 {
        eprintln!("usage: ironc <build|run> [--audit] <fichier.irn>");
        exit(2);
    }
    let input = Path::new(positional[1].as_str());
    let res = match positional[0].as_str() {
        "build" => driver::build(input, audit).map(|_| ()),
        "run" => driver::run(input, audit),
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
