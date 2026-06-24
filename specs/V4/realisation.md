# Iron — V4 / réalisation (trace)

> Phase 3 terminée. **Gate V4 vert (2026-06-24).** Stdlib = prélude de built-ins. **Dernière version
> du découpage — V0→V4 complet.**

## Gate V4 — atteint

| | Preuve |
|---|---|
| Built-ins stdlib | `stdlib.irn` → `42 / 7 / 3 / 8 / 4 / 10 / hello world` |
| Corpus V0–V3 vert · zéro `unsafe` · tests | **19/19** · `forbid` · `cargo test` 3/3 |

## Livré
- Built-ins (lowering `call()` + inférence) : `str`, `abs`, `min`, `max`, `sqrt` (conv/math) ;
  `push`, `sum` (listes) ; `concat` (chaînes) — wrappers fins sur `std` (I5).
- **Bonus** : littéraux numériques **suffixés** (`i64`/`f64`) → fixe globalement l'ambiguïté `{integer}`/
  `{float}` sur les appels de méthode (a aussi restauré `use_math`/`use_gen`).

## Écarts & dettes
- **DZ1 — Built-ins = noms réservés** : un `sum`/`sqrt`/… built-in **masque** un extern `.iri` homonyme
  (`use_math`/`use_gen` passent désormais par le built-in `sqrt` — même résultat).
- **DZ2 — Monomorphe** : `sum` câblé `list[i64]→i64` ; `concat` str-only. (Génériques de stdlib = ultérieur.)
- **DZ3 — Prélude réduit** : pas d'IO (stdin), tri, `map`/`filter` (closures → ultérieur).

## Phase 4
Corpus 19/19 (non-régression V0–V3) + programme stdlib. Métrique de vérité (corpus large) toujours différée.
