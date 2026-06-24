//! [70100] Driver cargo + CLI — réel (planify-2.7) + remontée diagnostics (planify-2.6, I8/I2).
//!
//! Câble le pipeline figé (I9) : `lex → parse → infer → resolve → lower → prettyplease`,
//! matérialise un projet cargo, invoque `cargo`. En cas d'échec `rustc`, **remonte** l'erreur
//! au `Span` Iron via les marqueurs (pire cas = erreur remontée proprement, jamais du Rust brut).

use crate::ir::*;
use crate::{infer, lex, lower, parse, resolve, sourcemap};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Résultat de la compilation (avant cargo) : Rust formaté + source map résolue.
pub struct Compiled {
    pub rust_src: String,
    pub source_map: SourceMap,
    pub mappings: Vec<sourcemap::Mapping>,
}

impl Compiled {
    /// `Span` Iron gouvernant une ligne Rust : le dernier marqueur à `rust_line <= line`.
    pub fn locate(&self, rust_line: u32) -> Option<Span> {
        self.mappings
            .iter()
            .filter(|m| m.rust_line <= rust_line)
            .max_by_key(|m| m.rust_line)
            .map(|m| m.iron)
    }
}

/// Pipeline figé (I9), sans invoquer cargo. Testable isolément.
pub fn compile(src: &str, file: FileId) -> CResult<Compiled> {
    let tokens = lex::lex(src, file)?;
    let ast = parse::parse(tokens)?;
    let types = infer::infer(&ast);
    let own = resolve::resolve(&ast, &types);
    let (ts, sm) = lower::lower(&ast, &types, &own);

    let syn_file: syn::File = syn::parse2(ts)
        .map_err(|e| CompileError::msg(format!("lowering a produit du Rust invalide : {e}")))?;
    let rust_src = prettyplease::unparse(&syn_file);
    let mappings = sourcemap::resolve(&rust_src, &sm);

    Ok(Compiled { rust_src, source_map: sm, mappings })
}

/// Écrit un projet cargo autonome `{Cargo.toml, src/main.rs}` dans `out_dir`.
pub fn emit_project(out_dir: &Path, rust_src: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(out_dir.join("src"))?;
    std::fs::write(out_dir.join("Cargo.toml"), CARGO_TOML)?;
    std::fs::write(out_dir.join("src").join("main.rs"), rust_src)?;
    Ok(())
}

// `[workspace]` vide → projet généré **autonome** (jamais membre d'un workspace parent).
const CARGO_TOML: &str = "[workspace]\n\n[package]\nname = \"iron_out\"\nversion = \"0.0.0\"\nedition = \"2021\"\n";

pub fn build(input: &Path) -> CResult<PathBuf> {
    let src = std::fs::read_to_string(input)
        .map_err(|e| CompileError::msg(format!("lecture {} : {e}", input.display())))?;
    let compiled = compile(&src, 0)?;
    let out_dir = out_dir_for(input);
    emit_project(&out_dir, &compiled.rust_src)
        .map_err(|e| CompileError::msg(format!("émission projet : {e}")))?;

    let output = Command::new("cargo")
        .args(["build"])
        .current_dir(&out_dir)
        .output()
        .map_err(|e| CompileError::msg(format!("cargo introuvable : {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CompileError::msg(remontee(&stderr, &compiled, &src, input)));
    }
    Ok(out_dir)
}

pub fn run(input: &Path) -> CResult<()> {
    let out_dir = build(input)?;
    let status = Command::new("cargo")
        .args(["run", "--quiet"])
        .current_dir(&out_dir)
        .status()
        .map_err(|e| CompileError::msg(format!("cargo run : {e}")))?;
    if !status.success() {
        return Err(CompileError::msg("exécution échouée"));
    }
    Ok(())
}

/// Remonte une erreur du Rust généré vers la source Iron (I8/I2).
fn remontee(stderr: &str, compiled: &Compiled, src: &str, path: &Path) -> String {
    if let Some(rust_line) = find_rust_error_line(stderr) {
        if let Some(span) = compiled.locate(rust_line) {
            let (l, c) = line_col(src, span.start);
            let detail = stderr
                .lines()
                .find(|x| x.trim_start().starts_with("error"))
                .unwrap_or("erreur du compilateur Rust")
                .trim();
            return format!(
                "{}:{}:{}: {}\n  ↳ remontée depuis le Rust généré (main.rs:{})",
                path.display(),
                l,
                c,
                detail,
                rust_line
            );
        }
    }
    format!("échec de compilation du Rust généré :\n{}", stderr.trim())
}

/// 1ʳᵉ ligne `main.rs:N` apparaissant **après** un `error…` (ignore les warnings).
fn find_rust_error_line(stderr: &str) -> Option<u32> {
    let mut armed = false;
    for line in stderr.lines() {
        if line.trim_start().starts_with("error") {
            armed = true;
        }
        if armed {
            if let Some(pos) = line.find("main.rs:") {
                let rest = &line[pos + "main.rs:".len()..];
                let num: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
                if let Ok(n) = num.parse::<u32>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

/// Ligne/colonne (1-based) d'un offset octet dans la source.
fn line_col(src: &str, byte: u32) -> (u32, u32) {
    let b = byte as usize;
    let mut line = 1u32;
    let mut col = 1u32;
    for (i, ch) in src.char_indices() {
        if i >= b {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn out_dir_for(input: &Path) -> PathBuf {
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
    std::env::temp_dir().join("ironc-out").join(stem)
}
