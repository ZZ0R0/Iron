//! [50300/50900] Auto-génération de bindings `.iri` depuis `rustdoc --output-format json`.
//!
//! Best-effort (D5) : extrait les fonctions/méthodes/constructeurs **publics** dont la signature se
//! **projette** vers le sous-ensemble Iron (I6) via `map_type` récursif (51000) ; les autres (génériques
//! nus, `unsafe`, types non mappables) sont **sautés et comptés**, jamais bindés à tort. Extrait la POO
//! (méthodes/ctors des `impl` inhérents, 51100) et reconnaît les idiomes builder/getter (51300).
//! Cible rustdoc-json **v57** (nightly) ; dégrade proprement.

use crate::ir::*;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::path::Path;
use std::process::Command;

/// Lance rustdoc-json sur la crate (dossier avec `Cargo.toml`) et renvoie le `.iri` généré.
pub fn bind(crate_dir: &Path, pkg: Option<&str>) -> CResult<String> {
    let manifest = std::fs::read_to_string(crate_dir.join("Cargo.toml"))
        .map_err(|e| CompileError::msg(format!("lecture Cargo.toml : {e}")))?;
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

    // Versions résolues des dépendances (transitives incluses) — pour déclarer les crates **étrangères**
    // référencées dans les chemins de traits UFCS (`num_traits::…`) comme deps Cargo (B12). Clé = nom de
    // module (underscores) ⇒ (nom publié, version) ; permet de retrouver `num-traits` depuis `num_traits`.
    let lock = std::fs::read_to_string(crate_dir.join("Cargo.lock")).unwrap_or_default();
    let mut versions: HashMap<String, (String, String)> = HashMap::new();
    let mut name: Option<String> = None;
    for line in lock.lines() {
        let l = line.trim();
        if let Some(n) = l.strip_prefix("name = \"").and_then(|s| s.strip_suffix('"')) {
            name = Some(n.to_string());
        } else if let Some(ver) = l.strip_prefix("version = \"").and_then(|s| s.strip_suffix('"')) {
            if let Some(n) = name.take() {
                versions.insert(n.replace('-', "_"), (n, ver.to_string()));
            }
        }
    }

    emit(&v, &target.replace('-', "_"), &target, &versions)
}

type Json = serde_json::Value;

/// Signature projetée d'une fn/méthode, ou `None` si non mappable.
struct Sig {
    /// `recv` éventuel (méthode) : `"self"` (ByRef), `"mut self"` (ByRefMut), `"own self"` (ByValue).
    recv: Option<&'static str>,
    /// `nom: type` Iron (sans le receveur).
    params: Vec<String>,
    ret: String,
    idioms: Vec<&'static str>,
}

/// `id → chemin **public**` en parcourant l'arbre des modules **publics** + les re-exports `pub use`
/// (y compris globs `pub use m::*`). rustdoc donne le chemin **canonique**, qui peut traverser des
/// modules **privés/restreints** (ex. `statrs::statistics::traits::Distribution`) ; le chemin réellement
/// compilable est le **re-export** (`statrs::statistics::Distribution`). Sans ça, une méthode de trait
/// dont le trait vit dans un module privé glob-réexporté donne un chemin UFCS qui ne compile pas
/// (`error[E0603]: module \`traits\` is private`). BFS ⇒ on garde le chemin le plus **court** par id.
fn build_public_paths(index: &serde_json::Map<String, Json>, root: u64, crate_name: &str) -> HashMap<u64, String> {
    let mut out: HashMap<u64, Vec<String>> = HashMap::new();
    let mut visited: HashSet<u64> = HashSet::new();
    let mut queue: VecDeque<(u64, Vec<String>)> = VecDeque::new();
    queue.push_back((root, vec![crate_name.to_string()]));
    let shorter = |out: &mut HashMap<u64, Vec<String>>, id: u64, p: Vec<String>| match out.get(&id) {
        Some(old) if old.len() <= p.len() => {}
        _ => {
            out.insert(id, p);
        }
    };
    let is_module = |id: u64| index.get(&id.to_string()).is_some_and(|t| t["inner"]["module"].is_object());
    while let Some((mid, prefix)) = queue.pop_front() {
        if !visited.insert(mid) {
            continue;
        }
        let Some(items) = index.get(&mid.to_string()).and_then(|m| m["inner"]["module"]["items"].as_array())
        else {
            continue;
        };
        for cid in items.iter().filter_map(|c| c.as_u64()) {
            let Some(child) = index.get(&cid.to_string()) else { continue };
            let inner = &child["inner"];
            let public = child["visibility"].as_str() == Some("public");
            if inner["use"].is_object() {
                // `pub use` seulement (un `use` privé ne crée pas de chemin public).
                if !public {
                    continue;
                }
                let Some(tgt) = inner["use"]["id"].as_u64() else { continue };
                if inner["use"]["is_glob"].as_bool() == Some(true) {
                    // `pub use m::*` : réexporte les items publics de `m` **au niveau courant** (même prefix).
                    queue.push_back((tgt, prefix.clone()));
                } else {
                    let name = inner["use"]["name"].as_str().unwrap_or("");
                    let mut p = prefix.clone();
                    p.push(name.to_string());
                    shorter(&mut out, tgt, p.clone());
                    if is_module(tgt) {
                        queue.push_back((tgt, p));
                    }
                }
            } else if !public {
                continue;
            } else if inner["module"].is_object() {
                let mut p = prefix.clone();
                p.push(child["name"].as_str().unwrap_or("").to_string());
                shorter(&mut out, cid, p.clone());
                queue.push_back((cid, p));
            } else if let Some(name) = child["name"].as_str() {
                let mut p = prefix.clone();
                p.push(name.to_string());
                shorter(&mut out, cid, p);
            }
        }
    }
    out.into_iter().map(|(k, v)| (k, v.join("::"))).collect()
}

