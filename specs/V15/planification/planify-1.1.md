# planify-1.1 — Schéma figé du diagnostic de portée + ABI const-générique

- Réalise (cadrage) : [50500] (taxonomie & diagnostic), [50600] (C3 — ABI const)
- Version : V15 · Vague : 1 · Piste : 1 · Statut : todo
- Dépend de : — (leaf : fige le contrat consommé par toutes les vagues suivantes)

## But
Geler, **avant toute logique**, le **schéma des données échangées** par le système de binding à portée connue :
le type des **raisons de non-binding**, le **diagnostic** par item, le compteur de **couverture**, et
l'**argument const** de `Type::ExternApp`. Aucune behavior — uniquement les types + leur sérialisation. C'est
la pièce « fondation fige le schéma » de la méthodo : si ce contrat bouge après le fan-out, deux pistes le
comblent différemment → intégration cassée.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/ir.rs` (ajouts de types uniquement).

## Interface produite / consommée
**Produite** (consommée par 2.1, 3.1, et toutes les capacités) :
```rust
// Taxonomie FERMÉE — toute nouvelle cause d'abandon DOIT s'ajouter ici (contrat de complétude).
pub enum SkipReason {
    UnmappedType(String),            // type de param/retour sans projection Iron
    GenericTraitMethod(String),      // signature dépend d'une borne de trait non-projetée
    ConstGenericShape,               // valeur-au-niveau-type (Const<N>, typenum)
    UnexpandableAlias(String),
    BorrowedReturnNonIdiom(String),  // retour emprunté hors idiome (ni Display ni clone)
    StreamBuilderApi,
    MacroOrHidden,
    ReexportSourceUndocumented(String),
    OrphanCuration,                  // entrée curée sans homologue auto (drift de version)
}
pub struct SkipDiagnostic { pub rust_path: String, pub item_kind: ItemKind, pub reason: SkipReason, pub hint: String }
pub enum ItemKind { Fn, Method, Ctor, Type, Field, Variant }
pub struct Coverage { pub seen: usize, pub bound: usize, pub skipped: usize } // invariant: seen == bound + skipped
```
**Étendre `Type::ExternApp`** pour porter des **arguments const** en plus des arguments de type :
```rust
// V12 : ExternApp(String, Vec<Type>) — chemin + args de type.
// V15 : porter aussi des args CONST (Const<N>, littéraux) pour C3.
//       choix d'encodage figé ici (ex. enum ExternArg { Ty(Type), Const(String) }) — à trancher dans l'étape.
```

## Étapes
1. Ajouter `SkipReason`/`SkipDiagnostic`/`ItemKind`/`Coverage` dans `ir.rs`, `#[derive(Debug, Clone)]`.
2. Trancher l'encodage const de `ExternApp` (variante additive : `enum ExternArg { Ty(Type), Const(String) }`,
   `ExternApp(String, Vec<ExternArg>)`) — **additif**, ne casse pas le lowering V12 (un `Vec<Type>` migre en
   `Vec<ExternArg::Ty>`). Mettre à jour les `match` exhaustifs strictement nécessaires (lsp.rs display arm).
3. Aucune autre logique. `cargo build` vert.

## Definition of Done (vérifiable)
- `cargo build -p ironc` vert.
- Les types compilent et sont `pub` (importables par `bind`/`lower`).
- Test unitaire : un `Coverage { seen: 5, bound: 3, skipped: 2 }` satisfait `seen == bound + skipped`
  (sanity du contrat, pas encore branché).

## Notes
- **Ne pas** brancher ces types dans `bind.rs` ici (c'est 2.1) — on ne fait QUE figer le schéma.
- L'encodage const de `ExternApp` doit rester **additif** (I9 pipeline figé : on n'altère pas le lowering
  existant, on l'étend). Le lowering const réel = 4.2 (C3).
