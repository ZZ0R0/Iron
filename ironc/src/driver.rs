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
    pub audit: Vec<Span>,
    pub crates: Vec<(String, String, Vec<String>)>,
    /// Couverture du moteur d'inférence (DA-5) : (signatures résolues, total). Reportée sous `--audit`.
    pub coverage: (usize, usize),
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
/// Registre de bindings curés **embarqués** (V9.1, 52200/52300) : crate → `.iri` compilé dans le binaire.
/// `import std` et les crates « bénis » viennent d'ici ; pas de fichier sur disque, pas de dépendance Cargo.
fn curated_iri(name: &str) -> Option<&'static str> {
    match name {
        "std" => Some(include_str!("../bindings/std.iri")),
        _ => None,
    }
}

pub fn compile(src: &str, file: FileId, base_dir: &Path) -> CResult<Compiled> {
    compile_with(src, file, base_dir, false)
}

/// Comme `compile`, mais choisit l'**émission propre** (`clean = true` → Rust lisible sans marqueurs de
/// source-map, pour `ironc emit` / la reprise à la main). `clean = false` = instrumenté (debug I8).
pub fn compile_with(src: &str, file: FileId, base_dir: &Path, clean: bool) -> CResult<Compiled> {
    let tokens = lex::lex(src, file)?;
    let (mut ast, mut next_id) = parse::parse_with_id(tokens, 0)?;
    // 70200 : charger + fusionner **à plat** les modules Iron importés. NodeId continus (clés des
    // tables annexes → pas de collision), FileId distincts (source maps multi-fichiers), cycles détectés.
    let mut next_file: FileId = file + 1;
    let (mut loaded, mut loading): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    let mut ext = std::mem::take(&mut ast.extern_types);
    let mut curated = Bindings::default();
    for m in std::mem::take(&mut ast.modules) {
        // Résolution d'`import` (V9.1, 52100/52200) — ordre : registre curé embarqué (`import std`) →
        // module Iron `m.irn` → binding `.iri` local du projet.
        if let Some(iri) = curated_iri(&m) {
            let bb = crate::iri::parse(iri, 0)?;
            curated.fns.extend(bb.fns);
            curated.types.extend(bb.types);
            curated.etypes.extend(bb.etypes);
            curated.mods.extend(bb.mods);
            curated.crates.extend(bb.crates);
            curated.consts.extend(bb.consts);
            curated.traits.extend(bb.traits);
            continue;
        }
        if base_dir.join(format!("{m}.irn")).exists() {
            load_module(&m, base_dir, &mut ast.items, &mut ast.imports, &mut ext, &mut next_id, &mut next_file, &mut loaded, &mut loading)?;
            continue;
        }
        // binding `.iri` : cherché à côté du source, puis à la **racine du projet** (parent) et dans le
        // **cwd** — c'est là qu'`ironc add` l'écrit. On pousse le chemin **absolu** (load_bindings le résout
        // tel quel : `base_dir.join(absolu) == absolu`).
        let cands = [
            base_dir.join(format!("{m}.iri")),
            base_dir.join("..").join(format!("{m}.iri")),
            PathBuf::from(format!("{m}.iri")),
        ];
        if let Some(found) = cands.iter().find(|p| p.exists()) {
            let abs = std::fs::canonicalize(found).unwrap_or_else(|_| found.clone());
            ast.imports.push(abs.to_string_lossy().into_owned());
            continue;
        }
        return Err(CompileError::msg(format!(
            "`import {m}` : ni module `{m}.irn`, ni binding `{m}.iri`. Lance `ironc add {m}` (ou fournis `{m}.iri`)."
        )));
    }
    ast.extern_types = ext;
    let mut bindings = load_bindings(&ast.imports, base_dir)?;
    // fusion du registre curé (std…).
    bindings.fns.extend(curated.fns);
    bindings.types.extend(curated.types);
    bindings.etypes.extend(curated.etypes);
    bindings.mods.extend(curated.mods);
    bindings.crates.extend(curated.crates);
    bindings.consts.extend(curated.consts);
    bindings.traits.extend(curated.traits);
    // async (V9, 21200) : si un `async def` est présent, injecter le runtime tokio (D4 : défaut tokio).
    let has_async = ast.items.iter().any(|it| matches!(&it.kind, ItemKind::Fn { is_async: true, .. }));
    if has_async && !bindings.crates.iter().any(|(n, _, _)| n == "tokio") {
        bindings.crates.push(("tokio".to_string(), "1".to_string(), vec!["full".to_string()]));
    }
    // serde typé (V12 [52900]) : si un `@derive(serde::Serialize/Deserialize)` est présent, le crate `serde`
    // doit être en dep AVEC la feature `derive` (qui fournit la macro de dérivation). Comme tokio pour async.
    let uses_serde_derive = ast
        .item_attrs
        .values()
        .flatten()
        .any(|a| a.contains("Serialize") || a.contains("Deserialize"));
    if uses_serde_derive && !bindings.crates.iter().any(|(n, _, _)| n == "serde") {
        bindings.crates.push(("serde".to_string(), "1".to_string(), vec!["derive".to_string()]));
    }
    // V10 (21700+) : moteur d'inférence — résout les types de params/retours non annotés (par contraintes,
    // lit les bindings). Réécrit l'AST avant inférence locale (subsume l'ancienne pré-passe `params`).
    infer::engine::resolve_params(&mut ast, &bindings)?;
    // Couverture du moteur (DA-5) mesurée sur l'AST résolu : signatures concrètes / total.
    let coverage = infer::engine::coverage(&ast);
    // Diagnostic de type Iron natif (D8/22200) : conflits de primitives signalés sur la ligne Iron, avant
    // `rustc` (qui reste le filet pour le reste, D7). Conservateur (zéro faux-positif sur le corpus).
    infer::engine::check_types(&ast, &bindings)?;
    let types = infer::infer(&ast, &bindings);
    // Diagnostics de forme (V11, DA-3) : champ/méthode inconnus + arité, sur type concret connu, avant
    // `rustc` (filet). Zéro faux-positif (n'agit que sur un `Struct` Iron résolu).
    infer::check_shapes(&ast, &types)?;
    let (own, param_modes) = resolve::resolve(&ast, &types);
    let (ts, sm, audit) = lower::lower(&ast, &types, &own, &param_modes, &bindings, clean);

    let syn_file: syn::File = syn::parse2(ts)
        .map_err(|e| CompileError::msg(format!("lowering a produit du Rust invalide : {e}")))?;
    let rust_src = prettyplease::unparse(&syn_file);
    let mappings = sourcemap::resolve(&rust_src, &sm);

    Ok(Compiled { rust_src, source_map: sm, mappings, audit, crates: bindings.crates, coverage })
}

