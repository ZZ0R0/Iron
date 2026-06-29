# V15 — PLANIFICATION — Réalisation du moteur de binding++ (réalise le cadrage V14)

> **Réalise** le cadrage **V14** : [50400] (système deux couches), [50500] (portée connue & rapport),
> [50600] (capacités C1–C4). La planif est nommée par **axe d'exécution** (`planify-<vague>.<piste>`),
> **pas** par numéro de spec — chaque unité **cite** en en-tête la/les spec(s) V14 qu'elle réalise.

> **Principe directeur de cette planif : l'anti-collision.** Les 4 capacités du moteur (C1–C4) touchent
> toutes le **cœur** de `bind.rs` (`project`, la boucle d'`impl`) et/ou `lower.rs`. Les lancer en 4 pistes
> parallèles = **collision** + intégration cassée (l'anti-pattern que la méthodo proscrit). **Solution :** la
> **fondation modularise `bind.rs`** en sous-modules à ownership disjoint, **puis** on ne parallélise QUE là
> où c'est vraiment disjoint. La vraie parallélisation de V15 est **limitée** côté moteur (2 pistes max) et
> **large** côté corpus (curation `.iri` ‖ projets de remplacement, fichiers totalement disjoints).

---

## Carte des vagues

| Vague | Piste | Réalise | Fichiers possédés | Parallèle ? |
|:--:|:--:|---|---|---|
| **1** | 1.1 | [50500][50600] | `ir.rs` (schéma figé : SkipReason/SkipDiagnostic/Coverage + `ExternApp` const-arg) | séquentiel (leaf — fige le contrat) |
| **2** | 2.1 | [50500] | `bind.rs` → `bind/{mapty,project,impls,render}.rs` (modularisation + `Option`→`Result<Sig,SkipDiagnostic>` + Coverage + invariant `seen==bound+skipped` + API reverse-alias) | séquentiel (touche tout le binder) |
| **3** | 3.1 | [50400][50500] | `driver.rs` (rapport `.md` + stderr + `--strict` + couches gen/curé), `iri.rs` (chargement two-layer), `main.rs` (flags) | séquentiel → **squelette e2e VERT** |
| **4** | 4.1 | [50600] C1 | `bind/impls.rs` (alias-monomorphisé → type nominal) | **‖** avec 4.2 |
| **4** | 4.2 | [50600] C3 | `bind/mapty.rs` (Const<N>→ExternApp const), `lower.rs` (turbofish const), `iri.rs` (syntaxe const) | **‖** avec 4.1 |
| **5** | 5.1 | [50600] C2 | `bind/impls.rs` (branche trait), `bind/project.rs` (matérialisation des génériques de trait) | séquentiel (partage `impls.rs` avec C1) |
| **6** | 6.1 | [50600] C4 | `bind/project.rs` (self-return), `lower.rs` (chaînage builder) | séquentiel (partage `project.rs` avec C2, `lower.rs` avec C3) |
| **7** | 7.1 | [50400] | `ironc/bindings/*.iri` (curation manuelle du **tail** que C1–C4 ne couvrent pas) | **‖** avec 7.2 |
| **7** | 7.2 | corpus | `specs/V11-test-2/<projets>/`, `CARTE-80.md`, `ETAT-IMPLEMENTATION.md` (remplacement HW **non-trivial** + validation) | **‖** avec 7.1 |

---

## Graphe de dépendances (ordre d'exécution réel)

```
1.1 ─► 2.1 ─► 3.1 ─┬─► 4.1 ─► 5.1 ─┐
   (fondation =     │                ├─► 6.1 ─┬─► 7.1
    squelette e2e)  └─► 4.2 ─────────┘        └─► 7.2
```
- **1.1→2.1→3.1** : fondation **séquentielle** (le squelette ; chaque étape consomme l'interface figée par la
  précédente). À l'issue de **3.1**, la colonne **passe l'e2e** : `ironc add` produit un `.iri` + un **rapport
  de couverture**, **zéro drop silencieux**, et une entrée **curée** écrase l'auto.
- **4.1 ‖ 4.2** : seule **vraie** parallélisation du moteur (impls.rs vs mapty.rs+lower.rs+iri.rs — disjoints).
- **5.1, 6.1** : **séquentiels** — ils partagent des modules avec la vague 4 (honnêteté : pas de fausse ‖).
- **7.1 ‖ 7.2** : parallélisation **large** côté corpus (types de fichiers totalement distincts).

---

## L'invariant e2e (phases 3-4) — ne JAMAIS le casser

À partir de la vague 3, la colonne **passe en permanence** l'e2e du binding :
1. `ironc add <crate>` sur un crate déjà vert (corpus V12/V13) produit un `.iri` **équivalent ou meilleur**
   (pas de régression de couverture).
2. Le **rapport** est émis, et `seen_public == bound + skipped` (assertion).
3. Un programme Iron utilisant un crate du corpus **compile + run** (cargo vert, 0 `unsafe` — I2).

Chaque vague de capacité (4→6) doit, après greffe, **re-valider l'e2e réel** (pas seulement ses tests isolés)
**et faire monter la couverture mesurée** ([50500]) de sa classe de skip. Si une piste casse l'e2e → on
ré-unifie avant de continuer.

---

## Pourquoi la modularisation de `bind.rs` est dans la fondation (et pas après)

La refonte `Option`→`Result<Sig,SkipDiagnostic>` (2.1) **touche déjà toutes** les fonctions du binder.
Découper en `bind/{mapty,project,impls,render}.rs` **au même moment** ne coûte presque rien de plus et
**crée les coutures** qui rendent la vague 4 parallèle (C1 sur `impls.rs` ‖ C3 sur `mapty.rs`). La fondation
**fige aussi les interfaces consommées** par le fan-out :
- `mapty::alias_for_monomorphization(...)` → consommée par C1 (qui ne modifie donc PAS `mapty.rs`).
- les hooks de sentinelles (`@arr`/`@read`/`@write`) dans `project.rs` → consommés par C2.
- `Type::ExternApp` const-arg (figé en 1.1) → consommé par C3.

> **Méthode.** Planif figée ici. **Prochaine étape = réalisation, sur ton feu vert.** Une fois lancée, je vais
> au bout des 7 vagues sans m'arrêter entre elles (autonomie de phase 3), sauf gate e2e final ou vrai blocage.
