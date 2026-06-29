# V14 — CADRAGE — Le système de génération de bindings (moteur + curation, à portée connue)

> **But.** Faire du binding un **vrai système**, sur le modèle Nim : un **moteur** d'auto-dérivation
> costaud (on l'a déjà à ~50% : `ironc add` → rustdoc JSON → `.iri`) **+** une couche de **curation
> manuelle** sanctionnée, les deux composant proprement. **Point critique** : le moteur doit **connaître
> sa propre portée** — quand il **sait** qu'il ne peut pas binder un item, il le **signale à la génération**
> (rapport), au lieu de le **droper en silence**. C'est ce silence qui nous a coûté les « murs » : on
> découvrait le trou à la compilation du programme, pas à la génération du binding.

> **Honnêteté d'entrée.** V14 est un **cadrage** (phase 1). La **réalisation** du moteur++ est **V15**
> (décision utilisateur). V14 fige l'**architecture** et les **contrats** ; il n'écrit pas le moteur.
> La cible affichée — **60/80 projets** sans matériel — n'est **pas** atteinte aujourd'hui (~31). Elle
> dépend surtout de **C1 (alias→nominal)** et **C2 (méthodes de trait projetées)** ; je ne la promets pas,
> je la **contractualise** (chaque capacité a un critère testable).

---

## Ancrage constitutionnel (rien d'inventé, on opérationnalise)

- **I7 — Interop par niveaux.** Le niveau **2** = `.iri` **manuels**, le niveau **3** = **auto-gen** rustdoc.
  La « séparation moteur/manuel » que tu demandes **est déjà dans la constitution** ; V14 définit **comment
  les deux couches coexistent** (merge last-wins par entrée), pas une philosophie neuve.
- **I7bis — Zéro lib Iron.** Un `.iri` (auto **ou** curé) **pointe un crate** ; il n'introduit jamais de
  code de bibliothèque Iron. La curation manuelle reste de la **donnée de binding**, pas une lib.
- **I8 — Diagnostics first-class.** Le **rapport de génération** EST un diagnostic I8 : item Rust ↔ raison
  de non-binding ↔ action (« écris à la main : … »). « Le moteur connaît sa portée » = I8 appliqué au binder.
- **I6 — Projection d'un sous-ensemble de Rust.** Les règles du moteur++ (C1–C4) restent des **projections
  générales** ; un item non projetable est **signalé** (50500), pas forcé.
- **Bloc « 0.1% » (generator.md).** Il dit déjà « couverture auto **mesurée 62–92%, jamais 100%** » et
  « **frontière mouvante : tenir cette liste à jour** ». V14 transforme cette frontière **prose** en
  frontière **machine, calculée et rapportée**.

> **La VRAIE règle « pas de cas par cas » (clarifiée).** Interdit = **nom de crate dans le COMPILATEUR**
> (`if crate=="scraper"` dans `ironc/src/*.rs`) — le compilateur reste **général**. **Autorisé** = **donnée
> de binding par-crate dans un `.iri`** — c'est ce qu'est **déjà `std.iri`** (« Bindings *curés* de la std »),
> c'est ce que fait Nim (`importc`) et cxx. J'avais sur-étendu l'interdit à « jamais de donnée par-crate »,
> ce qui est plus strict que **tout** langage et fabriquait les murs. **On corrige.**

---

## Les 3 specs de V14

| Spec | Sujet | Cœur |
|---|---|---|
| **[50400]** | **Système : moteur + curation** | Deux couches qui composent : `<crate>.gen.iri` (auto, régénérable, machine-owned) **+** `<crate>.iri` (curé, human-owned, **gagnant**), merge **last-wins par entrée**. Régénérer n'efface **jamais** la curation. Opérationnalise I7 niveaux 2+3. |
| **[50500]** | **Portée connue & rapport** *(critique)* | Chaque abandon du moteur devient un **`SkipDiagnostic`** structuré (taxonomie fermée de raisons) agrégé en **rapport de génération** + résumé stderr. **Invariant dur : zéro drop silencieux** (`seen_public == bound + skipped`). La couverture devient un **nombre calculé**. |
| **[50600]** | **Capacités du moteur++** *(réalisation V15)* | Les **règles générales** (zéro nom de crate) qui font passer des **familles** d'items de `skipped`→`bound` : **C1** alias-monomorphisé→type nominal (récupère `Vector3.new`), **C2** méthodes de trait projetées (AEAD `encrypt`/`decrypt`), **C3** const-génériques portés, **C4** modèle builder/flux (flate2). |

---

## Roadmap V14 → V15

- **V14 (ici) = cadrage.** Architecture des deux couches (50400), contrat du rapport & taxonomie (50500),
  contrat des capacités C1–C4 (50600). **Phase 1 uniquement.** Je m'arrête après ; la planification se fait
  **sur ton feu vert**.
- **V15 = planification + réalisation du moteur++.**
  - **Vague 1 (fondation/refactor)** = le **squelette à portée connue** : `project()` & collecteurs passent
    de `Option<Sig>` à `Result<Sig, SkipDiagnostic>` ; rapport + invariant `seen==bound+skipped` ; modèle
    deux-couches (50400/50500). **C'est la fondation : elle fige le schéma du diagnostic AVANT le fan-out.**
  - **Fan-out** = C1, C2, C3, C4 en pistes parallèles (ownership disjoint dans `bind.rs`/`lower.rs`), chacune
    **résorbant sa classe de skip** mesurée dans le rapport.
  - **Unification** = e2e réel sur les familles débloquées + recalcul de la couverture.

---

## Politique « remplacement des projets HW-bound » (décidée ici, sélection différée)

✅ **Décidé.** Le corpus reste **80 projets significatifs**. Les ~34 HW/OS-spécifiques sont **remplacés** par
des projets **non-triviaux** (surtout **pas** des cas jouets). Critères d'un remplaçant :
1. **Aucune** dépendance matériel/OS-spécifique.
2. **Exerce au moins une** classe dure du moteur (C1 dimensions, C2 trait-méthodes, C3 const-gen, C4 flux) —
   un remplaçant doit **mettre le moteur à l'épreuve**, pas le contourner.
3. **Programme réel** avec une **sortie vérifiable** (l'e2e du projet).

⏸️ **Différé** (tâche V15/phase 4) : la **sélection nominale** des remplaçants. La **politique** est figée
maintenant ; la liste se fait quand le moteur++ existe (on choisit des crates que C1–C4 visent).

---

## Plafond honnête

- **Aujourd'hui** : ~31/80 sans matériel (V12+V13). Murs = dimensions (ConstGenericShape), AEAD
  (GenericTraitMethod), flux (StreamBuilderApi) — **tous classés** par 50500.
- **Cible V15** : **60/80**, **conditionnée** à C1+C2 (haut levier) et au remplacement non-trivial des HW.
  **Je ne la garantis pas** : 50500 la rendra **mesurable** à chaque étape, donc on saura **où on en est**
  pour de vrai, sans estimation à la louche.

> **Méthode.** Cadrage figé ici (deux couches + rapport à portée connue + contrat C1–C4). **Prochaine étape
> = planification de V15 (vagues/pistes) sur ton feu vert.** Je ne démarre pas la réalisation de mon propre chef.
