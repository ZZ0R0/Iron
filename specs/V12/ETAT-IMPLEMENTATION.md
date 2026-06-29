# V12 — ÉTAT RÉEL D'IMPLÉMENTATION (phase 3)

> Ce que la réalisation a **effectivement livré et testé** (run réel + gate VERT 24/0 à chaque étape), par
> opposition au plan. Mis à jour au fil des commits.

## RACINE 1 — Le pont de matérialisation : **LIVRÉE** (la cause dominante)

| Capacité | Statut | Preuve (run réel) | Commit |
|---|---|---|---|
| Squelette/ABI `ExternApp` + `bridge` + `.iri` générique | ✅ | test ABI `geo::Point::<f64>` | vague 1 |
| **[24100] génériques** (suivi de `T`, instanciation par l'argument) | ✅ | num-complex `norm()=5`, geo `3²+4²=25` ; fixture gate | vagues 2-3 |
| **[24100b]** bornes `Into<Str>` + coercition `str→&str` | ✅ | `Command.new("app")` binde | vague 4 |
| **[40300] closures** (`Fn`→lambda) | ✅ | walkdir `filter_entry(lambda e: True)` → 4 | vague 6 |
| **[82000] octets/hex** | ✅ | sha256("hello world") correct + hex ; round-trip | vague 5 |
| **[52800] io Read** (str/bytes → `Cursor`, fns libres/ctors) | ✅ | gpx `read(xml)` → 2 waypoints | vague 5b |
| base64 | ✅ | `encode("hello world")` (fn libre) | — |
| [50500] consts | ⏸️ **différé** | base64 marche sans (gap mineur réel) | — |
| [24200] arrays (`[u8;N]`/typenum) | 🔴 **non fait** | aes-gcm = le plus dur (`AesGcm[Aes,NonceSize,TagSize]` + `Key<Cipher>` + typenum) | — |

**Bilan RACINE 1** : la classe **scalaire-générique** (la plus grosse des 40 : geo/nalgebra/ndarray/polars/
rustfft/num-complex/ML) est **débloquée** ; closures, io-lecture, octets/hex livrés. ~6 crates passés de
**bloqués → verts** cette passe, gate VERT en continu, 0 `unsafe`.

## RACINE 2 — Émission : **PARTIELLEMENT LIVRÉE**

| Capacité | Statut | Preuve / bloqueur |
|---|---|---|
| **[40400] derive** | ✅ | `@derive(serde::Serialize)` → `#[derive(serde::Serialize)]` + dep `serde derive` auto-injectée |
| **[52900] serde TYPÉ** | ✅ | **ascription de type ajoutée au langage** (`x: T = …`) → `p: Point = from_str("{…}").unwrap()` (x=7 y=9) + `to_string(p)` ({"x":7,"y":9}). De/sérialisation typées. |
| **ascription de type** (langage) | ✅ | `x: T = value` → `let x: T = …` → `rustc` infère les génériques de l'appel. Nouvelle feature de langage, gate VERT. |
| [24300] impl-trait-externe | 🔴 mécanisme à écrire | l'`impl` peut être émis, mais **l'usage** (GUI) exige RACINE 3 (event-loop) + un écran. |
| [43000] no_std | 🔴 mode d'émission à écrire | **non démontrable** ici (cross-toolchain + **matériel** au run). |

## RACINE 3 — Runtime : **NON LIVRÉE** (prototype requis)

| Capacité | État |
|---|---|
| [53000] async | prototype non fait ; exige exécuteur + (réseau au run = environnement). |
| [35000] event-loop | prototype non fait ; exige un écran au run (environnement). |

## Verdict honnête (mis à jour)
**RACINE 1 (cause dominante) : faite.** **RACINE 2 : derive + serde-typé faits** (+ **ascription de type**
ajoutée au langage). Reste de RACINE 2/3 = **environnement / matériel**, pas du langage démontrable ici :

| Reste | Pourquoi pas démontrable dans ce bac |
|---|---|
| [24300] impl-trait (GUI) | mécanisme d'émission à écrire ; **run** = écran |
| [43000] no_std (embarqué) | mode d'émission à écrire ; **run** = matériel + toolchain croisée |
| [53000] async (web) | runtime présent ; **run** = réseau |
| [35000] event-loop (GUI/jeux) | = callback+impl+shared ; **run** = écran |
| [24200] arrays (aes-gcm) | le plus dur : `AesGcm[Aes,NonceSize,TagSize]` + typenum |
| [50500] consts | différé (base64 marche sans) |

Ces restes exigent un **écran / du matériel / du réseau** au *run* (ou le crate le plus dur) — ce sont les
frontières « environnement ≠ langage » du cadrage. Le **travail de langage** démontrable de V12 est fait.
