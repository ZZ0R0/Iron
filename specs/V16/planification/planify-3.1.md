# planify-3.1 — Désucrage des opérateurs en appels de méthode (chemin ownership unique)

- Réalise (cadrage) : [30500] §2.1
- Version : V16 · Vague : 3 · Piste : 1 · Statut : todo
- Dépend de : planify-2.1 (partage `lower.rs` — séquencé après)

## But
Faire passer les **opérateurs surchargés** par **exactement la même analyse ownership** que les appels de
méthode, en **désucrant** `a OP b` → `a.__op__(b)` **avant** l'analyse. Résout §2.1 : `a + b` sur un `Vec3`
**emprunté** ne compile pas aujourd'hui (`E0369: cannot add Vec3 to &Vec3`), alors que `a.add(b)` marche (autoref).

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/parse.rs` (ou une passe de désucrage AST) — transformation `Binary{op,lhs,rhs}` → `Call{__op__, [rhs]}`
  sur le receveur `lhs`, **pour les types non-primitifs** (les primitifs gardent les opérateurs natifs).
- `ironc/src/lower.rs` (retrait/neutralisation du chemin codegen opérateur pour ces types — un seul chemin reste).

## Interface produite / consommée
**Consommée** : la table des dunders (`__add__`/`__sub__`/`__mul__`/… ; cf. `tests/v11/dunders.irn`) et l'info de
type (`TypeMap`) pour décider primitif vs type-utilisateur/externe. **Produite** : `a OP b` (types non-primitifs)
= un `Call` de méthode → routé par l'analyse de 1.1 (autoref du receveur, emprunt des opérandes lus).

## Étapes
1. Désucrer `Binary` en `Call` de la méthode dunder **uniquement** quand le type du `lhs` n'est pas un primitif
   numérique/bool (garder `i64 + i64` natif). Mapper chaque `BinOp` → nom de dunder.
2. S'assurer que la méthode dunder existe (définie par l'utilisateur ou via un trait externe `Add`…) ; sinon,
   diagnostic clair (pas de désucrage vers une méthode inexistante).
3. Retirer le chemin codegen opérateur redondant dans `lower.rs` pour ces types (un seul chemin = l'appel de méthode).

## Definition of Done (vérifiable)
- Un `Vec3` **emprunté** : `a + b` compile (test dédié) ; idem `a - b`, `a * s` selon les dunders définis.
- Le raytracer peut **revenir aux opérateurs** (`a + b` au lieu de `a.add(b)`) et compiler — *facultatif* (le
  raytracer marche déjà avec les méthodes ; ce test prouve la friction levée).
- Corpus 80 vert (les opérateurs sur primitifs **inchangés** — vérifier l'arithmétique de base).

## Notes
- **Risque** : ne pas désucrer les opérateurs **primitifs** (arithmétique i64/f64) — sinon régression massive.
  Garde stricte sur le type du `lhs`.
- Non sur le chemin critique du « bon Rust généré » du raytracer (qui utilise déjà des méthodes) : c'est la
  friction d'**ergonomie** §2.1 qu'on lève.
