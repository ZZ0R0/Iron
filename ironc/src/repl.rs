//! [72000] REPL Iron (V9.1) : compile-and-run par snippet (Iron n'a pas d'interpréteur, I9). Items
//! (`def`/`class`) accumulés au top-level, statements dans un `main`, expression seule → affichée. Sortie
//! en **diff** (seule la nouvelle s'affiche) ; **rollback** de l'accumulateur sur erreur.

use crate::ir::CResult;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::Command;

pub fn repl() -> CResult<()> {
    let mut items: Vec<String> = Vec::new();
    let mut body: Vec<String> = Vec::new();
    let mut prev = String::new();
    println!("Iron REPL — `exit` pour quitter.");
    let stdin = io::stdin();
    prompt();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let t = line.trim().to_string();
        if t.is_empty() {
            prompt();
            continue;
        }
        if t == "exit" || t == ":q" {
            break;
        }
        let is_item = ["def ", "class ", "enum ", "import ", "use ", "extern ", "@"]
            .iter()
            .any(|p| t.starts_with(p));
        // accumulateurs candidats.
        let mut try_items = items.clone();
        let mut try_body = body.clone();
        if is_item {
            try_items.push(line.clone());
        } else {
            try_body.push(if is_bare_expr(&t) { format!("print({t})") } else { line.clone() });
        }
        let prog = assemble(&try_items, &try_body);
        match run_capture(&prog) {
            Ok(out) => {
                // diff : n'afficher que la nouvelle sortie.
                let new = out.strip_prefix(&prev).unwrap_or(&out);
                print!("{new}");
                let _ = io::stdout().flush();
                prev = out;
                items = try_items; // commit
                body = try_body;
            }
            Err(e) => {
                eprintln!("{}", e.lines().next().unwrap_or(&e)); // rollback (pas de commit)
            }
        }
        prompt();
    }
    Ok(())
}

fn prompt() {
    print!(">>> ");
    let _ = io::stdout().flush();
}

/// Une ligne est-elle une **expression nue** (à `print`) plutôt qu'un statement ?
fn is_bare_expr(t: &str) -> bool {
    if t.ends_with(':') {
        return false;
    }
    // assignation `x = …` (mais pas `==`/`<=`/`>=`/`!=`).
    if t.contains('=') && !t.contains("==") && !t.contains("<=") && !t.contains(">=") && !t.contains("!=") {
        return false;
    }
    let kw = ["return", "print", "for", "while", "if", "raise", "assert", "break", "continue", "pass", "yield"];
    !kw.iter().any(|k| t.starts_with(k))
}

fn assemble(items: &[String], body: &[String]) -> String {
    let body_src = if body.is_empty() {
        "    pass".to_string()
    } else {
        body.iter().map(|l| format!("    {l}")).collect::<Vec<_>>().join("\n")
    };
    format!("{}\n\ndef main():\n{}\n", items.join("\n"), body_src)
}

/// Compile + exécute le programme, capture stdout. `Err` = message (rollback).
fn run_capture(prog: &str) -> Result<String, String> {
    let base = std::env::temp_dir().join("ironc-repl");
    let _ = std::fs::create_dir_all(&base);
    let irn = base.join("session.irn");
    std::fs::write(&irn, prog).map_err(|e| e.to_string())?;
    let compiled = super::driver::compile(prog, 0, &base).map_err(|e| e.msg)?;
    let out_dir = super::driver::out_dir_for(&irn);
    super::driver::emit_project(&out_dir, &compiled.rust_src, &compiled.crates).map_err(|e| e.to_string())?;
    let output = Command::new("cargo")
        .args(["run", "--quiet"])
        .current_dir(&out_dir)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(remap(&String::from_utf8_lossy(&output.stderr), &irn))
    }
}

fn remap(stderr: &str, _irn: &Path) -> String {
    stderr
        .lines()
        .find(|l| l.contains("error"))
        .unwrap_or("erreur de compilation")
        .to_string()
}
