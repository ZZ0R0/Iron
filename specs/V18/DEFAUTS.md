# V18 — Carte des défauts (audit éventail 1.2, **mesurée**)

> Produit par le fan-out 1.2 : 7 agents, 1 par catégorie, protocole d'oracle honnête (sortie correcte
> calculée à la main → `ironc run` → comparaison). **57 cas verts** au corpus (`ironc/tests/corpus/`),
> **10 défauts** parqués dans `<cat>/_defects/` (hors garde, repro prêts pour les vagues de correction).
> Ceci n'est PAS une hypothèse : chaque défaut a un `.irn` qui le reproduit.

Statuts : ✅ corrigé · 🔧 à corriger · ❓ décision de fond · 🔶 proposé

---

## A. Closures — capacité (→ vague 2 · [21000])
- **A1** 🔧 `inline_lambda` — IIFE `(lambda x: x*2)(21)` mal abaissé : l'argument est absorbé **dans** le
  corps → `move |x| (x*2)(21)` au lieu de `(|x| x*2)(21)`. E0282. *Bug structurel de lowering/précédence.*
  Repro : `closures/_defects/inline_lambda.irn`.
- **A2** 🔧 `stored_in_field` — `obj.f(6)` où `f: fn(i64)->i64` est un **champ** → pris pour un **dispatch de
  méthode** sur le type → « pas de méthode `f` ». Rejet frontend avant codegen. Repro : `closures/_defects/stored_in_field.irn`.
- **A3** 🔧 Stockage de closures **capturantes/hétérogènes** (`Vec`/champ) : Iron émet des types-closure
  distincts non boîtés → E0308. Manque **`Box<dyn Fn>`** (cf. `lower_type` `Type::Fn → impl Fn`, valide en
  arg/retour seulement). `stored_in_list` n'est vert **que** car ses lambdas sont **non-capturantes** (coercion
  `fn`-pointer). Repro : variante capturante (scratchpad de l'agent).
- **Déjà VERT** (pas à toucher) : lambda locale, capture d'env, ordre-supérieur `apply(f,x)` (monomorphisé),
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

## Synthèse pour l'exécution
| Bucket | Défauts | Vague cible | Nature |
|:--|:--:|:--:|:--|
| A closures | A1·A2·A3 | **2** [21000] | capacité (IIFE, field-call, `Box<dyn Fn>`) |
| B numérique | B1·B2·B3·B4 | **3** [21800] | cast/promotion/mod — **petits, localisés** |
| C codegen | C1·C2·C3 | **3** [21800] | index nég · clone dunder · impl générique |
| D parseur | D1 | **3** [21800] | destructuring en compréhension |
| E ergonomie | E1 | **3** [21800] | `?`-main |
| F fidélité Python | F1·F2·F3 | **3** (❓) | décision de fond |

**Lecture** : la vague 2 reste les closures (3 trous nets). La vague 3 absorbe un **backlog de correction réel**
(B/C/D/E = 9 défauts, la plupart de petits fixes project-global localisés) que l'audit a **mesuré** — c'est
exactement ce que la suite [70400] devait révéler. F = à trancher avec l'utilisateur (fidélité Python).
