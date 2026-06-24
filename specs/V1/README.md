# Iron — V1 / cadrage (`README.md`)

> **Fondation V1 = élision d'emprunt** (spec `30200`). Transforme le **clone-total** de V0 en
> **emprunts / moves** là où c'est prouvé sûr. I1 : le clone reste le fallback ; I2 : résolveur
> **optimiste**, `rustc` = filet ; I4 : warning perf `--audit`.
> Parent : `specs/generated.md`. Contrat : `../generator.md`.
> **Invariant clé de V1 : SÉMANTIQUE STRICTEMENT INCHANGÉE vs V0** — V1 est une *optimisation pure*.
> Statut : **🔶 proposé** — trancher §4 avant fan-out.

## 1. Périmètre

V0 a laissé une dette nette (`V0/realisation.md` D2) : le lowering **clone toute place non-`Copy`** lue
comme valeur. V1 **peuple réellement l'`OwnershipMap`** (vide en V0) et fait émettre à `lower` des
emprunts/moves. Trois leviers, du plus gros gain au plus petit :

- **(a) Params de fn *read-only* → `&T`** (escape analysis) + les call sites passent `&x`. ← le gros gain.
- **(b) Dernière utilisation d'un local → `move`** (pas de clone).
- **(c) Lecture non-dernière d'un local dont le consommateur accepte `&` → `borrow`.**
- **Clone fallback (I1)** sinon, avec **warning perf `--audit`** (I4, off par défaut).

**Hors V1** : aliasing mutable explicite `inout`/`ref` (I3) → ultérieur ; génériques/traits → V3.

## 2. Modèle d'analyse (escape analysis)

**Par paramètre `p` d'une `fn`**, classer selon son usage dans le corps :
- **Read** : `p` n'est que **lu** — jamais retourné, ni stocké (struct/collection), ni passé en valeur à
  un consommateur → le param devient **`&T`**.
- **Consumed** : `p` **s'échappe** (retourné / stocké / passé en valeur à un consommateur) → reste **`T`** (move-in).
- *(Mutated : V0 n'a pas de mutation-through-param → pas de `&mut T` en V1, reporté avec I3.)*

**Par lecture de variable locale** : dernière utilisation → **Move** ; sinon, si le consommateur accepte
`&` → **Borrow** ; sinon → **Clone** (I1).

## 3. Conventions figées

- **`Disposition` (ir.rs) gagne `Borrow`** : `{ Clone, Move, Borrow }` — extension **rétro-compatible** du
  schéma V0 (autorisé : nouvelle version).
- **Mode de param** : nouvelle table `fn → [ParamMode]`, `ParamMode ∈ { ByValue, ByRef }`, produite par
  l'escape analysis, consommée par `lower` (signature + call sites).
- **`lower`** : `ByRef` → `&T` en signature, `&x` au call site, déréférencement au besoin dans le corps ;
  `Borrow` → `&x` ; `Move` → `x` (sans clone) ; `Clone` → `x.clone()` (+ audit).
- **Optimisme borné (I2)** : on n'émet un emprunt **que dans les cas clairement sûrs** ; au moindre doute
  → `Clone`. `rustc` reste le filet pour les **bugs du résolveur**, pas une licence pour des emprunts
  douteux — il faut **préserver I1** (« compile toujours »).

## 4. Décisions ouvertes (fond — à trancher avant fan-out)

- 🔶 **Modèle de signature.** **(A, recommandé)** params read-only → `&T` + call sites adaptés (gros gain,
  plus complexe : touche signatures + appels) **vs (B)** signatures inchangées, emprunts **locaux seulement**
  (petit gain, simple). → **A** ?
- 🔶 **Agressivité.** **Conservateur** (emprunt seulement si évidemment sûr, sinon clone) — recommandé pour
  préserver I1. → ok ?
- ❓ **`inout`/`ref` (I3)** pour mutation-through-param : **reporté hors V1** ? (V0 n'a pas la syntaxe.)

## 5. Critères d'acceptation V1 (gate)

1. **Sémantique inchangée** : les **11 programmes du corpus V0** produisent une sortie **identique** (diff vide).
2. **Moins de clones** : le Rust généré pour le corpus contient **strictement moins de `.clone()`** qu'en V0 (mesurable).
3. **`--audit`** liste les clone-fallbacks restants (warning *perf*, off par défaut, I4).
4. **Zéro `unsafe`** (I2) ; **I1 préservé** (les programmes valides compilent toujours).
5. Aliasing mutable jamais auto-cloné (I3) — **N/A V1** (pas de mutation-through-param).

## 6. Suite (après validation)

Fan-out de la spec `30200` : escape analysis (`OwnershipMap` réel + `ParamModes`) → `lower` les respecte →
`--audit`. **Petit périmètre (1 spec)** → vraisemblablement 1 vague fondation + 1–2 pistes ; détail en phase 2.
Le corpus V0 (`ironc/tests/fixtures/`) **est** le banc de non-régression sémantique (critère 1).
