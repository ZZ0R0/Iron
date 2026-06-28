# planify-8.1 — Émission de **derives** (`#[derive(X)]`) sur un type Iron

- Réalise (cadrage) : [40400], [44000]
- Version : V12 · Vague : 8 · Piste : 1 · Statut : todo
- Dépend de : planify-3.1 (le typage est en place)

## But
Faire émettre par Iron un `#[derive(Serialize/Deserialize/Parser/…)]` sur un struct/enum Iron (RACINE 2). Iron a
déjà le pass-through (`item_attrs`/`@derive`) — en faire un mécanisme **piloté par les bindings** (le `.iri`
déclare le trait dérivable). Pré-requis de serde typé (8.2), pest, clap-derive.

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition** : `ironc/src/lower.rs` (émission des attributs derive), `ironc/src/ir.rs` (si `item_attrs` doit
  porter la provenance), `ironc/src/bind.rs` (marquer un trait externe comme **dérivable**).
- Séquentielle (vague 8).

## Interface produite / consommée
- **Consomme** : bindings (`etypes` du trait dérivable).
- **Produit** : `#[derive(Deserialize)]` émis sur un type Iron → type éligible aux API `T: Deserialize`.

## Étapes
1. `.iri`/binder : reconnaître les traits **dérivables** courants (serde, clap::Parser…) et le signaler.
2. Surface Iron : `@derive(Deserialize)` (déjà parsé) → émission propre, chemins complets (`serde::Deserialize`).
3. Émettre les attributs requis (`#[serde(...)]` minimal si besoin).

## Definition of Done (vérifiable)
1. Un struct Iron annoté émet `#[derive(serde::Deserialize)]` et **compile** avec la bonne dep/feature.
2. **Gate VERT** ; codegen via attrs/`TokenStream` (I10), 0 `unsafe`.

## Notes
- L'**usage** `from_str::<T>` (param `T: Deserialize`) = RACINE 1 (planify-8.2). Ici on rend le type **éligible**.
- Macros procédurales arbitraires (hors derives nommés) = hors périmètre, à cadrer si forcé.
