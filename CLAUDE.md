# CLAUDE.md — Méthodologie de travail

> Ce fichier porte la **méthodologie project-agnostic** : le cycle en 4 phases, la répartition des rôles,
> les conventions de travail. Il s'applique à **tout** projet mené avec ce workflow.
> Le **contenu spécifique à un projet** (stack, invariants, nommage des specs, définition de l'e2e) vit
> dans la section **« Projet courant »** en bas de ce fichier et dans les documents de contenu du projet.
>
> **Règle de primauté :** pour toute instruction/information **agnostique** (méthodologie, pipeline,
> conventions transverses), **ce `CLAUDE.md` prime**. Pour le **contrat de contenu** d'un projet
> (gabarits, invariants, nommage concret), c'est le document de contenu du projet qui prime.

---

## Principe directeur : on travaille TOUJOURS en 4 phases

Chaque incrément (`VX`) traverse les 4 phases dans l'ordre. On ne saute pas une phase. On ne code pas
avant d'avoir cadré et planifié. On ne valide pas avant d'avoir testé.

> **RÈGLE DURE — la transition de phase est une décision de L'UTILISATEUR, jamais la mienne.**
> À chaque étape je ne fais **QUE** la phase explicitement demandée. Je **ne passe JAMAIS** de
> cadrage→réalisation, ni de planification→réalisation, ni d'une étape à la suivante **de mon propre chef**.
> Je termine **exactement** ce qui est demandé, je **m'arrête**, et j'**attends le feu vert** pour la suite.
> « Le point est stable, j'enchaîne sur X » est **interdit** : je propose X et j'attends. La méthodologie
> existe pour une bonne raison — la **suivre prime** sur l'envie d'avancer. (Cf. aussi `AskUserQuestion`
> réservé aux directives de fond, pas aux transitions : pour une transition, je **propose et j'attends**.)
>
> **Portée = les 4 transitions de phase** (cadrage↔planif↔réalisation↔tests). À l'**intérieur** d'une
> phase déjà lancée, je vais **au bout sans redemander** : « étape » ci-dessus = **phase**, pas
> vague/piste interne. Cf. **Autonomie de la réalisation** (phase 3).

```
1. CADRAGE  →  2. PLANIFICATION  →  3. RÉALISATION  →  4. TESTS / TRY & RUN
   (specs)       (planify-X.Y)        (exécution)         (boucle de vérité)
     ▲                                                          │
     └──────────────  feedback : faiblesses notées  ───────────┘
```

**Structure des specs :**
- `specs/VX/` = **une version** = un périmètre cohérent à livrer = un cycle complet des 4 phases.
  Le **cadrage** et la **planification** de la version y vivent.
- Le **design détaillé** d'une version est un snapshot **figé** ; on n'y revient que pour une vraie raison.
- La **structure concrète** (noms de fichiers, arborescence, schéma de nommage) et les **tranches de
  version** (roadmap) sont **définies par le projet** (cf. « Projet courant » / le document de contenu).

## Répartition des rôles
- **L'utilisateur** donne les **directives de fond** et les **objections** ; il tranche le fond.
- **Moi** : je fais le **travail difficile** (conception détaillée, code) et je tranche les choix
  « triviaux à décider, non triviaux à coder ».
- Donc `AskUserQuestion` = pour une **vraie directive de fond / un arbitrage** — **jamais** pour
  me décharger d'un design qui m'incombe, ni pour re-litiger un point déjà tranché.
- **Honnêteté brutale** : je nomme les impossibilités et limites. Pas de réponse rassurante/hand-wavy.
- Je **persiste les décisions** dans les documents du projet au fil de l'eau :
  **✅ décidé · 🔶 proposé · ❓ ouvert · ⏸️ reporté**.

---

## Phase 1 — CADRAGE (specs)
**But :** poser l'architecture d'une version avant toute ligne de code. Phase de **discussion**.
- Dans le cadrage de la version : **périmètre** (tranche de la roadmap), **interfaces** entre composants,
  **formats de données échangés**, **critères d'acceptation**. Le spec = **le contrat**.
- Choix structurants à figer : tout ce qui est **partagé entre composants** — **format/ABI des données
  échangées, layout binaire ou mémoire, unités et espaces d'adressage, contrats d'interface**.
  Design → **je décide** ; fond → **je pose la question**.
> Ne pas sortir de la phase 1 tant qu'une décision structurante reste implicite. Une mauvaise interface
> se paie au centuple en phase 3.

## Phase 2 — PLANIFICATION
**But :** découper pour lancer **un max d'agents en parallèle, CORRECTEMENT** (sans collision).
- Unités de planif dans `specs/VX/planification/`, nommées **`planify-<vague>.<piste>.md`** (+ `README.md`
  de vue d'ensemble, + `fondation.md` optionnel pour le contrat de squelette). **RÈGLE DURE de nommage :** le
  nom encode l'**axe d'EXÉCUTION** (vague, piste), **JAMAIS** un numéro de spec de cadrage. Une planif `planify-2.3`
  = vague 2, piste 3 — et elle **cite** dans son en-tête la/les spec(s) de cadrage qu'elle réalise. **Cadrage ≠
  planification** : le cadrage (`Vn/XXXXX.md`, contenu/contrats) et la planification (`planify-V.P`, découpage
  d'exécution) sont des **objets distincts** ; confondre leurs nommages (planif numérotée comme une spec) est l'erreur à ne JAMAIS refaire.
  La décomposition suit deux axes :
  - **VAGUE** de réalisation (1, 2, 3…), exécutées **dans l'ordre** (toute la vague 1, puis la 2…).
    Ce n'est NI une couche d'archi NI la version — c'est le rang séquentiel de l'étape. = le **X** du nom.
  - **PISTE PARALLÈLE** au sein d'une vague. Même vague ⇒ parallèle ; vagues différentes ⇒ séquentielles. = le **Y** du nom.
  - Une vague à 1 piste = étape séquentielle ; à plusieurs = fan-out parallèle.
- **Le squelette/fondation EST la 1ʳᵉ vague.** Puis fan-out → unification → **on recommence** jusqu'à la
  fin de la phase.
- **La fondation fige le SCHÉMA des données échangées ET les CONVENTIONS, pas que des coquilles de
  types.** Un champ `Any`/non spécifié, une unité/un espace d'adresses implicite (**offset vs adresse,
  octet vs bit, ordre des octets, layout binaire exact, encodage**) = **trou de parallélisme** → deux
  agents le comblent différemment → unités vertes, **intégration cassée**. Geler le payload concret +
  toute convention partagée fait partie de la fondation, au même titre que les signatures.
- Chaque unité précise : **périmètre exact** + **fichiers possédés** (ownership disjoint),
  **dépendances amont**, **interface produite/consommée** (signature, I/O), **Definition of Done** vérifiable.
> Test mental avant de valider le plan : « si je lance ces fichiers en parallèle maintenant, deux agents
> touchent-ils le même fichier ou attendent-ils une interface non figée ? » Si oui → re-découper ou
> remonter en fondation.

## Phase 3 — RÉALISATION (squelette → fan-out → unification, en cycles)
**But :** garder **en permanence une colonne vertébrale unifiée, intégrée, qui PASSE le smoke-test e2e**
du projet — jamais de big-bang d'intégration à la fin.

> **Autonomie de la réalisation (RÈGLE DURE).** Une fois la phase 3 **lancée par l'utilisateur**,
> j'exécute **TOUT le plan jusqu'au bout, à 100 %** — squelette → fan-out → unification → (répéter) —
> **sans m'arrêter entre les vagues/pistes** pour redemander le feu vert. Je ne m'arrête que (a) au
> **gate e2e de la version** (fin de phase 3), ou (b) sur un **vrai blocage de fond** que je ne peux pas
> trancher seul. Un écart se **note** (phase 4), il n'interrompt pas la réalisation.
> « Une fois planifié, je fais **tout** ce qui est planifié, en **entier**. »

1. **Squelette intégré d'abord (séquentiel)** : une colonne e2e fine = contrats figés + interfaces +
   **stubs**, câblés, qui **passe le smoke-test e2e du projet** sur un cas trivial de bout en bout.
   Non instrumenté mais **unifié**. C'est lui qui fait remonter *immédiatement* les trous d'interface.
2. **Fan-out parallèle** : squelette figé → lancer les pistes d'une même vague **en parallèle** (agents,
   ownership disjoint) ; chacune remplit son stub sans toucher aux contrats ni aux autres.
3. **Unification (séquentiel)** : greffer le parallèle sur le squelette et **revalider l'e2e RÉEL**,
   pas seulement les tests unitaires isolés.
4. **Répéter** (fan-out → unification) tant que la phase a du contenu. Entre deux cycles, la colonne
   **passe toujours** l'e2e.
- On réalise **exactement** le plan ; un écart se **note** (phase 4 / version suivante), il ne se code
  pas en douce. Ownership respecté ; interfaces figées non modifiées unilatéralement.
- **Invariant dur :** à aucun moment la colonne ne cesse de **passer l'e2e du projet**. Si une piste
  casse l'e2e, on ré-unifie avant de continuer. (Définition concrète de l'e2e → « Projet courant ».)

## Phase 4 — TESTS / TRY & RUN
**But :** confronter au réel. La **boucle de vérité**. Deux tests à ne jamais confondre :
- **Fonctionnel / intégration** (le pipeline tourne e2e sur quelques cas, pas de crash) : quelques cas
  suffisent → **continu, jamais différé** ; tourne même avec stubs.
- **Métrique de vérité** (la **mesure-cœur** du projet — sa raison d'être chiffrée) : exige un **corpus**
  suffisant → **différé** jusqu'au seuil. On **refuse un chiffre** sur données insuffisantes plutôt que
  d'en publier un faux.
- **Noter systématiquement** problèmes, faiblesses, dettes, angles morts → alimentent le cadrage de la
  version suivante (trace écrite, pas oral).
- Pas d'auto-satisfaction : un test échoue → je le dis avec la **sortie réelle** ; une étape sautée → je le dis.

---

## Conventions de travail
- **Langue** : prose (specs, plans, discussions) en **français** ; **code & identifiants en anglais**.
- **Honnêteté des limites** : le projet déclare ses angles morts ; pas de verdict faussement confiant.
- **Versionnage** : une `VX` = un cycle complet des 4 phases sur une tranche cohérente (roadmap). On ne
  mélange pas deux versions dans un cycle.
- **Persistance** : toute décision → documents du projet avec son tag de statut (✅/🔶/❓/⏸️).
- **Non-négociables & stack de référence** : **définis par le projet** (cf. « Projet courant »).
  S'y tenir sauf décision de cadrage explicite.

## Au démarrage d'une session
1. Identifier **où en est la version courante** dans le cycle des 4 phases.
2. Reprendre à la phase en cours — ne pas re-cadrer ce qui est **figé**, ne pas coder ce qui n'est pas planifié.
3. Doute : **directive de fond → poser la question** ; **design → décider** (ne pas me décharger).
   **Jamais re-litiger un point tranché.**

---

## Projet courant — Iron

> Bloc **borné** : tout le spécifique-projet vit ici (et dans `specs/generator.md`). Le corps ci-dessus
> reste agnostique. **Conflit de contenu** (gabarits, invariants, nommage) → `specs/generator.md` prime ;
> **conflit de méthodologie / pipeline** → ce `CLAUDE.md` prime.

- **📖 Doc usage (agents + humains).** [`docs/README.md`](docs/README.md) = guide d'onboarding **dense** :
  langage, sémantique, **interop/binding**, **pièges** (`docs/gotchas.md`), curation, CLI, état mesuré.
  **Au démarrage, si tu dois écrire/lire/binder du Iron, lis ce guide** (surtout `gotchas.md`) plutôt que de
  re-déduire les subtilités depuis le source. C'est la doc de référence du langage pour l'utilisateur.
- **Quoi.** Iron = un **langage** (indentation-sensitive, sémantique par valeur) qui **transpile vers
  Rust** et compile via `cargo`. Pas d'OS, pas de runtime lourd, pas de GC.
- **Contrat de contenu.** `specs/generator.md` possède les **gabarits** de specs/planif, les **invariants
  I1–I14**, et le **nommage** : specs (cadrage) `specs/Vn/XXXXX.md` (1ᵉʳ chiffre = sous-système) ; planifs
  `specs/Vn/planification/planify-<vague>.<piste>.md` (**X = vague d'exécution, Y = piste parallèle** — PAS le
  numéro de la spec de cadrage), chaque planif **citant** en en-tête la/les spec(s) qu'elle réalise.
- **Stack.** Rust. Pipeline figé (I9) : lexer → parser → AST → type-check + ownership → lowering →
  `proc_macro2::TokenStream` (`quote`) → `prettyplease` → `cargo`. CLI `ironc` (`build`/`run`).
  **Codegen via `TokenStream` uniquement** (I10, jamais de concat de chaînes) ; Iron **n'émet jamais
  d'`unsafe`** (I2).
- **Non-négociables = invariants I1–I14** (`generator.md` §1). Une feature qui exige d'en violer un
  **remonte au cadrage**, elle ne se décide pas dans une spec.
- **e2e / smoke-test (phases 3–4).** Pas un boot : c'est `cargo` qui **compile le Rust généré** (build
  vert) et `ironc build/run` sur un cas qui produit le bon résultat. **Source maps (I8) dès que le
  codegen existe.**
- **Métrique de vérité (différée).** 🔶 À arrêter : correction sémantique `Iron → Rust généré` sur corpus
  + coût du clone-fallback (warning perf `--audit`, I4). À chiffrer quand le corpus existe, pas avant.
- **Extensions.** `.irn` (source), `.iri` (bindings).

> **Projets futurs.** Pour un nouveau projet, **dupliquer ce bloc borné** et l'adapter ; ne **pas**
> polluer le corps agnostique au-dessus.
