# V18 — Carte des défauts (audit éventail 1.2, **mesurée**)

> Produit par le fan-out 1.2 : 7 agents, 1 par catégorie, protocole d'oracle honnête (sortie correcte
> calculée à la main → `ironc run` → comparaison). **57 cas verts** au corpus (`ironc/tests/corpus/`),
> **10 défauts** parqués dans `<cat>/_defects/` (hors garde, repro prêts pour les vagues de correction).
> Ceci n'est PAS une hypothèse : chaque défaut a un `.irn` qui le reproduit.

Statuts : ✅ corrigé · 🔧 à corriger · ❓ décision de fond · 🔶 proposé

---

## A. Closures — capacité (vague 2 · [21000]) — ✅ CORRIGÉ
- **A1** ✅ IIFE `(lambda x: x*2)(21)` : callee lambda/closure désormais **parenthésé** au lowering →
  `(|x| x*2)(21)`. Cas vert : `closures/inline_lambda.irn`.
- **A2** ✅ `obj.f(6)` (champ-closure) : (1) infer accepte un **champ** appelé (pas que les méthodes) ;
  (2) lower détecte le champ-closure (`closure_fields`) ou un callee de type `Fn` → émet `(obj.f)(6)`.
  Cas vert : `closures/stored_in_field.irn`.
- **A3** ✅ (champ) — `lower_storage_type` : un champ `fn(...)` devient **`Box<dyn Fn>`** (impl Fn interdit
  en stockage) ; la valeur lambda est **boîtée** (`Box::new(...)`) ; la struct porteuse perd `Debug`/`Clone`
  auto (Box<dyn Fn> ne les dérive pas). Cas vert : `closures/stored_in_field.irn`.
  - **A3 écart (0.1%)** 🔧 Littéral de **liste** de closures **capturantes** (`[lambda x: x+base, …]`) :
    besoin de `Vec<Box<dyn Fn>>` + boxing/coercion par élément dans le littéral. Non fait (capture non triviale,
    flaggée 0.1% par planify-2.1). **Contournement** : champ de struct (A3) ou lambdas non-capturantes
    (`stored_in_list`, vert). Repro parqué : `closures/_defects/captured_list.irn`.
- **Déjà VERT avant V18** : lambda locale, capture d'env, ordre-supérieur `apply(f,x)` (monomorphisé),
  `map`/`filter`, **closure retournée chaînée `adder(10)(5)`**.

## B. Numérique — ergonomie + correction (→ vague 3 · [21800])
- **B1** 🔧 Pas de **promotion i64→f64** dans une op binaire : `2 + 3.5` rejeté (`f64 attendu, trouvé i64`).
  Python promeut → `5.5`. Repro : `maths/_defects/mix_int_float.irn`.
- **B2** 🔧 `int(f64)` abaissé en `(x).trim().parse::<i64>()` (suppose une **chaîne**) → E0599 sur `f64`.
  Attendu : **troncature** `x as i64`. Repro : `maths/_defects/int_truncate.irn`. *(= vague 3(c) confirmé.)*
- **B3** 🔧 `float(i64)` abaissé en `(x).trim().parse::<f64>()` → E0599 sur `i64`. Attendu : `x as f64`.
  Repro : `maths/_defects/float_cast.irn`.
- **B4** 🔧 `%` sur négatifs : reste **tronqué** Rust (`-7 % 2 = -1`) alors que `//` = `div_euclid`
  (`-7 // 2 = -4`) → **conventions mélangées**, viole `(a//b)*b + (a%b) == a`. Attendu (Python floor) :
  `%` → `rem_euclid`. Repro : `maths/_defects/neg_floordiv_mod.irn`.