fn emit(v: &Json, crate_name: &str, crate_real: &str, versions: &HashMap<String, (String, String)>) -> CResult<String> {
    let index = v["index"]
        .as_object()
        .ok_or_else(|| CompileError::msg("rustdoc-json : `.index` absent (format inattendu)"))?;
    // Crates **étrangères** référencées par des chemins de traits UFCS (`num_traits`, `rand`…) — à déclarer
    // comme deps Cargo, sinon `error[E0433]: cannot find module or crate num_traits` (B12).
    let mut foreign: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let paths = &v["paths"];
    let version = v["crate_version"].as_str().unwrap_or("*");
    // Chemins **publics** (re-exports inclus) des items locaux — corrige les chemins canoniques qui
    // traversent des modules privés glob-réexportés (B9).
    let public_paths = v["root"].as_u64().map(|r| build_public_paths(index, r, crate_name)).unwrap_or_default();
    let (mut bound, mut skipped) = (0usize, 0usize);

    // --- POO : méthodes/ctors par type (51100) ---
    let mut method_ids: HashSet<u64> = HashSet::new();
    let mut types: BTreeMap<String, (Vec<String>, Vec<String>)> = BTreeMap::new(); // nom → (ctors, méthodes)
    let mut type_paths: BTreeMap<String, String> = BTreeMap::new(); // nom court → chemin Rust complet (sous-modules)
    let mut seen: BTreeMap<String, HashSet<String>> = BTreeMap::new(); // dedup noms par type

    for item in index.values() {
        let imp = &item["inner"]["impl"];
        if !imp.is_object() {
            continue;
        }
        if imp["is_synthetic"].as_bool() == Some(true) || !imp["blanket_impl"].is_null() {
            continue;
        }
        // V9 (51600) : inhérent (`trait` null) OU `impl Trait for T` d'un trait **utile** (méthodes de
        // comportement : Digest, Datelike, Iterator, Read…). Les traits-bruit (Debug/Clone/From…) sautés.
        if let Some(tr) = imp["trait"]["path"].as_str() {
            if is_noise_trait(tr.rsplit("::").next().unwrap_or(tr)) {
                continue;
            }
        }
        let for_id = imp["for"]["resolved_path"]["id"].as_u64();
        let tyfull = match imp["for"]["resolved_path"]["path"].as_str() {
            Some(n) => n.to_string(),
            None => continue,
        };
        let tyname = tyfull.rsplit("::").next().unwrap_or(&tyfull).to_string();
        // type local à la crate (crate_id 0) — sinon on n'a pas son chemin public.
        if !for_id.map_or(false, |id| is_local(paths, id)) {
            continue;
        }
        // chemin Rust complet du type (sous-modules : `statrs::distribution::Beta`) — via `paths[id]`
        // (canonique), sinon `resolved_path` puis `crate::Type`. Sinon `crate::Beta` serait introuvable.
        type_paths.entry(tyname.clone()).or_insert_with(|| {
            // chemin public (re-export, B9) d'abord ; sinon `paths[id]` canonique + heuristique sous-module.
            for_id
                .and_then(|id| public_paths.get(&id).cloned())
                .or_else(|| {
                    for_id
                        .and_then(|id| paths.get(&id.to_string()))
                        .and_then(|p| p["path"].as_array())
                        .map(|segs| segs.iter().filter_map(|s| s.as_str()).collect::<Vec<_>>().join("::"))
                        .filter(|p| p.contains("::"))
                        .map(|p| drop_private_mod(&p))
                })
                .unwrap_or_else(|| if tyfull.contains("::") { tyfull.clone() } else { format!("{crate_name}::{tyfull}") })
        });
        // Chemin **complet** du trait d'un `impl Trait for T` (None = inhérent) → méthodes appelées en UFCS.
        // On résout via `paths[id]` (ex. `core::ops::Mul`, `core::fmt::Display`) — sinon les traits std/externes
        // seraient mal préfixés `{crate}::Trait` (bug : `num_bigint::Mul` au lieu de `core::ops::Mul`).
        let trait_id = imp["trait"]["id"].as_u64();
        let trait_short = trait_id
            .and_then(|id| public_paths.get(&id).cloned()) // chemin public (re-export, B9) d'abord
            .or_else(|| {
                trait_id
                    .and_then(|id| paths.get(&id.to_string()))
                    .and_then(|p| p["path"].as_array())
                    .map(|segs| segs.iter().filter_map(|s| s.as_str()).collect::<Vec<_>>().join("::"))
            })
            .or_else(|| imp["trait"]["path"].as_str().map(|t| t.to_string()))
            .map(|t| collapse_std_private(&t)); // modules privés de la stdlib (B9 bis)
        // crate racine du trait (UFCS) : si étrangère (ni core/std/alloc ni la crate courante), à déclarer
        // en dep Cargo (B12).
        if let Some(root) = trait_short.as_deref().and_then(|t| t.split("::").next()) {
            if !matches!(root, "core" | "std" | "alloc") && root != crate_name {
                foreign.insert(root.to_string());
            }
        }
        for mid in imp["items"].as_array().into_iter().flatten() {
            let Some(id) = mid.as_u64() else { continue };
            method_ids.insert(id);
            let Some(m) = index.get(&id.to_string()) else { continue };
            if m["inner"]["function"].is_object() && m["visibility"].as_str() != Some("private") {
                let name = m["name"].as_str().unwrap_or("");
                // méthode dont le nom est un mot-clé Iron (`not`/`and`/`in`/`is`/…) → non parsable comme
                // identifiant → sautée (ex. `Not::not`, `BitAnd` exposent `not`/`and`). Rare et remplaçable
                // par les opérateurs.
                if is_iron_keyword(name) {
                    skipped += 1;
                    continue;
                }
                // dedup par nom au sein du type (multi-`impl` → garder la 1ʳᵉ, 51100).
                if !seen.entry(tyname.clone()).or_default().insert(name.to_string()) {
                    continue;
                }
                match project(&m["inner"]["function"], &tyname) {
                    Some(sig) => {
                        let entry = types.entry(tyname.clone()).or_default();
                        if sig.recv.is_some() {
                            // méthode de trait → chemin UFCS `crate::Trait::method` (évite `use Trait`) ;
                            // méthode inhérente → nom court (syntaxe-méthode auto-ref/deref).
                            let mpath = trait_short.as_ref().map(|tr| format!("{tr}::{name}"));
                            entry.1.push(render(name, &sig, mpath.as_deref()));
                        } else {
                            // ctor / fn associée : **inhérente** → `crate::Type::ctor` ; **associée à un
                            // trait** (`One::one`, `Digest::new`, `FromPrimitive::from_u64`) → UFCS qualifié
                            // `<crate::Type as Trait>::ctor` (sinon `crate::Type::one` n'existe pas). B5.
                            let tp = type_paths.get(&tyname).cloned().unwrap_or_else(|| format!("{crate_name}::{tyname}"));
                            let full = match &trait_short {
                                Some(tr) => format!("<{tp} as {tr}>::{name}"),
                                None => format!("{tp}::{name}"),
                            };
                            entry.0.push(render(name, &sig, Some(&full)));
                        }
                        bound += 1;
                    }
                    None => skipped += 1,
                }
            }
        }
    }

    // --- fonctions libres (51000) ---
    let mut free = Vec::new();
    for (id, item) in index {
        if id.parse::<u64>().map_or(false, |n| method_ids.contains(&n)) {
            continue;
        }
        if !item["inner"]["function"].is_object() || item["visibility"].as_str() != Some("public") {
            continue;
        }
        let name = match item["name"].as_str() {
            Some(n) if !is_iron_keyword(n) => n,
            _ => continue,
        };
        match project(&item["inner"]["function"], "") {
            Some(sig) if sig.recv.is_none() => {
                free.push(render(name, &sig, Some(&format!("{crate_name}::{name}"))));
                bound += 1;
            }
            _ => skipped += 1,
        }
    }

    // --- fermeture du graphe (51200) : déclarer en opaque tout type référencé non déjà typé ---
    let mut referenced: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for (ctors, methods) in types.values() {
        for line in ctors.iter().chain(methods) {
            collect_type_refs(line, &mut referenced);
        }
    }
    for line in &free {
        collect_type_refs(line, &mut referenced);
    }
    let opaque: Vec<String> = referenced.into_iter().filter(|n| !types.contains_key(n)).collect();

    // --- assemblage .iri ---
    // Directive `crate` = nom **publié** (clé Cargo, peut avoir des tirets) ; les chemins de types et le nom
    // d'import restent en `crate_name` (underscore).
    let mut iri = format!("# bindings auto-générés depuis `{crate_name}` (ironc bind)\ncrate {crate_real} = \"{version}\"\n");
    // Crates étrangères des traits UFCS (B12) → deps Cargo, version **résolue** (Cargo.lock) pour éviter le
    // hasard d'unification de versions. Inconnue du lock (rare) → ignorée (le nom de module ≠ nom publié).
    for root in &foreign {
        if let Some((name, ver)) = versions.get(root) {
            iri.push_str(&format!("crate {name} = \"{ver}\"\n"));
        }
    }
    for ty in &opaque {
        iri.push_str(&format!("\nextern type {ty} = \"{crate_name}::{ty}\"\n")); // handle opaque (graphe fermé)
    }
    for (ty, (ctors, methods)) in &types {
        if ctors.is_empty() && methods.is_empty() {
            continue;
        }
        let typath = type_paths.get(ty).cloned().unwrap_or_else(|| format!("{crate_name}::{ty}"));
        iri.push_str(&format!("\nextern type {ty} = \"{typath}\":\n"));
        for line in ctors.iter().chain(methods.iter()) {
            iri.push_str(&format!("    {line}\n"));
        }
    }
    if !free.is_empty() {
        iri.push('\n');
        for line in &free {
            iri.push_str(&format!("extern {line}\n")); // fn libre au top-level → `extern fn …`
        }
    }
    iri.push_str(&format!("\n# {bound} item(s) bindé(s), {skipped} non mappable(s) sauté(s)\n"));
    Ok(iri)
}

