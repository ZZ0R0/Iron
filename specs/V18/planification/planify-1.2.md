# planify-1.2 — Audit fan-out : corpus par catégorie + remontée centralisée des défauts (sous-agents)

- Réalise (cadrage) : [70400]
- Version : V18 · Vague : 1 · Piste : 2 · Statut : todo
- Dépend de : planify-1.1 (le runner + le format existent)

## But
**Construire le corpus en éventail ET auditer le compilateur** : un **fan-out** de sous-agents écrit + lance des
programmes `.irn` réels **par catégorie**, chacun remontant les **défauts** (ne compile pas / mauvaise sortie),
qu'on **centralise** en un rapport → liste de **corrections compilateur** traitées au centre. C'est le scénario
« lecture globale → défauts centralisés → correction centralisée ».

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/tests/corpus/<catégorie>/*` — **chaque agent possède son sous-dossier de catégorie** (ownership
  disjoint strict → parallélisable sans collision). Aucun agent ne touche `corpus_runner.rs` (1.1).

## Interface produite / consommée
**Consommée** : le runner + le format (1.1). **Produite** : N cas `.irn`+attendus par catégorie + un **rapport
de défauts agrégé** (cas qui échouent + raison : compile-fail / diff de sortie).

## Étapes
1. Découper en **catégories disjointes** : CLI/fichiers · données/sérialisation · maths/vectoriel · closures
   (stubs, anticipant [21000]) · io/flux · texte/regex · un-crate-simple-curé · le raytracer.
2. **Fan-out** (workflow/sous-agents, **opt-in**) : un agent par catégorie écrit 5-10 programmes `.irn`
   **plausibles** + leur sortie attendue, les **lance** via le runner, et **retourne** un rapport structuré
   { cas, statut (vert/compile-fail/diff), détail }.
3. **Centralisation** : agréger les rapports → un **tableau de défauts** unique. Les **verts** rejoignent le
   corpus ; les **défauts** alimentent une liste de **corrections compilateur** (traitées au centre, vagues 2-4).
4. Honnêteté : **ne pas truquer** les attendus pour faire passer — un défaut réel reste un défaut (c'est l'audit).

## Definition of Done (vérifiable)
- ≥ 50 cas `.irn` ajoutés au corpus, répartis par catégorie ; `cargo test` les exécute.
- Un **rapport de défauts centralisé** existe (les cas qui ne passent pas + pourquoi) → la **vraie** carte des
  trous du compilateur (mesurée, pas inférée).
- Aucune collision (ownership de catégorie disjoint).

## Notes
- **Opt-in sous-agents** : confirmé au moment de lancer 1.2 (coût en agents/tokens). Sans opt-in → repli
  séquentiel (je rédige les catégories une à une, plus lent mais identique en résultat).
- Les défauts trouvés ici **orientent** [21000]/[21800]/[21300] (on corrige ce que l'audit révèle vraiment).
