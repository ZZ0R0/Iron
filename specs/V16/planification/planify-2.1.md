# planify-2.1 — For-élément emprunté + dernière-utilisation réelle (move/clone)

- Réalise (cadrage) : [30500] (corollaire §1 + §2.2)
- Version : V16 · Vague : 2 · Piste : 1 · Statut : todo
- Dépend de : planify-1.1 (le point fixe des modes existe)

## But
Deux compléments à l'analyse ownership : (a) **emprunter l'élément** d'une boucle `for` quand le corps ne le
consomme pas (`for obj in &xs` au lieu de `.iter().cloned()`) ; (b) **vraie analyse de dernière-utilisation**
voyant la réutilisation **intra-expression** (remplace l'heuristique single-use) → supprime le `.copy()` manuel.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/resolve.rs` (last-use réel + détection read-only de la variable de boucle).
- `ironc/src/lower.rs` (émission `for x in &xs` vs `.iter().cloned()` ; `for_iter` cas par défaut).

## Interface produite / consommée
**Consommée** : modes/call-graph de 1.1. **Produite** : `OwnershipMap` enrichie (move prouvé dernière-utilisation,
y compris ordre d'évaluation intra-expression) ; signal « variable de boucle empruntée » consommé par `for_iter`.

## Étapes
1. **Last-use** : remplacer `compute_moves` (single-use au niveau statement, `resolve.rs:49-63`) par une analyse
   de **dernière lecture par flot**, ordre d'évaluation **intra-expression** inclus (appels imbriqués). Move =
   prouvé dernier usage ; sinon emprunt (lecture seule) ou clone (consommé ≥2 fois). Conservateur : doute → clone.
2. **For-élément** : si le corps de la boucle ne **consomme** pas la variable (lecture seule : base de champ/
   index/méthode `&self`/comparaison), marquer l'élément **emprunté** → `lower.rs` `for_iter` émet `&xs` (au lieu
   de `#it.iter().cloned()`). Sinon, comportement actuel (cloned).

## Definition of Done (vérifiable)
- **Raytracer (emit)** : `for obj in &self.objects` (×2 : `hit`/`occluded`) ; le `.copy()` de `vec3.irn`
  (`reflect`/`refract`) devient **inutile** (`v.dot(n.scale(2.0*v.dot(n)))` sans `.copy()` compile).
- Corpus 80 vert. `cargo test -p ironc` vert. 0 `unsafe`.

## Notes
- Séquentiel après 1.1 (même `resolve.rs`). Parallèle avec 2.2/2.3 (fichiers disjoints).
- Conservatisme : un élément de boucle dont l'usage est ambigu reste **cloné** (sûr).
