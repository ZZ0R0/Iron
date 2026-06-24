# Iron — V4 / cadrage (`README.md`)

> **Fondation V4 = stdlib** (spec `80100`) : un **prélude de built-ins**, wrappers **fins** sur `std`
> (I5 : pas de runtime lourd ; I7 : interop). V2 (interop) donne déjà l'accès à `std`/aux crates ; V4 = la
> **couche de confort intégrée** (sans `use` manuel). Parent : `specs/generated.md`. Contrat : `../generator.md`.
> Statut : **🔶 proposé** (décidé : approche built-ins).

## 1. Décision de conception
**Built-ins** (comme `print`/`len`/`range`), **pas** des `.iri` : les opérations sur `list[T]` sont
**génériques**, non exprimables en `extern fn` monomorphe (50200). Un built-in se projette directement
(`.abs()`, `.push()`, …) — wrapper fin, I5/I7 respectés.

## 2. Périmètre (prélude)
- **Conversion/maths** : `str(x)` (→ `String`), `abs(x)`, `min(a,b)`, `max(a,b)`, `sqrt(x)` (`f64`).
- **Listes** : `push(lst, x)` (mute), `sum(lst)` (`list[i64]`→`i64`).
- **Chaînes** : `concat(a, b)` (→ `String`).

## 3. Hors-périmètre
IO (lecture stdin), tri, map/filter (closures → ultérieur), conteneurs avancés.

## 4. Comportement (Iron → Rust)
`str(42)`→`(42).to_string()` · `abs(-3)`→`(-3).abs()` · `sqrt(9.0)`→`(9.0).sqrt()` · `push(xs, 5)`→`xs.push(5)` ·
`sum(xs)`→`xs.iter().sum::<i64>()` · `concat(a, b)`→`format!("{}{}", a, b)`.

## 5. Critères d'acceptation V4 (gate)
Un programme utilisant `str/abs/min/max/sqrt/push/sum/concat` → `cargo` vert + sortie correcte ; zéro `unsafe` ;
corpus V0/V1/V2/V3 **vert**.

## 6. Suite
Fan-out `80100` : ajouter les built-ins à l'inférence (types de retour) + au lowering (`call()`). 1 piste.
