# V18 — État d'implémentation (consignation, phase 4)

> Réalisation V18 : **garde + audit éventail → closures → ergonomie/correction → async/scope → mesure**.
> Méthode : la garde (`cargo test --test corpus`) re-joue le corpus à chaque pas → toute régression échoue.

## Fait (vert, gardé)

| Vague | Livré | Cas corpus |
|:--:|:--|:--|
| 1.1 | **Garde** : runner intégré `cargo test`, 4 tiers `.out/.compile/.err/.sha`, `_*` hors-garde | core/ (seed) |
| 1.2 | **Audit éventail** (7 agents, oracle honnête) → 57 verts initiaux + carte de défauts mesurée | 7 catégories |
| 2.1 | **Closures** : IIFE (A1), champ-closure `obj.f()` (A2), stockage `Box<dyn Fn>` (A3) | closures/ (8) |
| 3.1a | **Numérique** : `int/float` casts (B2/B3), `%` plancher (B4), index négatif (C1) | maths/ |
| 3.1b | **Générique+méthode** `impl<T>` (C3), destructuring en compréhension (D1), `?`-main (E1) | structs_enums/, collections/, errors/ |
| 3.1c | **Promotion** `i64→f64` (B1) | maths/mix_int_float |
| 3.1d | **Clone-fallback sound (last-use)** — corrige le dunder réutilisé (C2) **sans régresser V17** | structs_enums/money_dunders |
| 4.1 | **async/await** (déjà ok) + **`thread.scope(&data)`** (pré-emprunt des libres non-Copy) | concurrency/ (3) |

**12/12 défauts durs de l'audit corrigés** (A1-3·B1-4·C1-3·D1·E1). Garde finale : **70 cas verts**, 0 KO, 0 `unsafe`.

## Régressions captées EN ROUTE par la garde (et corrigées)
- `for` sur scalaire Copy → `&i64 < i64` (régression V17 latente, invisible sans `cargo test`) → `.iter().copied()`.
- `rem_euclid` (méthode) cassait l'inférence d'un param de lambda (`x % 2`) → forme par opérateurs `((a%b)+b)%b`.
- index négatif lvalue (`xs[i] += v`) : `len()` dans l'index chevauchait l'emprunt mutable → index hoisté.

## Écarts notés (non bloquants, documentés)
- **A3 (0.1%)** : littéral de **liste** de closures **capturantes** (`Vec<Box<dyn Fn>>`) non émis →
  contournement champ de struct / lambdas non-capturantes. Repro : `closures/_defects/captured_list.irn`.
- **thread.scope borné** : corps = mono-expression (compréhension de spawns) ; orchestration riche / **sortie
  mutable disjointe** (raytracer plein) → `rust{}`. Le bénéfice §2.6 (**lecture partagée `&data` sans clone**)
  est livré + démontré (`concurrency/scope_shared`).
- **§2.4/§2.5 (planify-3.1 d/e)** : propagation inter-appels / annotation de retour prioritaire — **non**
  remontés comme défauts par l'audit ; le raytracer garde ses annotations de params (il compile + tourne).
  Refinement d'inférence laissé ouvert (mesuré non nécessaire à la correction).
- **F — fidélité Python (❓ décision)** : `print(True)`→`true`, `print(1.0)`→`1`, `int("abc")`→`0`. Choix
  « projection Rust » (I6) documentés dans `gotchas.md` #13. À trancher : garder, ou Display custom Python.

## Mesure (métrique de vérité)
- **Corpus : 70/70** cas verts (`cargo test --test corpus`), couvrant les 4 axes V18 + le noyau.
- **Raytracer** (programme réel, non trivial) : compile, tourne, **PNG correct**, ~0,09 s, 7 clones (= V17).
- Gate historique `gate.sh` : 50/50, 8 clones, 0 `unsafe`.
