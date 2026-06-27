# V11 — Complétude du langage + inférence (cadrage)

> **Statut : CADRAGE (phase 1).** Exhaustif : **toute** décision structurante est tranchée ici (aucune
> implicite) pour ne pas re-itérer. Combler **tous** les trous trouvés à l'audit + finir le moteur. Gate
> final = **la comparaison-test fournie par l'utilisateur** + non-régression.

## Pourquoi cette version

Audit de ~22 features : le langage est mature mais il reste des **trous concrets** (patterns or/range, traits
user, surcharge d'opérateurs, fonctions imbriquées, etc.) et le **moteur d'inférence** a des cas non couverts
(construction d'enum non typée) + de la dette reportée (V10/V10.1 : anti-unification, flux en contraintes).
V11 comble **tout**, sauf l'irréductible justifié (cf. hors-périmètre).

## Périmètre — 4 chantiers

- **A — Inférence (finir le moteur)** : construction d'enum typée + scrutateur de `match` informé · anti-
  unification (V10.1/D-e) · diagnostics de type riches · flux Option/Result en contraintes.
- **B — Pattern matching** : **or-patterns** (`1 | 2 | 3`) · **range patterns** (`0..10`, `0..=10`).
- **C — Traits** : **class implémente un trait** + **bornes user** (`def f[T: Show]`) · **méthodes par
  défaut** · **surcharge d'opérateurs** (`__add__`… → `std::ops`).
- **D — Surface syntaxique** : **fonctions imbriquées** · **while/for-else** · **affectation chaînée**.

## Décisions figées (exhaustives)

### A — Inférence
- **DA-1 ✅** `Enum.Variant(args)` est **typé comme l'enum** (gen) — corrige `area(s)` sans annotation. En
  complément, un `match s: Enum.X(...)` **informe** le type du scrutateur `s` (bidirectionnel) : `s: Enum`.
- **DA-2 ✅** **Anti-unification sur conflit** (V10.1/D-e) : deux appels donnant des types **structurellement
  semblables** différant à une position (`list[i64]` vs `list[str]`) → généralisation **à cette position**
  (`list[T]`), pas du param entier. Conflit irréconciliable (`i64` vs `str` à plat) → promotion param-entier
  (déjà V10.1). Conflit sur param **annoté** → erreur (D-b).
- **DA-3 ✅** **Diagnostics de type riches** (étend D8) natifs sur la ligne Iron, avant `rustc` : conflit de
  primitives (déjà) **+** champ/méthode inconnu sur un type connu **+** arité d'appel **+** borne de trait
  non satisfaite quand le moteur peut la trancher. Le reste → `rustc` filet (I2).
- **DA-4 ✅** **Flux Option/Result (I11) ré-exprimé en contraintes** dans le moteur → `flow.rs` **retiré**.
  La chute (`def f() -> i64` sans return partout → `Option`) et le `?` (→ `Result`) deviennent des contraintes
  générées (port de `effective_ret`/`definitely_returns`).
- **DA-5 ✅** **`local.rs` : objectif retrait, couverture mesurée.** Le moteur produit la `TypeMap` par nœud ;
  `local.rs` reste **uniquement** en repli pour les nœuds non couverts, avec un **audit de couverture** (% de
  nœuds typés par le moteur). Retrait total quand couverture = 100 % sur corpus+binding ; sinon on **mesure et
  on documente** le reliquat (honnêteté). Pas de retrait à l'aveugle qui casserait les 80 verts.

### B — Pattern matching
- **DB-1 ✅** **or-patterns** `p1 | p2 | p3` : le lexer accepte `|` (déjà `BitOr` ailleurs → réutilisé en
  contexte pattern), parser → `Pattern::Or(Vec<Pattern>)`, lowering → `p1 | p2 | p3` Rust. Bindings interdits
  dans un or-pattern hétérogène (règle Rust) → erreur claire.
- **DB-2 ✅** **range patterns** : `lo..hi` **exclusif** (cohérent avec `range()` Python) et `lo..=hi`
  **inclusif**. Parser → `Pattern::Range(lo, hi, inclusive)`, lowering → `lo..hi` / `lo..=hi` Rust. Sur
  entiers/chars.
- **DB-3 ✅** **Exhaustivité** : pas de vérif maison — `rustc` la fait (filet, I2). Le `match` non exhaustif
  → erreur `rustc` remontée (comportement actuel conservé).

### C — Traits
- **DC-1 ✅** **`class Dog(Show):`** (syntaxe base Python) = « **implémente le trait Show** ». Désucré en
  `struct Dog` + `impl Show for Dog` (les méthodes de la classe qui matchent la signature du trait). Plusieurs
  traits : `class Dog(Show, Ord):`. Une base **inconnue comme trait** → erreur (« base de classe = trait »).
- **DC-2 ✅** **Bornes user** `def f[T: Show](x)` → `fn f<T: Show>` ; `T.show()` résolu via le trait (le
  trait Show est **émis comme trait Rust**). Le moteur connaît `(trait, méthode) → signature` et les `impl`
  (Iron + classes implémentantes) → `T.show()` type le résultat.
- **DC-3 ✅** **Méthodes par défaut** : `trait Show: def show(self) -> str: <corps>` parse (corps optionnel),
  lowering → méthode par défaut dans le trait Rust. Une classe peut ne pas la redéfinir.
- **DC-4 ✅** **Surcharge d'opérateurs** par méthodes spéciales, ensemble **béni** → `impl std::ops` :
  `__add__`→`Add`, `__sub__`→`Sub`, `__mul__`→`Mul`, `__div__`→`Div`, `__eq__`→`PartialEq`,
  `__lt__`→`PartialOrd`, `__str__`→`Display`, `__getitem__`→`Index`. Une classe avec `__add__` →
  `impl Add for C` (méthode `add`). `a + b` sur type user → `a + b` Rust (résout via l'`impl`) ; le moteur
  voit `Trait(a, Add)` satisfaite par l'`impl`, résultat = type de retour de `__add__`. Hors ensemble → non
  surchargeable (filet).

### D — Surface syntaxique
- **DD-1 ✅** **Fonctions imbriquées** : `def inner(...)` dans un `def` → désucré en **closure** `let inner =
  |args| { corps }` (capture par valeur, sémantique-valeur I1). Récursivité d'une inner non supportée au v0
  (closure non récursive) → message clair si détectée.
- **DD-2 ✅** **while-else / for-else** (sémantique Python : `else` exécuté si la boucle finit **sans
  `break`**) → désucré via un drapeau `__broke` (lowering).
- **DD-3 ✅** **Affectation chaînée** `a = b = expr` → `expr` évaluée **une fois**, assignée de gauche à
  droite : désucré `__t = expr; a = __t; b = __t`.

## Hors-périmètre — l'irréductible justifié

- **État global MUTABLE** (`count = 0` au niveau module, muté dans une fn) : **interdit par I2** — exigerait
  `static mut` → `unsafe`, qu'Iron n'émet **jamais**. **Constantes** globales immuables : **dans le
  périmètre** (DD-bonus : `NAME = literal` au top-level → `const`/`static` Rust). Mutable global : **dehors**.
- **Trait objects / dispatch dynamique** (`dyn Trait`) : hors-périmètre (la polymorphie passe par les
  génériques monomorphisés, I6/coût nul). `rust { }` si vraiment besoin.
- **Inférence d'exhaustivité maison**, HM-global (déjà hors le « 0.1 % »).

## Impact invariants
- **I11** ré-exprimé en contraintes (DA-4), pas supprimé. **I15** étendu (DA-1/DA-2/DA-3). **I2/I6** conservés
  (génériques monomorphisés, jamais d'`unsafe`, `rustc` filet). **I9** pipeline figé (étapes inchangées,
  `flow` fusionné dans `infer`). I1 (valeur) : closures imbriquées capturent par valeur.

## e2e / gate V11
1. **corpus + binding + vitrine + zero-annot toujours verts** (non-régression dure) ;
2. **toutes les features de l'audit comblées** : chaque trou → une fixture qui compile+run ;
3. **la comparaison-test de l'utilisateur** (à fournir) : un programme conséquent → compile, run, sortie juste,
   et idéalement comparée à une implémentation de référence ;
4. **0 `unsafe`** · **couverture du moteur mesurée** (DA-5) · `flow.rs` retiré · tous les `🔶` tranchés.

## Carte des specs (sous-systèmes : 1=parser, 2=types/inférence, 4=lowering, 9=gate)

| Spec | Chantier | Titre |
|---|---|---|
| **11000** | B | Patterns : or-patterns + range patterns |
| **11100** | D | Surface : fonctions imbriquées · while/for-else · affectation chaînée · const globales |
| **23000** | A | Inférence : construction d'enum typée + scrutateur informé + anti-unification |
| **23100** | A | Diagnostics de type riches (champ/méthode/arité/borne) |
| **23200** | A | Flux Option/Result en contraintes + retrait `flow.rs` + couverture `local` |
| **23400** | C | Traits user : `class(Trait)` + bornes + méthodes par défaut |
| **23500** | C | Surcharge d'opérateurs (`__dunder__` → `std::ops`) |
| **90900** | gate | Gate V11 (fixtures + comparaison-test + verdict) |

> **Sortie de cadrage** : toutes les décisions (DA-1…DA-5, DB-1…3, DC-1…4, DD-1…3, hors-périmètre) **figées**.
> Aucune implicite. Prêt à planifier.