/// Charge récursivement un module Iron `name.irn` et fusionne ses items/imports (à plat) dans le
/// programme. `next_id` thread les `NodeId` (unicité globale), `next_file` les `FileId`. `loaded`
/// (déjà fusionné, évite le double-merge en diamant) et `loading` (pile en cours → cycle). (70200)
#[allow(clippy::too_many_arguments)]
fn load_module(
    name: &str,
    base_dir: &Path,
    items: &mut Vec<Item>,
    imports: &mut Vec<String>,
    extern_types: &mut Vec<(String, String)>,
    next_id: &mut u32,
    next_file: &mut FileId,
    loaded: &mut Vec<String>,
    loading: &mut Vec<String>,
) -> CResult<()> {
    let path = base_dir.join(format!("{name}.irn"));
    let key = path.to_string_lossy().to_string();
    if loaded.contains(&key) {
        return Ok(()); // import en diamant : déjà fusionné
    }
    if loading.contains(&key) {
        return Err(CompileError::msg(format!("cycle d'import détecté sur le module `{name}`")));
    }
    loading.push(key.clone());
    let src = std::fs::read_to_string(&path)
        .map_err(|e| CompileError::msg(format!("import `{name}` ({}) : {e}", path.display())))?;
    let fid = *next_file;
    *next_file += 1;
    let tokens = lex::lex(&src, fid)?;
    let (mod_ast, new_next) = parse::parse_with_id(tokens, *next_id)?;
    *next_id = new_next;
    let mod_dir = path.parent().map(Path::to_path_buf).unwrap_or_else(|| base_dir.to_path_buf());
    // dépendances du module d'abord (relatives au dossier du module).
    for sub in &mod_ast.modules {
        load_module(sub, &mod_dir, items, imports, extern_types, next_id, next_file, loaded, loading)?;
    }
    items.extend(mod_ast.items);
    imports.extend(mod_ast.imports);
    extern_types.extend(mod_ast.extern_types);
    loading.retain(|k| k != &key);
    loaded.push(key);
    Ok(())
}

