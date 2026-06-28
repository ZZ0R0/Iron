# V11-test-2 — Réalisation (try & run, un par un)

> Journal de réalisation des 80 projets. **Vague 0 = durcir le pipeline** (séquentiel) ; puis fan-out
> par risque. Statuts : ✅ vert · 🟡 partiel (bug noté) · 🔴 bloqué (mur) · ☐ à faire.

## Vague 0 — durcissement du pipeline `ironc add` + `import` (révélé par les pilotes)

| # | Bug | État |
|---|-----|:--:|
| B1 | Méthode nommée comme un **mot-clé Iron** (`not`/`and`/`in`…) → parse error. Sautée par le binder. | ✅ corrigé |
| B2 | **Nom de crate** hyphen/underscore : directive `.iri` portait le nom import (underscore, rejeté par cargo) → porte le nom publié ; parseur `.iri` accepte les tirets. | ✅ corrigé |
| B3 | **Résolution `.iri`** : `import` cherchait à côté du source ; `ironc add` écrit au cwd/racine. → cherche base_dir → racine → cwd (chemin absolu). | ✅ corrigé |
| B4 | **Chemin de méthode de trait** : `impl Trait for T` préfixé `{crate}::Trait` (faux pour std) → résolu via `paths[id]` rustdoc (`core::ops::Mul::mul`). | ✅ corrigé |
| B5 | **Constructeur associé à un trait** (`One::one`, `FromPrimitive::from_u64`) pathé `crate::Type::one` (faux). Spécifique num-traits. Nécessite UFCS `<T as Trait>::f()` avec le type. | ☐ à corriger quand bloquant |

## Projets réalisés

| # | Projet | Crate | Statut | Note |
|---|--------|-------|:--:|------|
| 60 | Détection de langue | `whatlang` | ✅ **vert** | `detect(t).unwrap().lang().eng_name()` → French/English. Crate **témoin** du pipeline durci. |
| 53 | Grands entiers | `num-bigint` | 🟡 partiel | binder OK (198 items) ; `.mul()`/`Display` OK après B4 ; mais ctors `one()`/`from_u64()` = trait-associés → B5. Crate num-traits = pire cas. |

## Constat & cadence (honnête)

- Le binder est **solide** (whatlang, num-bigint : 8/198 items bindés tout seuls). Le pipeline est
  nettement plus robuste après Vague 0.
- **« Tous les 80 un par un » est un grind séquentiel long** : chaque crate peut révéler un nouveau bug de
  binder (à corriger dans `ironc/src`, donc **pas parallélisable** — un agent ne peut pas éditer le compilo
  pendant qu'un autre l'édite). Réaliste : **plusieurs sessions**, pas une.
- **Plan** : continuer les 🟢 « feuilles » (fonctions/méthodes inhérentes : `flate2`, `csv`, `qrcode`,
  `chrono`…) pour verrouiller la couche commune ; corriger B5 quand un crate utile le réclame ; garder les
  🔴 (GUI/jeux/async/ML) et l'**ESP32/no_std** (chantier V12) pour la fin.
- À chaque crate : `ironc add` → mini-programme → `ironc run` → noter ✅/🟡/🔴 + le bug s'il y en a un.
