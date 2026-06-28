# V11-test-2 — Réalisation (try & run, un par un)

> Journal de réalisation des 80 projets. **Vague 0 = durcir le pipeline** (séquentiel) ; puis fan-out
> par risque. Statuts : ✅ vert · 🟡 partiel (bug noté) · 🔴 bloqué (mur) · ☐ à faire.

## Vague 0 — durcissement du pipeline `ironc add` + `import` (révélé par les pilotes)

Chaque crate réel a exposé un trou ; tous corrigés sans régresser le gate-v11 (30 bindings + 50 corpus).

| # | Bug | Couche | État |
|---|-----|--------|:--:|
| B1 | Méthode nommée comme un **mot-clé Iron** (`not`/`and`/`in`…) → parse error. | binder | ✅ |
| B2 | **Nom de crate** hyphen/underscore : directive `.iri` portait le nom import. → nom publié + parseur tolère les tirets. | binder/iri | ✅ |
| B3 | **Résolution `.iri`** : `import` cherchait à côté du source ; `ironc add` écrit au cwd. → base_dir → racine → cwd (absolu). | driver | ✅ |
| B4 | **Chemin de méthode de trait** : `impl Trait for T` préfixé `{crate}::Trait` (faux pour std). → résolu via `paths[id]`. | binder | ✅ |
| B5 | **Constructeur associé à un trait** (`One::one`, `from_u64`) pathé `crate::Type::one`. → UFCS `<T as Trait>::ctor`. | binder | ✅ |
| B6 | **Paramètre** nommé comme un mot-clé (`lambda`, `in`). → renommé `{nom}_` (appel positionnel). | binder | ✅ |
| B7 | **Chemin de type en sous-module** (`statrs::distribution::Beta`) + re-export `mod beta { pub Beta }`. → `paths[id]` + heuristique. | binder | ✅ |
| B8 | **`.unwrap()`/`.expect()` perdaient le type** (`Option[T]`/`Result[T]` → `Unit`) → méthode de trait suivante non routée en UFCS. | infer | ✅ |
| B9 | **Chemins canoniques à travers des modules privés** glob-réexportés (`statrs::statistics::traits::Distribution`). → BFS `id → chemin public`. | binder | ✅ |
| B10 | **Coercition numérique** aux appels externes : `i64` Iron → `i32`/`u32`/`usize`/`u64`/`f64` du binding (`(v) as T`). | lower | ✅ |
| B11 | (Faux bug — `ironc add` normalise déjà `num-bigint` → `num_bigint.iri`, `import num_bigint`.) | — | n/a |
| B12 | **Crate étrangère d'un trait UFCS** non déclarée (`num_traits::…`) → `E0433`. → lit `Cargo.lock` (versions exactes), émet `crate <dep>`. | binder | ✅ |
| — | **Modules privés stdlib** (`core::ops::arith::Mul`, `core::str::traits::FromStr`) → collapse vers le re-export public. | binder | ✅ |
| B13 | **Fns génériques** sautées en bloc → monomorphisation des bornes d'entrée (`impl AsRef<Path>` → `str`) + `Self` résolu. Débloque `new(path: impl AsRef<Path>)`. | binder | ✅ |
| B14 | **Boucle `for` sur type externe** émettait `.iter()` (absent) → itération **par valeur** (`for x in #it`, couvre `IntoIterator` ET `Iterator`). | lower | ✅ |

**Gains généraux B8/B9/B10/B12/B13/B14** (débloquent des classes entières) :
- B8 → tout le motif `ctor().unwrap().méthode()` (omniprésent).
- B9 + collapse stdlib → tout crate organisant ses traits en modules privés glob-réexportés (convention Rust courante).
- B10 → tout crate à entiers dimensionnés (`u32`/`usize`…).
- B12 → tout l'écosystème `num-traits`/`rand` (traits exposés depuis une autre crate).
- B13 → tout ctor/méthode `… (path: impl AsRef<Path>)` (APIs fichier : walkdir, csv, image, hound, File…).
- B14 → toute itération `for x in <type externe itérable>` (walkdir, lecteurs csv…).

## Projets réalisés

