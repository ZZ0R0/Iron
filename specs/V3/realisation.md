# Iron — V3 / réalisation (trace)

> Phase 3 terminée. **Gate V3 vert (2026-06-24).** Système de types avancé, **les 4 features**.
> Stratégie : **projection vers Rust + `rustc` = vérificateur** (I6/I2), inférence Iron légère.

## Gate V3 — atteint

| Feature | Preuve |
|---|---|
| **Enum + match** | `enum_match.irn` → `12 / 12 / 42 / -1` |
| **Dette V0 levée** | `Option` consommé par `match` (`Some(x)`/`None`) |
| **Génériques** `[T]`→`<T>` | `generics.irn` → `42 / true / 7 / 10` (fn générique à 2 types + struct générique) |
| **Trait + impl + méthode** | `traits.irn` → `woof / woof / 3` (méthode appelant `self.sound()`) |
| `match` non-exhaustif → `rustc` remonté (I2) | `nonexhaustive.irn:7:5: error[E0004]` |
| Corpus V0/V1/V2 vert · zéro `unsafe` · tests | **18/18** · `forbid` · `cargo test` 3/3 |

## Livré
- **Enums** : variantes tuple (`Circle(f64)`, `Empty`) → `enum` Rust ; construction `E.V(…)` → `E::V(…)`.
- **`match`** (instruction) : motifs variante (`E.V(x)`, `Some(x)`, `None`), `_`, variable, littéral → `match` Rust.
- **Génériques** : `[T]`/`[T: Trait]` sur fn/struct/enum → `<T>` ; `Name[args]` → `Name<args>` (`Type::App`).
- **Traits/impl** : `trait`/`impl T for Ty` → Rust ; `self` → `&self` ; appel `x.m()` via la chaîne d'appel.
- **Bonus** : `is_annotatable` restreint aux primitifs (fixe l'annotation de struct générique) ; `return x`
  = **move** (supprime un clone + permet le retour d'un générique non-`Clone`).

## Écarts & dettes → V4 / au-delà
- **DX1 — Motifs limités** : variante tuple / `_` / variable / littéral. Pas de gardes, ni motifs de struct à
  champs nommés, ni motifs imbriqués profonds. Littéral `str` en motif peut exiger `scrutinee.as_str()`.
- **DX2 — Retour d'un champ générique via emprunt** (`fn first[T](p) -> T: return p.a`) échoue : pas
  d'analyse de move (dette V1 DV1) + exigerait une borne `Clone`. `return x` (var nue) → move, OK.
- **DX3 — Inférence d'appel de méthode = `Unit`** (rustc tranche) ; `print` d'un résultat de méthode utilise
  `{}` (OK primitifs/strings ; `{:?}` parfois souhaitable).
- **DX4 — Construction/motif de variante** : `E.V(…)` (user) ou built-in `Some/None/Ok/Err` ; variante unité
  user en motif via convention majuscule.
- **DX5 — `Type::App`** (struct générique annotée) : accès de champ → inférence incomplète (→ `Unit`) ; rustc filet.
- **DX6 — Sélection d'`impl`** déléguée à `rustc` (pas de résolution de méthode côté Iron).

## Phase 4 (boucle de vérité)
Corpus 18/18 (non-régression V0/V1/V2) + 3 programmes V3 + remontée `match` non-exhaustif. Métrique de
vérité (corpus large) toujours différée.
