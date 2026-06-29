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

/// Lance `cargo +nightly rustdoc --output-format json` sur la crate (ou un package `-p`) et renvoie le JSON.
fn run_rustdoc(crate_dir: &Path, pkg: Option<&str>, stem: &str) -> CResult<Json> {
    let mut cmd = Command::new("cargo");
    cmd.args(["+nightly", "rustdoc"]);
    if let Some(p) = pkg {
        cmd.args(["-p", p]);
    }
    // `--lib` : documenter la **bibliothèque** (une crate à plusieurs cibles — lib + bin(s)/exemple(s),
    // ex. `qrcode` — sinon `cargo rustdoc` échoue « passing --lib or --bin NAME »). No-op pour une crate à
    // lib unique (B20).
    cmd.arg("--lib")
        .arg("--manifest-path")
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
    let json_path = crate_dir.join("target/doc").join(format!("{stem}.json"));
    let json = std::fs::read_to_string(&json_path)
        .map_err(|e| CompileError::msg(format!("lecture {} : {e}", json_path.display())))?;
    serde_json::from_str(&json).map_err(|e| CompileError::msg(format!("JSON rustdoc invalide : {e}")))
}

/// Versions résolues (Cargo.lock) : nom de **module** (underscores) ⇒ (nom **publié**, version). Sert à
/// déclarer les crates étrangères en deps (B12) et à retrouver `geo-types` depuis le module `geo_types`.
fn read_lock_versions(crate_dir: &Path) -> HashMap<String, (String, String)> {
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
    versions
}

/// Crates (modules, underscores) dont la crate **ré-exporte publiquement** des types (`pub use geo_types::…`)
/// → à documenter aussi et fusionner (mur « crate façade », 52700). On ne retient que les **vraies crates
/// externes** (présentes dans `external_crates`), pas les modules internes (`self`/`crate`/`super`).
fn reexport_source_crates(v: &Json) -> Vec<String> {
    let Some(idx) = v["index"].as_object() else { return vec![] };
    let ext_names: HashSet<&str> = v["external_crates"]
        .as_object()
        .map(|o| o.values().filter_map(|c| c["name"].as_str()).collect())
        .unwrap_or_default();
    let mut out: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for it in idx.values() {
        if it["inner"]["use"].is_object() && it["visibility"].as_str() == Some("public") {
            if let Some(root) = it["inner"]["use"]["source"].as_str().and_then(|s| s.split("::").next()) {
                if !matches!(root, "core" | "std" | "alloc") && ext_names.contains(root) {
                    out.insert(root.to_string());
                }
            }
        }
    }
    out.into_iter().collect()
}