/// Charge et fusionne les bindings `.iri` importés (chemins relatifs au `.irn`).
fn load_bindings(imports: &[String], base_dir: &Path) -> CResult<Bindings> {
    let mut b = Bindings::default();
    for imp in imports {
        let path = base_dir.join(imp);
        let src = std::fs::read_to_string(&path)
            .map_err(|e| CompileError::msg(format!("lecture binding {} : {e}", path.display())))?;
        let bb = crate::iri::parse(&src, 0)?;
        b.fns.extend(bb.fns);
        b.types.extend(bb.types);
        b.etypes.extend(bb.etypes);
        b.mods.extend(bb.mods);
        b.crates.extend(bb.crates);
        b.consts.extend(bb.consts);
        b.traits.extend(bb.traits);
    }
    Ok(b)
}

/// Écrit un projet cargo autonome `{Cargo.toml, src/main.rs}` dans `out_dir`.
fn deps_toml(crates: &[(String, String, Vec<String>)]) -> String {
    // dédup par nom de crate (deux imports peuvent tirer la même dép transitive, B12) → pas de clé
    // Cargo dupliquée. On garde la 1ʳᵉ occurrence.
    let mut seen = std::collections::HashSet::new();
    crates
        .iter()
        .filter(|(n, _, _)| seen.insert(n.clone()))
        .map(|(n, v, feats)| {
            if feats.is_empty() {
                format!("{n} = \"{v}\"\n")
            } else {
                let fs = feats.iter().map(|f| format!("\"{f}\"")).collect::<Vec<_>>().join(", ");
                format!("{n} = {{ version = \"{v}\", features = [{fs}] }}\n")
            }
        })
        .collect()
}

pub fn emit_project(out_dir: &Path, rust_src: &str, crates: &[(String, String, Vec<String>)]) -> std::io::Result<()> {
    std::fs::create_dir_all(out_dir.join("src"))?;
    let deps = deps_toml(crates);
    // `[workspace]` vide → projet généré **autonome** (jamais membre d'un workspace parent).
    let cargo = format!(
        "[workspace]\n\n[package]\nname = \"iron_out\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[dependencies]\n{deps}"
    );
    std::fs::write(out_dir.join("Cargo.toml"), cargo)?;
    std::fs::write(out_dir.join("src").join("main.rs"), rust_src)?;
    Ok(())
}

/// `ironc lib` (V9, 51800) : émet un crate Rust `{Cargo.toml [lib], src/lib.rs}` réutilisable depuis Rust.
pub fn emit_lib(out_dir: &Path, rust_src: &str, crates: &[(String, String, Vec<String>)]) -> std::io::Result<()> {
    std::fs::create_dir_all(out_dir.join("src"))?;
    let deps = deps_toml(crates);
    let cargo = format!(
        "[workspace]\n\n[package]\nname = \"iron_lib\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n\n[dependencies]\n{deps}"
    );
    std::fs::write(out_dir.join("Cargo.toml"), cargo)?;
    std::fs::write(out_dir.join("src").join("lib.rs"), rust_src)?;
    Ok(())
}

/// Exporte un module Iron en crate Rust (inverse de l'interop, 51800) ; compile pour vérifier.
pub fn lib(input: &Path) -> CResult<()> {
    let src = std::fs::read_to_string(input)
        .map_err(|e| CompileError::msg(format!("lecture {} : {e}", input.display())))?;
    let base_dir = input.parent().unwrap_or(Path::new("."));
    let compiled = compile(&src, 0, base_dir)?;
    let out_dir = out_dir_for(input);
    emit_lib(&out_dir, &compiled.rust_src, &compiled.crates)
        .map_err(|e| CompileError::msg(format!("émission lib : {e}")))?;
    let output = Command::new("cargo")
        .args(["build"])
        .current_dir(&out_dir)
        .output()
        .map_err(|e| CompileError::msg(format!("cargo introuvable : {e}")))?;
    if !output.status.success() {
        return Err(CompileError::msg(remontee(&String::from_utf8_lossy(&output.stderr), &compiled, &src, input)));
    }
    println!("crate Rust émis (réutilisable) : {}", out_dir.display());
    Ok(())
}

