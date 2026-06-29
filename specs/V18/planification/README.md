# V18 — PLANIFICATION — « Iron suffisant à lui-même » (réalise [70400]·[21000]·[21800]·[21300])

> Découpage d'**exécution**. Les 4 axes sont d'égale importance ; **l'ordre** suit la protection : **[70400]
> d'abord** (la garde), puis les capacités/ergonomie **sous sa protection** (corpus re-joué à chaque greffe).

## Orchestration : un seul axe en fan-out, le reste séquentiel

Leçon V16/V17 : un **refactor de cœur** (closures, async) est **couplé** (`infer`/`lower`) → **séquentiel**
(paralléliser = collisions/fausse parallélisation). En revanche **[70400]** (la suite de correction) est le cas
idéal du **« lecture globale → défauts centralisés → correction centralisée »** : un **fan-out** d'agents écrit+
lance des programmes `.irn` par catégorie, remonte les défauts (ne compile pas / mauvaise sortie), on corrige
**au centre**. → **Audit fan-out = piste 1.2** (sous-agents) ; tout le reste = séquentiel.

## Carte des vagues

| Vague | Piste | Réalise | Fichiers POSSÉDÉS | Note |
|:--:|:--:|---|---|---|
| **1** | 1.1 | [70400] | `ironc/tests/` (runner + format + intégration `cargo test` + noyau de cas) | séquentiel — **la garde** |
| **1** | 1.2 | [70400] | `ironc/tests/corpus/*` (cas `.irn`+attendus, **par catégorie**) | **FAN-OUT** (sous-agents) — audit + corpus |
| **2** | 2.1 | [21000] | `infer/`, `lower.rs`, `ir.rs` (closures 1ʳᵉ classe) | séquentiel — cœur |
| **3** | 3.1 | [21800] | `infer/`, `parse.rs`, `lower.rs` (ergonomie : §2.4/§2.5/`?`-main/stdlib) | séquentiel |
| **4** | 4.1 | [21300] | `lower.rs`, `iri/bindings` (async/await + `thread.scope`) | séquentiel — dépend de 2.1 (closures) |
| **5** | 5.1 | toutes | unification + **mesure de self-suffisance** ([70400] re-joué : N/N) | séquentiel — **gate final** |

## Graphe de dépendances

```
1.1 ─► 1.2 (audit fan-out) ─► [garde en place]
                │
                ▼
        2.1 closures ─► 4.1 async (thread.scope a besoin des closures)
                │
                └─► 3.1 ergonomie
                          │
                          ▼
                       5.1 unification + mesure
```
- **1.1→1.2** : la garde d'abord (le runner, puis le corpus en éventail). À partir de là, **tout est protégé**.
- **2.1 closures** : la capacité-pivot (l'async en dépend pour les lambdas de worker).
- **3.1 ergonomie** ‖ conceptuellement avec async, mais partage `lower`/`infer` → séquencé.
- **4.1 async** après closures. **5.1** = unification + **chiffre la self-suffisance**.

## L'invariant central (le critère de « fini »)
- **Dès la vague 1** : `cargo test` **re-joue le corpus** ; toute régression d'une vague suivante **fait
  échouer le test** (la garde mord — fini le spot-check manuel).
- **5.1** : la suite [70400] **chiffre** « N/N programmes réels (raytracer + closures + async + ergonomie)
  compilent **et** produisent la bonne sortie ». C'est la **mesure** de self-suffisance.

## Sur le fan-out 1.2 (les sous-agents)
- Chaque agent prend **une catégorie** (CLI/fichiers, données/sérialisation, maths/vectoriel, closures,
  io/flux, un crate simple…), écrit 5-10 programmes `.irn` + attendus, les **lance**, et **remonte** : verts /
  défauts (compile-fail / mauvaise sortie, avec le diff). **Ownership disjoint** (chaque agent ses fichiers de cas).
- **Centralisation** : on agrège les défauts → un **rapport unique** → liste de **corrections compilateur**
  (traitées au centre, séquentiellement, dans les vagues 2-4 ou un lot de fixes dédié).
- C'est **opt-in** (coût en agents/tokens) — confirmé au moment de la réalisation de 1.2.

> **Méthode.** Planif figée ici. **Prochaine étape = réalisation sur ton feu vert.** Vu l'ampleur (4 axes), la
> réalisation s'étalera ; chaque vague est un incrément protégé par la garde [70400].
