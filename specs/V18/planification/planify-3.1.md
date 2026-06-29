# planify-3.1 — Ergonomie : `?`-main, stdlib Python, propagation (§2.4), annotation de retour (§2.5)

- Réalise (cadrage) : [21800]
- Version : V18 · Vague : 3 · Piste : 1 · Statut : todo
- Dépend de : planify-1.1/1.2 (garde) ; partage `infer`/`lower` avec 2.1 → après 2.1

## But
Lever les gotchas du quotidien (4 points) pour qu'on écrive du Iron **naturel** sans contournement.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/infer/` (§2.4 propagation inter-appels + membre non-appelé ; §2.5 annotation de retour / flux).
- `ironc/src/parse.rs` / `ironc/src/lower.rs` (`?`-dans-`main` ; `int(f64)`→cast ; `f64 ** n`→`powi/powf`).

## Interface produite / consommée
**Consommée** : moteur I15 + flux (`infer/engine/flux.rs`) + le graphe d'appels. **Produite** : `main` auto-promu
`Result` si `?` ; types propagés inter-appels ; annotation de retour prioritaire ; `int`/`**` sémantique Python.

## Étapes
1. **(a) `?`-dans-`main`** : `main` sans annotation contenant `?`/`raise` → émis `-> Result<(), IronError>` + `Ok(())`.
2. **(c) stdlib** : `int(x)` sur `f64` → `as i64` (pas le parseur) ; `x ** n` sur `f64` → `powi`/`powf`.
3. **(d) §2.4** : propager un type concret le long de la chaîne d'appels (point fixe) ; membre non-appelé →
   inférer du corps ou **diagnostic** (plus de `__T<n>` vers `rustc`).
4. **(e) §2.5** : annotation de retour **prioritaire** ; prouver l'exhaustivité du `return` ou **diagnostic**
   (jamais de réécriture en `Option`).

## Definition of Done (vérifiable, via [70400])
- `main` avec `?` compile sans annotation ; `int(3.9)`→3 ; `2.0**5`→32.0.
- **Le raytracer compile SANS** ses annotations de contournement (`cross/rec/scene/r`) — ou diagnostic clair (d).
- `random_in_unit_sphere() -> Vec3` reste `Vec3`. Corpus [70400] vert (Option/Result légitime **préservé**).

## Notes
- Séquentiel après 2.1 (partage `infer`/`lower`). (d)/(e) = cœur d'inférence (prudent) ; (a)/(c) = petits.