## C. Codegen — correction (→ vague 3 · [21800], passe correction)
- **C1** 🔧 Index de liste négatif `xs[-1]` → `#b[(i) as usize]` sans normalisation → **panic usize**, alors que
  slices ET index de `str` gèrent le négatif. `lower.rs:~1108`. (Source-vérifié par l'agent collections.)
- **C2** 🔧 **Clone-fallback de dunder incohérent** : un opérande réutilisé comme **rhs** d'un dunder de
  comparaison (`==`/`<`) n'est pas cloné → E0382 « borrow of moved value » ; il l'EST s'il sert aussi à
  `__sub__`/`__mul__`. Réduction fidèle de `v11/dunders.irn`. Territoire ownership V17. Repro : `structs_enums/_defects/money_dunders.irn`.
- **C3** 🔧 **Struct générique + méthode** : le bloc `impl` omet `<T>` → `impl Box {` + `fn get(&self) -> T`
  → E0107/E0425. Déclencheur = présence d'une méthode (sans méthode, `Pair[A,B]` compile). Repro : `structs_enums/_defects/generic_box.irn`.

## D. Parseur (→ vague 3 · [21800])
- **D1** 🔧 Compréhension `{k: v for k, v in items}` : destructuring tuple **non parsé** en compréhension
  (`parse_comp` exige UN motif puis `in`), alors qu'il marche en **instruction** (`for k,v in`) — et que
  `language.md:96` le documente valide. **Asymétrie** for-instruction / for-compréhension. Repro : `collections/_defects/dict_from_pairs.irn`.

## E. Ergonomie (→ vague 3 · [21800])
- **E1** 🔧 `?`-dans-`main` : `?` hors `try` abaissé en `(expr?)` brut + `main` non auto-promu → **E0277 rustc
  brut** fuit (la source-map localise bien la ligne ; c'est le **message** qui pèche). Confirme gotcha #1.
  Attendu : auto-promotion de `main` à `Result` si `?`/`raise`, OU diagnostic Iron-natif. Repro : `errors/_defects/q_in_main.irn`.

## F. Divergences sémantiques Python (❓ à trancher — directive de fond)
> Non-bloquantes, **non** enshrinées en vert par les agents (honnêteté). Iron = syntaxe Python / codegen Rust :
> jusqu'où pousser la **fidélité Python** ? Ma reco entre parenthèses.
- **F1** ❓ `int("abc")` → `0` au lieu d'une exception (Python `ValueError`). *(Reco : corriger — réponse
  silencieusement fausse ; mais edge.)*
- **F2** ❓ `print(True)` → `true`/`false` (Display Rust) au lieu de `True`/`False`. *(Reco : Display bool
  custom Python — cosmétique mais visible pour un langage à syntaxe Python.)*
- **F3** ❓ Flottant entier `print(1.0)` → `1` (Display Rust drop `.0`) ; collections via `{:?}` (chaînes entre
  guillemets). *(Reco : documenter comme intentionnel OU Display float custom. Trancher en vague 3.)*

---

## Synthèse — ÉTAT FINAL
| Bucket | Défauts | Vague | État |
|:--|:--:|:--:|:--|
| A closures | A1·A2·A3 | 2 [21000] | ✅ (A3 écart liste-capturante 0.1% noté) |
| B numérique | B1·B2·B3·B4 | 3 [21800] | ✅ cast/promotion/mod/index |
| C codegen | C1·C2·C3 | 3 [21800] | ✅ index nég · **clone dunder (last-use)** · impl générique |
| D parseur | D1 | 3 [21800] | ✅ destructuring en compréhension |
| E ergonomie | E1 | 3 [21800] | ✅ `?`-main auto-promu Result |
| F fidélité Python | F1·F2·F3 | — | ❓ **décision de fond** (documenté gotchas #13, défaut « projection Rust ») |

**12/12 défauts durs corrigés**, project-global, sans régresser V17 (raytracer 7 clones, ~0,09 s). Seul **F**
reste ouvert = un choix de fond (projection Rust vs fidélité Python sur l'affichage/parse). Détail : [`ETAT-IMPLEMENTATION.md`](ETAT-IMPLEMENTATION.md).
