# planify-1.1 — Schéma figé `OwnershipPlan` / `Disp`

- Réalise (cadrage) : [30800]
- Version : V17 · Vague : 1 · Piste : 1 · Statut : todo
- Dépend de : — (leaf : fige le contrat consommé par 2.1/3.x)

## But
Geler le **schéma de la sortie ownership** avant toute logique : `Disp` (disposition par site), l'extension de
`ParamMode` (`ByRefMut`), et la struct `OwnershipPlan`. Aucune behavior — juste les types + leur place dans `ir.rs`.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/ir.rs` (ajouts de types).

## Interface produite / consommée
**Produite** (consommée par 2.1, 3.1, 40500) :
```rust
pub enum Disp { Borrow, BorrowMut, Move, Clone }     // disposition par site d'usage
// ParamMode existant étendu : ByRef | ByRefMut | ByValue (ajout ByRefMut)
pub struct OwnershipPlan {
    pub sites: HashMap<NodeId, Disp>,                 // par site de lecture de valeur
    pub params: HashMap<String, Vec<ParamMode>>,      // par (fn libre nom | méthode "Type::m"), subsume ParamModes
}
```

## Étapes
1. Ajouter `Disp` (`#[derive(Debug, Clone, Copy, PartialEq, Eq)]`).
2. Étendre `ParamMode` avec `ByRefMut` (mapping lowering `&mut` ; `InOut` reste pour `inout` explicite, ou
   fusion à trancher — décision figée ici : garder `InOut` distinct, `ByRefMut` = mutation inférée).
3. Ajouter `OwnershipPlan`. **Aucune** logique ; `cargo build` vert.

## Definition of Done (vérifiable)
- `cargo build -p ironc` vert. Types `pub`, importables par `resolve`/`lower`.

## Notes
- Ne PAS brancher dans resolve/lower ici (c'est 2.1). On fige le **schéma des données échangées** (la fondation
  de parallélisme : sans ça, 3.1 et 3.2 le combleraient différemment).
