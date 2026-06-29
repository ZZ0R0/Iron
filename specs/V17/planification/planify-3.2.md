# planify-3.2 — Exploitation : désucrage des opérateurs (opérandes via le plan)

- Réalise (cadrage) : [40500] (+ [30800] §opérateurs)
- Version : V17 · Vague : 3 · Piste : 2 · Statut : todo
- Dépend de : planify-2.1 (`emit_use` en place) — **‖ planify-3.1** (fichiers disjoints)

## But
Faire passer les **opérateurs surchargés** par le même chemin ownership que les méthodes, en **désucrant**
`a OP b` → `a.__op__(b)` **avant** l'analyse. Leurs opérandes deviennent alors des sites d'usage normaux,
décidés par le moteur et émis par `emit_use` (autoref cohérent). Résout la friction `a + b` sur `Vec3` emprunté.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/parse.rs` (ou une passe de désucrage AST) — `Binary{op,lhs,rhs}` → `Call{__op__, recv=lhs, [rhs]}`
  pour les types **non-primitifs**. Disjoint de `resolve.rs` (3.1).

## Interface produite / consommée
**Consommée** : la table des dunders (`__add__`/`__sub__`/… ; `tests/v11/dunders.irn`) ; le type du `lhs`
(`TypeMap`) pour décider primitif vs type-utilisateur/externe. **Produite** : `a OP b` (non-primitif) = un `Call`
de méthode → opérandes décidés par le moteur, émis par `emit_use`.

## Étapes
1. Désucrer `Binary` en `Call` du dunder **uniquement** si le type du `lhs` n'est pas un primitif numérique/bool
   (garder `i64+i64` natif). Mapper chaque `BinOp` → nom de dunder.
2. Si le dunder n'existe pas (ni défini ni trait externe `Add`…) → ne pas désucrer / diagnostic clair.

## Definition of Done (vérifiable)
- `a + b` sur `Vec3` **emprunté** compile (test dédié) ; idem `-`/`*` selon les dunders définis.
- Le raytracer peut **revenir aux opérateurs** (`a + b`) et compiler (facultatif — il marche déjà en méthodes).
- **Corpus 80 vert** — l'arithmétique des **primitifs** est INCHANGÉE (garde stricte sur le type du `lhs`).

## Notes
- **Risque** : ne jamais désucrer les opérateurs primitifs (régression massive). Garde stricte.
- Parallèle avec 3.1 (resolve) — fichiers disjoints. Dépend de 2.1 (`emit_use`).
