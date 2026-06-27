# V10.1 — Planification (vagues & pistes)

> Refactor du moteur V10 vers l'**inférence pilotée par les sites d'appel** (boucle de point fixe +
> promotion). Le moteur est **couplé autour de `gen.rs`** → le parallélisme est **plus limité** qu'en V10
> (honnêteté : la plupart des changements touchent `gen.rs`, donc séquentiels). Une seule vraie piste
> parallèle (`generalize.rs`, fichier disjoint). 5 pistes · 4 vagues.

## Carte des vagues

| Vague | But | Pistes | Parallèle ? |
|---|---|---|---|
| **1 — Fondation** | Boucle de point fixe + type `Promoted` + contrat `gen` réentrant (comportement = V10, vert) | 1.1 | séquentiel |
| **2 — Cœur** | Basculement **monomorphe** + **promotion sur conflit** (D-a) — atomique (sinon `id` casse) | 2.1 | séquentiel |
| **3 — Raffinements** | Incrémental structurel + HOF *(gen.rs)* · Display/defaulting/retours *(generalize.rs)* | 3.1 ∥ 3.2 | **2 en parallèle** |
| **4 — Gate** | Gate V10.1 + fixtures (count/label/struct/HOF/récursion) + verdict | 4.1 | séquentiel |

## Piste ↔ décisions cadrées ↔ fichier possédé

| Piste | Décisions couvertes | Fichier(s) **possédé(s)** |
|---|---|---|
| 1.1 | D-g *(structure de boucle)* · D-i *(whole-program)* · le **contrat** | `infer/engine/mod.rs` (boucle) + **crée** `infer/engine/promote.rs` (type `Promoted`) |
| 2.1 | **D-a** *(promote)* · **D-b** *(conflit annoté = erreur)* · bascule monomorphe · cœur D-g *(itère)* | `infer/engine/gen.rs` (appels) + `infer/engine/promote.rs` (détection conflit) |
| 3.1 | **D-g** *(incrémental : champ/méthode/index résolus une fois la base connue)* · **D-h** *(HOF)* · D-d | `infer/engine/gen.rs` |
| 3.2 | **D-c / R2** *(Display→générique borné)* · **D-j** *(defaulting final)* · **D-l** *(retours génériques)* | `infer/engine/generalize.rs` |
| 4.1 | **D-k** *(externes génériques hors-scope, documenté)* · **D-f** *(best-effort, documenté)* · gate | `tests/gate-v10.1.sh` · `tests/zero-annot/` (+ count/label/struct) · `tests/fixtures/type-errors/` · `generator.md` |

> **D-e** *(anti-unification structurelle, `count` en `list[i64]` ET `list[str]`)* = **reporté par cadrage**,
> donc **non planifié pour le code** (cas rare). Listé ici pour traçabilité : à rouvrir si besoin réel.

## La fondation (1.1) fige

1. **Le type `Promoted`** = ensemble des métavariables de params **promues en génériques** (`HashSet<u32>`).
   C'est le format d'échange entre la détection de conflit (2.1) et la génération (`gen` freshise un param
   **seulement** s'il est déclaré générique **ou** dans `Promoted`).
2. **Le contrat `gen` réentrant** : `Gen::program(ast, &Promoted)` — la génération est **re-exécutable** avec
   la `Subst` accumulée + l'ensemble promu (c'est l'incrémentalité D-g).
3. **La boucle** dans `mod.rs` : `build_env → { gen → solve → détecte conflits → promeut } * jusqu'au point
   fixe → defaulting → writeback`. En 1.1 : **un seul tour, `Promoted` vide** → comportement V10 → corpus vert.

## Invariant dur de réalisation

À chaque vague la colonne **passe l'e2e** (corpus 50 + binding 30 + vitrine + zero-annot). Bascule atomique :
**la vague 2 (mono + promotion) se valide d'un bloc** — `id` doit rester polymorphe (par promotion) ET
`count(xs)` doit marcher (mono depuis l'appel) **avant** de clore la vague. Si la bascule casse `id` → on
n'avance pas tant que la promotion ne le rattrape pas.

## Test mental de parallélisme

- **Vague 3** : `gen.rs` (3.1) vs `generalize.rs` (3.2) → **fichiers disjoints**, contre le contrat figé en
  1.1/2.1. `generalize` tourne **après** la boucle (defaulting) ; `gen` **dans** la boucle. Interface = la
  `Subst` + les bornes par var (gelées). ✅ pas de collision.
- **Vagues 1/2/4** : touchent `mod.rs`/`gen.rs`/`promote.rs` de façon centrale → **séquentielles assumées**.

## Definition of Done — par vague

- **1** : boucle + `Promoted` + `gen` réentrant compilent ; un tour, promotion vide → corpus+binding VERTS
  (comportement V10 préservé).
- **2** : appels monomorphes + promotion sur conflit ; **`id` polymorphe (par promotion)** ET **`count(xs)`
  inféré depuis l'appel** ; corpus+binding+vitrine+zero-annot VERTS.
- **3** : (3.1) `return x.name` / `obj.m()` d'un param non annoté résolus via l'appel ; HOF typé depuis
  l'appel. (3.2) `label` → `T: Display` ; defaulting en dernier ; retours génériques. Unifié → tout vert.
- **4** : `gate-v10.1.sh` agrège tout + nouvelles fixtures (count/label/struct/HOF/récursion) ; `generator.md`
  (I15) à jour ; **0 unsafe** ; verdict **GATE V10.1 VERT**.