/// `ironc doc` (V9.1, 72200) : génère la doc via **overlay rustdoc** — émet le module en crate (lib) et lance
/// `cargo doc`. Les items publics Iron → doc rustdoc.
pub fn doc(input: &Path) -> CResult<()> {
    let src = std::fs::read_to_string(input)
        .map_err(|e| CompileError::msg(format!("lecture {} : {e}", input.display())))?;
    let base_dir = input.parent().unwrap_or(Path::new("."));
    let compiled = compile(&src, 0, base_dir)?;
    let out_dir = out_dir_for(input);
    emit_lib(&out_dir, &compiled.rust_src, &compiled.crates)
        .map_err(|e| CompileError::msg(format!("émission lib : {e}")))?;
    let output = Command::new("cargo")
        .args(["doc", "--quiet", "--no-deps"])
        .current_dir(&out_dir)
        .output()
        .map_err(|e| CompileError::msg(format!("cargo introuvable : {e}")))?;
    if !output.status.success() {
        return Err(CompileError::msg(remontee(&String::from_utf8_lossy(&output.stderr), &compiled, &src, input)));
    }
    println!("doc générée : {}/target/doc/iron_lib/index.html", out_dir.display());
    Ok(())
}

/// `ironc remove <crate>` (V9.1, 72100) : retire le crate d'`ironc.toml` + supprime `<crate>.iri`.
pub fn remove(crate_name: &str) -> CResult<()> {
    let crate_name = &crate_name.replace('-', "_"); // même normalisation que `add` (tirets → underscores)
    let _ = std::fs::remove_file(format!("{crate_name}.iri"));
    if let Ok(toml) = std::fs::read_to_string("ironc.toml") {
        let filtered: Vec<&str> = toml
            .lines()
            .filter(|l| !l.trim_start().starts_with(&format!("{crate_name} =")))
            .collect();
        let _ = std::fs::write("ironc.toml", filtered.join("\n"));
    }
    println!("`{crate_name}` retiré (ironc.toml + {crate_name}.iri).");
    Ok(())
}

/// `ironc update` (V9.1, 72100) : re-binde tous les `.iri` curés du projet (versions à jour, invalide le cache).
pub fn update() -> CResult<()> {
    let mut n = 0;
    if let Ok(entries) = std::fs::read_dir(".") {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("iri") {
                if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                    if add(stem).is_ok() {
                        n += 1;
                    }
                }
            }
        }
    }
    println!("{n} binding(s) mis à jour.");
    Ok(())
}

/// `ironc check` (V9.1, 60800) : diagnostics haute-fidélité par **overlay rustc** (la surcouche). Transpile
/// → `cargo check` → remonte les erreurs rustc en `Span` Iron. Erreurs Iron (lex/parse/inférence) attrapées
/// d'abord par `compile` ; erreurs profondes (types/traits/borrow) par rustc. Zéro faux-positif.
pub fn check(input: &Path) -> CResult<()> {
    let src = std::fs::read_to_string(input)
        .map_err(|e| CompileError::msg(format!("lecture {} : {e}", input.display())))?;
    let base_dir = input.parent().unwrap_or(Path::new("."));
    // niveau rapide : erreurs Iron (front-end). On rend le span en `fichier:ligne:col:` pour que le
    // checkOnSave de VSCode le souligne au bon endroit (I8) — comme la remontée rustc.
    let compiled = compile(&src, 0, base_dir).map_err(|e| located(e, &src, input))?;
    let out_dir = out_dir_for(input);
    emit_project(&out_dir, &compiled.rust_src, &compiled.crates)
        .map_err(|e| CompileError::msg(format!("émission projet : {e}")))?;
    let output = Command::new("cargo")
        .args(["check", "--quiet"])
        .current_dir(&out_dir)
        .output()
        .map_err(|e| CompileError::msg(format!("cargo introuvable : {e}")))?;
    if !output.status.success() {
        // niveau profond : erreurs rustc → remontées en Span Iron (I8).
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CompileError::msg(remontee(&stderr, &compiled, &src, input)));
    }
    println!("✓ aucune erreur ({}).", input.display());
    Ok(())
}

