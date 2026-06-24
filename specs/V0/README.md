# Iron — V0 / cadrage (`README.md`)

> **Fondation de la version V0** (phase 1 — cadrage). Gèle le **sous-ensemble source** et le **contrat de données partagé** (IR) que les 8 specs `V0/XXXXX.md` élaborent ensuite.
> Carte de la version : `V0/generated.md`. Gabarits / invariants / nommage : `../generator.md`.
> Principe : **on gèle ici le SCHÉMA partagé + les conventions** avant tout fan-out de specs (anti-collision, cf. `CLAUDE.md` phases 2–3).
> Statut : **🔶 proposé** — décisions de cadrage (1–4) intégrées ; reste les 🔶/❓ du §7.

## 1. Périmètre source de V0

> Objectif **« max dès le départ »** (décision utilisateur) : on maximise la surface **non génériquement-difficile** et on **repousse seulement le dur**. V0 devient costaud — la réalisation reste **squelette-d'abord** (colonne fine qui compile, puis fan-out), donc pas de big-bang.

**Inclus :**
- **Items** : `fn f(params) -> T:` · `struct S:`. Un fichier `.irn` = un module = des items de top-level.
- **Contrôle de flux** : `if/elif/else`, `while`, **`for x in (range(...) | collection)`**, **`break`**, **`continue`**, `return [expr]`, `pass`.
- **Types** (monotypes **côté utilisateur**, I6) : `i64`, `f64`, `bool`, `str`, `()` ; structs ; `Option<T>`/`Result<T>` **inférés par le flux** (I11–I13) ; **collections natives `list[T]`→`Vec<T>`, `dict[K,V]`→`HashMap<K,V>`** (types paramétrés **natifs**, pas de génériques *utilisateur*).
- **Instructions** : liaison `x = expr` (copie, I1) · affectation (`x = …`, `e.f = …`, `e[i] = …`) · `if` · `while` · `for` · `return` · `break` · `continue` · instruction-expression · `pass`.
- **Expressions** : littéraux (`int`→`i64`, `float`→`f64`, `bool`, `str`, **littéral `list` `[…]`**, **littéral `dict` `{k: v}`**) · variable · unaire (`-`, `not`) · binaire (arith `+ - * / %`, comparaison `== != < <= > >=`, booléen `and/or`) · appel `f(args)` · construction struct **`S(champ=expr, …)`** · accès champ `e.champ` · **indexation `e[i]`** · sucre `?` (propagation `Result`, I11).
- **Double syntaxe de bloc** (§1bis).
- **Sémantique Option/Result** (I11–I13) : chemin « pas de valeur » → `Option<T>` ; opération faillible / `?` → `Result<T, IronError>` (I13). `NaN` reste `f64` (I12).

