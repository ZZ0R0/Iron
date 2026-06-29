# planify-1.1 — Squelette : call graph + point fixe des modes de paramètres (interprocédural)

- Réalise (cadrage) : [30500]
- Version : V16 · Vague : 1 · Piste : 1 · Statut : todo
- Dépend de : — (squelette ; fige le call graph + le contrat de modes whole-program)

## But
Remplacer l'escape analysis **intra-procédurale + whitelist `print`/`len`** de `resolve.rs` par une **analyse
de modes interprocédurale à point fixe** : tout paramètre non-`Copy` démarre `ByRef`, et n'est rétrogradé en
`ByValue` que s'il s'échappe **réellement** — un argument passé à une position **`ByRef` du callee** n'échappe
**pas**. Couvre les **fns libres ET les méthodes**. Livre le gros gain : `ray_color`/`shade` en `&Scene`/`&Ray`.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/resolve.rs` (l'analyse — réécriture du calcul de `ParamModes`).
- `ironc/src/lower.rs` (consommation des modes au site d'appel **de méthode** ; les fns libres consomment déjà
  `param_modes`, cf. `lower.rs:1894-1897`).

## Interface produite / consommée
**Consommée** : l'AST + la **`TypeMap`** (résolue par l'inférence, qui tourne avant — nécessaire pour résoudre
un appel de méthode `x.m()` → `(type, méthode)`). Modes des fns **externes** = `ExternFn.recv`/`param_owned`.
**Produite (contrat figé)** :
- `ParamModes` whole-program : clé = **fn libre (nom)** OU **méthode (`Type::méthode`)** ; valeur = `Vec<ParamMode>`.
- Un **call graph** : `(appelant, site) → (callee, position d'arg)`, callee ∈ {fn libre, méthode, externe}.
- `lower.rs` lit les modes de **méthode** pour passer `&arg`/`arg` au bon endroit (parité avec les fns libres).

## Étapes
1. **Collecte** : énumérer toutes les fonctions analysables — `ItemKind::Fn` (clé = nom) **et** les méthodes
   d'`ItemKind::Impl` (clé = `format!("{type}::{method}")`). Pour chacune : params (nom, type) + corps.
2. **Call graph** : parcourir les corps ; pour chaque `Call`, résoudre le callee :
   - `f(args)` (callee `Var(f)`) → fn libre `f`, positions = index d'arg.
   - `x.m(args)` (callee `Field`) → via `TypeMap[x]` = type concret → méthode `Type::m`. Type non résolu → **conservateur** (les args de cet appel comptent comme échappants).
   - callee externe (binding) → modes connus (`recv`/`param_owned`), positions empruntées = `recv ByRef` / param non-`own`.
3. **Init** : pour chaque param non-`Copy` et non-`inout` → `ByRef`. (`is_copy`, `Type::Ref` inchangés.)
4. **Point fixe (worklist)** : recalculer l'ensemble `escaped` de chaque fonction :
   - **échappements définitifs** (indépendants des modes, calculés une fois) : `return p` / `return p.champ`
     (l.230-241 actuel), stockage (struct-lit/append/assign survivant), capture `move`, builtin consommant.
   - **échappements conditionnels** : `p` passé à la position `j` d'un callee `g` → échappe **ssi `modes[g][j] == ByValue`**.
   - `modes[f][p] = ByValue si p ∈ escaped(f) sinon ByRef`. Itérer jusqu'à stabilité (monotone → termine).
5. **Conservatisme** : tout cas non prouvé emprunt-seul (type de receveur inconnu, callee non résolu, expr
   exotique) → `ByValue`. **Dans le doute, clone.**
6. `lower.rs` : au site d'appel de **méthode**, consulter `param_modes["Type::m"]` pour passer `&arg` quand `ByRef`
   (symétrie avec le chemin fn-libre existant).

## Definition of Done (vérifiable)
- `cargo build -p ironc` + `cargo test -p ironc` verts.
- **Raytracer (emit)** : `fn ray_color(scene: &Scene, r: &Ray, depth: i64)`, `fn shade(mat: &Material, scene:
  &Scene, …)`, `fn hit(&self, r: &Ray, …)` apparaissent dans `main-rust/src/main.rs`.
- **Corpus 80 reste VERT** (échantillon re-testé : aucun programme qui passait ne casse).
- **Terminaison** : le point fixe converge sur la récursion mutuelle `ray_color`↔`shade` (pas de boucle infinie).
- 0 `unsafe`.

## Notes
- **Ordre pipeline (I9)** : `infer` avant `resolve` — la `TypeMap` est disponible. Le call graph se construit
  dans `resolve` (ou exposé pour réutilisation par 2.3 ; sinon 2.3 reconstruit le sien — borné en 2.3).
- **Risque rayon de souffle** : commits intermédiaires ; à chaque pas, vérifier un échantillon corpus + le
  raytracer emit. Si un `&T` casse `rustc`, **rétrograder** (le conservatisme manque un échappement → l'ajouter).
- Ne PAS faire ici : for-élément, last-use intra-expression, opérateurs (vagues 2-3). Juste les **modes de params**.
