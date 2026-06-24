# V6 — Réalisation (notes & gate)

> Trace honnête de la phase 3–4 de V6 : ce qui est **fait + prouvé**, ce qui est **différé** (avec raison),
> et les **faiblesses/dettes** → cadrage V7. (CLAUDE.md : « noter systématiquement ».)

## Gate V6 (phase 4) — ✅ VERT

- **Corpus positif : 37/37** programmes `.irn` compilent (Rust généré vert via `cargo`) **et s'exécutent**
  avec le bon résultat.
- **0 `unsafe`** dans tout le Rust généré (`#![forbid(unsafe_code)]`, I2).
- **3 clones** générés sur tout le corpus, **tous intentionnels** : `Rc::clone` (partage I5) et copie de
  prédicat `filter`. Aucun clone profond accidentel.
- **Multi-fichiers** (`import`) : `mods_demo` (2 modules → 25/30) OK ; cycle → erreur claire.
- **Remontée I8** (tests négatifs) : `type_error` → `:3:5 E0277 cannot add String to i64` ;
  `nonexhaustive` → `:7:5 E0004 Color::Green/Blue not covered` ; chaque erreur `rustc` pointe le **Span
  Iron** exact (ligne:col), jamais un dump Rust brut.
- `cargo test` : 3/3 (squelette + Rust valide + fixtures).

## Specs réalisées + prouvées (10/12)

| Spec | Feature | Preuve (fixture → sortie) |
|------|---------|---------------------------|
| 20400 / **2.1** | I11 : Option/Result **par le flux** | `f_flow` → Some(5)/None/Some(5)/None/42 |
| 30300 / **2.2** | Partage explicite **Rc/RefCell** (cœur, I5) | `r_shared` → 42/100/102 (2 propriétaires) |
| 20700 / **2.3** | **Closures** : lambda, map, filter | `c_closures` → [1,4,9,16]/[3,4]/42/[20,40,60] |
| 20500 / **3.1** | Inférence des **types de params** | `p_infer` → 42 / 7,hi / 1,x |
| 20600 / **3.2** | Motifs match : **gardes + imbrication** | `match_guard` → positif/…/42/30/-1 |
| 30400 / **3.3** | Ownership : **move-hors-struct** (DX2) | `m_moveout` → `unwrap<T>` sans borne Clone |
| 50400 / **4.1** | Interop : **`extern type`** opaque inline | `x_extern` → "duration = 3 s" |
| 70200 / **4.2** | **Modules** multi-fichiers `import` | `mods_demo` → 25/30 ; cycle détecté |
| 80200 / **4.3** | **Stdlib élargie** | `s_stdlib2` → sort/sum/concat/upper/split… |
| 10300 / **4.4** | **Comparaison chaînée** `a < b < c` | `cmp_chain` → petit/moyen/grand/croissant |

## Différé — par conception (2/12)

- **60200 / 3.4 — source maps granularité expression.** La spec elle-même flague 🔶 le mécanisme central
  (marqueurs par expression survivant à `prettyplease` → pollution / déplacement). Le mapping **niveau
  instruction** (I8) **fonctionne et est prouvé** (remontée ligne:col exacte ci-dessus). Une couche de
  marqueurs par expression bâclée régresserait la robustesse au reformatage (dé-risque n°1). → V7, avec un
  mécanisme « marqueur invisible » (`#[line]`-like) cadré.
- **90100 / 5.1 — corpus de validation / métrique de vérité.** Différé **par méthodologie** : la métrique
  exige un **langage gelé** (sinon cible mouvante) et un corpus ≫ 37. Le corpus fonctionnel actuel (37
  programmes, intégration continue) tient lieu de filet ; le **chiffre de correction** sera publié quand le
  langage sera figé (post-V7). Refuser un chiffre sous le seuil > publier un faux.

## Différé — sous-features (dans des specs réalisées)

- **2.2 (30300)** : `weak()`/`Weak` (cycles = charge utilisateur, I5) ; lint aliasing I3 (`--audit`).
- **3.2 (20600)** : variantes à **champs nommés** (syntaxe de liaison ❓-ouverte ; les variantes tuple suffisent).
- **3.3 (30400)** : escape **point-fixe** inter-signatures (DV2, optimisation : moins de clones) ; **scoping
  for-var post-boucle** (D7b — cf. `p_forvar.irn`, échoue proprement « cannot find value `v` » ; subtil à
  cause de l'itérable vide → pas de `Default` générique sûr).
- **4.1 (50400)** : `bind` élargi (str/struct/sous-modules — nécessite nightly `rustdoc-json`, non vérifiable
  ici) ; forme indentée `rust:` (sucre sur `rust {}` qui fonctionne).

## Faiblesses / angles morts → cadrage V7

1. **Print générique** : un `T` retourné s'imprime en `{:?}` (Debug) plutôt que `{}` — sûr (Debug plus
   largement dérivé) mais incohérent avec les types concrets. À cadrer (trait-bound `Display` ? choix explicite).
2. **`filter` Copy/Clone** : prédicat sur **copie** de l'élément (robuste mais clone par élément). À optimiser.
3. **Comparaison chaînée** : opérande médian **dupliqué** → effet de bord évalué 2× (let temporaire à figer).
4. **RefCell** : `borrow()`/`borrow_mut()` → panics runtime possibles (footgun I5) ; le double-emprunt
   trivial est paré (RHS dans un temp), pas les cas complexes.
5. **D7b / point-fixe escape / champs nommés / weak / source-maps expr / bind nightly** : cf. ci-dessus.
