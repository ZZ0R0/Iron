# V18 — CADRAGE — « Iron suffisant à lui-même » (fiable · capable · plaisant)

> **But.** Faire passer Iron de « un bon transpileur dont le cœur marche » à **un langage sur lequel on
> s'appuie**. Le cœur (inférence de types I15, ownership V16/V17) est **mûr** ; les gains y sont décroissants.
> Ce qu'il manque pour la **self-suffisance**, ce ne sont **pas** plus d'inférence/ownership — c'est :
> **la confiance** (une garde de non-régression), **la capacité** (closures, async) et l'**ergonomie**
> (lever les frictions quotidiennes). Les **4 axes sont d'égale importance** (décision utilisateur) et tous
> réalisés en V18.

> **Preuve d'entrée.** Le raytracer (vrai programme non-trivial, ~5× plus rapide après V17) prouve qu'Iron
> EST suffisant pour une **classe** de programmes. V18 élargit la classe **et** la rend **fiable**.

---

## Les 4 axes (specs)

| Spec | Axe | Pilier | Rôle |
|---|---|---|---|
| **[70400]** | **Suite de correction / non-régression** | *fiable* | Un corpus de programmes `.irn` à **sortie attendue**, re-joué à chaque changement (compile + run + assert). La **métrique de vérité** toujours différée. Fini le spot-check manuel ; sécurise tout refactor. |
| **[21000]** | **Closures / fonctions d'ordre supérieur** | *capable* | Closures de 1ʳᵉ classe : passées en argument à des **fns utilisateur**, retournées, stockées ; lowering trait-objet/`impl Fn`. Débloque callbacks, combinateurs, `map`/`filter` riches, crates closure-lourds (nom, itertools). |
| **[21800]** | **Ergonomie (lever les gotchas)** | *plaisant* | `?`-dans-`main` auto, `int()`/`**` sémantique Python, **propagation de type inter-appels** (§2.4), **annotation de retour respectée** (§2.5). Réduit la friction quotidienne. |
| **[21300]** | **Async / concurrence** | *capable* | Un modèle async/concurrent **usable** (serveurs, outils concurrents) dans les contraintes d'Iron (valeur, zéro `unsafe`) : `async`/`await` + runtime opt-in, `thread.scope(&data)` (partage en lecture). Périmètre **borné** (l'async exotique reste `rust{}`). |

---

## La hiérarchie *dans* V18 (priorité d'exécution, pas d'importance)

Les 4 sont aussi importants, **mais l'ordre de réalisation compte** : la **suite de correction d'abord**, car
elle **protège les 3 autres**. Closures, async et ergonomie touchent le cœur (inférence/lowering) à fort rayon
de souffle ; sans garde automatisée, on régresse en silence (comme le refactor ownership V16/V17 fait à la
main). Donc : **[70400] = vague 1**, puis les 3 autres se réalisent **sous sa protection** (corpus vert
re-joué à chaque greffe).

```
[70400] suite de correction ──► protège ──► [21000] closures
                                       ├──► [21800] ergonomie
                                       └──► [21300] async
```

---

## Ancrage constitutionnel
- **Métrique de vérité (CLAUDE.md « Projet courant »)** : [70400] **EST** cette métrique enfin matérialisée
  (correction sémantique `Iron → Rust généré` sur corpus, re-jouée).
- **I15** : closures = l'**exception** assumée de l'inférence (la *déclaration* d'un générique et le *type
  d'une closure renvoyée* restent explicites). [21000] respecte ça ; [21800] §2.4 étend la propagation.
- **I11/I13** : [21800] §2.5 respecte l'annotation de retour (pas de réécriture en `Option`).
- **I5 + bloc « 0.1% »** : [21300] commit à un async **basique** (runtime opt-in, injecté seulement si `async`
  utilisé) ; l'async exotique (`select!`, pinning, streams, async-traits) reste **hors-périmètre** (`rust{}`).
- **I2** : tout reste sans `unsafe` ; `rustc` filet.

## La self-suffisance, mesurée
Le critère de « suffisant » n'est pas subjectif : à la fin de V18, **[70400] le chiffre** — N programmes `.irn`
réels (dont le raytracer, des programmes à closures, un programme async/concurrent) compilent **et** produisent
la **bonne sortie**, re-joués automatiquement. C'est ça, un langage fiable et capable.

> **Méthode.** Cadrage figé ici (phase 1). **Prochaine étape = planification sur ton feu vert.** Je ne démarre
> pas la réalisation de mon propre chef. Vu l'ampleur (4 axes), la réalisation s'étalera probablement sur
> plusieurs cycles — la planif le découpera.
