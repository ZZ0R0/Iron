//! CLI `ironc build|run <fichier.irn>` — squelette (planify-1.1).

use ironc::driver;
use std::path::Path;
use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: ironc <build|run> <fichier.irn>");
        exit(2);
    }
    let input = Path::new(&args[2]);
    let res = match args[1].as_str() {
        "build" => driver::build(input).map(|_| ()),
        "run" => driver::run(input),
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
