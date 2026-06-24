# Iron — V5 / réalisation (trace)

> Version de complétude. **Gate vert (2026-06-25).** Résorbe les manques de l'audit invariants/dettes.
> Corpus passé de 19 → **22 programmes**, toujours vert.

## Cibles — état

| # | Cible | État | Preuve |
|---|---|---|---|
| **G1** | `move` dernière utilisation (**I1**) | ✅ | `a_move.irn` : `t = s` **déplacé**, `--audit` = 0 clone |
| **G2** | Inférence d'appel de méthode (DX3) | ✅ | table `(type, méthode)→ret` ; `print` typé correctement |
| **G3** | `inout`/`ref` + `&mut` (**I3**, était « non fait ») | ✅ | `inout.irn` : `incr(n)`×2 → `7`, `add_to(n,10)` → `17` |
| **G6** | `NaN ≠ null` (**I12**) | ✅ | `nan.irn` : `NaN != NaN` → `true`, `== ` → `false` |
| **G8** | Métrique de vérité (mesurée) | ✅ | **22/22** corrects · **0 clone** / ~784 l. générées · **0 `unsafe`** (forbid) |
| **G4** | Inférence Option/Result par flux (**I11**) | 🔶 **scopé** | risqué à l'arrache (mé-typerait des fns du corpus en `Option` sans analyse « definitely-returns » correcte). L'annotation `Option[T]` + `some/none/ok/err` reste la réalisation d'I11 (`?`, `IronError` OK). |
| **G5** | Source maps granularité expression (**I8**) | 🔶 **scopé** | gain marginal sur l'instruction (qui pointe déjà la bonne ligne) ; pollution de marqueurs. |
| **G7** | Partage explicite `Rc`/`ref` (**I5**) | 🔶 **scopé** | `Rc<RefCell>` mal fait = footguns (cycles, `Weak`) ; mérite un vrai design. |

## Bilan invariants (post-V5)
- **I1** : clone ✅ + emprunt ✅ + **move dernière utilisation ✅** (single-use, conservateur). Complet pour l'essentiel.
- **I3** : `inout` → `&mut`, mutation propagée ✅. (Warning « tu voulais partager » : non fait — DZ.)
- **I12** : testé ✅.
- **I11** : `?`/`IronError` ✅, mais **inférence-flux pure non faite** (annotation) — reste le principal écart d'invariant.
- **I5** : « pas de GC » ✅ ; **partage `Rc` explicite non fait**.
- **I8** : spans + remontée ✅, **granularité instruction** (pas expression).

## Reste honnêtement ouvert
G4 (I11 flux), G5 (I8 expr), G7 (I5 Rc) — chacun un vrai morceau de design, pas un à-l'arrache sûr. Et la
métrique de vérité est chiffrée sur **22 programmes**, pas un corpus de milliers (la vraie validation large
reste à constituer).
