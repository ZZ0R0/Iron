# planify-3.1 — Exploitation : modes de méthode + last-use réel (dans le moteur)

- Réalise (cadrage) : [30800]
- Version : V17 · Vague : 3 · Piste : 1 · Statut : todo
- Dépend de : planify-2.1 (socle neutre en place) — **‖ planify-3.2** (fichiers disjoints)

## But
**Améliorer les décisions du moteur** (pas le `emit_use`, qui est figé et suit le plan) : (a) **modes de
méthode** — une méthode dont un param est seulement lu → `ByRef` (`hit(&self, r: &Ray)`) ; (b) **last-use réel**
— dernière-utilisation par flot, **ordre d'évaluation intra-expression** inclus (remplace l'heuristique
single-use). `emit_use` traduit automatiquement → plus de `r.clone()` par objet, plus de `rec.clone()`, plus de
`.copy()` manuel.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/resolve.rs` (les **décisions** du moteur ; `emit_use` de `lower` est inchangé). Disjoint de `parse.rs` (3.2).

## Interface produite / consommée
**Consommée** : le moteur/plan de 2.1. **Produite** : `plan.params["Type::m"]` exploité (méthodes en `ByRef`) ;
`plan.sites` avec des `Move` là où le single-use ratait (réutilisation intra-expression → la 1ʳᵉ lecture
`Borrow`, la dernière `Move`).

## Étapes
1. **Méthodes** : activer la consommation des modes de méthode (V16 les calcule déjà dans `params`) — le socle
   2.1 les gardait « par valeur » pour la neutralité ; ici on **lève le frein** : `emit_use` lit `plan.params`
   pour les méthodes comme pour les fns libres. Vérifier que les signatures de méthode émettent `&` (lower_method)
   et que les corps gèrent les params empruntés (le `byref` est désormais dans le plan, plus en champ ad-hoc).
2. **Last-use** : remplacer le comptage single-use par une analyse de **dernière lecture** tenant compte de
   l'ordre d'évaluation des sous-expressions. `Move` = prouvé dernier usage ; sinon `Borrow` (lu) ou `Clone`
   (consommé ≥2 fois). Conservateur : doute → `Clone`.

## Definition of Done (vérifiable)
- **Raytracer** : `fn hit(&self, r: &Ray, …)` ; `obj.hit(r)` sans `.clone()` ; `nearest = rec` (move, sans
  `.clone()`) ; `v.dot(n.scale(2.0*v.dot(n)))` compile **sans `.copy()`**.
- **Corpus 80 vert** (les méthodes du corpus passent en `&` sans casser) ; `cargo test` vert ; 0 `unsafe`.

## Notes
- Parallèle avec 3.2 (parse) — fichiers disjoints. Séquentiel après 2.1 (même `resolve.rs`).
- Conservatisme strict : une méthode dont le param s'échappe reste `ByValue` (le point fixe le gère).
