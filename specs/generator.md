# Iron — `specs/generator.md`

> Méta-document. Définit **comment** sont produites les specs et les unités de planification.
> À lire avant d'écrire le moindre `Vn/XXXXX.md` ou `planification/X.Y.md`.
> Les chemins du dépôt et les conventions de pipeline vivent dans `claude.md` ; **ce fichier possède le contrat de contenu**. En cas de conflit de convention, `claude.md` prime.

---

## 0. Position dans le pipeline

```
cadrage (décisions validées)
        │
        ▼
 [ generator.md ]  ← ce fichier : procédure + gabarits + invariants. NE CONTIENT AUCUNE SPEC.
        │
        ▼
 specs/Vn/XXXXX.md            ← une spec = UN composant cohérent
        │
        ▼
 specs/Vn/planification/X.Y.md ← une unité = UNE tâche atomique (une session Claude Code)
        │
        ▼
 implémentation
```

Distinction stricte :
- **Spec** = décrit *un composant cohérent* en termes observables (le quoi), jamais l'implémentation détaillée.
- **Planif** = décrit *une tâche atomique* implémentable en une session, build vert à la fin (le comment, découpé).

---

## 1. Invariants — la constitution

Décisions verrouillées au cadrage. **Aucune spec ne peut les contredire.** Toute spec cite les invariants qui la concernent par leur numéro. Une feature qui exige d'en violer un n'est pas spécifiée : elle **remonte au cadrage**, elle ne se décide pas dans une spec.

- **I1 — Sémantique par valeur.** `y = x` est une copie conceptuelle. Le résolveur l'abaisse en `clone` (défaut), `move` (dernière utilisation), ou emprunt `&`/`&mut` (si prouvé sûr). Le clone est un **fallback total** : il compile toujours.
- **I2 — Résolveur optimiste, `rustc` = filet.** Le résolveur émet des emprunts là où il *pense* que c'est sûr ; si erreur, `rustc` rejette. Iron n'émet **jamais** d'`unsafe`. Pire cas = erreur `rustc` remontée en source Iron, **jamais** une corruption mémoire.
- **I3 — Aliasing mutable ≠ ownership.** Ne **jamais** auto-cloner pour résoudre un partage mutable : ça changerait la sémantique (les copies divergent au lieu de partager). Ce cas exige `inout`/`ref` explicite. À défaut → warning « tu voulais peut-être partager ».
- **I4 — Warnings = lint de perf.** Le clone-fallback émet un warning de catégorie *perf*, **off par défaut**, activable via `--audit`. Jamais une erreur de compilation.
- **I5 — Pas de GC, pas de runtime lourd.** Types valeur possédés. `Rc`/`Arc` uniquement sur partage *explicite* (équiv. `ref` Nim). Pas de ramasse-cycles → `Weak` à la charge de l'utilisateur, à documenter.
- **I6 — Cible Rust = héritage cargo.** Le système de types d'Iron est une **projection propre d'un sous-ensemble du système de types de Rust**. Toute construction de type qui ne se mappe pas vers Rust est suspecte et doit être justifiée dans la spec.
- **I7 — Interop par niveaux.** 1 : bloc `rust: { }` inline → 2 : bindings `.iri` manuels → 3 : auto-génération via `rustdoc --output-format json`. Ne pas spécifier/implémenter un niveau avant le précédent.
- **I8 — Diagnostics first-class.** Source maps (span `.irn` ↔ ligne Rust générée) dès que le codegen existe. **Toute spec touchant au codegen DOIT spécifier sa contribution aux source maps** (section dédiée obligatoire).
- **I9 — Pipeline figé.** lexer indentation-sensitive → parser → AST → type-check + ownership → lowering AST → `proc_macro2::TokenStream` (`quote`) → `prettyplease` → `cargo`.
- **I10 — Codegen via `TokenStream` uniquement.** Jamais de concaténation de chaînes pour produire du Rust.
- **I11 — Inférence Option/Result depuis le flux.** Chemin retournant `None` → `Option<T>`. Opération faillible → `Result<T, IronError>`, propagée par sucre `?`. `try/except` = **sucre** abaissé vers `match`, pas un mécanisme runtime.
- **I12 — NaN ≠ null.** `NaN` reste un `f64` valide et ne déclenche **pas** d'`Option`. Seul `None`/null → `Option`.
- **I13 — Type d'erreur unifié `IronError` au v0.** Pas d'inférence d'un `E` précis (post-1.0).
- **I14 — Extensions.** Source `.irn`, bindings `.iri`.