/// Collapse les modules **privés** internes de `core`/`std`/`alloc` (stables, non documentés en JSON donc
/// hors `build_public_paths`) : `core::ops::arith::Mul` → `core::ops::Mul`, `core::str::traits::FromStr` →
/// `core::str::FromStr`, `core::iter::traits::iterator::Iterator` → `core::iter::Iterator`. Sans ça les
/// méthodes d'opérateurs/conversions en UFCS échouent (`error[E0603]: module \`arith\` is private`).
fn collapse_std_private(path: &str) -> String {
    if !(path.starts_with("core::") || path.starts_with("std::") || path.starts_with("alloc::")) {
        return path.to_string();
    }
    // Modules d'organisation **privés** de la lib standard (les traits sont re-exportés au parent).
    const PRIV: &[&str] = &[
        "arith", "bit", "deref", "function", "index", "range", "unsize", "control_flow", "try_trait",
        "coroutine", "async_function", "traits", "iterator", "collect", "double_ended", "exact_size",
        "accum", "marker",
    ];
    path.split("::").filter(|s| !PRIV.contains(s)).collect::<Vec<_>>().join("::")
}

/// Heuristique de re-export (rustdoc donne le chemin **canonique** = module privé interne ; le chemin
/// **public** est le re-export). Cas très courant `mod beta { pub struct Beta }` re-exporté plus haut :
/// `a::b::beta::Beta` → `a::b::Beta` (on retire le segment-module égal au type en minuscules).
fn drop_private_mod(path: &str) -> String {
    let segs: Vec<&str> = path.split("::").collect();
    if segs.len() >= 2 {
        let last = segs[segs.len() - 1];
        if segs[segs.len() - 2] == last.to_lowercase() {
            let mut s = segs.clone();
            s.remove(segs.len() - 2);
            return s.join("::");
        }
    }
    path.to_string()
}