/// `ironc add <crate>` (V9.1, 52300) : binde un crate Rust et écrit `<crate>.iri` dans le dossier courant
/// (que `import <crate>` trouvera) + l'ajoute à `ironc.toml`. Crée un projet temporaire pour rustdoc.
pub fn add(crate_name: &str) -> CResult<()> {
    // Le **nom de fichier** `.iri` (= identifiant d'import) normalise les tirets en underscores :
    // `import num_integer`, pas `num-integer` (illégal comme identifiant). Le **nom de crate** garde son
    // tiret pour `cargo`/le binder (rustdoc), et les chemins Rust sont déjà en underscore.
    let stem = crate_name.replace('-', "_");
    if curated_iri(crate_name).is_some() {
        println!("`{crate_name}` est déjà un binding curé (import direct, rien à ajouter).");
        return Ok(());
    }
    // projet temporaire avec la dépendance → rustdoc-json.
    let tmp = std::env::temp_dir().join(format!("ironc-add-{crate_name}"));
    std::fs::create_dir_all(tmp.join("src"))
        .map_err(|e| CompileError::msg(format!("création projet temp : {e}")))?;
    std::fs::write(
        tmp.join("Cargo.toml"),
        format!("[package]\nname = \"ironc_add\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[dependencies]\n{crate_name} = \"*\"\n"),
    )
    .map_err(|e| CompileError::msg(format!("écriture Cargo.toml : {e}")))?;
    std::fs::write(tmp.join("src/main.rs"), "fn main() {}\n")
        .map_err(|e| CompileError::msg(format!("écriture main.rs : {e}")))?;
    let _ = Command::new("cargo").args(["fetch"]).current_dir(&tmp).output();
    // bind → `.iri`.
    let iri = crate::bind::bind(&tmp, Some(crate_name))?;
    std::fs::write(format!("{stem}.iri"), &iri)
        .map_err(|e| CompileError::msg(format!("écriture {stem}.iri : {e}")))?;
    // ajout à `ironc.toml` (si présent).
    if let Ok(mut toml) = std::fs::read_to_string("ironc.toml") {
        if !toml.contains(&format!("{crate_name} =")) {
            if toml.contains("[dependencies]") {
                toml = toml.replacen("[dependencies]", &format!("[dependencies]\n{crate_name} = \"*\""), 1);
            } else {
                toml.push_str(&format!("\n[dependencies]\n{crate_name} = \"*\"\n"));
            }
            let _ = std::fs::write("ironc.toml", toml);
        }
    }
    println!("`{crate_name}` ajouté : {stem}.iri écrit ({} fns/types bindés). `import {stem}` est prêt.",
        iri.matches("extern ").count());
    Ok(())
}