---

## 2. Procédure — générer une spec `Vn/XXXXX.md`

1. **Cibler UN composant cohérent** (lexer, parser, inférence, résolveur, lowering, niveau d'interop, source maps, CLI…). Un *milestone* peut couvrir plusieurs specs ; une spec ne couvre jamais plusieurs composants.
2. **Lister les invariants applicables** (§1). Si la feature exige d'en violer un → **STOP**, remonter au cadrage.
3. **Résoudre les dépendances.** Une spec ne référence que des specs de numéro inférieur ou explicitement notées.
4. **Remplir le gabarit** (§4). Comportement décrit en termes *observables*, avec exemples `Iron → Rust généré` pour tout ce qui touche la sémantique.
5. **Écrire les critères d'acceptation testables** *avant* toute décomposition. Si un critère n'est pas vérifiable objectivement, il est mal formulé.
6. **Décomposer en unités de planif** (§5).

> **Règle d'or :** une spec doit être compréhensible par une session Claude Code *fraîche*, avec pour seul contexte ce fichier + le cadrage. Pas de contexte implicite.

---

## 3. Convention de nommage

> Schéma **par défaut**. Si `claude.md` en fixe un autre, `claude.md` prime — supprimer alors cette section.

- **`Vn`** — version majeure du langage (`V1` = boucle source→binaire de bout en bout ; suite définie par la roadmap).
- **`XXXXX`** — code à 5 chiffres. **Premier chiffre = sous-système** :

  | Préfixe | Sous-système |
  |---|---|
  | `1xxxx` | front-end : lexer, parser, AST |
  | `2xxxx` | système de types & inférence |
  | `3xxxx` | ownership & résolveur |
  | `4xxxx` | codegen & lowering |
  | `5xxxx` | interop Rust |
  | `6xxxx` | diagnostics & source maps |
  | `7xxxx` | CLI & tooling (`ironc`, driver cargo) |
  | `8xxxx` | stdlib & bindings de base |
  | `9xxxx` | réservé / méta |

  Chiffres 2–3 = composant, 4–5 = sous-composant/itération. Ex. `10100` lexer, `10200` parser, `20100` inférence locale, `30100` résolveur clone/move, `40100` lowering core, `50100` blocs `rust:` inline, `60100` source maps.

- **`X.Y`** — décomposition d'une spec en tâches. `X` = identifiant court de la spec parente (table de correspondance en tête de `planification/`), `Y` = numéro de tâche atomique. Chaque `X.Y` = une session Claude Code, build vert à la fin. Le fichier référence sa spec parente dans son en-tête.

---

## 4. Gabarit de spec — `Vn/XXXXX.md`

```markdown
# [XXXXX] <titre>

- Version : Vn
- Milestone : Mn
- Statut : draft | validé | implémenté
- Dépend de : [XXXXX], [XXXXX]
- Invariants applicables : I1, I6, I9, …

## Objectif
<1 paragraphe : ce que le composant fait et pourquoi il existe.>

## Périmètre
<ce qui est dans cette spec.>

## Hors-périmètre
<ce qui appartient à une autre spec (donner l'ID si connu).>

## Comportement attendu
<le cœur. Le QUOI, en termes observables — pas le détail d'implémentation.
Pour toute sémantique Iron, fournir des exemples `Iron → Rust généré`.>

## Contrat / interface
<entrées, sorties, types, surface d'API que les autres composants peuvent attendre.>

## Source maps
<OBLIGATOIRE si I8 s'applique. Comment ce composant préserve/contribue
le mapping span .irn ↔ Rust généré. Sinon : "N/A".>

## Critères d'acceptation
<liste testable. Chaque critère = un test ou une vérification objective.>

## Décomposition
<liste des unités de planif X.Y produites depuis cette spec.>

## Risques / questions ouvertes
<honnête. Ce qui n'est pas tranché.>
```

---

## 5. Procédure — décomposer en `planification/X.Y.md`

1. Chaque `X.Y` = **une tâche atomique**, taille d'une PR, testable isolément, **laisse le build vert**.
2. Ordonner par dépendances : une tâche ne dépend que de tâches déjà faites ou explicitement listées.
3. **Pas de tâche fourre-tout.** Si une tâche ne tient pas en une session → la scinder.
4. Chaque `X.Y` porte au moins **un critère de done objectif** (test qui passe + build vert).

---

## 6. Gabarit de planif — `planification/X.Y.md`

```markdown
# [X.Y] <titre de la tâche>

- Spec parente : [XXXXX]
- Version : Vn
- Dépend de : [X.Y], [X.Y]
- Statut : todo | wip | done

## But
<une tâche atomique, implémentable en une session, build vert à la fin.>

## Fichiers touchés
<chemins attendus.>

## Étapes
<séquence d'implémentation.>

## Critère de done
<test(s) qui passe(nt) + build vert. Objectif, vérifiable.>

## Notes
<pièges connus, références aux invariants concernés.>
```

---

## 7. Conventions transverses

- **Langue** : prose en français, code & identifiants en anglais.
- **Auto-suffisance** : spec et planif lisibles par une session fraîche, sans contexte implicite.
- **Traçabilité** : planif → spec parente → invariants → cadrage. Toujours remontable.
- **Statuts explicites**, mis à jour au fil de l'avancement.
- Une spec/planif ne **paraphrase pas** le cadrage : elle le **cite** (numéro d'invariant) et le spécialise.

---

## 8. Exemple — carte des specs V1 (illustratif)

Décomposition de la roadmap M0→M6 en specs. À régénérer/compléter par version ; ceci montre la convention en action.

| ID | Composant | Milestone | Invariants clés |
|---|---|---|---|
| `10100` | Lexer indentation-sensitive (INDENT/DEDENT) | M0 | I9 |
| `10200` | Parser + AST (sous-ensemble Python-ish) | M0 | I9 |
| `20100` | Inférence locale, monotypes, structs | M1 | I6 |
| `20200` | Inférence Option/Result depuis le flux | M1 | I6, I11, I12, I13 |
| `30100` | Résolveur : clone défaut + move dernière utilisation | M2 | I1, I2, I4 |
| `40100` | Lowering AST → TokenStream (core) | M2 | I9, I10 |
| `70100` | Driver cargo + CLI `ironc build/run` | M2 | — |
| `60100` | Source maps + remontée diagnostics `rustc` | M3 | I8, I2 |
| `50100` | Blocs `rust:` inline | M4 | I7 |
| `50200` | Bindings `.iri` manuels | M4 | I7, I6 |
| `50300` | Auto-génération bindings via rustdoc JSON | M4 | I7, I6 |
| `30200` | Élision d'emprunt (escape analysis) + warning fallback | M2+ | I1, I3, I4 |
| `20300` | Enums + `match` + traits + génériques | M5 | I6 |
| `80100` | Stdlib (wrappers fins sur std + crates) | M6 | I7 |

> Ordre de dérisquage : `60100` (source maps) est le risque n°1 — le spécifier dès que `40100` produit du Rust, pas en fin de V1.