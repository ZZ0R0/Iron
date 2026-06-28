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
| 02/80 | Hash / PoW | `sha2` | 🟡 | `Sha256` : `new`/`finalize` vivent dans le trait **blanket** `Digest` (`impl<T> Digest for T`) → le binder les saute (les blankets sont partout). Seul `update` (impl direct `Update`) est bindé. **Mur : API de convenance via trait blanket.** |
| 05 | Renommeur regex | `regex` | 🟡 | Le binder a retenu `regex::bytes::Regex` (variante octets) au lieu de `regex::Regex` (str) sur collision de nom court → `replace_all("…","X")` veut un `bytes::Replacer`. Mur : **dédup choisit la mauvaise variante** (préférer le chemin le plus court). |
| 04 | Texte coloré | `colored` | 🔴 | `.red()` est ajouté à `&str` via le trait `Colorize` ; Iron `str` est un builtin (pas un type externe) → impossible d'y greffer une méthode de trait. **Mur dur : extension d'un type primitif.** |
| 63 | Gzip | `flate2` | ☐ | `import` OK ; usage réel (Encoder/Write, io) à tester. |

## Murs identifiés (frontière)

1. **Traits blanket de convenance** (sha2 `Digest`, et tout `Read`/`Write`/`Iterator` blanket) : l'API
   utile (`new`/`finalize`/`read`/`collect`) est dans un `impl<T> Trait for T` (le `for` est un `T`
   générique, pas un type concret) → sauté par nécessité (sinon explosion). → piste : *whitelist* de
   traits-blanket utiles + synthèse de leurs méthodes sur les types concrets qui satisfont les bornes.
2. **Dédup variante de type** (regex `Regex` vs `bytes::Regex`) : sur collision de nom court, préférer le
   chemin **public le plus court** (la variante « principale »). Nécessite un pré-passage type.
3. **Extension d'un type primitif** (colored sur `str`) : Iron ne peut greffer une méthode de trait sur son
   `str` builtin. Mur dur (sémantique du langage).

> ✔ *Résolu cette session* : « ctors sur types génériques » (csv `Reader<R>`) — c'était la borne
> `impl AsRef<Path>` non monomorphisée, corrigée par **B13**.

## Constat & cadence (honnête)

- Le binder est **solide** et nettement durci : **6 crates réels verts** (whatlang, statrs, chrono,
  num-bigint, walkdir, csv), **14 trous bouchés**, dont **6 généraux** (B8/B9/B10/B12/B13/B14). Le gate
  reste VERT à chaque pas.
- **« Tous les 80 un par un » reste un grind séquentiel** : chaque crate peut révéler un bug à corriger dans
  `ironc/src` (non parallélisable). Réaliste : **plusieurs sessions**.
- **Plan** : continuer les feuilles 🟢/🟡 (CLI/données/temps/maths à API simples) ; adresser les 3 murs
  ci-dessus quand un crate utile les réclame ; garder 🔴 (GUI/jeux/async/ML) et **ESP32/no_std** (chantier
  V12) pour la fin.
