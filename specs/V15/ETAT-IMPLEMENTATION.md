# V15 — ÉTAT RÉEL D'IMPLÉMENTATION (phase 3/4)

> **Honnête, étayé par des runs réels.** 2 livrables majeurs **vérifiés e2e** (fondation à portée connue +
> C1 alias→nominal). C2/C3/C4 : investigués avec **preuves**, classés par tractabilité réelle. Le système
> deux-couches + rapport transforme le reste en **workflow de curation** (le but de [50400]/[50500]).

## ✅ Livré et vérifié

### Fondation (vagues 1-3) — [50500] portée connue + [50400] deux couches
- `project()` : `Option<Sig>` → `Result<Sig, SkipReason>` (taxonomie **fermée** de 13 raisons). Chaque
  abandon devient un `SkipDiagnostic { rust_path, kind, reason, hint }` — **zéro drop silencieux**.
- `bind()` → `BindOutput { iri, diagnostics, Coverage }`. Couverture **calculée** (`bound/seen`).
- `ironc add` écrit **`<crate>.gen.iri`** (en-tête AUTO, régénérable) + **`<crate>.iri.report.md`** +
  résumé stderr ; `import` charge `gen.iri` PUIS `iri` curé (**last-wins, le curé gagne**). `--strict`.
- **Run réel** : `ironc add semver` → « 35 candidats, 28 liés, 7 sautés (80 %) », rapport listant chaque
  skip avec sa raison (`clone_to_uninit`=UnsafeFn, `from_iter`=UnmappedType:I) **avant** toute compilation.
  `import semver` + `Version.new(1,2,3)` → `1 2 3`. 12 tests verts, pipeline cœur intact.

### C1 (vague 4) — [50600] alias monomorphisé → type nominal : **LE MUR DES DIMENSIONS BRISÉ**
- `sig_of()` : signature structurelle (alias `resolved_path` **expansés** → `U3`=`Const<3>`, génériques→`_`,
  consts gardées). Table `alias_shapes` : sig → plus court nom d'alias public à dims concrètes (tie-break
  canonique `Vector3` > alias de vue). Reroutage de l'`impl` vers le **type nominal de l'alias**.
- **100 % général** : zéro nom de crate dans `ironc/src`. La règle lit alias + impl depuis rustdoc.
- **Run réel** : `Vector3.new(1.0,2.0,3.0)` → un vrai `nalgebra::Vector3` affiché (matrice 3×1). nalgebra :
  `Vector2..6.new`, `Matrix2/3.new` bindés ; couverture **39.7 → 40.8 %** (+103 items). Le mécanisme
  s'applique à **tout** crate à alias-de-monomorphisation (ndarray, rustfft… même pattern).
### C1+ — héritage alias→base (extension, FAITE) : nalgebra devient *opérable*
- `ExternType.base: Option<String>` ; bind émet `base Matrix` dans le bloc de l'alias ; infer (`extern_method_ret`)
  et lower (`try_extern_method`) **retombent sur les méthodes du type de base** quand l'alias ne les a pas.
  Un `Vector3` EST un `Matrix` en Rust → la méthode compile. **Zéro nom de crate** (lien lu de l'alias).
- **Run réel** : `Vector3.new(3.0,4.0,0.0).norm()` → **5**. Les méthodes **unaires/scalaires** de Matrix
  (norm, magnitude, …) sont héritées et marchent.
- **Limite honnête mesurée** : les méthodes **binaires** prenant le **type de base en argument**
  (`dot(rhs: Matrix)`, `+`, `cross`) **échouent** (E0308) — le Rust généré n'unifie pas `Vector3` avec le
  paramètre `Matrix`. Donc nalgebra est **opérable en unaire** (construction + norm/magnitude…), **pas encore
  en binaire**. Fixer = traiter un arg de type-base comme « accepte tout alias de cette base » (non fait).

## 🔬 Investigué avec preuves — pas des gains nets comme C1

### C2 (AEAD : aes-gcm/chacha20) — **rabbit-hole, pas une généralité propre**
`ironc add aes-gcm` → 40.6 %. Le rapport pointe le gap **exact** : `AesGcm::encrypt/decrypt
[UnmappedType: impl Into]`. Preuves de la dureté :
- plaintext = `impl Into<Payload>` ; **`From<&[u8]> for Payload` : 0 détecté** (lifetimes `'msg,'aad`) → la
  règle générale **propre** « `Into<T>` où `T: From<&[u8]>` » **ne se déclenche pas**. La seule alternative
  (`impl Into<X>` → octets pour tout X) est l'**heuristique ambiguë** str-vs-bytes déjà rejetée (V12).
- nonce = `Array<u8, NonceSize>` (typenum) : handle **opaque**, non-constructible côté Iron sans règle dédiée.
- **Verdict honnête** : AEAD = soit heuristiques crate-flavored (interdit), soit **curation** `<crate>.iri`
  (le rapport donne le hint). Ce n'est PAS le cas propre de C1.

### C3 (const-génériques portés hors-alias)
**Valeur marginale faible** depuis C1 : le cas dimensions (le seul du corpus) est déjà couvert par les
alias (C1). Les const-génériques **sans alias dédié** sont rares dans les 80. ABI `ExternArg` non implémentée
(touche ir.rs/lower/lsp/iri pour un gain marginal). **Différé** — à activer si un crate du corpus l'exige.

### C4 (builder/flux : flate2) — structurel, non investigué en profondeur
API à état porté entre appels (`Encoder::new(w).write_all().finish()`). Hors d'une fn one-shot ;
le rapport le classera `StreamBuilderApi`. **Le plus dur** ; curation probable.

## Plafond honnête après V15 (partiel)
- **Mécanisme décisif livré** : le moteur **connaît et dit** sa portée (exigence #1) + **C1 brise le mur des
  dimensions** (le plus gros levier auto). Tout est poussé, vert, vérifié e2e.
- **Le reste = couverture incrémentale** via deux voies que le système supporte **maintenant** : (a) plus de
  règles générales (seulement quand une vraie généralité existe, comme C1 — PAS comme C2), (b) **curation
  `<crate>.iri`** guidée par le rapport (la voie sanctionnée I7-2 pour les cas durs : AEAD, flux).
- **Cible 60/80** : non atteinte ici. Honnêtement, elle dépend surtout de **curation** pour la longue traîne
  dure (AEAD, flux, API exotiques), car les vraies *généralités* auto-dérivables sont rares au-delà de C1.

## Décision en attente (fond — utilisateur)
La V12/V13 promettait C1-C4 comme « capacités ». La réalité mesurée : **C1 est une vraie généralité (faite),
C2-C4 sont surtout des cas de curation**. Deux directions :
1. **Curation guidée** : utiliser le rapport pour curer la longue traîne (aes-gcm, flate2…) à la main —
   sûr, c'est le design I7-2, mais « binding manuel » que tu voulais éviter.
2. **Plus d'auto** : ne chercher de nouvelles règles générales que là où une vraie généralité existe
   (incertain au-delà de C1) — ROI faible, beaucoup d'effort pour peu de crates.

> **Note de méthode (écart phase 4).** Modularisation de `bind.rs` différée (refonte in-place pour préserver
> le vert ; la modularisation ne servait qu'un fan-out d'agents, écarté car le moteur est trop couplé). C3
> différé (marginal post-C1). C2/C4 classés curation après preuves.
