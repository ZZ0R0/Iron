# planify-2.2 — Coercition : numérique unifié + monomorphisation (lower.rs)

- Réalise (cadrage) : [24000], [24100]
- Version : V12 · Vague : 2 · Piste : 2 · Statut : todo
- Dépend de : planify-1.1

## But
Unifier la dette numérique éparpillée en **un** point, et **monomorphiser** les types externes génériques. Face
« Iron → Rust » de [24100].

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition exclusive** : `ironc/src/lower.rs` (`coerce_to`, `extern_arg`, `extern_arg_to`, `coerce_sized_binop`,
  `try_extern_ctor`/`try_extern_method` pour le routage `ExternApp`, bras `lower_type` numériques/ExternApp ;
  `num_kind_str` supprimé au profit de `bridge::rust_num_name`).
- **Lecture seule** : `ironc/src/bridge.rs`, `ironc/src/ir.rs`.
- **N'édite PAS** `bind.rs` (2.1) ni `infer/` (2.3).

## Interface produite / consommée
- **Produit** : Rust final monomorphisé (`geo::Point::<f64>::new(…)`) + coercition table-pilotée.
- **Consomme** : `bridge::{coerce_arg, cast_numeric, rust_num_name, BYTES, CONTAINERS}`, `Type::ExternApp`, et
  (à l'unification, vague 3) les types instanciés fournis par 2.3.

## Étapes
1. **R1 — un seul cast numérique.** `coerce_to`, le volet numérique d'`extern_arg_to`, et `coerce_sized_binop`
   délèguent à `bridge::cast_numeric(expr, from, to)`. `num_kind_str` → `bridge::rust_num_name`.
2. **Monomorphiser `ExternApp`.** Router un récepteur/ctor `ExternApp(path, args)` comme `Extern` (path dans
   `etypes`) en propageant l'instanciation pour les retours `T` (le type vient de 2.3 à l'unification).
3. **Coercition d'args table-pilotée.** `extern_arg`/`extern_arg_to` (non-numérique : `&`/`.as_ref()`/octets)
   lisent `bridge::{BYTES, CONTAINERS}` au lieu de `match Type::…` en dur (B22/B27/B29 deviennent des entrées).

## Definition of Done (vérifiable)
1. **Refactor numérique prouvé NEUTRE** : sur le corpus, le Rust généré des cas numériques (B10/B22/B26/B29) est
   **byte-identique** avant/après (diff vide) — on dé-duplique, on ne change pas la sémantique.
2. `grep` : plus aucun `num_kind_str` ni `match (Type::Num…, Type::I64)` hors `bridge`.
3. `geo` : `Point.new(2.0, 3.0)` émet un `geo::Point::<f64>::new(…)` qui **compile** (`cargo` vert).
4. **Gate VERT** ; 0 `unsafe` (I2) ; codegen `TokenStream` (I10).

## Notes
- L'étape 1 est un refactor **pur** (sortie identique) : c'est la dé-duplication qui paie la dette, pas un changement de comportement.
- Le type instancié des retours `T` n'est pleinement disponible qu'après greffe de 2.3 → finaliser le routage `ExternApp` à l'**unification** (vague 3).
