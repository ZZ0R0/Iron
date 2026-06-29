//! [70400] **La garde** — runner du corpus intégré à `cargo test`.
//!
//! Remplace les `gate-vN.sh` épars (jamais joués par `cargo test`) par UN test Rust qui
//! re-joue tout le corpus à chaque `cargo test` → toute régression échoue automatiquement.
//!
//! Un *cas* = un `<nom>.irn` accompagné d'**exactement un** fichier-frère qui déclare l'attendu :
//!   - `<nom>.out`     → compile + **exécute** (`ironc run`) ; le stdout doit égaler le fichier (oracle).
//!   - `<nom>.compile` → compile **in-process** vers du Rust valide (rapide, sans cargo) ; vérifie I2
//!                       (`forbid(unsafe_code)`, 0 `unsafe` réel) + source-map non vide. Contenu = note libre.
//!   - `<nom>.err`     → la compilation **doit échouer** ; chaque ligne non vide = un fragment que le
//!                       diagnostic doit contenir (test négatif).
//!   - `<nom>.sha`     → compile + exécute ; l'**empreinte stable** du stdout (FNV-1a, sans dépendance)
//!                       doit égaler le fichier. Pour les sorties volumineuses/déterministes.
//!
//! Un `.irn` **sans** frère est un **module auxiliaire** (importé par un cas) : listé, jamais joué seul.
//! Deux tiers exécutent cargo (`.out`/`.sha`) : pour une boucle interne rapide, `IRON_CORPUS_FAST=1`
//! les saute (ne garde que `.compile`/`.err`, in-process).

use std::path::{Path, PathBuf};
use std::process::Command;

const CORPUS: &str = "tests/corpus";
/// Chemin du binaire `ironc` construit par cargo avant ce test d'intégration (e2e réel).
const IRONC: &str = env!("CARGO_BIN_EXE_ironc");

enum Kind {
    Out(String),
    Compile,
    Err(Vec<String>),
    Sha(String),
}

impl Kind {
    fn label(&self) -> &'static str {
        match self {
            Kind::Out(_) => "out",
            Kind::Compile => "compile",
            Kind::Err(_) => "err",
            Kind::Sha(_) => "sha",
        }
    }
    fn runs_cargo(&self) -> bool {
        matches!(self, Kind::Out(_) | Kind::Sha(_))
    }
}

/// Empreinte stable, sans dépendance (FNV-1a 64 bits — mêmes constantes que `driver::out_dir_for`).
fn digest_hex(s: &str) -> String {
    let mut h: u64 = 1469598103934665603;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(1099511628211);
    }
    format!("{h:016x}")
}

fn collect_irn(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            // Dossiers `_*` (ex. `_defects/`) = **hors garde** : cas en attente de correction,
            // parqués sur disque pour les vagues de fix, jamais joués (ne cassent pas le vert).
            if p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.starts_with('_')) {
                continue;
            }
            collect_irn(&p, out);
        } else if p.extension().is_some_and(|x| x == "irn") {
            out.push(p);
        }
    }
}

/// Détermine le tier d'un `.irn` d'après son unique frère. `None` = module auxiliaire.
/// `Err` = déclaration ambiguë (plusieurs frères) → remontée comme échec de cas.
fn classify(irn: &Path) -> Result<Option<Kind>, String> {
    let present: Vec<&str> = ["out", "compile", "err", "sha"]
        .into_iter()
        .filter(|x| irn.with_extension(x).exists())
        .collect();
    match present.as_slice() {
        [] => Ok(None),
        [one] => {
            let sidecar = irn.with_extension(one);
            let read = || std::fs::read_to_string(&sidecar).unwrap_or_default();
            Ok(Some(match *one {
                "out" => Kind::Out(read()),
                "compile" => Kind::Compile,
                "sha" => Kind::Sha(read().trim().to_string()),
                "err" => Kind::Err(
                    read().lines().map(str::trim).filter(|l| !l.is_empty()).map(String::from).collect(),
                ),
                _ => unreachable!(),
            }))
        }
        many => Err(format!("déclaration ambiguë (frères multiples : {many:?})")),
    }
}

