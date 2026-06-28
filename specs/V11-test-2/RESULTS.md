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
| 04 | Texte coloré | `colored` | ✅ **vert** | **B19** (méthodes de trait sur primitif `str` + méthodes fournies) : `"error".red()` → `\e[31m`, `"ok".green().bold()` → `\e[1;32m` (chaînage). |
| 67 | Distances géo | `geo` | 🟡 | **Façade corrigée** (52700, binding multi-crate) → `Point` porte ses méthodes geo_types. **Mais** `Point<T: CoordNum>` est générique sur le **scalaire** en entrée *et* en sortie (`new(x: T)`, `distance() -> T`) → monomorphiser `T`→`f64` exigerait de reconnaître `CoordNum`/`Num` (traits de crate) : **interdit par la règle globale**. Limite dure. |
| 63 | Gzip | `flate2` | 🔴 | **Modèle io.** Tout passe par `Write`/`Read`/`BufRead` (puits/sources d'octets streaming) + params `&mut Vec` (out-params) + variantes `read`/`write`/`bufread`. Même en mappant `Write`/`Read` (std) → `list[u8]`, l'usage cogne sur : littéraux `list[u8]` (la liste est `i64`), `&[u8]` vs `&Vec` pour `BufRead`, out-params. Stack de features hors sémantique par valeur d'Iron. **Limite de modèle** (chantier io V12). |

## Murs — 4/5 corrigés (regex B15, sha2 B16, hound B17/B18, colored B19) ; 2 restants = **gros**

Tous corrigés en **règles globales** (zéro nom de crate) :

1. ✅ ~~**sha2 — traits blanket**~~ → **B16** : blanket **matérialisés par type concret** (le `for` d'`impl<D>
   Digest for D` apparaît `Sha256` dans la liste d'impls de `Sha256`) + générique du blanket = `Self`.
2. ✅ ~~**hound — struct/enum externes**~~ → **B17** (littéral `WavSpec(...)` + `SampleFormat.Int`) + **B18**
   (arg par valeur `own T`). Écrit un WAV valide.
3. ✅ ~~**colored — extension de primitif**~~ → **B19** : `impl Trait for &str` rattaché au pseudo-type
   `__str`, routé en UFCS `Trait::m(&*x)` ; + binding des méthodes **fournies** (corps par défaut) du trait.
4. 🟡 **geo — crate façade** → **52700 corrigé** (binding multi-crate : `rustdoc -p geo_types` + fusion).
   `Point` porte ses méthodes. **Reste bloqué** par le scalaire générique `T: CoordNum` (entrée **et**
   sortie) : monomorphiser `T`→`f64` exige de reconnaître `CoordNum`/`Num` (traits **de crate**) → interdit.
   **Limite de modèle** : Iron ne suit pas les paramètres de type (`Point<T>`).
5. 🔴 **flate2 — modèle io** : tout est `Write`/`Read`/`BufRead` + out-params `&mut Vec` + variantes
   `read`/`write`/`bufread`. Stack de features hors sémantique par valeur. **Limite de modèle** (io V12).

> regex était un **bug** → corrigé. sha2/hound/colored étaient des **features** → construites en règles
> globales (B16–B19). **geo** : façade corrigée (52700), mais scalaire générique = **limite de modèle**
> (pas globalement contournable sans hack crate). **flate2** : modèle io streaming = **limite de modèle**.
> Ces deux-là ne sont pas des bugs ni des features bindables — ce sont les **frontières dures** d'Iron sous
> la contrainte « global-only » : suivi de paramètres de type générique, et io par flux. Vrais sujets V12.

## Constat & cadence (honnête)

- Le binder est **solide** et nettement durci : **10 crates réels verts** (whatlang, statrs, chrono,
  num-bigint, walkdir, csv, regex, sha2, hound, colored), **B5–B19** (trous + features), dont les gros
  gains généraux : B8/B9/B10/B12/B13/B14 + **blanket** (B16), **struct/enum externes** (B17), **arg par
  valeur** (B18), **trait sur primitif + méthodes fournies** (B19). Le gate reste VERT à chaque pas.
- **« Tous les 80 un par un » reste un grind séquentiel** : chaque crate peut révéler un bug à corriger dans
  `ironc/src` (non parallélisable). Réaliste : **plusieurs sessions**.
- **Plan** : continuer les feuilles 🟢/🟡 (CLI/données/temps/maths à API simples) ; adresser les 3 murs
  ci-dessus quand un crate utile les réclame ; garder 🔴 (GUI/jeux/async/ML) et **ESP32/no_std** (chantier
  V12) pour la fin.