**Repoussés (génériquement durs / par découpage) :**
- génériques **utilisateur** + traits → V3 · `match` + `enum` **utilisateur** → V3 (séparables, cf. note) · closures/lambda → ultérieur · interop `rust:` → V2 · stdlib au-delà des primitives+collections → V4 · imports multi-fichiers → ultérieur.
- élision d'emprunt → V1 · **syntaxe d'emprunt source : jamais** (l'emprunt est une sortie du résolveur, I1).

> **🔶 Tension assumée** : les collections natives imposent des **types paramétrés natifs** (`Vec`/`HashMap`) dans le `TypeMap` — **entaille contrôlée** dans « monotypes only ». Ça ne donne **pas** les génériques utilisateur (toujours V3), mais ça grossit `20100`/`20200`/`40100`. Inclus pour le « max » ; **vetoable** pour un V0 plus mince. *(NB : `Vec`/`HashMap` de la std Rust sont une cible de codegen, pas la « stdlib Iron » du V4.)*

> **Note enum/match** : `Option`/`Result` sont déjà des enums gérés nativement par le flux (`20200`). Les enums **utilisateur** + `match` exhaustif restent en V3 (bundle `20300`), **séparables** des génériques/traits si tu veux les tirer en V0 — à dire.

## 1bis. Double syntaxe de bloc (Python ⟷ accolades)

**Tout bloc s'écrit au choix**, décidé **par bloc** :
- **Python** : `entête:` puis **bloc indenté** (off-side rule, indentation significative).
- **Accolades** : `entête { … }` (**sans** `:`) — indentation **non** significative à l'intérieur, instructions séparées par `Newline`/`;`.

**Désambiguïsation figée : `:` ⇒ indentation · `{` ⇒ accolades.**

Coût assumé (le plus cher au regard de sa valeur — sucre syntaxique). Points durs **délégués à `10100`/`10200`** : (a) `{…}` **bloc** vs `{k: v}` **littéral dict** → désambiguïsé par la **position** (instruction vs expression) ; (b) imbrication inter-styles ; (c) le lexer **suspend `Indent`/`Dedent`** entre `{` et `}` apparié.

## 2. Pipeline figé (I9)

```
.irn ──Lexer(10100)──▶ Vec<Token>
     ──Parser(10200)──▶ Ast (+ NodeId, Span)
     ──Inférence(20100/20200)──▶ TypeMap
     ──Résolveur(30100)──▶ OwnershipMap
     ──Lowering(40100)──▶ TokenStream + SourceMap
     ──prettyplease──▶ Rust source
     ──cargo(70100)──▶ binaire
            ▲
   SourceMaps(60100) : Span .irn ↔ Rust, remontée diagnostics rustc
```

Codegen **exclusivement** via `proc_macro2::TokenStream` (I10) — jamais de concaténation de chaînes. **Aucun `unsafe` émis** (I2), garanti au niveau crate (§4.8).

## 3. Clé de jointure partagée : `NodeId` + `Span`

Linchpin anti-collision : **chaque nœud AST porte un `NodeId` et un `Span`.** Toutes les passes communiquent par **tables annexes indexées par `NodeId`** — l'AST reste **immuable** après le parse, les passes sont des fonctions pures.

- **`Span` = `{ file: FileId, start: u32, end: u32 }`** : offsets **OCTETS** (demi-ouverts `[start, end)`). Unité figée = **octet**. Ligne/colonne dérivées à la demande.
- **`NodeId` = `u32`** : attribué par le parser en **pré-ordre DFS**, dense à partir de 0. Clé partagée entre `Ast`, `TypeMap`, `OwnershipMap`, `SourceMap`.

## 4. Contrat de données partagé (gelé)

> Schémas **conceptuels** (le Rust exact d'`ironc` est détaillé dans chaque spec).

### 4.1 Token — produit par `10100`
`Token = { kind: TokenKind, span: Span }`. `TokenKind` ∈ littéraux | identifiant | mots-clés (`fn struct if elif else while for in break continue return pass and or not true false`) | opérateurs/ponctuation (`{ } ; [ ] ( ) . , : = -> + - * / % == != < <= > >= ?`) | **synthétiques `Indent`/`Dedent`/`Newline`** (I9). Indentation = **espaces** ; tabulation/mélange = **erreur lexicale** (quand l'off-side est actif). Le lexer **suspend `Indent`/`Dedent`** dans une région `{ … }`.

### 4.2 Ast — produit par `10200` (immuable)
- **Item** : `Fn { name, params: [(name, Type)], ret: Type, body: [Stmt] }` · `Struct { name, fields: [(name, Type)] }`
- **Stmt** : `Let{name,value}` · `Assign{target,value}` *(target ∈ Var|Field|Index)* · `If{cond,then,elifs,else}` · `While{cond,body}` · **`For{var,iter,body}`** · **`Break`** · **`Continue`** · `Return{value:Option<Expr>}` · `ExprStmt` · `Pass`
- **Expr** : `Lit` · `Var{name}` · `Unary` · `Binary` · `Call{callee,args}` · `StructLit{name,fields}` · `Field{base,name}` · **`Index{base,idx}`** · **`ListLit{items}`** · **`DictLit{pairs}`** · `Try{inner}`

Un `body`/bloc = **liste de `Stmt`**, **identique quelle que soit la syntaxe source** (`:`/`{}` produisent le même AST ; seul le `Span` retient l'origine). Chaque nœud = `{ id: NodeId, span: Span, kind }`. **Types et ownership hors AST** (tables annexes).

### 4.3 Type — vocabulaire partagé inférence ↔ lowering
`Type ∈ { I64, F64, Bool, Str, Unit, Struct(name), Option(Type), Result(Type), List(Type), Dict(Type,Type) }`. Monotypes côté utilisateur (V0) ; `List`/`Dict` = paramétriques **natifs** seulement. `IronError` = le `E` de tout `Result` (I13).

### 4.4 TypeMap — produit par `20100`/`20200`
`TypeMap : NodeId → Type`. Tout `Expr` a un monotype résolu. `20100` = locaux/structs + **éléments de collections** (I6) ; `20200` = `Option`/`Result` par flux (I11–I13).

### 4.5 OwnershipMap — produit par `30100`
`OwnershipMap : NodeId(lecture d'un `Var`) → Disposition`, `Disposition ∈ { Clone, Move }`. V0 : **clone défaut** (I1) + **move dernière utilisation**. Itération `for x in coll` = clone des éléments (I1-cohérent). **`Borrow` hors V0** (→ V1). I3 non déclenché (pas d'emprunt).

### 4.6 SourceMap — produit par `40100`, consommé par `60100`
**Granularité cible V0 = expression** (décision « max »). **Dérisquage** : le squelette prouve d'abord la **survie au reformatage `prettyplease`** (le point dur, risque n°1) à la granularité **instruction**, puis on descend à l'expression. Mécanisme : marqueurs liés au `NodeId`, ré-localisés par `60100` après `prettyplease`.

### 4.7 Sortie codegen — `40100`/`70100`
`ironc build <f.irn>` → projet cargo : `Cargo.toml` + `src/main.rs` (Rust `prettyplease`). `ironc run` = build + `cargo run`. Un `.irn` → un `main.rs` (mono-fichier V0).

### 4.8 Invariants émis dans le crate généré
- `#![forbid(unsafe_code)]` en tête de crate → **I2 garanti par le compilateur**.
- Définition unique du type `IronError` (I13) injectée dans le crate.

## 5. Interfaces inter-composants (table anti-trou)

| Spec | Produit | Consomme | Invariants |
|---|---|---|---|
| 10100 lexer | `Vec<Token>` (+ Indent/Dedent/Newline, suspension accolades), `Span` | source `.irn` | I9 |
| 10200 parser | `Ast` (+ `NodeId`, `Span`) — double syntaxe → même AST | `Vec<Token>` | I9 |
| 20100 inférence locale | `TypeMap` (monotypes, structs, collections) | `Ast` | I6 |
| 20200 Option/Result | `TypeMap` (Option/Result) | `Ast`, `TypeMap` | I6, I11, I12, I13 |
| 30100 résolveur | `OwnershipMap` (Clone/Move) | `Ast`, `TypeMap` | I1, I2, I4 |
| 40100 lowering | `TokenStream` + `SourceMap` | `Ast`, `TypeMap`, `OwnershipMap` | I9, I10 |
| 60100 source maps | `Span` .irn↔Rust + remontée `rustc` | `SourceMap`, sortie `cargo` | I8, I2 |
| 70100 ironc/cargo | projet cargo, `build`/`run` | `TokenStream`+`SourceMap` (via prettyplease) | — |

## 6. Critères d'acceptation V0 (le gate)

1. **e2e** : un `.irn` représentatif (fn + struct + `if`/`while`/`for` + récursion + `list`/`dict` + `Option`/`Result` + `?`, **dans les deux syntaxes de bloc**) → `ironc build` → `cargo` **vert** → `ironc run` → **sortie correcte**.
2. **I2 compilateur** : `#![forbid(unsafe_code)]` présent → zéro `unsafe`.
3. **Source maps (I8)** : map à la granularité **expression** ; une erreur `rustc` forcée **remonte au `Span` Iron** exact.
4. **Déterminisme** : même entrée → même Rust (modulo `prettyplease`).
5. **I1 fallback** : tout programme compile **au pire** par clone — jamais d'`unsafe`/corruption ; pire cas = erreur `rustc` remontée (I2).

## 7. Décisions de cadrage (tranchées)

- ✅ **Collections natives** (§1, §4.3–4.4) : **en V0**. N'étaient planifiées **nulle part** (`generator §8` ne les nomme pas ; `80100`/stdlib = wrappers std, pas la syntaxe collection du langage) → règle utilisateur « non-planifié ⇒ V0 ».
- ✅ **enum/match utilisateur** : **en V3**. **Effectivement planifiés** (`20300`) → laissés en place (règle « déjà planifié ⇒ on n'avance pas »).
- ✅ **Construction struct** `S(champ=expr)`.
- ✅ **Double syntaxe de bloc** (`:`/`{}`), détails → `10100`/`10200`.
- ✅ **Source map** : cible **expression**, dérisquée via instruction.
- ✅ **Indentation** : espaces only (tab/mélange = erreur lexicale quand off-side actif) ; suspendue dans les accolades.

## 8. Suite (après validation de cette fondation)

Fan-out des **8 specs** `V0/XXXXX.md` (gabarit `generator §4`), chacune élaborant sa ligne du §5 contre les schémas gelés §3–4. **`10100`/`10200` portent une charge de design en plus** (double syntaxe + collections). **Ordre de dérisquage : `60100` (source maps) prioritaire** dès que `40100` produit du Rust. Puis phase 2 (planification `V0/planification/X.Y.md`).
