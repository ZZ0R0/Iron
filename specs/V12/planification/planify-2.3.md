# planify-2.3 — Inférence : substitution des paramètres de type (infer/)

- Réalise (cadrage) : [24000], [24100]
- Version : V12 · Vague : 2 · Piste : 3 · Statut : todo
- Dépend de : planify-1.1

## But
Faire **circuler et instancier** les paramètres de type à travers les chaînes de méthodes externes : une méthode
de `Point[f64]` rendant `T` doit être typée `f64`. C'est le cœur « inférence » de [24100].

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition exclusive** : `ironc/src/infer/local.rs`, `ironc/src/infer/mod.rs` (et `engine`/`flux` si la
  substitution doit passer par le moteur de contraintes).
- **Lecture seule** : `ironc/src/bridge.rs`, `ironc/src/ir.rs`.
- **N'édite PAS** `bind.rs` (2.1) ni `lower.rs` (2.2).

## Interface produite / consommée
- **Produit** : les types **instanciés** (`ExternApp` concret) consommés par `lower` (2.2) à l'unification.
- **Consomme** : `ExternType.params` (de 1.1) et les `.iri` à params (produits par 2.1).

## Étapes
1. **Substitution `param → arg`.** Récepteur `ExternApp(path,[f64])`, méthode (params `["T"]`) rendant
   `Generic("T")` → renvoyer `f64`. Construire la table de subst depuis `ExternType.params` × args de l'`ExternApp`.
   Toucher `extern_method_ret` (`infer/mod.rs`) + le routage d'appel (`infer/local.rs`).
2. **Instanciation par l'argument (ctors).** `Point.new(2.0, 3.0)` → unifier args↔params ⇒ `T=f64` ⇒ résultat
   `ExternApp("geo::Point",[F64])`. C'est ce qui rend la whitelist de bornes inutile pour ce cas.
3. **Propager dans les chaînes.** `builtin_method_ret`/`extern_iter_elem` consultent la même substitution (un
   itérateur `Vec[T]` instancié rend le bon `T` — généralise B30 au cas générique).
4. **Invariant D2** : aucun `Var`/`Generic` non résolu n'atteint `lower` (defaulting documenté ou erreur native).

## Definition of Done (vérifiable)
1. Test d'inférence ciblé : une méthode rendant `T` sur `Foo[i32]` est typée `i32`.
2. `Point.new(2.0,3.0)` infère `T=f64` (résultat `ExternApp` concret).
3. **Invariant D2** vérifié (assert/diagnostic : pas de `Generic` résiduel en entrée de `lower`).
4. **Gate VERT** ; pipeline I9 intact.

## Notes
- Ne pas émettre de Rust ici (c'est 2.2) ni produire des `.iri` (c'est 2.1). Cette piste **type**, point.