/// Lance rustdoc-json sur la crate (dossier avec `Cargo.toml`) et renvoie le `.iri` généré.
pub fn bind(crate_dir: &Path, pkg: Option<&str>) -> CResult<String> {
    let manifest = std::fs::read_to_string(crate_dir.join("Cargo.toml"))
        .map_err(|e| CompileError::msg(format!("lecture Cargo.toml : {e}")))?;
    let target = match pkg {
        Some(p) => p.to_string(),
        None => pkg_name(&manifest).ok_or_else(|| CompileError::msg("nom de crate introuvable dans Cargo.toml"))?,
    };
    let v = run_rustdoc(crate_dir, pkg, &target.replace('-', "_"))?;
    let versions = read_lock_versions(crate_dir);

    let mut iri = emit(&v, &target.replace('-', "_"), &target, &versions)?;

    // Mur « crate façade » (52700) : `geo` ré-exporte `geo_types::Point` dont les ctors/méthodes vivent dans
    // `geo_types` (invisibles au bind mono-crate). On documente aussi ces crates-sources et on **fusionne**
    // leurs blocs **après** (le parseur `.iri` est last-wins → `geo_types::Point` typé l'emporte sur le
    // `Point` opaque de geo). Borné à 4 pour éviter les explosions.
    for src_mod in reexport_source_crates(&v).into_iter().take(4) {
        let Some((published, _)) = versions.get(&src_mod) else { continue };
        let published = published.clone();
        match run_rustdoc(crate_dir, Some(&published), &src_mod)
            .and_then(|sub| emit(&sub, &src_mod, &published, &versions))
        {
            Ok(sub_iri) => {
                iri.push_str(&format!("\n# --- re-export façade depuis `{published}` (52700) ---\n"));
                iri.push_str(&sub_iri);
            }
            Err(_) => continue, // crate-source non documentable → on garde le binding mono-crate
        }
    }
    Ok(iri)
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
    /// V12 [52800] : index d'un paramètre `impl Read`/`BufRead` (source io) — la fn libre est alors émise
    /// avec un **chemin-closure** qui enveloppe l'argument str dans un `std::io::Cursor`. `None` si aucun.
    read_param: Option<usize>,
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
    // B33 : recense les alias `pub type FooResult<T> = Result<T, E>` pour les **expanser** en leur forme
    // Iron (`ImageResult<DynamicImage>` → `Result[DynamicImage]`) au lieu d'un handle opaque inutilisable.
    populate_alias_map(index);
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
    // V12 [24000] : paramètres de type d'un type externe générique (`Point<T>` → `["T"]`) → entête
    // `extern type Point[T]`. Base de la substitution en inférence (`Point[f64]`.méthode rendant `T` → `f64`).
    let mut type_params: BTreeMap<String, Vec<String>> = BTreeMap::new();

    // Pré-passe « variante canonique » : sur **collision de nom court** (ex. `regex::Regex` ET
    // `regex::bytes::Regex` → tous deux `Regex`), retenir la variante au chemin public **le plus court** (la
    // « principale ») ; sinon le binder fusionne arbitrairement les méthodes des deux (B15).
    let mut canonical_ty: HashMap<String, String> = HashMap::new();
    for item in index.values() {
        let imp = &item["inner"]["impl"];
        if !imp.is_object() || imp["is_synthetic"].as_bool() == Some(true) {
            continue;
        }
        if let Some(tr) = imp["trait"]["path"].as_str() {
            if is_noise_trait(tr.rsplit("::").next().unwrap_or(tr)) {
                continue;
            }
        }
        let for_id = imp["for"]["resolved_path"]["id"].as_u64();
        let Some(tyfull) = imp["for"]["resolved_path"]["path"].as_str() else { continue };
        if !for_id.map_or(false, |id| is_local(paths, id)) {
            continue;
        }
        let tyname = tyfull.rsplit("::").next().unwrap_or(tyfull).to_string();
        let path = resolve_type_path(for_id, tyfull, &public_paths, paths, crate_name);
        canonical_ty
            .entry(tyname)
            .and_modify(|e| {
                if path.split("::").count() < e.split("::").count() {
                    *e = path.clone();
                }
            })
            .or_insert(path);
    }

    for item in index.values() {
        let imp = &item["inner"]["impl"];
        if !imp.is_object() {
            continue;
        }
        if imp["is_synthetic"].as_bool() == Some(true) {
            continue;
        }
        // Note : on **ne saute plus** les impls blanket. rustdoc les **matérialise par type concret** (le `for`
        // d'`impl<T> Digest for T` apparaît comme `Sha256` dans la liste d'impls de `Sha256`) → on attache
        // alors ses méthodes en UFCS comme tout `impl Trait`. Un vrai blanket générique (`for: T`) n'a pas de
        // `resolved_path` → il retombe au `continue` ci-dessous (tyfull `None`). Règle 100 % générale.
        // V9 (51600) : inhérent (`trait` null) OU `impl Trait for T` d'un trait **utile** (méthodes de
        // comportement : Digest, Datelike, Iterator, Read…). Les traits-bruit (Debug/Clone/From…) sautés.
        if let Some(tr) = imp["trait"]["path"].as_str() {
            if is_noise_trait(tr.rsplit("::").next().unwrap_or(tr)) {
                continue;
            }
        }
        let for_id = imp["for"]["resolved_path"]["id"].as_u64();
        // Récepteur : type **nommé** (resolved_path) OU **primitif** (`impl Trait for &str` → extension d'un
        // primitif, ex. colored). Sinon (slice/tuple/dyn…) → sauté.
        let (tyname, is_prim) = match imp["for"]["resolved_path"]["path"].as_str() {
            Some(n) => (n.rsplit("::").next().unwrap_or(n).to_string(), false),
            None => match prim_for_type(&imp["for"]) {
                Some(p) => (p, true),
                None => continue,
            },
        };
        if is_prim {
            // pas de `is_local`, pas de dédup de variante, pas de chemin résolu : les méthodes seront des
            // **traits** appelées en UFCS (`Colorize::red(&x)`) ; le pseudo-type pointe vers le primitif Rust.
            type_paths.entry(tyname.clone()).or_insert_with(|| "str".to_string());
        } else {
            let tyfull = imp["for"]["resolved_path"]["path"].as_str().unwrap_or(&tyname).to_string();
            // type local à la crate (crate_id 0) — sinon on n'a pas son chemin public.
            if !for_id.map_or(false, |id| is_local(paths, id)) {
                continue;
            }
            let typath = resolve_type_path(for_id, &tyfull, &public_paths, paths, crate_name);
            // variante non principale d'un nom en collision (ex. `regex::bytes::Regex` quand `regex::Regex`
            // existe) → ignorée (B15), sinon ses méthodes pollueraient le type retenu.
            if canonical_ty.get(&tyname).is_some_and(|c| c != &typath) {
                continue;
            }
            type_paths.entry(tyname.clone()).or_insert(typath);
        }
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
        // Pour un **impl blanket matérialisé** (`impl<D> Digest for D`), le générique `D` dénote le type
        // concret (`Sha256`) → on le mappe vers `tyname`, comme `Self` ; sinon `new() -> D` serait sauté.
        let mut blanket_self: Generics = Generics::new();
        if let Some(g) = imp["blanket_impl"]["generic"].as_str() {
            blanket_self.insert(g.to_string(), Some(tyname.clone()));
        }
        // V12 [24000] : **garder** les paramètres de type de l'impl (`impl<T: CoordNum> Point<T>`) au lieu de
        // les jeter — sinon `new(x: T, y: T)` et `euclidean_distance(self) -> T` sont droppés. Un param à borne
        // reconnue (`AsRef<Path>`) prend son type Iron ; sinon il **reste générique** (`Some("T")`), instancié
        // par l'argument à l'appel (infer). Et on enregistre les params du type pour l'entête `Point[T]`.
        for p in imp["generics"]["params"].as_array().into_iter().flatten() {
            if let Some(name) = p["name"].as_str() {
                let ty = &p["kind"]["type"];
                if ty.is_object() {
                    blanket_self
                        .entry(name.to_string())
                        .or_insert_with(|| bound_to_iron(&ty["bounds"]).filter(|s| s != "@read").or_else(|| Some(name.to_string())));
                }
            }
        }
        // Bornes en clause `where` de l'impl (`impl<P> … where P: FnMut(&DirEntry)->bool`) — fréquent pour les
        // closures (V12 [40300]). Elles **améliorent** un param resté générique (`Some("P")`) en son type Iron.
        for w in imp["generics"]["where_predicates"].as_array().into_iter().flatten() {
            let bp = &w["bound_predicate"];
            if let Some(name) = bp["type"]["generic"].as_str() {
                // `@read` (sentinelle de param io) ne doit PAS devenir un **paramètre de type** (sinon il fuit
                // en `StreamDeserializer[@read, T]`) → on garde un type param `Read` comme générique.
                if let Some(t) = bound_to_iron(&bp["bounds"]).filter(|s| s != "@read") {
                    blanket_self.insert(name.to_string(), Some(t));
                }
            }
        }
        let for_params: Vec<String> = imp["for"]["resolved_path"]["args"]["angle_bracketed"]["args"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|a| a["type"]["generic"].as_str().map(str::to_string))
            .collect();
        if !for_params.is_empty() {
            // `Self` = type **appliqué** (`Complex[T]`) → `new(...) -> Self` rend `Complex[T]` (substituable).
            blanket_self.insert("Self".to_string(), Some(format!("{tyname}[{}]", for_params.join(", "))));
            type_params.entry(tyname.clone()).or_insert(for_params);
        }
        // Méthodes de l'impl + méthodes **fournies** (corps par défaut) du trait **non surchargées** (52600,
        // ex. colored `red`/`green`, déclarées avec défaut dans `Colorize` et absentes des items de l'impl).
        // Trait local uniquement (sa définition doit être dans l'index).
        let impl_items: Vec<u64> =
            imp["items"].as_array().into_iter().flatten().filter_map(|x| x.as_u64()).collect();
        let provided: Vec<u64> = imp["trait"]["id"]
            .as_u64()
            .and_then(|tid| index.get(&tid.to_string()))
            .and_then(|t| t["inner"]["trait"]["items"].as_array())
            .map(|a| a.iter().filter_map(|x| x.as_u64()).filter(|x| !impl_items.contains(x)).collect())
            .unwrap_or_default();
        for &id in impl_items.iter().chain(provided.iter()) {
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
                // dedup par nom **après projection réussie** : un même nom exposé par plusieurs impls (ex.
                // `Digest::update` à arg générique non mappable PUIS `Update::update` à `&[u8]` concret) ne
                // doit pas être « réservé » par la 1ʳᵉ tentative **ratée** — sinon la variante mappable est
                // bloquée à tort (51100). On ne marque `seen` que quand on **binde** réellement.
                if seen.get(&tyname).is_some_and(|s| s.contains(name)) {
                    continue;
                }
                match project(&m["inner"]["function"], &tyname, &blanket_self) {
                    Some(sig) => {
                        // extension d'un primitif : on ne garde que les **méthodes de trait** (un ctor/fn
                        // associée sur un pseudo-type `__str` n'a pas de sens).
                        if is_prim && sig.recv.is_none() {
                            skipped += 1;
                            continue;
                        }
                        seen.entry(tyname.clone()).or_default().insert(name.to_string());
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
    let mut seen_free: HashSet<String> = HashSet::new();
    for (id, item) in index {
        let Some(idn) = id.parse::<u64>().ok() else { continue };
        if method_ids.contains(&idn) {
            continue;
        }
        if !item["inner"]["function"].is_object() || item["visibility"].as_str() != Some("public") {
            continue;
        }
        let name = match item["name"].as_str() {
            Some(n) if !is_iron_keyword(n) => n,
            _ => continue,
        };
        match project(&item["inner"]["function"], "", &Generics::new()) {
            Some(sig) if sig.recv.is_none() => {
                if !seen_free.insert(name.to_string()) {
                    continue; // dedup par nom (une fn libre homonyme dans deux modules → garder la 1ʳᵉ)
                }
                // chemin **public** de la fn (re-export/sous-module, ex. `zstd::bulk::compress`), pas
                // `crate::name` qui suppose le top-level (B23).
                let fpath = public_paths
                    .get(&idn)
                    .cloned()
                    .or_else(|| {
                        paths
                            .get(&idn.to_string())
                            .and_then(|p| p["path"].as_array())
                            .map(|segs| segs.iter().filter_map(|s| s.as_str()).collect::<Vec<_>>().join("::"))
                            .filter(|p| p.contains("::"))
                            .map(|p| collapse_std_private(&p))
                    })
                    .unwrap_or_else(|| format!("{crate_name}::{name}"));
                free.push(render(name, &sig, Some(&fpath)));
                bound += 1;
            }
            _ => skipped += 1,
        }
    }

    // --- structs à champs publics (construction par littéral) + enums à variantes (52400) ---
    let mut struct_fields: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    let mut enum_variants: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (id, item) in index {
        let Some(idn) = id.parse::<u64>().ok() else { continue };
        if !is_local(paths, idn) || item["visibility"].as_str() != Some("public") {
            continue;
        }
        let Some(tyname) = item["name"].as_str().map(str::to_string) else { continue };
        // struct à **tous** champs publics nommés (sinon non constructible par littéral).
        let plain = &item["inner"]["struct"]["kind"]["plain"];
        if plain.is_object() && plain["has_stripped_fields"].as_bool() == Some(false) {
            // V12 [24000] : params de type de la struct → gardés génériques pour ne pas dropper les champs
            // typés `T` (`Complex { re: T, im: T }`). Et enregistrés pour l'entête `extern type Complex[T]`.
            let mut sgen: Generics = Generics::new();
            let mut sparams = Vec::new();
            for p in item["inner"]["struct"]["generics"]["params"].as_array().into_iter().flatten() {
                if let Some(name) = p["name"].as_str() {
                    let ty = &p["kind"]["type"];
                    if ty.is_object() {
                        sgen.insert(name.to_string(), bound_to_iron(&ty["bounds"]).filter(|s| s != "@read").or_else(|| Some(name.to_string())));
                        sparams.push(name.to_string());
                    }
                }
            }
            let mut fs = Vec::new();
            let mut ok = true;
            for fid in plain["fields"].as_array().into_iter().flatten().filter_map(|x| x.as_u64()) {
                let Some(fm) = index.get(&fid.to_string()) else { ok = false; break };
                match (fm["name"].as_str(), map_type(&fm["inner"]["struct_field"], &sgen)) {
                    (Some(fname), Some(t)) if !is_iron_keyword(fname) => fs.push((fname.to_string(), t)),
                    _ => { ok = false; break }
                }
            }
            if ok && !fs.is_empty() {
                struct_fields.insert(tyname.clone(), fs);
                if !sparams.is_empty() {
                    type_params.entry(tyname.clone()).or_insert(sparams);
                }
                type_paths.entry(tyname.clone()).or_insert_with(|| resolve_type_path(Some(idn), &tyname, &public_paths, paths, crate_name));
            }
        }
        // enum : variantes **nullaires** (`SampleFormat.Int`) ; les variantes à données sont ignorées.
        if let Some(vars) = item["inner"]["enum"]["variants"].as_array() {
            let vs: Vec<String> = vars
                .iter()
                .filter_map(|x| x.as_u64())
                .filter_map(|vid| index.get(&vid.to_string()))
                .filter(|vm| vm["inner"]["variant"]["kind"].as_str() == Some("plain"))
                .filter_map(|vm| vm["name"].as_str())
                // variante **non référençable** depuis Iron (nom = mot-clé/littéral `None`/`True`, ou
                // non-identifiant) → sautée, sinon le `.iri` ne parse pas (B24). Le reste de l'enum reste OK.
                .filter(|n| {
                    !is_iron_keyword(n)
                        && n.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                        && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                })
                .map(str::to_string)
                .collect();
            if !vs.is_empty() {
                enum_variants.insert(tyname.clone(), vs);
                type_paths.entry(tyname.clone()).or_insert_with(|| resolve_type_path(Some(idn), &tyname, &public_paths, paths, crate_name));
            }
        }
    }

    // --- constantes publiques (V12 [50500]) : `pub const NAME: T` → `extern const NAME: T = "path"` ---
    let mut consts: Vec<(String, String, String)> = Vec::new(); // (nom, type Iron, chemin Rust)
    for (id, item) in index {
        let Some(idn) = id.parse::<u64>().ok() else { continue };
        if !is_local(paths, idn) || item["visibility"].as_str() != Some("public") {
            continue;
        }
        let Some(c) = item["inner"].get("constant") else { continue };
        let Some(name) = item["name"].as_str() else { continue };
        if is_iron_keyword(name) || !name.chars().next().is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_') {
            continue;
        }
        if let Some(t) = map_type(&c["type"], &Generics::new()) {
            let path = resolve_type_path(Some(idn), name, &public_paths, paths, crate_name);
            // Nom RÉFÉRENÇABLE = dernier segment du chemin **public** (re-export prélude), p.ex.
            // `base64::prelude::BASE64_STANDARD` → `BASE64_STANDARD` (et non l'item canonique `STANDARD`,
            // qui collisionne avec `alphabet::STANDARD`).
            let pub_name = path.rsplit("::").next().unwrap_or(name).to_string();
            if pub_name.chars().next().is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_') && !is_iron_keyword(&pub_name) {
                consts.push((pub_name, t, path));
            }
        }
    }
    consts.sort();
    consts.dedup_by(|a, b| a.0 == b.0); // dédup par nom (collisions de re-export)

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
    let opaque: Vec<String> = referenced
        .into_iter()
        .filter(|n| !types.contains_key(n) && !struct_fields.contains_key(n) && !enum_variants.contains_key(n))
        .collect();

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
    // union des types ayant du contenu : ctors/méthodes, **champs** (struct) ou **variantes** (enum).
    let mut all_types: std::collections::BTreeSet<&String> = types.keys().collect();
    all_types.extend(struct_fields.keys());
    all_types.extend(enum_variants.keys());
    let empty: (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());
    for ty in all_types {
        let (ctors, methods) = types.get(ty).unwrap_or(&empty);
        let fields = struct_fields.get(ty);
        let variants = enum_variants.get(ty);
        if ctors.is_empty() && methods.is_empty() && fields.is_none() && variants.is_none() {
            continue;
        }
        let typath = type_paths.get(ty).cloned().unwrap_or_else(|| format!("{crate_name}::{ty}"));
        // V12 [24000] : entête `extern type Point[T] = …` si le type a des paramètres.
        let hdr = match type_params.get(ty) {
            Some(p) if !p.is_empty() => format!("{ty}[{}]", p.join(", ")),
            _ => ty.clone(),
        };
        iri.push_str(&format!("\nextern type {hdr} = \"{typath}\":\n"));
        for (fname, fty) in fields.into_iter().flatten() {
            iri.push_str(&format!("    field {fname}: {fty}\n"));
        }
        for v in variants.into_iter().flatten() {
            iri.push_str(&format!("    case {v}\n"));
        }
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
    // constantes publiques (V12 [50500]) : `extern const NOM: T = "chemin"`.
    if !consts.is_empty() {
        iri.push('\n');
        for (name, ty, path) in &consts {
            iri.push_str(&format!("extern const {name}: {ty} = \"{path}\"\n"));
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

/// Chemin Rust complet d'un type : chemin **public** (re-export, B9) d'abord ; sinon `paths[id]` canonique
/// + heuristique sous-module ; sinon `resolved_path` puis `crate::Type`.
fn resolve_type_path(
    for_id: Option<u64>,
    tyfull: &str,
    public_paths: &HashMap<u64, String>,
    paths: &Json,
    crate_name: &str,
) -> String {
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
        .unwrap_or_else(|| if tyfull.contains("::") { tyfull.to_string() } else { format!("{crate_name}::{tyfull}") })
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

/// Type récepteur **primitif** d'un `impl Trait for <prim>` (extension d'un primitif via trait, ex. colored
/// `impl Colorize for &str`) → pseudo-type Iron interne. `str`/`&str`/`String` → `__str` (les méthodes y
/// sont rattachées et routées sur le `str` builtin au lowering). Les autres primitifs : à étendre au besoin.
fn prim_for_type(for_json: &Json) -> Option<String> {
    let t = for_json.get("borrowed_ref").map(|b| &b["type"]).unwrap_or(for_json);
    let name = t["primitive"]
        .as_str()
        .or_else(|| t["resolved_path"]["path"].as_str().map(|p| p.rsplit("::").next().unwrap_or(p)));
    match name {
        Some("str" | "String") => Some("__str".to_string()),
        _ => None,
    }
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
            // littéraux Python capitalisés (le lexer en fait des tokens, pas des idents).
            | "True" | "False" | "None"
    )
}

/// `true` si l'id de type appartient à la crate documentée (`crate_id == 0`).
fn is_local(paths: &Json, id: u64) -> bool {
    paths[id.to_string()]["crate_id"].as_u64() == Some(0)
}

/// Projette la signature d'une `function` rustdoc-json vers Iron. `owner` = type si méthode (résolution
/// du receveur). `None` si non mappable (générique de fonction, `unsafe`, type non projetable).
fn project(f: &Json, owner: &str, extra_gen: &Generics) -> Option<Sig> {
    if f["header"]["is_unsafe"].as_bool() == Some(true) {
        return None;
    }
    // Génériques de fonction résolus par leurs **bornes d'entrée** (`P: AsRef<Path>` → `str`) ; un générique
    // sans borne mappable reste `None` → la fn est sautée si ce générique est réellement utilisé (via
    // `map_type` qui renverra `None`). Couvre `new(path: impl AsRef<Path>)` (ubiquitaire) sans débinder les
    // vraies fns génériques (`fn parse<T: FromStr>()` → `T` non mappable → sautée comme avant).
    // `extra_gen` : alias du type Self apportés par le bloc `impl` — notamment le **générique d'un blanket**
    // (`impl<D> Digest for D` → `D` dénote le type concret matérialisé, comme `Self`).
    let mut gen: Generics = extra_gen.clone();
    // `Self` (type de retour des ctors `fn new(...) -> Self`) → le type impl'é. Sans ça `new` est sauté.
    // V12 [24000] : si l'impl a déjà fixé `Self` au type **appliqué** (`Complex[T]`), ne pas l'écraser par le
    // nom nu — sinon `new` rendrait `Complex` (opaque) et la substitution des génériques serait perdue.
    if !owner.is_empty() {
        gen.entry("Self".to_string()).or_insert_with(|| Some(owner.to_string()));
    }
    for p in f["generics"]["params"].as_array().into_iter().flatten() {
        if let Some(name) = p["name"].as_str() {
            let ty = &p["kind"]["type"];
            if ty.is_object() {
                // Générique **propre à la fonction** : drop si non matérialisable, SAUF borne **serde**
                // (`T: Serialize`/`Deserialize`, V12 [52900]) → gardé générique pour que `to_string`/`from_str`
                // bindent ; `T` est instancié par l'argument (serialize) ou l'ascription (deserialize).
                gen.insert(name.to_string(), map_fn_generic(name, &ty["bounds"]));
            }
        }
    }
    // Bornes en clause **`where`** (`fn http<A>(addr: A) where A: ToSocketAddrs`) — pas dans les bounds
    // inline du param (B28). Sans ça `A` reste `None` et la fn est sautée à tort.
    for w in f["generics"]["where_predicates"].as_array().into_iter().flatten() {
        let bp = &w["bound_predicate"];
        if let Some(name) = bp["type"]["generic"].as_str() {
            if !matches!(gen.get(name), Some(Some(_))) {
                gen.insert(name.to_string(), map_fn_generic(name, &bp["bounds"]));
            }
        }
    }
    let inputs = f["sig"]["inputs"].as_array()?;
    let mut recv = None;
    let mut params = Vec::new();
    let mut idioms = Vec::new();
    let mut read_param: Option<usize> = None;
    for (i, inp) in inputs.iter().enumerate() {
        let pn = inp[0].as_str().unwrap_or("x");
        let ty = &inp[1];
        if i == 0 && pn == "self" {
            recv = Some(recv_kind(ty));
            continue;
        }
        // Nom de param → identifiant Iron valide (l'appel est **positionnel**, le nom n'importe pas) :
        // - **motif** non-identifiant (`(x, y)` d'un `fn index(&self, (x, y): …)` déstructuré) → `argN` (B21) ;
        // - **mot-clé** Iron (`lambda`/`in`/`type`…) → `{nom}_`.
        let valid = !pn.is_empty()
            && pn.chars().next().is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
            && pn.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        let pn = if !valid {
            format!("arg{i}")
        } else if is_iron_keyword(pn) {
            format!("{pn}_")
        } else {
            pn.to_string()
        };
        match map_type(ty, &gen) {
            // V12 [52800] : source io → param `str`, mémoriser l'index pour le chemin-closure (un seul supporté).
            Some(t) if t == "@read" => {
                if read_param.is_some() {
                    return None; // deux sources io dans une même fn : non géré → on saute (honnête)
                }
                read_param = Some(params.len());
                params.push(format!("{pn}: str"));
            }
            // `@read` **imbriqué** (`list[@read]`, `(@read, …)`) : non représentable → on saute la fn (honnête).
            Some(t) if t.contains("@read") => return None,
            Some(t) => {
                // param **par valeur** (pas d'emprunt Rust) d'un *handle* (type externe/struct, PascalCase) →
                // marqué `own` pour que le lowering **ne mette pas** de `&` (ex. `create(spec: WavSpec)`).
                let owned = ty.get("borrowed_ref").is_none()
                    && t.chars().next().is_some_and(|c| c.is_ascii_uppercase());
                params.push(if owned { format!("{pn}: own {t}") } else { format!("{pn}: {t}") });
            }
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
        match map_type(inner, &gen) {
            Some(t) => {
                idioms.push("clone");
                t
            }
            None => return None,
        }
    } else {
        map_type(out, &gen)?
    };
    // `@read` est une sentinelle de **param** (source io) ; en position de retour/imbriquée elle n'a pas de
    // sens → saute la fn plutôt que d'émettre un `.iri` illisible.
    if ret.contains("@read") {
        return None;
    }
    Some(Sig { recv, params, ret, idioms, read_param })
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

/// Borne **serde** sur un générique (`T: Serialize`/`Deserialize`/`DeserializeOwned`) → V12 [52900].
fn has_serde_bound(bounds: &Json) -> bool {
    bounds.as_array().into_iter().flatten().any(|b| {
        let t = b["trait_bound"]["trait"]["path"].as_str().unwrap_or("");
        matches!(t.rsplit("::").next().unwrap_or(""), "Serialize" | "Deserialize" | "DeserializeOwned")
    })
}

/// Générique **propre à une fonction** → type Iron. Drop (`None`) si non matérialisable, SAUF borne serde →
/// gardé générique (`Some(name)`) : `to_string`/`from_str` bindent, `T` instancié à l'appel/par ascription.
fn map_fn_generic(name: &str, bounds: &Json) -> Option<String> {
    if has_serde_bound(bounds) {
        Some(name.to_string())
    } else {
        bound_to_iron(bounds)
    }
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
    let mut path = free_path.map(|p| p.to_string()).unwrap_or_else(|| name.to_string());
    // V12 [52800] : source io → chemin-closure qui enveloppe l'argument str dans un `Cursor` (fn libre/ctor à
    // chemin complet). `read("…")` → `(|__a0| gpx::read(std::io::Cursor::new(__a0)))(&*"…")`.
    if let Some(ri) = sig.read_param {
        if path.contains("::") {
            let n = sig.params.len();
            let names: Vec<String> = (0..n).map(|j| format!("__a{j}")).collect();
            let call: Vec<String> = (0..n)
                .map(|j| if j == ri { format!("std::io::Cursor::new({})", names[j]) } else { names[j].clone() })
                .collect();
            path = format!("(|{}| {}({}))", names.join(", "), path, call.join(", "));
        }
    }
    let idioms = if sig.idioms.is_empty() {
        String::new()
    } else {
        format!(" [{}]", sig.idioms.join(", "))
    };
    format!("fn {name}({params}) -> {} = \"{path}\"{idioms}", sig.ret)
}

/// Projection récursive type-Rust → type-Iron (51000, D1). `None` si non mappable. **Aucun widening.**
/// Génériques d'une fonction résolus en type Iron : `P: AsRef<Path>` → `Some("str")`, `T: Clone` → `None`
/// (non mappable → la fn sera sautée si `T` est utilisé). Clé = nom du générique.
type Generics = HashMap<String, Option<String>>;

/// Monomorphise un faisceau de bornes de trait **en position d'entrée** vers un type Iron concret :
/// `AsRef<Path>`/`AsRef<str>`/`Into<String>`/`ToString` → `str` (couvre le motif ubiquitaire
/// `new(path: impl AsRef<Path>)`). Tout le reste → `None`.
fn bound_to_iron(bounds: &Json) -> Option<String> {
    for b in bounds.as_array().into_iter().flatten() {
        let tr = &b["trait_bound"]["trait"];
        let tname = tr["path"].as_str().unwrap_or("").rsplit("::").next().unwrap_or("");
        // V12 [40300] : borne closure `Fn(A,…)->B`/`FnMut`/`FnOnce` → type fonction Iron `fn(A,…) -> B`
        // (l'argument sera une **lambda** Iron). Args concrets requis (mappés en Generics vide) ; sinon None.
        // V12 [52800] : source io `Read`/`BufRead` → sentinelle `@read` (projetée en `str` + chemin-closure
        // qui enveloppe dans un `Cursor` au binding). `Write` (puits) reste à part (out-param, plus dur).
        if matches!(tname, "Read" | "BufRead") {
            return Some("@read".to_string());
        }
        if matches!(tname, "Fn" | "FnMut" | "FnOnce") {
            let par = &tr["args"]["parenthesized"];
            let inputs: Option<Vec<String>> = par["inputs"]
                .as_array()
                .map(|a| a.iter().map(|t| map_type(t, &Generics::new())).collect())
                .unwrap_or(Some(vec![]));
            let out = match par.get("output") {
                Some(o) if !o.is_null() => map_type(o, &Generics::new())?,
                _ => "unit".to_string(),
            };
            return inputs.map(|i| format!("fn({}) -> {out}", i.join(", ")));
        }
        let arg = tr["args"]["angle_bracketed"]["args"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|x| x.get("type"));
        let arg_name = arg
            .and_then(|t| t["resolved_path"]["path"].as_str().or_else(|| t["primitive"].as_str()))
            .map(|p| p.rsplit("::").next().unwrap_or(p));
        match (tname, arg_name) {
            // `Str` (V12 [24100b]) : newtype-chaîne de crate (`clap::Str`, etc.) — un type **nommé `Str`** est
            // un emballage de chaîne ; `Into<Str>` accepte donc une chaîne Iron (`Command.new("app")`).
            ("AsRef" | "Into" | "AsMut", Some("Path" | "PathBuf" | "OsStr" | "OsString" | "str" | "String" | "Str")) => {
                return Some("str".to_string())
            }
            ("ToString", _) => return Some("str".to_string()),
            // `ToSocketAddrs` (std) : une adresse se passe en **chaîne** (`"127.0.0.1:8080"`) ;
            // `&String: ToSocketAddrs` via le blanket `impl ToSocketAddrs for &T`. Trait std, pas crate.
            ("ToSocketAddrs", _) => return Some("str".to_string()),
            _ => {}
        }
        // `AsRef<[u8]>` (entrée d'octets, ex. `Digest::digest`) → `str` : Iron passe une chaîne, source
        // d'octets (`String: AsRef<[u8]>`). Règle standard (trait std), non spécifique à une crate.
        if matches!(tname, "AsRef" | "Into" | "AsMut")
            && arg.and_then(|t| t["slice"]["primitive"].as_str()) == Some("u8")
        {
            return Some("str".to_string());
        }
    }
    None
}

thread_local! {
    /// Alias de type `pub type FooResult<T> = Result<T, E>` (B33) : id rustdoc → (noms de params, type
    /// sous-jacent). Peuplé par `populate_alias_map` au début de chaque `emit`. Permet d'**expanser**
    /// l'alias en sa forme Iron au lieu de le binder en handle opaque non déballable (`.unwrap()` cassé).
    static ALIAS_MAP: std::cell::RefCell<HashMap<u64, (Vec<String>, Json)>> =
        std::cell::RefCell::new(HashMap::new());
    /// Garde de récursion (alias → alias → …) : borne la profondeur d'expansion.
    static ALIAS_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Recense les `type_alias` de l'index → `ALIAS_MAP` (B33). Appelé une fois par `emit`.
fn populate_alias_map(index: &serde_json::Map<String, Json>) {
    ALIAS_MAP.with(|m| {
        let mut m = m.borrow_mut();
        m.clear();
        for (id, item) in index {
            let Some(ta) = item["inner"].get("type_alias") else { continue };
            let Some(under) = ta.get("type") else { continue };
            let params: Vec<String> = ta["generics"]["params"]
                .as_array()
                .map(|a| a.iter().filter_map(|p| p["name"].as_str().map(String::from)).collect())
                .unwrap_or_default();
            if let Ok(id) = id.parse::<u64>() {
                m.insert(id, (params, under.clone()));
            }
        }
    });
}

/// Tente d'expanser un `resolved_path` qui pointe sur un alias de type (B33). Substitue les params de
/// l'alias par les args **mappés dans le `gen` courant**, puis projette le type sous-jacent. Renvoie `None`
/// si l'id n'est pas un alias, si la profondeur déborde, ou si l'expansion ne mappe pas — auquel cas
/// l'appelant **retombe** sur le handle opaque (jamais de régression : on n'expanse que si ça améliore).
fn try_expand_alias(id: u64, targs: &[&Json], gen: &Generics) -> Option<String> {
    let entry = ALIAS_MAP.with(|m| m.borrow().get(&id).cloned())?;
    let (params, under) = entry;
    if ALIAS_DEPTH.with(|d| d.get()) > 16 {
        return None;
    }
    let mut alias_gen: Generics = Generics::new();
    for (i, pname) in params.iter().enumerate() {
        // chaque param de l'alias ← l'arg correspondant, mappé dans le `gen` de l'appelant.
        alias_gen.insert(pname.clone(), targs.get(i).and_then(|t| map_type(t, gen)));
    }
    ALIAS_DEPTH.with(|d| d.set(d.get() + 1));
    let out = map_type(&under, &alias_gen);
    ALIAS_DEPTH.with(|d| d.set(d.get() - 1));
    out
}

fn map_type(t: &Json, gen: &Generics) -> Option<String> {
    if let Some(p) = t["primitive"].as_str() {
        return map_prim(p);
    }
    if let Some(g) = t["generic"].as_str() {
        return gen.get(g).cloned().flatten(); // générique résolu par ses bornes (ou None → sauté)
    }
    if let Some(bounds) = t.get("impl_trait") {
        return bound_to_iron(bounds); // `impl AsRef<Path>` en position d'argument
    }
    if let Some(b) = t.get("borrowed_ref") {
        return map_type(&b["type"], gen); // emprunt en entrée → intérieur (& réinjecté au lowering)
    }
    if let Some(s) = t.get("slice") {
        return map_type(s, gen).map(|x| format!("list[{x}]"));
    }
    if let Some(items) = t["tuple"].as_array() {
        let parts: Option<Vec<String>> = items.iter().map(|x| map_type(x, gen)).collect();
        return parts.map(|p| format!("({})", p.join(", ")));
    }
    if let Some(rp) = t.get("resolved_path") {
        let name = rp["path"].as_str()?.rsplit("::").next().unwrap_or("");
        let targs: Vec<&Json> = rp["args"]["angle_bracketed"]["args"]
            .as_array()
            .map(|a| a.iter().filter_map(|x| x.get("type")).collect())
            .unwrap_or_default();
        // B33 : si le chemin pointe sur un alias de type, l'expanser (`ImageResult<DynamicImage>` →
        // `Result[DynamicImage]`). On ne le fait que si l'expansion **mappe** ; sinon on retombe sur le
        // handle (`map_named`) — strictement non régressif.
        if let Some(id) = rp["id"].as_u64() {
            if let Some(expanded) = try_expand_alias(id, &targs, gen) {
                return Some(expanded);
            }
        }
        return map_named(name, &targs, gen);
    }
    None // dyn Trait, qualified_path, raw_pointer → couche B/C
}

fn map_named(name: &str, targs: &[&Json], gen: &Generics) -> Option<String> {
    let one = |i: usize| targs.get(i).and_then(|t| map_type(t, gen));
    match name {
        "String" | "str" | "Cow" | "PathBuf" | "OsString" => Some("str".to_string()),
        "Vec" | "VecDeque" => Some(format!("list[{}]", one(0)?)),
        "Option" => Some(format!("Option[{}]", one(0)?)),
        "Result" => Some(format!("Result[{}]", one(0)?)),
        "HashMap" | "BTreeMap" => Some(format!("dict[{}, {}]", one(0)?, one(1)?)),
        "HashSet" | "BTreeSet" => Some(format!("set[{}]", one(0)?)),
        "Box" | "Rc" | "Arc" => one(0),
        // struct/enum nommé (PascalCase) → réf de type externe. V12 [24000] : si **appliqué** (`Point<T>`) et que
        // tous les args mappent, émettre `Point[args]` (→ `ExternApp`) au lieu de jeter les args. Un arg non
        // mappable → handle opaque `Point` (pas de drop ; non régressif).
        other if other.chars().next().map_or(false, |c| c.is_ascii_uppercase()) => {
            if targs.is_empty() {
                Some(other.to_string())
            } else {
                let mapped: Option<Vec<String>> = targs.iter().map(|t| map_type(t, gen)).collect();
                match mapped {
                    Some(args) if !args.is_empty() => Some(format!("{other}[{}]", args.join(", "))),
                    _ => Some(other.to_string()),
                }
            }
        }
        _ => None,
    }
}

/// Traits « bruit » (dérives/marqueurs/conversions) dont on ne binde PAS les méthodes (51600) : elles
/// polluent l'API (fmt/clone/eq/from…) sans valeur. On garde les traits de **comportement**.
fn is_noise_trait(name: &str) -> bool {
    matches!(
        name,
        // `Default` n'est **pas** bruit : son `default()` est un **constructeur** très utile (builders/config
        // qui n'ont pas de `new`, ex. `ChannelBuilder::default()`). Une seule méthode → pas de clutter (B25).
        "Debug" | "Display" | "Clone" | "Copy" | "Send" | "Sync" | "Sized" | "Unpin"
            | "PartialEq" | "Eq" | "PartialOrd" | "Ord" | "Hash"
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
            // V12 : un nom d'**un seul caractère** majuscule (`T`, `R`, `K`, `W`…) est un **paramètre de type**,
            // pas un vrai type → ne pas l'émettre en `extern type T = …` (pollution + .iri illisible).
            if word.len() > 1 && word != "Option" && word != "Result" {
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