/// Nom = mot-clé Iron → ne peut pas être un identifiant de méthode/fonction (binder : on saute).
fn is_iron_keyword(name: &str) -> bool {
    matches!(
        name,
        "not" | "and" | "or" | "in" | "is" | "if" | "elif" | "else" | "for" | "while" | "def"
            | "class" | "return" | "match" | "case" | "import" | "from" | "as" | "pass" | "break"
            | "continue" | "raise" | "try" | "except" | "with" | "lambda" | "yield" | "async"
            | "await" | "true" | "false" | "none" | "self" | "enum" | "trait" | "impl" | "struct"
            | "fn" | "let" | "const" | "type" | "mut" | "inout" | "shared" | "extern"
    )
}

/// `true` si l'id de type appartient à la crate documentée (`crate_id == 0`).
fn is_local(paths: &Json, id: u64) -> bool {
    paths[id.to_string()]["crate_id"].as_u64() == Some(0)
}

/// Projette la signature d'une `function` rustdoc-json vers Iron. `owner` = type si méthode (résolution
/// du receveur). `None` si non mappable (générique de fonction, `unsafe`, type non projetable).
fn project(f: &Json, owner: &str) -> Option<Sig> {
    if f["header"]["is_unsafe"].as_bool() == Some(true) {
        return None;
    }
    // générique au niveau de la fonction → sauté (couche B / instance-pinning, 51400).
    if f["generics"]["params"]
        .as_array()
        .map_or(false, |a| a.iter().any(|p| !p["kind"]["lifetime"].is_object()))
    {
        return None;
    }
    let inputs = f["sig"]["inputs"].as_array()?;
    let mut recv = None;
    let mut params = Vec::new();
    let mut idioms = Vec::new();
    for (i, inp) in inputs.iter().enumerate() {
        let pn = inp[0].as_str().unwrap_or("x");
        let ty = &inp[1];
        if i == 0 && pn == "self" {
            recv = Some(recv_kind(ty));
            continue;
        }
        // param nommé comme un mot-clé Iron (`lambda`, `in`, `type`…) → renommé (le nom n'a pas
        // d'importance, l'appel est positionnel) pour rester un identifiant valide.
        let pn = if is_iron_keyword(pn) { format!("{pn}_") } else { pn.to_string() };
        match map_type(ty) {
            Some(t) => params.push(format!("{pn}: {t}")),
            None => return None,
        }
    }
    // retour + idiomes builder/getter (51300).
    let out = &f["sig"]["output"];
    let ret = if out.is_null() {
        "unit".to_string()
    } else if recv == Some("mut self") && is_self_or_owner(out, owner) {
        // builder : `&mut self -> &mut Self`/`&mut Owner` → mutation en place, retour jeté.
        idioms.push("builder");
        "unit".to_string()
    } else if let Some(inner) = borrowed_inner(out) {
        // getter : retour emprunté `&T` → valeur possédée (clone, D4).
        match map_type(inner) {
            Some(t) => {
                idioms.push("clone");
                t
            }
            None => return None,
        }
    } else {
        map_type(out)?
    };
    Some(Sig { recv, params, ret, idioms })
}

