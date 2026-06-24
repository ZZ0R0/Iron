# Iron — V0 / `generated.md`

> Carte de la version **V0** = la boucle source→binaire qui compile (`generator §3`).
> Parent : `specs/generated.md`. Contrat de contenu : `generator.md` (gabarits, invariants, nommage `XXXXX`).
> Statut : **🔶 proposé**.

## Objectif

La **boucle e2e minimale qui compile** : un programme Iron (fonctions, `struct`, variables locales, `if`/retour, `Option`/`Result` inférés par le flux) traverse **tout le pipeline figé (I9)** et produit du Rust que `cargo` compile et exécute correctement, **source maps présentes dès le codegen (I8)**. C'est le **squelette intégré** du projet : tout le reste (V1+) s'y greffe.

## Specs de V0

| ID | Composant | Dépend de | Invariants | Place pipeline |
|---|---|---|---|---|
| 10100 | Lexer indentation-sensitive (INDENT/DEDENT) | — | I9 | entrée |
| 10200 | Parser + AST (sous-ensemble Python-ish) | 10100 | I9 | entrée |
| 20100 | Inférence locale (monotypes, structs) | 10200 | I6 | milieu |
| 20200 | Inférence Option/Result depuis le flux | 20100 | I6, I11, I12, I13 | milieu |
| 30100 | Résolveur : clone (défaut) + move (dernière utilisation) | 20100 | I1, I2, I4 | milieu |
| 40100 | Lowering AST → `proc_macro2::TokenStream` (core) | 20200, 30100 | I9, I10 | sortie |
| 60100 | Source maps (span `.irn` ↔ Rust) + remontée diagnostics `rustc` | 40100 | I8, I2 | transverse codegen |
| 70100 | Driver `cargo` + CLI `ironc build`/`run` | 40100 | — | sortie |

## Réalisation (phase 3 — rappel méthodo)

1. **Squelette d'abord (1ʳᵉ vague, séquentiel)** : `ironc` lit un `.irn` trivial → **lexer/parser réels** → inférence/résolveur **stubés** (types triviaux) → lowering minimal → `prettyplease` → `cargo` vert sur un « hello ». Câblé bout en bout : fait remonter immédiatement les trous d'interface.
2. **Fan-out** (ownership disjoint) : inférence (20100/20200), résolveur (30100), source maps (60100) en pistes parallèles.
3. **Unification** : greffe + revalidation e2e **RÉELLE** (`ironc build/run` → `cargo` vert → exécution correcte).

## Gate V0 (phase 4)

- **Fonctionnel / continu** : `ironc build` sur un programme représentatif → `cargo` **vert** → exécution = résultat attendu ; spans Iron↔Rust présents ; **zéro `unsafe` émis** (I2).
- **Métrique de vérité** : différée (corpus) — on ne chiffre pas sur données insuffisantes.

## Décomposition → specs

Les 8 specs ci-dessus sont à **rédiger en phase cadrage de V0** (gabarit `generator §4`), puis planifiées (`V0/planification/X.Y.md`, gabarit `generator §6`). **Ordre de dérisquage** : **60100 (source maps) en priorité** dès que 40100 produit du Rust (`generator §8`).

## Risques / questions ouvertes

- ❓ Périmètre exact du sous-ensemble source V0 : quelles constructions de flux déclenchent `Option`/`Result` (I11) ?
- ❓ 60100 : remontée `rustc` **complète** dès V0, ou **baseline** V0 (spans seuls) + remontée complète plus tard ?
- 🔶 Frontière inférence : V0 = **monotypes only** ; génériques/traits → V3 (20300).
