# Iron — V0 / planification (`README.md`)

> Phase 2. Découpe les 8 specs `V0/*.md` en unités `planify-X.Y.md` pour **réalisation parallèle sans collision**.
> Convention (cf. `CLAUDE.md` phase 2 ; `generator §3` défère le schéma de nommage à `claude.md`) : **`planify-X.Y`** où **X = VAGUE** (séquentielle), **Y = PISTE** (parallèle dans la vague). Une unité = une session, **build vert** à la fin.
> Contrats **gelés** (interdits de modification en réalisation) : `V0/README.md §3–5` (types IR, `NodeId`/`Span`, interfaces). Gabarit : `generator §6` + ownership/interface.

## Correspondance piste → spec parente

| Piste | Spec(s) | Module **possédé** |
|---|---|---|
| 1.1 | toutes (fondation) | workspace + `src/ir.rs` + stubs + `tests/fixtures/` |
| 2.1 | 10100 | `src/lex.rs` |
| 2.2 | 10200 | `src/parse.rs` |
| 2.3 | 20100, 20200 | `src/infer/` |
| 2.4 | 30100 | `src/resolve.rs` |
| 2.5 | 40100 | `src/lower.rs` |
| 2.6 | 60100 | `src/sourcemap.rs` |
| 2.7 | 70100 | `src/driver.rs`, `src/main.rs` |
| 3.1 | unification | greffe + e2e réel (aucun module exclusif) |

## Vagues (ordre d'exécution)

- **Vague 1 — `planify-1.1` (séquentiel)** : fondation. Gèle les types IR en Rust, câble le squelette e2e (boote, `cargo` vert sur un `.irn` trivial), **prototype le mécanisme source-map** (dérisquage n°1), fournit les fixtures partagées. → tant que 1.1 n'est pas **vert + gelé**, aucune vague 2.
- **Vague 2 — `planify-2.1…2.7` (parallèle)** : 7 pistes, ownership **disjoint**, chacune remplit son module contre les **types gelés + fixtures** (jamais contre l'impl des autres). **Démarrer `2.6` en priorité** (risque n°1).
- **Vague 3 — `planify-3.1` (séquentiel)** : unification. Remplace les stubs par le réel, revalide l'**e2e RÉEL** = gate V0.

## Règle anti-collision (test mental)

Chaque piste `2.x` : touche **un seul module**, consomme/produit **des types gelés**, se teste sur **fixtures** (`tests/fixtures/`), ne touche **jamais** `src/ir.rs` (gelé en 1.1). Un manque dans `ir.rs` = **bug de la vague 1**, remonté avant la vague 2.

## Gate V0 (DoD global = `V0/README.md §6`)

`ironc run` sur le programme représentatif (fn + struct + `if/while/for` + récursion + `list/dict` + `Option/Result` + `?`, **les 2 syntaxes de bloc**) → `cargo` **vert** → sortie correcte ; `#![forbid(unsafe_code)]` ; erreur `rustc` forcée → remonte au `Span` Iron.

## État de réalisation (2026-06-24)

**Gate V0 : ✅ vert** (cf. `../realisation.md`). `1.1` ✅ · `2.1`–`2.3` ✅ · `2.4` 🔶 (clone-default, move-opt → V1) · `2.5`–`2.7` ✅ · `3.1` ✅ (intégration continue par incréments). Écarts/dettes : `../realisation.md`.
