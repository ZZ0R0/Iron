# V16 — CADRAGE — Ownership *whole-program* (la racine du clonage par défaut)

> **But.** Mettre la couche **ownership** au **même niveau que l'inférence de types** : une **analyse de flot
> interprocédurale à point fixe**, là où elle est aujourd'hui **locale, mono-passe et heuristique**. C'est *la*
> correction de fond révélée par le ray tracer (`RayTracer/RAPPORT_IRON.md`) : Iron compile tout le programme
> pour les **types** (I15) mais résout l'**ownership** fonction par fonction — d'où le clonage massif qui
> plafonne les perfs et la famille de frictions (opérateurs sur valeurs empruntées, `.copy()` manuel, ergonomie
> du parallélisme).

> **Origine (vérifiée dans le source).** `resolve.rs:14-42` traite chaque fonction **isolément** ; `resolve.rs:293`
> a une **whitelist `print`/`len` codée en dur** → tout autre argument d'appel « échappe » → `ByValue` → clone.
> `resolve.rs:49-63` (`compute_moves`) est une heuristique **single-use au niveau statement**, aveugle à la
> réutilisation intra-expression. Le critère actuel n'est pas « la donnée est-elle possédée ? » mais
> « apparaît-elle comme argument ? » — sans rapport avec le vrai besoin d'ownership.

> **Honnêteté de cadrage.** Le BINDER a plafonné (C1 était le pic) ; l'**ownership, lui, est un sous-système
> franchement incomplet et à fort levier**. C'est le gain moteur qui reste, et il livre la **promesse-cœur**
> d'Iron : « Python qui compile en Rust *sûr ET rapide*, presque sans annotations ».

---

## Ancrage constitutionnel

- **I15** — l'inférence de types est déjà whole-program à point fixe. V16 **calque l'ownership dessus** (même
  forme d'analyse : graphe d'appels, treillis monotone, point fixe). C'est le modèle de référence.
- **I1** — sémantique par valeur : `y = x` = copie conceptuelle, abaissée emprunt/move/clone. V16 ne change pas
  la sémantique ; il **choisit mieux** la forme d'abaissement (plus d'emprunts, moins de clones).
- **I2** — résolveur **optimiste**, `rustc` = filet, **jamais d'`unsafe`**. V16 reste optimiste mais **sûr par
  construction** (cf. §contrat de sûreté ci-dessous).
- **I3** — aliasing mutable ≠ ownership : pas d'auto-clone pour partager ; `inout`/`shared` explicite. Inchangé.
- **I4** — le clone-fallback reste un **warning de perf** (`--audit`). V16 en émet **beaucoup moins**.

---

## Les specs de V16

| Spec | Sujet | Rôle |
|---|---|---|
| **[30500]** | **Ownership whole-program unifié** *(LA fondation)* | Escape analysis **interprocédurale à point fixe** (modes de params) **+** une **seule** analyse read/borrow/consume pour args, méthodes, **opérateurs**, variable de `for`, captures, move/clone. Résout §1, §2.1, §2.2, §2.6 du rapport. |
| **[50700]** | **Résolution d'import `.iri` récursive** | Appliquer la résolution `module .irn` *ou* `binding .iri` à **tous** les sous-modules, pas seulement au fichier d'entrée. Résout §2.3. |
| **[21800]** | **Inférence whole-program : propagation inter-appels + annotation de retour** | Propager un **type concret** le long des arêtes d'appel (§2.4b), inférer/diagnostiquer un membre **jamais appelé** (§2.4a), et **respecter l'annotation de retour** au lieu d'emballer en `Option` (§2.5). |

§2.7 (sémantique Python de `int()`/`**`) = **polish**, noté ici, traité en fin de réalisation (pas une spec).

---

## Le contrat de SÛRETÉ (la décision structurante à figer)

C'est le point qui ne doit **pas** rester implicite (sinon deux pistes le comblent différemment) :

- **Direction optimiste, rétrogradation monotone.** Tout paramètre non-`Copy` démarre `ByRef` ; l'analyse ne
  fait que **rétrograder** `ByRef → ByValue`. Monotone ⇒ **terminaison garantie** (même en récursion mutuelle).
- **Sûr par sur-approximation.** On rétrograde en `ByValue` (= clone, toujours correct) **dès qu'on ne peut PAS
  prouver** que le paramètre est seulement emprunté. **Dans le doute → `ByValue`.** On ne laisse `ByRef` que
  quand l'emprunt est **sûr**. Conséquence : l'analyse **clone moins** qu'aujourd'hui **sans jamais** produire
  un `&T` là où l'ownership est requis.
- **Invariant dur (non négociable) : zéro régression du corpus.** Le corpus 80 (vert aujourd'hui) doit **rester
  vert** à chaque étape de la réalisation. C'est l'e2e de V16. Un `ByRef` qui casserait `rustc` = bug bloquant,
  pas « le filet rustc fera son travail ».
- **Hors-périmètre (reste clone/`rust{}`)** : durées de vie **essentielles** (référence stockée qui survit),
  borrow-checking complet (c'est `rustc`, I2). V16 améliore le **choix emprunt/clone**, il ne devient pas un
  borrow-checker.

---

## L'e2e de V16 (phases 3-4)

1. **Corpus 80 reste vert** (l'invariant ci-dessus) — re-mesuré à chaque vague.
2. **Le ray tracer** (`RayTracer/raytracer_iron/`) : `ray_color`/`shade` passent `&Scene` (plus de clone par
   échantillon ni par rebond) ; **gain de perf mesuré** (rapport : ~1,5× sur 4 sphères, croissant).
3. **Les frictions disparaissent** : `a + b` compile sur un `Vec3` emprunté (sans passer en méthode) ;
   `v.dot(n.scale(...))` compile **sans `.copy()`** ; un binding importé dans un **sous-module** se résout.

## Roadmap V16 → V17
- **V16 (ici) = cadrage.** Architecture + contrat de sûreté + critères. Phase 1 uniquement. **Je m'arrête après ;
  la planification se fait sur ton feu vert.**
- **V17 = réalisation** (probable, vu le rayon de souffle) : fondation [30500] d'abord (squelette : la passe à
  point fixe qui garde le corpus vert), puis fan-out des pistes secondaires.

> **Méthode.** Cadrage figé ici. Prochaine étape = **planification sur ton feu vert**. Je ne démarre pas la
> réalisation de mon propre chef.
