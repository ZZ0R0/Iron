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
- **Run réel** : `Vector3.new(3.0,4.0,0.0).norm()` → **5** (unaire), `a.dot(b)` → **32** (binaire). nalgebra
  est **pleinement opérable** : construction + méthodes héritées unaires ET binaires.
- **Fix associé** : `extern_arg` passait un `ExternApp` (type générique appliqué, ex. `Vector3[f64]`) **par
  valeur** au lieu de `&` (il couvrait `Extern` mais pas `ExternApp`) → les méthodes binaires héritées
  (`a.dot(b)` veut `&Matrix`) échouaient en E0308. Ajout d'`ExternApp` à l'arm `&` : cohérent avec `Extern`,
  les params `own` restent interceptés en amont. **Zéro régression** (semver/12 tests verts).

## 🔬 Investigué avec preuves — pas des gains nets comme C1

### C2 (AEAD : aes-gcm) — **LIVRÉ via curation** (auto non généralisable sans risque)
**Pourquoi pas auto** : `ironc add aes-gcm` → 40.6 % ; le rapport pointe le gap exact (`encrypt/decrypt
[UnmappedType: impl Into]`). Preuves que l'auto-propre échoue : plaintext = `impl Into<Payload>` et
**`From<&[u8]> for Payload` = 0 détecté** (lifetimes) → la règle propre `Into<T> où T: From<&[u8]>` ne se
déclenche pas ; nonce = `Array<u8,N>` (hybrid_array) dont la reconnaissance `@arr` **casserait ndarray**
(`Array<u8,_>` ambigu crypto-vs-ndarray) — régression inacceptable sur un crate que C1 vient d'activer.
**Donc : curation** (le filet I7-2, rendu facile par la fondation).

**Livré (run réel)** : `specs/V15/examples/aes_gcm.iri` (couche curée) binde `Aes256Gcm.new/encrypt/decrypt`
(clé/nonce via `try_into`, plaintext `&[u8]`). Round-trip vérifié : chiffrer 3 octets → **19** (3 + tag GCM
16), déchiffrer → **3**. **Zéro nom de crate dans le compilateur**, zéro régression. Démontre le système
deux couches sur un crate dur, exactement comme prévu en [50400].

### C3 (const-génériques portés) — **LIVRÉ** (ABI + parse + test)
Valeur de **langage** (pas de déblocage de crate du corpus : C1 couvre les dimensions, `@arr` les octets).
Implémenté **additivement, sans nouvelle variante de type** (donc zéro churn des matchs exhaustifs) : un
entier en position d'argument de type (`ArrayVec[u8, 16]`) est porté en `Type::Extern("16")` et **émis
littéralement** au lowering → `arrayvec::ArrayVec::<u8, 16>`. Parse ajouté aux **deux** parseurs (`.irn` et
`.iri`). **Vérifié** : test `extern_app_const_arg_lowers` vert (13 tests au total). Aucun crate du corpus ne
l'exerce aujourd'hui (capacité présente, prête à servir via curation pour les crates à cap const).

### C4 (builder/flux : flate2) — **différé** (hors de la demande « C2 C3 puis corpus »)
API à état porté entre appels (`Encoder::new(w).write_all().finish()`) — structurel, classé
`StreamBuilderApi`. Comme C2, c'est un cas de **curation**, pas une généralité auto. Non demandé dans la
dernière directive utilisateur (« C2 C3 puis corpus ») → différé, à curer au besoin.

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
