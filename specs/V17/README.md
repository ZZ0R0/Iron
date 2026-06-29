# V17 — CADRAGE — Le moteur d'ownership unifié (le socle borrow/move/clone)

> **But.** Faire de l'**ownership** un **vrai moteur whole-program**, au même niveau que l'inférence de types
> (I15) : **une seule** analyse de flot — borrow / move / clone — à point fixe, qui décide pour **chaque** site
> d'usage d'une valeur, et **une seule** sortie (`OwnershipPlan`) consommée **uniformément** par le lowering. On
> remplace le **patchwork** actuel (mécanismes séparés, règles ad-hoc par construct) par un **socle**.

> **Le constat (vérifié).** L'auto-typage est un socle : un moteur de contraintes, une vérité. L'auto-clonage,
> lui, est **éclaté** : `resolve.rs` a **deux** analyses hétérogènes (escape → `ParamModes`, interprocédurale
> depuis V16 ✓ ; `compute_moves` → `OwnershipMap`, heuristique single-use intra-fonction) ; `lower.rs` a **~5
> sites de consommation** chacun avec sa règle (`byref_params` fns-libres-seulement, for-élément — une analyse
> **dupliquée** en V16 —, passage d'args, codegen **opérateurs** séparé, `extern_arg`). Symptôme : on a dû
> **recopier** le walk d'échappement pour le for-élément. Quand on duplique une analyse, c'est qu'il manque un socle.

> **V16 = le 1ᵉʳ pilier** (escape analysis interprocédurale à point fixe → scène/objets par `&`, **1.7× mesuré**).
> V17 **généralise** ce pilier à **tous** les sites d'usage et **subsume** les règles dispersées.

---

## Ancrage constitutionnel

- **I15** — l'inférence de types EST déjà un moteur whole-program à point fixe. V17 **calque** l'ownership
  dessus : même forme (graphe de flot/d'appels unique, treillis monotone, point fixe, une sortie).
- **I1** — sémantique par valeur : V17 ne change pas la sémantique, il **choisit** mieux l'abaissement
  (borrow/move/clone) de façon **cohérente** partout.
- **I2** — optimiste, `rustc` filet, **jamais d'`unsafe`** ; **sûr par sur-approximation** (doute → clone).
- **I3** — aliasing mutable ≠ ownership : `inout`/`shared` explicite, inchangé.
- **I4** — clone-fallback = warning perf (`--audit`), beaucoup plus rare.

---

## Les specs de V17

| Spec | Sujet | Rôle |
|---|---|---|
| **[30800]** | **Le moteur d'ownership unifié** | L'analyse whole-program **unique** borrow/move/clone à point fixe + sa sortie figée **`OwnershipPlan`** + la **taxonomie des sites d'usage** + le contrat de sûreté. Subsume `ParamModes` + `OwnershipMap` (escape + moves). |
| **[40500]** | **Consommation uniforme au lowering** | `lower.rs` consomme l'`OwnershipPlan` par **un seul chemin** à **tous** les sites (args, récepteurs, **opérateurs**, for-élément, retours, champs/collections, captures). Subsume `byref_params` / for-élément / codegen-opérateurs / `extern_arg`. |

---

## La décision structurante à figer : `OwnershipPlan` (le contrat de données)

C'est l'interface qui ne doit **pas** rester implicite (sinon le moteur et le lowering divergent) :

- **`OwnershipPlan`** = la **seule** sortie du moteur, consommée par `lower` :
  - **par site d'usage** (`NodeId` d'une lecture de valeur) → `Disp { Borrow, BorrowMut, Move, Clone }` ;
  - **par (fonction/méthode, paramètre)** → `ParamMode { ByRef, ByRefMut, ByValue }` (subsume `ParamModes`).
- **Un seul graphe de flot/d'appels**, construit une fois, partagé (le rapport le notait) — fns libres + méthodes
  (résolues via `TypeMap`) + externes (modes des bindings).
- **Le modèle, partout le même** : un usage est `Borrow` si lu-seulement / prêté à une position empruntante ;
  `Move` si **dernière-utilisation prouvée** et ownership requis ; `Clone` si réutilisé après consommation /
  échappé / **incertain**.
- **Point fixe monotone** (rétrogradation `ByRef→ByValue`, `Move→Clone`) → **terminaison** ; **conservateur**.

> **Invariant dur : zéro régression du corpus.** Le moteur **reproduit d'abord** le comportement courant
> (corpus 80 + raytracer inchangés), puis on **branche un site d'usage à la fois**, chacun remplaçant une règle
> ad-hoc. Un `&T` qui casserait `rustc` = bug bloquant, pas « le filet fera son travail ».

---

## L'e2e de V17

1. **Corpus 80 reste VERT** + **4 suites de tests** vertes — à chaque greffe.
2. **Ray tracer** : non seulement `&Scene`/`&Ray`/`&élément` (déjà V16), mais aussi les **clones résiduels
   éliminés** (`r.clone()` par objet, `rec.clone()`) **gratuitement**, parce que méthodes/last-use sont
   désormais **dans le moteur**, pas des patches. Perf re-mesurée (gain ≥ V16).
3. **Frictions levées** : `a + b` sur valeur empruntée (opérateurs via le plan), plus de `.copy()` manuel.

## Migration (intelligente, stagée — pas un one-shot)

1. **Cadrage (ici)** : `OwnershipPlan`, le modèle unique, le contrat de sûreté, la taxonomie des sites.
2. **Réalisation (V18 probable, vu le rayon de souffle)** : (a) le moteur produit l'`OwnershipPlan` en
   **reproduisant** l'existant (corpus vert, zéro changement) ; (b) `lower` bascule sur le plan **site par site**,
   chacun retirant une règle ad-hoc ; (c) on branche moves-réels, méthodes, for-élément, opérateurs, captures.

> **Méthode.** Cadrage figé ici (phase 1). **Prochaine étape = planification sur ton feu vert.** Je ne démarre
> pas la réalisation de mon propre chef.
