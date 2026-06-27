# V10 — Planification (vagues & pistes)

> Découpage pour lancer **un max d'agents en parallèle, sans collision**. Axes : **VAGUE** (séquentielle,
> dans l'ordre) × **PISTE** (parallèle au sein d'une vague). La **fondation EST la vague 1**. Entre deux
> vagues : unification (greffe + revalidation e2e RÉEL). 10 pistes, 1:1 ou n:1 avec les 7 specs.

## Carte des vagues

| Vague | But | Pistes | Parallèle ? |
|---|---|---|---|
| **1 — Fondation** | Geler le SCHÉMA partagé + la structure du module `engine/` | 1.1 | séquentiel (1 piste) |
| **2 — Fan-out moteur** | Remplir gen / solve / assume contre le contrat figé | 2.1 · 2.2 · 2.3 | **3 en parallèle** |
| **3 — Fan-out clôture** | Defaulting/généralisation + diagnostics Iron | 3.1 · 3.2 | **2 en parallèle** |
| **4 — Migration** | Basculer le pipeline sur le moteur, retirer l'ancien | 4.1 → 4.2 → 4.3 | séquentiel (collision `infer/mod.rs`) |
| **5 — Gate V10** | Corpus zéro-annotation + audit + verdict | 5.1 | séquentiel (1 piste) |

## Piste ↔ spec ↔ fichier possédé (ownership disjoint)

| Piste | Spec | Fichier(s) **possédé(s)** | Interface produite / consommée |
|---|---|---|---|
| 1.1 | 21700 | `ir.rs` (variante `Type::Var`) · **crée** `infer/engine/{mod,subst,constraint,gen,solve,assume,generalize,diag}.rs` | **produit** : `Type::Var`, `Subst`, `Constraint`, `TraitBound`, signature `infer_types()` |
| 2.1 | 21800 | `infer/engine/gen.rs` | consomme contrat 1.1 · **produit** `Vec<Constraint>` + TypeMap provisoire |
| 2.2 | 21900 | `infer/engine/solve.rs` | consomme `Constraint` · **produit** `Subst` / `TypeError` |
| 2.3 | 22000 | `infer/engine/assume.rs` | consomme `Bindings` · **produit** table d'assomptions + instances |
| 3.1 | 22100 | `infer/engine/generalize.rs` | consomme `Subst` · **produit** defaulting + schémas `[T]` |
| 3.2 | 22200 | `infer/engine/diag.rs` | consomme `TypeError` · **produit** `CompileError` (span Iron) |
| 4.1 | 22300 | `infer/engine/mod.rs` + `driver.rs` | câble le moteur ; retire `local.rs` |
| 4.2 | 22300 | `infer/engine/mod.rs` + `tests/zero-annot/` | retire `params.rs` ; corpus dé-annoté |
| 4.3 | 22300 | `infer/engine/mod.rs` + `infer/flow.rs` | I11 en contraintes ; retire `flow.rs` |
| 5.1 | 22300 | `tests/gate-v10.sh` · `tests/fixtures/type-errors/` · `generator.md` | audit + gate + maj invariant |

## La fondation (1.1) fige — non négociable

1. **Le SCHÉMA des données échangées** : `Type::Var(u32)`, `Subst` (union-find : `fresh/union/apply/
   occurs_check`), `Constraint::{Eq(Type,Type,Span), Trait(Type,TraitBound,Span)}`, `TraitBound` (ensemble
   béni). Aucun champ `Any`, aucune unité implicite.
2. **Les CONVENTIONS partagées** : un `NodeId` → un `Type` (var ou concret) ; **toute contrainte porte un
   span** (I8) ; **aucune `Var` ne sort** de l'inférence (invariant de sortie, D2).
3. **La structure du module** : `engine/mod.rs` déclare **tous** les sous-modules + l'entrée
   `infer_types(ast, bindings) -> Result<TypeMap, CompileError>` (stub). Les fichiers `gen/solve/assume/
   generalize/diag.rs` sont **créés vides** → les pistes 2.x/3.x n'éditent **que** leur fichier, **jamais**
   `mod.rs` (pas de collision sur les déclarations `mod …;`).

## Invariant dur de réalisation (D9)

L'**ancienne inférence reste branchée** jusqu'à la vague 4. À chaque vague la colonne **passe l'e2e** (corpus
50 + binding 30 + vitrine) **via l'ancien chemin** ; le moteur se construit **à côté**, testé en isolation,
puis branché à la migration. On ne retire `local`/`params`/`flow` **que** quand le moteur couvre leur rôle,
prouvé par le corpus. Une bascule qui casse l'e2e → on ré-unifie avant d'avancer.

## Test mental de parallélisme (avant de lancer)

- **Vague 2** : `gen.rs` / `solve.rs` / `assume.rs` → **3 fichiers disjoints**, contre le contrat figé 1.1.
  Testables en isolation : `gen` par inspection des contraintes (sans solveur), `solve` par contraintes
  montées à la main (sans gen), `assume` par lecture d'un `.iri`. ✅ pas de collision, pas d'interface non figée.
- **Vague 3** : `generalize.rs` / `diag.rs` → **2 fichiers disjoints**, après vague 2 unifiée. ✅
- **Vague 4** : tout touche `infer/engine/mod.rs` (l'entrée) → **séquentiel assumé** (pas parallélisable). ✅

## Definition of Done — par vague

- **1 (fondation)** : contrat compile · `subst`/`constraint` testés unitairement · corpus+binding VERTS via
  l'ancienne inférence · `lower`/`resolve` reçoivent zéro `Var` (garde).
- **2 (fan-out + unif.)** : les 3 modules remplis + testés isolément ; **unification** : le moteur type un
  programme trivial e2e (`def add(a,b): return a+b` → `(i64,i64)->i64`) en parallèle de l'ancien.
- **3 (fan-out + unif.)** : moteur produit des TypeMaps **complètes** (defaulting+généralisation) + des
  **erreurs Iron** sur un sous-ensemble.
- **4 (migration)** : driver sur le moteur ; `local`/`params`/`flow` retirés ; corpus+binding+vitrine VERTS
  **par le moteur** ; corpus **zéro-annotation** VERT.
- **5 (gate)** : `gate-v10.sh` agrège tout ; audit types-concrets ; diagnostics-de-type ; `generator.md` à
  jour ; **0 unsafe** ; verdict **GATE V10 VERT**.
