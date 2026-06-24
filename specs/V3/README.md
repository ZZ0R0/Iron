# Iron — V3 / cadrage (`README.md`)

> **Fondation V3 = système de types avancé** (spec `20300`) : **enums + `match` + traits + génériques**.
> Le morceau dur du découpage. Parent : `specs/generated.md`. Contrat : `../generator.md`.
> Statut : **🔶 proposé** — trancher §5.

## 1. Stratégie clé : projection + `rustc` = vérificateur

L'inférence V0–V2 est **monotype**. Plutôt qu'un Hindley-Milner maison (lourd, risqué), V3 **émet du Rust
générique/trait/`match`** et laisse **`rustc`** faire l'inférence, la monomorphisation et la vérification
d'exhaustivité. Les paramètres de type sont **opaques** côté Iron. C'est exactement I6 (projection propre
d'un sous-ensemble Rust) + I2 (résolveur optimiste, `rustc` = filet, pire cas = erreur remontée I8). **Ça
rend V3 tractable** : Iron projette, Rust vérifie.

## 2. Périmètre (4 features, ordre de construction)

- **Enums** (types somme) → `enum` Rust. *(Option/Result sont déjà des enums natifs.)*
- **`match`** (filtrage) → `match` Rust. **Lève enfin la dette V0** : la consommation terminale
  d'`Option`/`Result` (et des enums) par `match`.
- **Traits + `impl`** → `trait`/`impl` Rust ; appel de méthode `x.m(args)`.
- **Génériques** (`fn`/`struct`/`enum` + bornes) → generics Rust.

## 3. Syntaxe figée (proposée)

- `enum Name:` + variantes indentées : `Variant` | `Variant(T, …)` | `Variant(champ: T, …)`.
  Construction : **`Name.Variant(…)`** (explicite, non ambigu).
- `match expr:` + bras `motif -> expr` (ou `motif:` bloc). **Motifs V3** : littéral, variante `Variant(x)`
  (lie `x`), `_`, variable. **Exhaustivité → `rustc`.**
- `trait Name:` (signatures) ; `impl Name for Type:` (corps). Appel : `x.method(args)`.
- **Génériques : `[T]`** (cohérent avec `list[T]`/`dict[K,V]`). `fn f[T](x: T) -> T:`, `struct Box[T]:`,
  bornes `[T: Trait]`.

## 4. Conventions / IR

- AST : `Item::Enum`, `Item::Trait`, `Item::Impl` ; `Match` (expr/stmt) + `Pattern` ; paramètres `[T]`.
- `Type` gagne `Enum(name)` et `Generic(name)` (param opaque).
- Lowering = **projection directe** (enum→enum, match→match, trait→trait, `[T]`→`<T>`).
- L'inférence annote ce qu'elle peut ; les trous génériques sont **laissés à `rustc`** (remontée I8).

## 5. Décisions ouvertes (fond — à trancher)

- 🔶 **Scope.** **Les 4 d'un coup** (enums → match → traits → génériques, ils s'enchaînent) — recommandé,
  mais les **génériques** sont le point profond. → ok, ou **enums+match d'abord** (= lève la dette V0) puis
  traits+génériques ?
- 🔶 **Syntaxe génériques** : **`[T]`** (cohérent collections) vs `<T>` (familier Rust mais conflit avec `<`
  comparaison). → `[T]` ?
- ❓ **Construction de variante** : `Name.Variant(…)` (explicite) vs `Variant(…)` (court, ambigu avec un appel). → `Name.Variant` ?

## 6. Critères d'acceptation V3 (gate)

1. **Enum + match** : un `enum` filtré par `match` (toutes variantes) → compile + exécute.
2. **Dette V0 levée** : `Option`/`Result` consommés par `match`.
3. **Trait + impl** : une méthode de trait appelée → compile + exécute.
4. **Générique** : une `fn`/`struct` générique instanciée à **≥ 2 types** → compile + exécute.
5. `match` non exhaustif → **erreur `rustc` remontée** (I2). Zéro `unsafe`. Corpus V0/V1/V2 **vert**.

## 7. Suite (après validation)

Fan-out `20300` : enums + `match` (fondation du filtrage) → traits/`impl` → génériques. Détail en phase 2.
