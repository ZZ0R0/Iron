# Interop : utiliser des crates Rust

C'est **le cœur d'Iron** : il n'y a pas de stdlib Iron (I7bis), tout vient de crates.io. Tu importes un crate
Rust et le **binder** génère l'interop. Ce fichier explique le système, son **rapport de portée**, ses
**idiomes**, et sa **frontière**.

## Les 3 niveaux d'interop (I7)

| Niveau | Mécanisme | Quand |
|---|---|---|
| 1 | `rust { … }` inline | échappatoire ponctuelle pour du Rust verbatim |
| 2 | `.iri` **curé** (écrit à la main) | quand l'auto ne suffit pas — voir [curation.md](curation.md) |
| 3 | **auto-génération** (`ironc add`) | le défaut : binder un crate automatiquement |

## `ironc add <crate>` — la passe automatique

```bash
ironc add serde_json
```
1. crée un projet temporaire avec la dépendance, lance `cargo +nightly rustdoc --output-format json` ;
2. **projette** les items publics (fns, méthodes, ctors, champs de struct, variantes d'enum, constantes,
   traits) dont la signature se mappe vers le sous-ensemble Iron ;
3. écrit **`<crate>.gen.iri`** (le binding, régénérable) + **`<crate>.iri.report.md`** (le rapport) ;
4. ajoute le crate à `ironc.toml`.

Sortie console = **résumé de couverture** :
```
binding `serde_json` : 356 candidats — 219 liés, 137 sautés (couverture 61.5 %)
  · serde_json::Value::... [GenericTraitMethod: ...]
  ...
```

## Le **rapport de portée** — le moteur dit ce qu'il ne sait pas faire

`<crate>.iri.report.md` liste **chaque item sauté** avec **sa raison** et une **action**. C'est l'outil n°1 :
avant de te demander « pourquoi ça marche pas », **lis le rapport**. Taxonomie des raisons :

| Raison | Sens | Que faire |
|---|---|---|
| `UnmappedType(T)` | type de param/retour non projetable | curer l'entrée à la main |
| `GenericTraitMethod(Tr)` | signature dépend d'un trait générique | curer (matérialiser concret) |
| `ConstGenericShape` | valeur au niveau type (`Const<N>`, typenum) | C1 (alias) ou curation |
| `BorrowedReturnNonIdiom(T)` | retour emprunté hors idiome | durée de vie essentielle → curer/`rust{}` |
| `StreamBuilderApi` | API builder/flux à état | chaîner à la main / curer |
| `UnsafeFn` | fn `unsafe` | ignoré (I2) — normal |
| `MacroOrHidden` | item généré par macro / caché | `rust { }` si besoin |
| … | (taxonomie fermée, voir le rapport) | |

**Invariant** : `seen == bound + skipped` — **zéro item sauté en silence**. La couverture est un nombre **calculé**.

## Le système **deux couches** (gen + curé)

```
<crate>.gen.iri   ← AUTO, régénérable par `ironc add`, NE PAS éditer
<crate>.iri       ← CURÉ, écrit à la main, optionnel, GAGNE par entrée (kind, nom)
```
À l'import, Iron charge `.gen.iri` **puis** `.iri` : une entrée curée **écrase** son homologue auto. Régénérer
le binding (`ironc add`) réécrit `.gen.iri` et **laisse `.iri` intact**. → tu peux corriger un binding à la
main sans perdre ton travail à la prochaine régénération. (`import std` est un `.iri` 100 % curé, embarqué.)

## Ce qui se projette bien (auto)

- **Primitifs & collections** : `i64/f64/bool/str/char`, `Vec→list`, `HashMap→dict`, `HashSet→set`, tuples,
  `Option`/`Result`, `Box/Rc/Arc` (transparents).
- **POO** : ctors (`Type.new(...)`), méthodes (`obj.m(...)`), champs publics (`Type(field=val)`), variantes
  d'enum (`Enum.Variant`).
- **Génériques scalaires** : `Point<T>`, `Complex<T>`, `Ratio<T>` → `Point[f64]` etc.
- **Dimensions (const-génériques via alias)** : `Vector3<T>`, `Matrix3<T>` → `Vector3.new(x,y,z)` **+ héritage
  des méthodes du type de base** (`v.norm()`, `v.dot(o)` viennent de `Matrix`). *(mécanisme C1.)*
- **Octets taille fixe** : `[u8; N]`, `GenericArray<u8, N>`, `Key`/`Nonce`/`Tag` → `list[u8]` (matérialisé).
- **serde typé** : `@derive(serde::Serialize)` + ascription `cfg: Config = from_str(s)?`.
- **Constantes & traits publics** : `extern const`, `extern trait` (implémentable : `class My(App):`).

## Les **idiomes** (glue reconnue automatiquement)

Quand le binder reconnaît un motif Rust, il émet la glue idiomatique :

| Idiome | Motif Rust | Effet Iron |
|---|---|---|
| `builder` | `&mut self -> &mut Self` | mutation en place, retour jeté (chaînage) |
| `clone` | retour emprunté `&T` | valeur possédée (`.clone()`/`.to_string()`) |
| `collect` | retour itérateur | `.collect::<Vec<_>>()` → `list[T]` |
| `closure(i)` | param `Fn`-borné | argument = **lambda** Iron |
| `display` | retour `Display`-borné emprunté | `.to_string()` (chaîne possédée) |

## La **frontière** (ce que l'auto ne fait PAS — mesuré)

| Famille | Exemples | Pourquoi | Issue |
|---|---|---|---|
| **Type-level lourd** | uom (unités), palette (couleur), nom (combinateurs) | dimensions/espaces encodés en types | curation partielle / `rust{}` |
| **Phantom types** | euclid `Point2D<T, Unit>` | unité fantôme non inférable (E0282) | annoter l'unité / curer |
| **Storage générique** | ndarray `ArrayBase<S, D>` | param de stockage non résolu (E0308) | curation |
| **AEAD / trait générique** | aes-gcm `encrypt(impl Into<Payload>)` | méthode de trait + `Into<crate-type>` | **curation** (gabarit fourni) |
| **Flux / builder à état** | flate2 `Encoder::new(w)…finish()` | état porté entre appels | chaîner à la main |
| **Event-loop / async / no_std / ML lourd** | GUI, axum, esp-hal, candle | architecture, pas binding | hors-périmètre langage |

> **Règle d'or honnête** : la couverture auto est **mesurée 16–92 %** selon le crate. Ce n'est **jamais** promis
> à 100 %. Le rapport te dit exactement quoi curer ([curation.md](curation.md)). Au-delà de C1, peu de
> *généralités* restent — la frontière est documentée et mesurée dans [status.md](status.md).
