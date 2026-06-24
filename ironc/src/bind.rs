//! [50300] Auto-génération de bindings `.iri` depuis `rustdoc --output-format json` (interop niveau 3).
//!
//! Best-effort : extrait les fonctions **publiques** dont la signature se **projette** vers le
//! sous-ensemble Iron (I6) ; les autres (génériques, `unsafe`, types non mappables) sont **sautées et
//! comptées**, jamais bindées à tort. Cible rustdoc-json **v57** (nightly) ; dégrade proprement.

use crate::ir::*;
use std::path::Path;
use std::process::Command;

/// Lance rustdoc-json sur la crate (dossier avec `Cargo.toml`) et renvoie le `.iri` généré.
pub fn bind(crate_dir: &Path, pkg: Option<&str>) -> CResult<String> {
    let manifest = std::fs::read_to_string(crate_dir.join("Cargo.toml"))
        .map_err(|e| CompileError::msg(format!("lecture Cargo.toml : {e}")))?;
    // Crate ciblée : `pkg` (dépendance) ou la crate locale.
    let target = match pkg {
        Some(p) => p.to_string(),
        None => pkg_name(&manifest).ok_or_else(|| CompileError::msg("nom de crate introuvable dans Cargo.toml"))?,
    };

    let mut cmd = Command::new("cargo");
    cmd.args(["+nightly", "rustdoc"]);
    if let Some(p) = pkg {
        cmd.args(["-p", p]);
    }
    cmd.arg("--manifest-path")
        .arg(crate_dir.join("Cargo.toml"))
        .args(["--", "-Zunstable-options", "--output-format", "json"]);
    let out = cmd
        .output()
        .map_err(|e| CompileError::msg(format!("`cargo +nightly rustdoc` : {e} (nightly requis)")))?;
    if !out.status.success() {
        return Err(CompileError::msg(format!(
            "rustdoc-json a échoué :\n{}",
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }

    let json_path = crate_dir
        .join("target/doc")
        .join(format!("{}.json", target.replace('-', "_")));
    let json = std::fs::read_to_string(&json_path)
        .map_err(|e| CompileError::msg(format!("lecture {} : {e}", json_path.display())))?;
    let v: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| CompileError::msg(format!("JSON rustdoc invalide : {e}")))?;

    emit(&v, &target)
}

fn emit(v: &serde_json::Value, crate_name: &str) -> CResult<String> {
    let index = v["index"]
        .as_object()
        .ok_or_else(|| CompileError::msg("rustdoc-json : `.index` absent (format inattendu)"))?;
    let version = v["crate_version"].as_str().unwrap_or("*");
    let mut iri = format!(
        "# bindings auto-générés depuis `{crate_name}` (ironc bind)\ncrate {crate_name} = \"{version}\"\n"
    );
    let (mut bound, mut skipped) = (0usize, 0usize);

    for item in index.values() {
        let f = &item["inner"]["function"];
        if !f.is_object() || item["visibility"].as_str() != Some("public") {
            continue;
        }
        let name = match item["name"].as_str() {
            Some(n) => n,
            None => continue,
        };
        let has_generics = f["generics"]["params"].as_array().map_or(false, |a| !a.is_empty());
        let is_unsafe = f["header"]["is_unsafe"].as_bool() == Some(true);
        let inputs = f["sig"]["inputs"].as_array();
        if has_generics || is_unsafe || inputs.is_none() {
            skipped += 1;
            continue;
        }
        let mut params = Vec::new();
        let mut mappable = true;
        for inp in inputs.unwrap() {
            let pn = inp[0].as_str().unwrap_or("x");
            match map_type(&inp[1]) {
                Some(t) => params.push(format!("{pn}: {t}")),
                None => {
                    mappable = false;
                    break;
                }
            }
        }
        let ret = match &f["sig"]["output"] {
            serde_json::Value::Null => Some("unit".to_string()),
            t => map_type(t),
        };
        match (mappable, ret) {
            (true, Some(r)) => {
                iri.push_str(&format!(
                    "extern fn {name}({}) -> {r} = \"{crate_name}::{name}\"\n",
                    params.join(", ")
                ));
                bound += 1;
            }
            _ => skipped += 1,
        }
    }

    iri.push_str(&format!("# {bound} fn(s) bindée(s), {skipped} item(s) non mappable(s) sauté(s)\n"));
    Ok(iri)
}

/// Projette un type rustdoc-json vers le sous-ensemble Iron (I6). Mapping **exact** (pas de
/// widening : un `i32` mal mappé en `i64` produirait un binding faux).
fn map_type(t: &serde_json::Value) -> Option<String> {
    match t["primitive"].as_str() {
        Some("i64") => Some("i64".to_string()),
        Some("f64") => Some("f64".to_string()),
        Some("bool") => Some("bool".to_string()),
        _ => None,
    }
}

fn pkg_name(manifest: &str) -> Option<String> {
    let mut in_pkg = false;
    for line in manifest.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            in_pkg = l == "[package]";
        }
        if in_pkg {
            if let Some(rest) = l.strip_prefix("name") {
                if let Some(eq) = rest.find('=') {
                    return Some(rest[eq + 1..].trim().trim_matches('"').to_string());
                }
            }
        }
    }
    None
}