fn recv_kind(ty: &Json) -> &'static str {
    if let Some(b) = ty.get("borrowed_ref") {
        if b["is_mutable"].as_bool() == Some(true) {
            return "mut self";
        }
        return "self";
    }
    "own self"
}

/// `&Self`/`&mut Self` ou `&mut Owner` (retour builder, chaînable).
fn is_self_or_owner(t: &Json, owner: &str) -> bool {
    borrowed_inner(t).map_or(false, |inner| {
        inner["generic"].as_str() == Some("Self")
            || inner["resolved_path"]["path"]
                .as_str()
                .map_or(false, |p| p.rsplit("::").next() == Some(owner))
    })
}

/// Si `t` est `&T`/`&[T]`, renvoie l'intérieur emprunté.
fn borrowed_inner(t: &Json) -> Option<&Json> {
    t.get("borrowed_ref").map(|b| &b["type"])
}

fn render(name: &str, sig: &Sig, free_path: Option<&str>) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(r) = sig.recv {
        parts.push(r.to_string());
    }
    parts.extend(sig.params.iter().cloned());
    let params = parts.join(", ");
    // chemin : fn libre → chemin complet fourni ; méthode → nom court (résolu par la lower) ; ctor → nom court.
    let path = free_path.map(|p| p.to_string()).unwrap_or_else(|| name.to_string());
    let idioms = if sig.idioms.is_empty() {
        String::new()
    } else {
        format!(" [{}]", sig.idioms.join(", "))
    };
    format!("fn {name}({params}) -> {} = \"{path}\"{idioms}", sig.ret)
}