/// `ironc build --debug <file>` (V9.1, 72300) : build debug (DWARF rustc conservé) **+** export de la
/// source-map I8 dans `sourcemap.json` (ligne Rust → ligne Iron) **+** un `launch.json` CodeLLDB. C'est
/// l'application directe de I8 : le debugger est le consommateur ultime des source-maps. Le mapping fin
/// `.irn`-breakpoint → `.rs`-line est dérivable de `sourcemap.json` par un adaptateur DAP (post-v0).
pub fn build_debug(input: &Path) -> CResult<PathBuf> {
    let src = std::fs::read_to_string(input)
        .map_err(|e| CompileError::msg(format!("lecture {} : {e}", input.display())))?;
    let base_dir = input.parent().unwrap_or(Path::new("."));
    let compiled = compile(&src, 0, base_dir)?;
    let out_dir = out_dir_for(input);
    emit_project(&out_dir, &compiled.rust_src, &compiled.crates)
        .map_err(|e| CompileError::msg(format!("émission projet : {e}")))?;
    let output = Command::new("cargo")
        .args(["build"]) // cargo build = profil debug ⇒ DWARF présent (pas de strip)
        .current_dir(&out_dir)
        .output()
        .map_err(|e| CompileError::msg(format!("cargo introuvable : {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CompileError::msg(remontee(&stderr, &compiled, &src, input)));
    }
    write_debug_artifacts(input, &src, &out_dir, &compiled)?;
    let bin = out_dir.join("target/debug/iron_out");
    println!("build debug : {}", bin.display());
    println!("source-map  : {}", out_dir.join("sourcemap.json").display());
    println!("launch.json : {} (CodeLLDB ; copier dans .vscode/)", out_dir.join("launch.json").display());
    Ok(out_dir)
}

/// Écrit `sourcemap.json` (map ligne Rust → ligne Iron, fichier principal) + `launch.json` (CodeLLDB).
fn write_debug_artifacts(input: &Path, src: &str, out_dir: &Path, compiled: &Compiled) -> CResult<()> {
    // Map ligne Rust → ligne Iron (1-based) pour le fichier principal (FileId 0). Modules multi-fichiers
    // omis au v0 (à indexer par FileId quand le besoin émerge — noté hors-périmètre 72300).
    let mut pairs: Vec<serde_json::Value> = Vec::new();
    for m in &compiled.mappings {
        if m.iron.file == 0 {
            let (line, _) = line_col(src, m.iron.start);
            pairs.push(serde_json::json!({ "rust": m.rust_line, "iron": line }));
        }
    }
    let abs = std::fs::canonicalize(input).unwrap_or_else(|_| input.to_path_buf());
    let bin = out_dir.join("target/debug/iron_out");
    let rs = out_dir.join("src/main.rs");
    let map = serde_json::json!({
        "iron_source": abs.to_string_lossy(),
        "rust_source": rs.to_string_lossy(),
        "binary": bin.to_string_lossy(),
        "note": "I8 source-map : ligne `.rs` → ligne `.irn` (fichier principal). Un adaptateur DAP pose un \
                 breakpoint Iron en cherchant la 1ʳᵉ ligne Rust dont `iron` == la ligne demandée.",
        "mappings": pairs
    });
    std::fs::write(out_dir.join("sourcemap.json"), serde_json::to_string_pretty(&map).unwrap_or_default())
        .map_err(|e| CompileError::msg(format!("écriture sourcemap.json : {e}")))?;
    // launch.json CodeLLDB : débogue le binaire (DWARF complet sur le `.rs` généré). Les lignes Iron se
    // relisent via `sourcemap.json` (I8). Écrit dans out_dir pour ne PAS écraser un `.vscode/` utilisateur.
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("iron");
    let launch = serde_json::json!({
        "version": "0.2.0",
        "configurations": [{
            "type": "lldb",
            "request": "launch",
            "name": format!("Iron: debug {stem}"),
            "program": bin.to_string_lossy(),
            "args": [],
            "cwd": out_dir.to_string_lossy(),
            "sourceLanguages": ["rust"],
            "//": "Breakpoints sur le `.rs` généré (DWARF). Lignes Iron ↔ Rust via sourcemap.json (I8)."
        }]
    });
    std::fs::write(out_dir.join("launch.json"), serde_json::to_string_pretty(&launch).unwrap_or_default())
        .map_err(|e| CompileError::msg(format!("écriture launch.json : {e}")))?;
    Ok(())
}

/// `ironc fmt <file>` (V9.1, 71400) : formate la source **en place** en préservant les commentaires.
/// Valide d'abord que le fichier lexe (indentation cohérente) — on ne reformate jamais un fichier cassé.
pub fn fmt(input: &Path) -> CResult<()> {
    let src = std::fs::read_to_string(input)
        .map_err(|e| CompileError::msg(format!("lecture {} : {e}", input.display())))?;
    crate::lex::lex(&src, 0)?; // garde-fou : indentation lexicalement valide avant reformatage
    let formatted = crate::fmt::format(&src);
    if formatted != src {
        std::fs::write(input, &formatted)
            .map_err(|e| CompileError::msg(format!("écriture {} : {e}", input.display())))?;
        println!("formaté : {}", input.display());
    } else {
        println!("déjà formaté : {}", input.display());
    }
    Ok(())
}

/// `ironc lint` (V9, 60500) : passe de lint sur l'AST ; jamais bloquant (warnings, I4).
pub fn lint(input: &Path) -> CResult<()> {
    let src = std::fs::read_to_string(input)
        .map_err(|e| CompileError::msg(format!("lecture {} : {e}", input.display())))?;
    let toks = crate::lex::lex(&src, 0)?;
    let ast = crate::parse::parse(toks)?;
    let lints = crate::lint::lint(&ast);
    for l in &lints {
        let (line, col) = line_col(&src, l.span.start);
        eprintln!("{}:{}:{}: {} {}", input.display(), line, col, l.code, l.msg);
    }
    println!("{} lint(s).", lints.len());
    Ok(())
}

pub fn build(input: &Path, audit: bool) -> CResult<PathBuf> {
    let src = std::fs::read_to_string(input)
        .map_err(|e| CompileError::msg(format!("lecture {} : {e}", input.display())))?;
    let base_dir = input.parent().unwrap_or(Path::new("."));
    let compiled = compile(&src, 0, base_dir)?;
    if audit {
        for span in &compiled.audit {
            let (l, c) = line_col(&src, span.start);
            eprintln!("{}:{}:{}: perf: clone-fallback (I4, --audit)", input.display(), l, c);
        }
        let (res, tot) = compiled.coverage;
        let pct = if tot == 0 { 100.0 } else { 100.0 * res as f64 / tot as f64 };
        eprintln!("couverture moteur d'inférence : {res}/{tot} signatures résolues ({pct:.0} %) [DA-5]");
    }
    let out_dir = out_dir_for(input);
    emit_project(&out_dir, &compiled.rust_src, &compiled.crates)
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

pub fn run(input: &Path, audit: bool) -> CResult<()> {
    let out_dir = build(input, audit)?;
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

/// Configuration `[iron]` du manifeste `ironc.toml` (le « cargo » d'Iron). Tout est optionnel.
struct IronCfg {
    /// `emit = "clean"|"instrumented"` — lisibilité du Rust de `ironc emit` (défaut : `clean`).
    emit_clean: bool,
    /// `rust-out = "dir"` — où `ironc emit` écrit le projet Rust (défaut : `<stem>-rust/`).
    rust_out: Option<String>,
}

fn iron_cfg(base_dir: &Path) -> IronCfg {
    let mut cfg = IronCfg { emit_clean: true, rust_out: None };
    let text = std::fs::read_to_string(base_dir.join("ironc.toml"))
        .or_else(|_| std::fs::read_to_string("ironc.toml"))
        .ok();
    if let Some(t) = text {
        let mut in_iron = false;
        for line in t.lines() {
            let l = line.trim();
            if l.starts_with('[') {
                in_iron = l == "[iron]";
                continue;
            }
            if !in_iron {
                continue;
            }
            if let Some((k, v)) = l.split_once('=') {
                let (k, v) = (k.trim(), v.trim().trim_matches('"'));
                match k {
                    "emit" => cfg.emit_clean = v != "instrumented",
                    "rust-out" | "rust_out" => cfg.rust_out = Some(v.to_string()),
                    _ => {}
                }
            }
        }
    }
    cfg
}

/// `[iron] entry` du manifeste `ironc.toml` du dossier courant — fichier source par défaut d'un projet.
pub fn manifest_entry() -> Option<String> {
    let t = std::fs::read_to_string("ironc.toml").ok()?;
    let mut in_iron = false;
    for line in t.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            in_iron = l == "[iron]";
            continue;
        }
        if in_iron {
            if let Some((k, v)) = l.split_once('=') {
                if k.trim() == "entry" {
                    return Some(v.trim().trim_matches('"').to_string());
                }
            }
        }
    }
    None
}

/// `ironc emit <fichier>` : transpile en **Rust lisible** (sans marqueurs de source-map) dans un dossier
/// **visible** (projet cargo complet `Cargo.toml` + `src/main.rs`), **sans compiler** — pour lire, éditer et
/// fignoler le Rust à la main, puis `cargo run` dessus. C'est l'esprit `nimcache` : le `.rs` intermédiaire
/// devient un artefact de premier ordre. Mode via `[iron] emit` (défaut `clean`), emplacement via
/// `[iron] rust-out`. `--instrumented` force la version avec marqueurs (source-maps).
pub fn emit(input: &Path, instrumented_flag: bool, no_std: bool) -> CResult<()> {
    let src = std::fs::read_to_string(input)
        .map_err(|e| CompileError::msg(format!("lecture {} : {e}", input.display())))?;
    let base_dir = input.parent().unwrap_or(Path::new("."));
    let cfg = iron_cfg(base_dir);
    let clean = if instrumented_flag { false } else { cfg.emit_clean };
    let compiled = compile_with(&src, 0, base_dir, clean).map_err(|e| located(e, &src, input))?;
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
    let out_dir = match &cfg.rust_out {
        Some(d) => base_dir.join(d),
        None => base_dir.join(format!("{stem}-rust")),
    };
    // V12 [43000] : mode `no_std` (embarqué). On préfixe l'attribut crate `#![no_std]` + `alloc` (qui fournit
    // `String`/`Vec`/`format!` sans `std`) et on émet une **lib** (pas de `main`/panic-handler à fournir ici —
    // le binaire final / le HAL les apporte). Iron continue d'émettre **0 `unsafe`** (I2) : l'`unsafe` éventuel
    // vit dans la crate-HAL. NB : `print` (→ `println!`) est std-only → un programme no_std passe par un HAL.
    if no_std {
        // Les attributs internes `#![…]` doivent précéder tout `use`/item. On insère `#![no_std]` en tête, on
        // conserve les `#![…]` existants, PUIS on ajoute `extern crate alloc` + les imports alloc, PUIS le corps.
        let lines: Vec<&str> = compiled.rust_src.lines().collect();
        let split = lines.iter().position(|l| !l.trim_start().starts_with("#![")).unwrap_or(0);
        let (attrs, body) = lines.split_at(split);
        let imports = "extern crate alloc;\nuse alloc::string::{String, ToString};\nuse alloc::vec::Vec;\nuse alloc::{vec, format, boxed::Box};";
        let lib_src = format!("#![no_std]\n{}\n{}\n\n{}", attrs.join("\n"), imports, body.join("\n"));
        emit_lib(&out_dir, &lib_src, &compiled.crates)
            .map_err(|e| CompileError::msg(format!("émission lib no_std : {e}")))?;
        println!(
            "Rust no_std (lib) : {0}/src/lib.rs\n  → cible embarquée : cd {0} && cargo build [--target <thumb…/xtensa…>]",
            out_dir.display()
        );
        return Ok(());
    }
    emit_project(&out_dir, &compiled.rust_src, &compiled.crates)
        .map_err(|e| CompileError::msg(format!("émission projet : {e}")))?;
    let mode = if clean { "propre" } else { "instrumenté" };
    println!(
        "Rust {mode} : {0}/src/main.rs\n  → édite-le, puis : cd {0} && cargo run",
        out_dir.display()
    );
    Ok(())
}

/// `ironc test <fichier>` : compile + lance `cargo test` (les `def test_*()` deviennent des `#[test]`).
pub fn test(input: &Path) -> CResult<()> {
    let out_dir = build(input, false)?;
    let status = Command::new("cargo")
        .args(["test"])
        .current_dir(&out_dir)
        .status()
        .map_err(|e| CompileError::msg(format!("cargo test : {e}")))?;
    if !status.success() {
        return Err(CompileError::msg("des tests ont échoué"));
    }
    Ok(())
}

/// `ironc new <nom>` : scaffolding d'un projet Iron (manifeste + src/main.irn + .gitignore).
pub fn new_project(name: &str) -> CResult<()> {
    let dir = Path::new(name);
    if dir.exists() {
        return Err(CompileError::msg(format!("le dossier `{name}` existe déjà")));
    }
    let w = |p: PathBuf, c: &str| {
        std::fs::write(&p, c).map_err(|e| CompileError::msg(format!("écriture {} : {e}", p.display())))
    };
    std::fs::create_dir_all(dir.join("src"))
        .map_err(|e| CompileError::msg(format!("création {name}/src : {e}")))?;
    w(dir.join("src/main.irn"), "def main():\n    print(\"hello from Iron\")\n")?;
    w(
        dir.join("ironc.toml"),
        &format!(
            "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[iron]\nentry = \"src/main.irn\"\n\
             # emit = \"clean\"      # Rust produit par `ironc emit` : clean (lisible) | instrumented (source-maps)\n\
             # rust-out = \"rust\"   # dossier où `ironc emit` écrit le Rust (défaut : <stem>-rust/)\n"
        ),
    )?;
    w(dir.join(".gitignore"), "/target\n/ironc-out\n")?;
    println!("Projet `{name}` créé.  cd {name} && ironc run src/main.irn");
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

/// Rend une `CompileError` front-end avec un `Span` sous la forme `fichier:ligne:col: msg` (I8) — pour le
/// CLI et le regex `checkOnSave` de VSCode. Sans span : message inchangé.
fn located(e: CompileError, src: &str, input: &Path) -> CompileError {
    match e.span {
        Some(sp) => {
            let (l, c) = line_col(src, sp.start);
            CompileError { span: Some(sp), msg: format!("{}:{}:{}: {}", input.display(), l, c, e.msg) }
        }
        None => e,
    }
}

pub fn out_dir_for(input: &Path) -> PathBuf {
    let stem = input.file_stem().and_then(|s| s.to_str()).unwrap_or("out");
    // Hash du chemin **complet** → projets parallèles distincts (sinon tous les `main.irn` collisionnent).
    let full = std::fs::canonicalize(input).unwrap_or_else(|_| input.to_path_buf());
    let mut h: u64 = 1469598103934665603; // FNV-1a
    for b in full.to_string_lossy().bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    std::env::temp_dir().join("ironc-out").join(format!("{stem}_{h:016x}"))
}
