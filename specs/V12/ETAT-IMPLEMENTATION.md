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

## RACINE 2 — Émission : **NON LIVRÉE** (murs secondaires réels)

| Capacité | Bloqueur secondaire honnête |
|---|---|
| [52900] serde **typé** | `from_str::<T>` (désérialisation) exige une **ascription de type** (`x: T = …`) — **fonctionnalité de langage absente d'Iron**. La sérialisation seule exigerait une stratégie `Serialize` ciblée (garder `T` struct-matérialisable) + dep `serde derive`. |
| [40400] derive | mécanisme `@derive` présent, mais sans cas d'usage complet (serde-typé ci-dessus). |
| [24300] impl-trait-externe | l'`impl` peut être émis, mais **l'usage** (GUI) exige RACINE 3 (event-loop) + un écran. |
| [43000] no_std | mode d'émission faisable, mais **non démontrable** ici (cross-toolchain + **matériel** au run). |

## RACINE 3 — Runtime : **NON LIVRÉE** (prototype requis)

| Capacité | État |
|---|---|
| [53000] async | prototype non fait ; exige exécuteur + (réseau au run = environnement). |
| [35000] event-loop | prototype non fait ; exige un écran au run (environnement). |

## Verdict honnête
**RACINE 1 (la cause dominante) est faite et testée** — le plus gros débloquage des 40. **RACINE 2 et 3 ne
sont PAS faites** : elles butent sur des **besoins secondaires réels** — une **ascription de type** (langage,
pour serde-typé), la **boucle d'événements** (pour GUI), et l'**environnement** (écran/matériel/réseau/runtime
async). Ce ne sont pas des oublis : ce sont les frontières que le cadrage avait marquées « à prototyper » et
« hors-langage (environnement) ». Les livrer proprement = d'autres cycles (dont un **ajout de langage** :
l'ascription de type), pas une simple passe de binder.
