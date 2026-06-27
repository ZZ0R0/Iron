# V10 — Moteur d'inférence complet (cadrage)

> **Statut : CADRAGE (phase 1).** Discussion + contrat. La planification et la réalisation viennent après,
> sur feu vert. Ambition tranchée avec l'utilisateur : **complète-pragmatique** (voir D0).

## Pourquoi cette version

L'inférence actuelle (`infer/local.rs` + `params.rs` + `flow.rs`, ~1125 lignes) est un **faisceau
d'heuristiques forward, best-effort, sans contraintes ni unification**. Elle annote ce qu'elle peut et
laisse `rustc` finir via des génériques fuités. Résultat : des annotations encore exigées sans raison
(param passé à une fonction externe, type d'élément d'une liste, retour d'un calcul de local…).

V10 remplace ce faisceau par un **vrai moteur** : génération de contraintes → unification → résolution de
traits → `TypeMap` complète. **L'annotation devient inutile pour tout code déterminé par l'usage et les
bindings.** C'est, avec l'auto-binding, l'un des deux cœurs du projet — et les deux sont **couplés** : le
moteur lit les signatures des `.iri` (ce que l'auto-binder produit) pour typer les appels de crates.

## Décisions figées (cadrage)

- **D0 — Ambition : complète-pragmatique** (tranchée). Inférence bidirectionnelle + contraintes de trait,
  lit les bindings. **Zéro annotation en pratique** pour params, retours, locaux, méthodes, overloading
  (`+`/`<`/`==`). Restent **explicites par conception** (comme Rust/Swift) : la **déclaration** d'un
  générique `def f[T]` et le **type d'une closure renvoyée** (`-> fn(A) -> R`). Pas de HM-global pur.
- **D1 — Architecture** : `constraint-gen (AST → contraintes) → unification (union-find + substitution) →
  résolution des contraintes de trait → TypeMap complète`. **Bidirectionnel** : mode *synth* (le type
  remonte) et mode *check* (un type attendu descend), pour les littéraux, méthodes, et la frontière D0.
- **D2 — Représentation = `Type::Var(TyVar)`** ajouté à l'IR (métavariable d'inférence). C'est le **format
  d'échange figé** entre génération et solveur. **Invariant de sortie : aucune `Var` ne subsiste** après
  inférence (sinon → defaulting D5, sinon → erreur D8). [Décision structurante : on étend l'enum `Type` —
  légitime, le moteur EST le sous-système `2xxxx` ; les autres composants ne voient que des types résolus.]
- **D3 — Overloading via contraintes de trait.** `+`→`Add`, `-`/`*`/`/`→`Sub`/`Mul`/`Div`, `<`/`<=`/`>`/
  `>=`→`PartialOrd`, `==`/`!=`→`PartialEq`, f-string/`print`→`Display`. Ensemble **béni**, mappé **1:1 vers
  les traits Rust** (I6). Résolu contre les instances connues : primitives + structs/enums Iron + **bindings**.
  Hindley-Milner pur ne gère pas l'overloading ad-hoc — c'est **le point technique dur** de la version.
- **D4 — Bindings = assomptions de typage.** Le moteur ingère `Bindings` (`.iri`) : `extern fn`/méthodes/
  types deviennent des faits de typage. **C'est le couplage inférence ↔ auto-binding** : `query` dans
  `levenshtein(query, n)` s'infère `str` **parce que** le moteur lit `levenshtein(a: str, b: str)`.
- **D5 — Defaulting.** Variable numérique non contrainte en clôture → `i64` (entier) / `f64` (flottant),
  comme le defaulting Rust/Haskell. Appliqué **après** unification, jamais avant (sinon on re-crée le bug
  « str dans `+` → i64 »).
- **D6 — Généralisation (let-polymorphism).** `def f[T]` introduit un **schéma** quantifié sur `T` ; chaque
  appel l'instancie fraîchement. La **déclaration** `[T]` reste explicite (D0) ; le **lien** `[T]`↔usage est
  inféré (`pair_of[T](x)` → `x: T` sans l'écrire). Récursion mutuelle : typer par composantes fortement
  connexes (variables de signature partagées dans une SCC).
- **D7 — I2 conservé : `rustc` reste le filet de sûreté.** Le moteur devient **autoritaire pour émettre des
  types concrets** (fini les génériques fuités non voulus), mais `rustc` garde mémoire/emprunts et rattrape
  tout trou résiduel. Défense en profondeur, pas redondance inutile.
- **D8 — Erreurs de type Iron (I8).** Un échec d'unification produit un **diagnostic Iron mappé sur le
  span** (« attendu `str`, trouvé `i64` ») **avant** `rustc`. La surcouche rustc (60800) reste en second
  rideau pour ce que le moteur ne couvre pas.
- **D9 — Migration sans big-bang.** `local`/`params`/`flow` subsumés **progressivement** ; à chaque vague la
  colonne (corpus 50 + binding 30 + vitrine) **passe l'e2e**. I11 (Option/Result par flux) **ré-exprimé en
  contraintes**, jamais perdu. `rust { }` opaque → variable de sortie défautée/annotée (noté hors-périmètre).

## Carte des specs (sous-système `2xxxx` — types & inférence)

| Spec | Titre | Rôle |
|---|---|---|
| **21700** | Socle : variables de type + substitution + langage de contraintes | le **contrat figé** (D2, D1) |
| **21800** | Génération de contraintes (AST → contraintes) | flux Option/Result intégré (I11) |
| **21900** | Unification + résolution des contraintes de trait | overloading (D3) — le point dur |
| **22000** | Assomptions depuis les bindings + résolution de méthodes | couplage auto-binding (D4), bidirectionnel |
| **22100** | Defaulting + généralisation (let-polymorphism) | D5/D6, frontière d'annotation D0 |
| **22200** | Diagnostics de type Iron (I8) | D8 ; rustc filet (D7/I2) |
| **22300** | Migration + gate V10 | subsume l'existant (D9) ; corpus « zéro annotation » |

## Impact sur les invariants (constitution)

- **Réverse** la ligne du « 0.1 % » (`generator.md`) : *Hindley-Milner global hors-périmètre* → l'inférence
  complète est désormais **dans le scope et core**, bornée par D0 (génériques déclarés + closures renvoyées
  restent explicites). À mettre à jour à la clôture de V10.
- **I2** (résolveur optimiste, rustc filet, jamais d'`unsafe`) : **conservé** (D7).
- **I6** (types = projection de Rust) : **renforcé** — les contraintes de trait sont les traits Rust.
- **I9** (pipeline figé) : inchangé — l'inférence reste l'étape `infer`, **réécrite** ; lexer→parser→**infer**
  →resolve→lower intact.
- **I11** (Option/Result par flux) : **ré-exprimé** en contraintes, pas supprimé. **I13** (E typé opt-in) :
  inchangé.

## e2e / gate V10 (définition)

Le smoke-test e2e du projet (cargo compile le Rust généré + `ironc run` juste) **plus** :
1. **corpus + binding-30 + vitrine** restent verts (non-régression dure, D9) ;
2. un **corpus « zéro annotation »** : les programmes du tuto (et plus) **sans annotations** sauf D0 →
   compile + run + sortie juste ;
3. **types concrets émis** : audit « zéro générique fuité non voulu » sur le Rust généré ;
4. **diagnostics de type Iron** : un jeu de programmes mal typés → message Iron sur le bon span (D8).

## Risques / questions ouvertes

- 🔶 **Overloading (D3)** = le risque principal. Si la résolution de traits devient ambiguë (ex. `+` sur deux
  variables libres) → **dégradation gracieuse** : on exige une annotation à cet endroit précis, pas de crash.
- 🔶 **Récursion mutuelle** (D6) : ordre de typage par SCC. Bien posé en théorie, à coder soigneusement.
- 🔶 **`rust { }`** : opaque pour le moteur → la variable assignée depuis un bloc Rust reste à défauter/annoter.
- ❓ **Inférence du `E` de `Result[T, E]`** depuis les `raise` (étendre I13 ?) : **reporté** — reste opt-in
  au cadrage V10, à rouvrir si pertinent (⏸️).
- 🔶 **Performance** : unification quasi-linéaire (union-find), acceptable ; à surveiller sur très gros AST.

> **Sortie de cadrage** : ces décisions (D0–D9) figées, on peut **planifier** (phase 2). Tant qu'une reste
> implicite, on ne sort pas de la phase 1.
