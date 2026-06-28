# planify-2.1 — Projection : produire les génériques + tables pilotées-donnée (bind.rs)

- Réalise (cadrage) : [24000], [24100]
- Version : V12 · Vague : 2 · Piste : 1 · Statut : todo
- Dépend de : planify-1.1

## But
Faire **produire** au binder les arguments génériques (au lieu de jeter `T`) et migrer la projection vers les
tables-données du pont. C'est la face « Rust → Iron » de [24100].

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition exclusive** : `ironc/src/bind.rs` (`map_type`, `map_named`, `map_prim`, `project`, `bound_to_iron`).
- **Lecture seule** : `ironc/src/bridge.rs`, `ironc/src/ir.rs`.
- **N'édite PAS** `lower.rs` (piste 2.2) ni `infer/` (piste 2.3) → aucune collision.

## Interface produite / consommée
- **Produit** : des `.iri` où un type externe générique est `Name[args]`, les params déclarés `extern type
  Name[T] = …`, et les `T` de méthodes **préservés** (`Type::Generic("T")`, plus de drop). C'est l'entrée que
  2.3 (infer) substitue et 2.2 (lower) monomorphise.
- **Consomme** : `bridge::{project, BOUND_TO_IRON, CONTAINERS}`, `Type::ExternApp`, `ExternType.params` (de 1.1).

## Étapes
1. Branche PascalCase de `map_named`/`map_type` : `Point<f64>` → `ExternApp("geo::Point",[F64])` (ne plus
   renvoyer le nom nu sans args).
2. Un paramètre `T` en arg/retour de méthode → conservé `Generic("T")` (porté dans le `.iri`).
3. `project` peuple `ExternType.params` (ex. `["T"]`) et émet l'en-tête `extern type Point[T] = "geo::Point":`.
4. **Bornes : suivre, pas whitelister.** `bound_to_iron` devient une **lecture de table** (`bridge::BOUND_TO_IRON`,
   garde le cas `AsRef<Path>`→`str`). Le **fallback** d'un `T` non reconnu n'est plus « drop la fn » → « garder
   `Generic("T")` », résolu à l'appel (⇒ `Point::new`, `Series::new`, FromSql… cessent d'être droppés).
5. `map_prim`/conteneurs → consultent `bridge` (R3) ; supprimer les `match name { "Vec" => … }` dupliqués.

## Definition of Done (vérifiable)
1. `ironc add geo` régénéré : `extern type Point[T] = "geo::Point":`, `Point.new(x: T, y: T) -> Point[T]`,
   `euclidean_distance(self, other: Point[T]) -> T` (params présents).
2. `ironc add nalgebra`/`ndarray` : ctors scalaires-génériques **apparaissent** (non droppés).
3. `grep` : plus de table de conteneurs/primitifs dupliquée dans `bind.rs` (une seule def dans `bridge`).
4. **Gate VERT** (les `.iri` commités du gate ne sont pas régénérés). Diff `.iri` d'un crate non générique = vide.

## Notes
- Ne PAS toucher la coercition ni l'inférence ici — elles appartiennent à 2.2/2.3. Tout écart se **note**, pas se code.
