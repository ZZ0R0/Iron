# planify-6.1 — Stratégie **closure** : `Fn`-borné → lambda Iron

- Réalise (cadrage) : [40300]
- Version : V12 · Vague : 6 · Piste : 1 · Statut : todo
- Dépend de : planify-3.1 (pont) ; bénéficie de 5.1 si la closure prend une source

## But
Matérialiser une borne `F: Fn(A)->B`/`FnMut`/`FnOnce` par une **lambda Iron** (RACINE 1). Iron a déjà `lambda`
et `Idiom::Closure(idx)` — il faut en faire une **stratégie** du pont (types des params/retour issus de la borne).
Débloque rusqlite(`query_row(|row| …)`), dialoguer(validators), `sort_by`, callbacks de crates.

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition** : `ironc/src/bridge.rs` (entrée `STRATEGIES` pour la famille `Fn`), `ironc/src/lower.rs` (émission
  `|args| { … }` avec coercition des params/retour), `ironc/src/infer/` (typer les params de la lambda depuis la borne).
- Séquentielle (vague 6).

## Interface produite / consommée
- **Consomme** : `bridge::{accepts, materialize}` + (1.1) ; la substitution de génériques (2.3) pour `A`/`B` génériques.
- **Produit** : appels prenant une closure (`q.query_row(params, lambda)`), types de la lambda inférés.

## Étapes
1. `accepts(Fn(A)->B) = Type::Fn([A],B)` ; le param Iron est une lambda.
2. `materialize` : émettre la closure Rust, **coercer** ses params (entrée Rust→Iron) et son retour (Iron→Rust) via le pont.
3. infer : donner aux params de la lambda le type tiré de la borne (`row: &Row` → `Extern("Row")`).

## Definition of Done (vérifiable)
1. rusqlite : `query_row(sql, params, |row| row.get(0))` **lit** une ligne (run réel) — débloque la **lecture** (le partiel noté en passe 80).
2. un `xs.sort_by(|a,b| …)` ou un validator dialoguer compile et tourne.
3. **Gate VERT** ; 0 `unsafe`.

## Notes
- Closure **rendant un trait-objet** (`Box<dyn App>`) = cas avancé : si nécessaire pour event-loop, traité en 12.1 (RACINE 3).
- `row.get::<T>(0)` générique : la substitution (2.3) fournit `T` depuis le contexte.
