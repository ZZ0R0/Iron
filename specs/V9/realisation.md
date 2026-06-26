# V9 — Réalisation (tracker)

> **État : cadrage + planification FAITS. Réalisation NON commencée** (attend le feu vert — transition
> planif→réalisation = décision utilisateur, CLAUDE.md). Se remplit en phase 3-4. **Objectif : langage fini
> à 99.9% à la fin de V9.**

## Avancement des pistes (6 vagues, 22 pistes)

| Piste | Spec | Titre | Vague | Statut |
|---|---|---|---|---|
| 1.1 | — | Fondation : gel IR (SharedSync, async/await, yield, proto) | 1 | ☐ todo |
| 2.1 | 51600 | ⭐ Binding des méthodes de traits + protocoles bénis | 2 | ☐ todo |
| 2.2 | 51700 | Robustesse du bind (ré-exports, collisions, getters) | 2 | ☐ todo |
| 2.3 | 51800 | Inverse : export Iron → crate Rust (`ironc lib`) | 2 | ☐ todo |
| 2.4 | 70700 | `import <crate>` (auto-bind + cache) | 2 | ☐ todo |
| 2.5 | 70800 | Registre de bindings curés + `ironc add` | 2 | ☐ todo |
| 3.1 | 40300 | Fix `try/except` + hoisting | 3 | ☐ todo |
| 3.2 | 40400 | Auto-wrap I11 complet | 3 | ☐ todo |
| 3.3 | 21300 | Inférence élargie (HOF fiable) | 3 | ☐ todo |
| 3.4 | 10900 | Générateurs `yield` → itérateurs | 3 | ☐ todo |
| 4.1 | 30700 | `shared_sync[T]` → `Arc<Mutex>` | 4 | ☐ todo |
| 4.2 | 70900 | Module `thread` (spawn/join) | 4 | ☐ todo |
| 4.3 | 21200 | `async`/`await` → Rust async | 4 | ☐ todo |
| 5.1 | 60500 | Passe de lint réelle | 5 | ☐ todo |
| 5.2 | 60600 | Erreurs de type niveau Iron | 5 | ☐ todo |
| 5.3 | 60700 | Source-maps granularité expression | 5 | ☐ todo |
| 5.4 | 71000 | `ironc fmt` | 5 | ☐ todo |
| 5.5 | 71100 | LSP | 5 | ☐ todo |
| 5.6 | 71200 | `ironc repl` | 5 | ☐ todo |
| 6.1 | 90400 | Batterie « 30 vrais crates » | 6 | ☐ todo |
| 6.2 | 90100 | Métrique de vérité chiffrée | 6 | ☐ todo |
| 6.3 | 90500 | **Gate V9 final** | 6 | ☐ todo |

## Hérité de V8 à combler (références)

Bords rugueux notés dans `specs/V8/realisation.md` → adressés ici : collisions noms courts (→ 2.2), getter
`&str`→`to_string` (→ 2.2), auto-wrap `ok()` I11 (→ 3.2), limite try/except hoisting (→ 3.1), `extern mod`
(→ 2.4/2.5 import/registre), auto-détection closures (→ 2.1 traits + reste 0.1%), split `bind/` (cosmétique,
optionnel).

## Écarts / faiblesses (à remplir en phase 3-4)

_(rien encore — réalisation non commencée)_

## Le 0.1% assumé (hors V9, par conception — D6)

- Inférence **Hindley-Milner complète** (on reste local + flux dirigé par l'usage).
- **Auto-hébergement** (compilateur Iron écrit en Iron).
- **async exotique** : `select!`, pinning manuel, streams complexes → `rust{}`.
- **Coroutines/`yield` complexes** (plusieurs points de reprise, `send`) → `rust{}`.
- **Binding garanti de N'IMPORTE quel crate** : les traits user non-monomorphisables → `rust{}`/`.iri` curé.

> À la fin de V9, `realisation.md` ne doit plus lister que **ce 0.1%**. Tout le reste = **fait**.