fn compile_in_process(irn: &Path) -> Result<ironc::driver::Compiled, String> {
    let src = std::fs::read_to_string(irn).map_err(|e| e.to_string())?;
    let base = irn.parent().unwrap_or_else(|| Path::new("."));
    ironc::driver::compile(&src, 0, base).map_err(|e| e.to_string())
}

fn run_via_cli(irn: &Path) -> Result<String, String> {
    let out = Command::new(IRONC)
        .args(["run", &irn.to_string_lossy()])
        .output()
        .map_err(|e| format!("lancement de ironc : {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "`ironc run` a échoué (code {:?})\n--- stderr ---\n{}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn run_case(irn: &Path, kind: &Kind) -> Result<(), String> {
    match kind {
        Kind::Compile => {
            let c = compile_in_process(irn)?;
            if !c.rust_src.contains("forbid(unsafe_code)") {
                return Err("I2 : `forbid(unsafe_code)` absent du Rust généré".into());
            }
            if c.mappings.is_empty() {
                return Err("I8 : source-map vide".into());
            }
            let real_unsafe = c
                .rust_src
                .lines()
                .filter(|l| l.contains("unsafe") && !l.contains("forbid"))
                .count();
            if real_unsafe > 0 {
                return Err(format!("I2 : {real_unsafe} `unsafe` réel(s) émis"));
            }
            Ok(())
        }
        Kind::Err(needles) => match compile_in_process(irn) {
            Ok(_) => Err("compilation réussie alors qu'une erreur était attendue".into()),
            Err(diag) => {
                for n in needles {
                    if !diag.contains(n) {
                        return Err(format!("le diagnostic ne contient pas «{n}» :\n{diag}"));
                    }
                }
                Ok(())
            }
        },
        Kind::Out(want) => {
            let got = run_via_cli(irn)?;
            if got.trim_end() != want.trim_end() {
                return Err(format!(
                    "sortie différente :\n--- attendu ---\n{}\n--- obtenu ---\n{}",
                    want.trim_end(),
                    got.trim_end()
                ));
            }
            Ok(())
        }
        Kind::Sha(want) => {
            let got = run_via_cli(irn)?;
            let h = digest_hex(got.trim_end());
            if &h != want {
                return Err(format!("empreinte {h} ≠ attendue {want}"));
            }
            Ok(())
        }
    }
}

fn rel(p: &Path) -> String {
    p.strip_prefix(CORPUS).unwrap_or(p).to_string_lossy().into_owned()
}

#[test]
fn corpus() {
    let root = Path::new(CORPUS);
    assert!(root.exists(), "répertoire corpus absent : {}", root.display());

    let mut irns = Vec::new();
    collect_irn(root, &mut irns);
    irns.sort();

    let fast = std::env::var_os("IRON_CORPUS_FAST").is_some();
    let mut helpers = Vec::new();
    let mut fails = Vec::new();
    let (mut pass, mut skipped) = (0u32, 0u32);
    let mut by_tier = std::collections::BTreeMap::<&str, u32>::new();

    for irn in &irns {
        match classify(irn) {
            Ok(None) => helpers.push(rel(irn)),
            Ok(Some(kind)) => {
                if fast && kind.runs_cargo() {
                    skipped += 1;
                    continue;
                }
                match run_case(irn, &kind) {
                    Ok(()) => {
                        pass += 1;
                        *by_tier.entry(kind.label()).or_default() += 1;
                    }
                    Err(e) => fails.push(format!(
                        "  ✗ {} [{}]\n      {}",
                        rel(irn),
                        kind.label(),
                        e.replace('\n', "\n      ")
                    )),
                }
            }
            Err(e) => fails.push(format!("  ✗ {} : {}", rel(irn), e)),
        }
    }

    let tiers: Vec<String> = by_tier.iter().map(|(k, v)| format!("{k}:{v}")).collect();
    eprintln!(
        "[corpus] {} cas verts ({}) · {} module(s) aux. · {} sauté(s){}",
        pass,
        tiers.join(" "),
        helpers.len(),
        skipped,
        if fast { " [FAST]" } else { "" }
    );

    if !fails.is_empty() {
        panic!(
            "{} cas KO / {} :\n{}",
            fails.len(),
            pass + fails.len() as u32,
            fails.join("\n")
        );
    }
}
