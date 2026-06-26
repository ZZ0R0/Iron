# V7 — Réalisation (état honnête avant batterie de tests)

> Iron est passé de « transpileur qui marche » à **clone Python en syntaxe** (sémantique Rust dessous).
> Gate **vert en continu** (44/44 corpus, 0 `unsafe`, 3 clones intentionnels), ~30 commits, + 3 sweeps de
> robustesse (20 programmes Python variés). Trace pour la phase 4.

## ✅ FAIT et prouvé

**Langage cœur (chantier A)** : tuples + destructuring + retour multiple · f-strings · compréhensions
liste/dict/set · affectation augmentée (`+= ** //`) · ternaire · `in`/`not in` · `assert` · slicing ·
zoo numérique complet + `char` + casts `to_<t>` · type `set`.

**Surface Python (chantier P)** : `def` · `class` (champs + méthodes) · méthodes + `self` mutant (champ
ET méthode mutante) · arguments **par défaut** (fn + méthodes) · arguments **nommés** (struct + fn
tout-nommé) · `True`/`False`/`None` · chaînes `'…'` **et** `"…"` · concaténation `s1 + s2` ·
`try`/`except`/`raise` · `for a, b in …` (destructuring de boucle) · HOF (param fonction appelé).

**Stdlib (chantier C)** : `io` (fichiers, `std::fs`) · méthodes built-in str (upper/lower/strip/split/
replace/startswith/endswith/contains) / list (append/pop/sort/reverse/contains/len) / dict (keys/values/
items/get/contains) · itertools (enumerate/zip/sorted/reversed/any/all/reduce) · `os`/`time` ·
join/int/float/round/min-max-sur-liste · combinateurs Option (`.map`/`.unwrap_or`/`.is_some`).

**Tooling (chantier E)** : `ironc new` (scaffold) · `ironc test` (`def test_*()` + assert → `cargo test`).

**Robustesse (3 sweeps, bugs corrigés)** : inférence des **collections vides** depuis `d[k]=v` (pattern
`counts = {}`) · méthode mutante sur `self` · param fonction HOF · concaténation de chaînes.

## ⚠️ Limites documentées (workaround trivial — la batterie peut les éviter)

- **Arguments nommés sur méthode / appel mixte** (`c.m(k=v)`, `sorted(xs, key=f)`) → **erreur claire** ;
  utiliser le **positionnel** (`c.m(v)`, `sorted(xs)`). (Refacto `Call.kwargs` → V8.)
- **`return <valeur>` dans un `try`** → utiliser une **variable-résultat** assignée. (closure-IIFE.)
- **HOF** : param fonction supposé `i64→i64` (heuristique) ; autres signatures → annoter / lambda inline.

## ⏸️ Différé par conception (rationnel honnête)

- **Chantier B — crates réelles / JSON** : nécessite le binding via **nightly `rustdoc-json`** (format
  instable) + résolution de deps. Non fait : dépend d'un toolchain externe non garanti ici.
- **Chantier D — lint réel** (au-delà du comptage de clones `--audit`).
- **`ironc fmt`** (formateur) ; **source-maps granularité expression** (60200, mécanisme 🔶-recherche).
- **Variantes enum à champs nommés** (2.8) ; **opérateurs bit-à-bit** (`& | ^ << >>`) — niche en code Python.
- **for-var post-boucle** (D7b, `p_forvar.irn`) ; **escape point-fixe** (DV2).

## Verdict

Le **langage + la stdlib sont prêts pour une batterie de tests costaude** sur des programmes Python
idiomatiques (classes, méthodes, compréhensions, f-strings, try/except, fichiers, dict/list/str…). Écrire
les tests en `def test_*()` + `assert`, lancer `ironc test`. Éviter les 3 limites ci-dessus (workarounds
documentés). Les chantiers différés (crates/JSON, lint, fmt) sont des **outils**, pas du langage cœur.
