# Sémantique & invariants

Iron a une **sémantique par valeur** : tu raisonnes comme en Python (les valeurs se copient), et le résolveur
choisit sous le capot la forme Rust efficace (emprunt quand c'est sûr, sinon clone). Tu **n'écris jamais** de
`&`, `&mut`, `.clone()`, lifetimes — c'est inféré.

## Le modèle en bref

```python
y = x        # COPIE CONCEPTUELLE. Abaissée en :
             #   • emprunt &x        si x est encore utilisé après et lu seulement (prouvé sûr)
             #   • move x            si c'est la dernière utilisation de x
             #   • x.clone()         sinon (fallback TOTAL : compile toujours)
```

- **Le clone est le filet** : si le résolveur n'est pas sûr, il clone → ça compile **toujours**. Un clone
  inutile est juste une **perte de perf** (jamais une erreur), signalée par un *warning* sous `--audit`.
- **`rustc` est le second filet** : le résolveur est **optimiste** (il emprunte là où il *pense* sûr) ; si
  c'est faux, c'est une **erreur `rustc` remontée sur ta ligne Iron**. Pire cas = erreur claire, **jamais**
  une corruption mémoire. Iron n'émet **jamais** d'`unsafe`.

## Muter une valeur

Par défaut, passer une valeur à une fonction ne la modifie pas (sémantique valeur). Pour un **partage mutable
explicite** :

```python
def push_into(inout buf: list[i64], x):   # inout T → &mut T
    buf.append(x)

shared_counter = shared[i64]              # shared[T] → Rc<RefCell<T>> (partage explicite, I5)
```

- **Ne JAMAIS compter sur un auto-clone pour partager** : cloner changerait la sémantique (les copies
  divergent au lieu de partager). Le partage mutable veut `inout`/`shared` **explicite** (I3).
- `shared_sync[T]` → `Arc<Mutex<T>>` (partage thread-safe, pour passer un état à `thread.spawn`).

## Sous-ensemble propre de Rust (I6)

Le système de types d'Iron est une **projection d'un sous-ensemble** de celui de Rust. Tout ce qui se mappe
proprement marche ; le reste est **suspect** et remonte au design (ou passe par `rust { }`).

## Les invariants (la constitution — règles dures)

Ces décisions sont **verrouillées** ; aucune feature ne les viole. Utile à un agent pour savoir ce qui est
possible **par construction**.

| # | Invariant | En clair |
|---|---|---|
| **I1** | Sémantique par valeur | `y = x` = copie conceptuelle ; abaissée clone/move/emprunt ; clone = fallback total. |
| **I2** | Résolveur optimiste, `rustc` filet | Emprunts optimistes ; erreur `rustc` remontée ; **jamais d'`unsafe`**, jamais de corruption. |
| **I3** | Aliasing mutable ≠ ownership | Pas d'auto-clone pour partager ; `inout`/`shared` explicite requis. |
| **I4** | Warnings = lint de perf | Clone-fallback → warning perf, off par défaut, `--audit`. Jamais bloquant. |
| **I5** | Pas de GC, pas de runtime lourd | Types valeur ; `Rc`/`Arc` seulement sur `shared`/`shared_sync` explicite. |
| **I6** | Cible Rust, héritage cargo | Types = projection d'un sous-ensemble de Rust. |
| **I7** | Interop par niveaux | 1 `rust { }` → 2 `.iri` manuels → 3 auto-gen rustdoc. |
| **I7bis** | Zéro bibliothèque Iron | Aucune stdlib Iron ; seules des **primitives** intégrées (`print`/`len`/`range`/opérateurs/compréhensions). Tout le reste = `import` d'un crate. |
| **I8** | Diagnostics first-class | Source-maps span `.irn` ↔ Rust ; erreurs remontées à la ligne Iron. |
| **I9** | Pipeline figé | lexer → parser → AST → infer + ownership → lowering `TokenStream` → `prettyplease` → `cargo`. |
| **I10** | Codegen via `TokenStream` | Jamais de concaténation de chaînes pour produire du Rust. |
| **I11** | Option/Result depuis le flux | Chemin `None` → `Option` ; opération faillible → `Result`, propagée par `?`. |
| **I12** | `NaN` ≠ null | `NaN` reste un `f64` valide ; ne déclenche pas d'`Option`. |
| **I13** | `IronError` par défaut, `Result[T, E]` typé opt-in | Faillible non annoté → `Result<T, IronError>` ; annoter `Result[T, E]` pour un `E` concret. |
| **I14** | Extensions | source `.irn`, bindings `.iri`. |
| **I15** | Inférence par contraintes, pilotée par les appels | Types params/retours résolus par un moteur de contraintes ; polymorphie seulement sur conflit (monomorphisation, coût nul). |

> Les primitives **intégrées** (pas besoin d'import) : `print`, `len`, `range`, `abs`, `min`/`max`,
> opérateurs, compréhensions, threading de base. **Tout le reste** (fichiers, regex, http, maths avancées…)
> vient d'un `import <crate>` — voir [interop.md](interop.md).