| # | Projet | Crate | Statut | Note |
|---|--------|-------|:--:|------|
| 60 | Détection de langue | `whatlang` | ✅ **vert** | `detect(t).unwrap().lang().eng_name()` → French/English. Crate témoin. |
| 54 | Lois de probabilité | `statrs` | ✅ **vert** | `Beta.new(2,5).unwrap().mean() = 0.2857…` exact. A révélé B6/B7/B8/B9. |
| 66 | Calendrier | `chrono` | ✅ **vert** | `NaiveDate.from_ymd_opt(2026,6,28)` → `Sun`, ordinal `179`. A révélé B10. |
| 53 | Grands entiers | `num-bigint` | ✅ **vert** | `30! = 265252859812191058636308480000000` exact. A révélé B5/B12 + collapse stdlib. |
| 03/04 | Parcours d'arbo | `walkdir` | ✅ **vert** | `for e in WalkDir.new("."): n += 1`. A révélé B13 (ctor `AsRef<Path>` + `Self`) et B14 (itération). |
| 06 | CSV → JSON | `csv` | ✅ **vert** | `Reader.from_path(p).unwrap()` + `for rec in rdr.records()` → 3 lignes. Débloqué par B13 + B14. |
| 02/80 | Hash / PoW | `sha2` | ✅ **vert** | **B16** (impls blanket matérialisés) : `Sha256.digest("hello world")` → digest **32 octets** correct. Reste hors-mur : afficher le hex (ergonomie d'octets). |
| 42 | Synthé / WAV | `hound` | ✅ **vert** | **B17** (struct/enum externes) + **B18** (arg par valeur) : écrit un WAV valide (`RIFF…WAVE`, en-tête 44 o, `sample_rate=44100`). Reste hors-mur : `write_sample<S: Sample>` (générique sur trait de crate). |
| 05 | Renommeur regex | `regex` | ✅ **vert** (cœur) | **B15** : dédup retient `regex::Regex` (chemin court). `Regex.new("[0-9]+").unwrap()` → `is_match`/`find().as_str()` OK. `replace_all` reste non bindé (arg `R: Replacer` ; coercition `&String`→`&str` à part). |
| 04 | Texte coloré | `colored` | 🔴 | `.red()` est ajouté à `&str` via le trait `Colorize` ; Iron `str` est un builtin (pas un type externe) → impossible d'y greffer une méthode de trait. **Mur dur : extension d'un type primitif.** |
| 67 | Distances géo | `geo` | 🔴 | `Point` est **re-exporté de la dép `geo_types`** où vit `Point::new` → invisible au bind mono-crate. Mur : **crate façade** (binding multi-crate). |
| 63 | Gzip | `flate2` | 🔴 | `GzEncoder::new(w: impl Write, …)` consomme un **puits d'octets** en streaming → cogne avec la sémantique par valeur. Mur : **modèle io `Write`/sink**. |

## Murs — 3/5 corrigés (regex B15, sha2 B16, hound B17/B18) ; 3 restants = **grosses features**

Tous corrigés en **règles globales** (zéro nom de crate). Restent (par difficulté) :

1. ✅ ~~**sha2 — traits blanket**~~ → **B16** : on lie les blanket **matérialisés par type concret** par
   rustdoc (le `for` d'`impl<D> Digest for D` apparaît `Sha256` dans la liste d'impls de `Sha256`) + le
   générique du blanket = `Self`. Règle générale (Digest, Read, Write…).
2. ✅ ~~**hound — struct/enum externes**~~ → **B17** (littéral `WavSpec(...)` + `SampleFormat.Int`) + **B18**
   (arg par valeur `own T`). Écrit un WAV valide.
3. 🔴 **colored — extension d'un type primitif** : `.red()` greffé sur `&str` via `Colorize` ; Iron `str`
   est un builtin, pas un type externe → router `x.red()` vers `<str as Colorize>::red(&x)` exige de binder
   les impls de trait sur les **primitifs** + router au lowering. **Moyen-dur** (touche le modèle du langage).
4. 🔴 **geo — crate façade** : `Point` re-exporté de la dép `geo_types`, son ctor `Point::new` y vit →
   invisible au bind mono-crate. → piste : binder les crates **sources** des re-exports. **Gros (multi-crate).**
5. 🔴 **flate2 — modèle io `Write`/sink** : `GzEncoder::new(w: impl Write, …)` consomme un puits d'octets en
   streaming → cogne avec la sémantique par valeur. **Gros (modèle io, pas qu'un binding).**

> regex était un **bug** (variante) → corrigé. sha2/hound étaient des **features** → construites en règles
> globales (B16/B17/B18). Restent colored (moyen-dur), geo et flate2 (gros) — chacune un vrai chantier.

## Constat & cadence (honnête)

- Le binder est **solide** et nettement durci : **9 crates réels verts** (whatlang, statrs, chrono,
  num-bigint, walkdir, csv, regex, sha2, hound), **18 trous/features** (B5–B18), dont les gros gains
  généraux B8/B9/B10/B12/B13/B14 + **blanket** (B16), **struct/enum externes** (B17), **arg par valeur**
  (B18). Le gate reste VERT à chaque pas.
- **« Tous les 80 un par un » reste un grind séquentiel** : chaque crate peut révéler un bug à corriger dans
  `ironc/src` (non parallélisable). Réaliste : **plusieurs sessions**.
- **Plan** : continuer les feuilles 🟢/🟡 (CLI/données/temps/maths à API simples) ; adresser les 3 murs
  ci-dessus quand un crate utile les réclame ; garder 🔴 (GUI/jeux/async/ML) et **ESP32/no_std** (chantier
  V12) pour la fin.
