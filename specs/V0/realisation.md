# Iron — V0 / réalisation (trace)

> Phase 3 terminée. Trace de réalisation : ce qui est livré, et les **écarts/dettes** qui alimentent
> le cadrage V1 (méthodo : « noter systématiquement problèmes, faiblesses, dettes »).
> Statut : **gate V0 vert (2026-06-24).**

## Gate V0 — atteint

Compilateur `ironc/` (crate lib+bin, Rust). `ironc run <f.irn>` → Rust via `TokenStream` (I10) →
`prettyplease` → `cargo` → exécution.

| Critère (README §6) | État | Preuve |
|---|---|---|
| e2e programme représentatif (struct, fn, if/while/for, récursion, list/dict, Option/Result, `?`, **2 syntaxes**) | ✅ | `tests/fixtures/gate.irn` → `25 / 15 / 55 / Some(20) / 0 1 2` |
| `#![forbid(unsafe_code)]` → zéro `unsafe` (I2) | ✅ | présent en tête de chaque crate généré |
| source maps : erreur `rustc` → `Span` Iron (I8) | ✅ | `type_error.irn` → `…:3:5: error[E0277]…` (remonté de `main.rs:19`) |
| déterminisme | ✅ | passes pures, sortie stable |
| fallback I1 (clone) compile toujours | ✅ | places non-`Copy` clonées par défaut |

7 fixtures vertes : `factorial, collections, result, structs, braces, gate, type_error`. `cargo test` : 3/3.

## Pistes livrées

- **2.1 lexer** (`lex.rs`) : off-side rule + suspension accolades + spans octets + erreurs lexicales. ✅
- **2.2 parser** (`parse.rs`) : grammaire V0, `NodeId`, double syntaxe `:`/`{}`, `S(x=…)`→StructLit. ✅
- **2.3 inférence** (`infer/`) : monotypes, structs, collections, index/`for`. ✅ (flux Option/Result : cf. D1)
- **2.4 résolveur** (`resolve.rs`) : cf. D2 (clone-default dans le lowering). 🔶
- **2.5 lowering** (`lower.rs`) : tout le sous-ensemble + marqueurs + place/valeur (clone I1). ✅
- **2.6 source maps** (`sourcemap.rs` + driver) : relocalisation post-`prettyplease` + remontée `cargo`. ✅
- **2.7 driver** (`driver.rs`) : `ironc build/run`, projet cargo autonome, capture diagnostics. ✅
- **3.1 unification** : réalisée en **incréments toujours-verts** (cf. note méthode). ✅

> **Note méthode.** Le fan-out vague 2 a été exécuté **en incréments séquentiels gardant l'e2e vert**
> (front-end → collections → Option/Result → structs → accolades → remontée), pas en 7 agents parallèles :
> pour un compilateur, la cohérence inter-passes prime (le « CORRECTEMENT » de la planif). Résultat identique,
> intégration continue plutôt que big-bang.

## Écarts & dettes → cadrage V1

- **D1 — Option/Result par annotation, pas par flux.** V0 : `-> Option[T]`/`Result[T]` + `some/none/ok/err`.
  `flow.rs` est un no-op. L'inférence pure par le flux (I11) reste à faire. **Dette.**
- **D2 — Résolveur = clone-total.** Le lowering clone toute place non-`Copy` lue comme valeur (fallback I1,
  compile toujours). Pas de passe de liveness « move à la dernière utilisation » ; `OwnershipMap` non peuplée.
  **Cohérent avec le découpage : l'optimisation move/emprunt EST V1 (30200).**
- **D3 — Builtins hors sous-ensemble formel** : `print`, `len`, `range`, `some/none/ok/err`. `print` est
  nécessaire à toute sortie observable. À cadrer proprement (stdlib V4 ?).
- **D4 — Types de params/retour REQUIS (annotés).** Pas d'inférence des types de paramètres.
- **D5 — Pas de `:`-bloc dans un `{}`-bloc** (indentation suspendue dans les accolades). Limite §1bis documentée.
- **D6 — `NodeId` en ordre de construction** (dense, déterministe) plutôt que strict pré-ordre DFS ; petits
  trous (parenthèses). Fonctionnellement suffisant.
- **D7 — Scoping Python : liaisons ✅ corrigé en phase 4** (hoisting des locales `Let` en tête de fonction).
  **Reste D7b** : variable de boucle `for` utilisée *après* la boucle → block-scopée Rust → erreur. Mineur. **Dette V1.**
- **D8 — Source map granularité instruction** (cible expression annoncée). Mécanisme marqueur prouvé ;
  descente expression = raffinement.
- **D9 — Annotations de type non strictement vérifiées** (inférence best-effort) ; `rustc` reste le filet
  (I2) et la remontée fonctionne.
- **D10 — Comparaisons chaînées** `a < b < c` non supportées (parsées `(a<b)<c`, sémantique ≠ Python).
  À cadrer (sucre `and`). **Dette.**

## Phase 4 — boucle de vérité (2026-06-24)

11 programmes fonctionnels verts. Sondage adverse (10 sondes `s_*`/`p_*`) → **1 bug critique trouvé ET corrigé**
(D7 affectation conditionnelle, pattern courant) + **2 gaps mineurs documentés** (D7b var-de-boucle-après, D10
comparaison chaînée). Liste vide, struct retournée, clés string, récursion mutuelle, flottants, booléens, collections
imbriquées : ✅. **Métrique de vérité** (correction `Iron→Rust` sur corpus + coût clone-fallback) : toujours **différée**
faute de corpus — on ne chiffre pas sur données insuffisantes.