/// Projection récursive type-Rust → type-Iron (51000, D1). `None` si non mappable. **Aucun widening.**
fn map_type(t: &Json) -> Option<String> {
    if let Some(p) = t["primitive"].as_str() {
        return map_prim(p);
    }
    if let Some(b) = t.get("borrowed_ref") {
        return map_type(&b["type"]); // emprunt en entrée → intérieur (& réinjecté au lowering)
    }
    if let Some(s) = t.get("slice") {
        return map_type(s).map(|x| format!("list[{x}]"));
    }
    if let Some(items) = t["tuple"].as_array() {
        let parts: Option<Vec<String>> = items.iter().map(map_type).collect();
        return parts.map(|p| format!("({})", p.join(", ")));
    }
    if let Some(rp) = t.get("resolved_path") {
        let name = rp["path"].as_str()?.rsplit("::").next().unwrap_or("");
        let targs: Vec<&Json> = rp["args"]["angle_bracketed"]["args"]
            .as_array()
            .map(|a| a.iter().filter_map(|x| x.get("type")).collect())
            .unwrap_or_default();
        return map_named(name, &targs);
    }
    None // generic nu, dyn Trait, qualified_path, raw_pointer → couche B/C
}

fn map_named(name: &str, targs: &[&Json]) -> Option<String> {
    let one = |i: usize| targs.get(i).and_then(|t| map_type(t));
    match name {
        "String" | "str" | "Cow" | "PathBuf" | "OsString" => Some("str".to_string()),
        "Vec" | "VecDeque" => Some(format!("list[{}]", one(0)?)),
        "Option" => Some(format!("Option[{}]", one(0)?)),
        "Result" => Some(format!("Result[{}]", one(0)?)),
        "HashMap" | "BTreeMap" => Some(format!("dict[{}, {}]", one(0)?, one(1)?)),
        "HashSet" | "BTreeSet" => Some(format!("set[{}]", one(0)?)),
        "Box" | "Rc" | "Arc" => one(0),
        // struct/enum nommé (PascalCase) → réf de type externe (handle).
        other if other.chars().next().map_or(false, |c| c.is_ascii_uppercase()) => Some(other.to_string()),
        _ => None,
    }
}

/// Traits « bruit » (dérives/marqueurs/conversions) dont on ne binde PAS les méthodes (51600) : elles
/// polluent l'API (fmt/clone/eq/from…) sans valeur. On garde les traits de **comportement**.
fn is_noise_trait(name: &str) -> bool {
    matches!(
        name,
        "Debug" | "Display" | "Clone" | "Copy" | "Send" | "Sync" | "Sized" | "Unpin"
            | "PartialEq" | "Eq" | "PartialOrd" | "Ord" | "Hash" | "Default"
            | "From" | "Into" | "TryFrom" | "TryInto" | "AsRef" | "AsMut" | "Borrow" | "BorrowMut"
            | "Deref" | "DerefMut" | "Drop" | "ToOwned" | "ToString"
            | "Serialize" | "Deserialize" | "Freeze" | "RefUnwindSafe" | "UnwindSafe"
    )
}

fn map_prim(p: &str) -> Option<String> {
    match p {
        "bool" => Some("bool".to_string()),
        "char" => Some("char".to_string()),
        "str" => Some("str".to_string()),
        "()" | "unit" => Some("unit".to_string()),
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" | "f32" | "f64" => Some(p.to_string()),
        _ => None,
    }
}

/// Extrait les noms de types externes (PascalCase) de la **signature** Iron (partie avant `=`),
/// hors `Option`/`Result` (constructeurs Iron). Sert à fermer le graphe de types (51200).
fn collect_type_refs(line: &str, out: &mut std::collections::BTreeSet<String>) {
    let sig = line.split(" = ").next().unwrap_or(line);
    let b = sig.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i].is_ascii_uppercase() {
            let start = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            let word = &sig[start..i];
            if word != "Option" && word != "Result" {
                out.insert(word.to_string());
            }
        } else {
            i += 1;
        }
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
