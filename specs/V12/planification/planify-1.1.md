# planify-1.1 — Squelette : le contrat gelé du pont de correspondance

- Réalise (cadrage) : [24000] (fondation ; prépare [24100])
- Version : V12 · Vague : 1 · Piste : 1 · Statut : todo
- Dépend de : — (racine de la V12)

## But
Poser le **schéma partagé** de la frontière Iron↔Rust et le câbler en **pass-through vert** (comportement
inchangé). Cette vague **fige les interfaces** que les pistes de la vague 2 (`planify-2.1/2.2/2.3`) rempliront
en parallèle sans collision. Séquentiel : rien d'autre ne démarre tant que c'est rouge.

## Fichiers POSSÉDÉS (ownership disjoint)
- **Création** : `ironc/src/bridge.rs`.
- **Édition** : `ironc/src/ir.rs` (variante `ExternApp`, champ `ExternType.params`, `mod bridge`),
  `ironc/src/lower.rs` (uniquement le bras `lower_type` de `ExternApp`),
  `ironc/src/iri.rs` (parse `Name[args]` + en-tête `extern type Name[T] = …`),
  `ironc/src/bind.rs` (émettre `[T]`/`Name[args]` **seulement si params** — sinon sortie inchangée).
> En vague 1 c'est UNE piste séquentielle : pas de conflit possible. La disjonction stricte commence en vague 2.

## Interface produite / consommée
**Produit (gelé, consommé par TOUTES les vagues suivantes)** :
- Module `bridge` avec signatures **définitives** (corps en délégation pass-through pour l'instant) :
  - `project(rd, cx) -> Option<Type>` (façade projection) · `coerce_arg(expr, from, target) -> TokenStream`
    (façade coercition) · `cast_numeric(expr, from, to) -> TokenStream` (point numérique unique) ·
    `rust_num_name(NumKind) -> &str` (table canonique, reprend `num_kind_str`).
  - **Le mécanisme « borne → stratégie » (RACINE 1, extensible)** — l'interface que les vagues 4-6/8.2 remplissent
    sans toucher au pont : `accepts(bound) -> Option<Type>` (face projection : quel type Iron une borne accepte)
    et `materialize(bound, iron_expr, target) -> TokenStream` (face coercition : comment produire la valeur Rust).
    Table `STRATEGIES` (borne→stratégie) **vide en vague 1** sauf l'entrée scalaire (vague 2) — l'ajout d'une
    stratégie (io/closure/serde…) = **une entrée**, jamais une modif du pont. **C'est ce hook qui rend l'archi
    cohérente** : io/closures/serde-param ne sont pas des refontes, ce sont des entrées.
  - Tables-**données** exposées en lecture : `CONTAINERS`, `BOUND_TO_IRON`, `BYTES`, `STRATEGIES`.
- ABI génériques : `Type::ExternApp(String /*rust_path*/, Vec<Type> /*args*/)` **additive** ; lowering
  `ExternApp(p,[])` → `p` et `ExternApp(p,args)` → `p::<args>`.
- Schéma `.iri` : `extern type Name[T,…] = "path":` → `ExternType.params: Vec<String>` ; `Name[args]` en
  position de type → `Type::ExternApp`. Rétro-compat : `.iri` sans params reste valide (`params` vide).

**Consommé** : —

## Étapes
1. Ajouter `Type::ExternApp` + `ExternType.params` dans `ir.rs` (additif). Bras `lower_type` pour `ExternApp`.
2. Créer `bridge.rs` : signatures des 3 faces + tables, **corps = appels aux fonctions actuelles** (`map_type`,
   `extern_arg_to`, `num_kind_str`) → zéro changement de sortie.
3. `iri.rs` : parser `Name[args]` (réutilise la lecture `[...]` de `list[T]`) et l'en-tête `[T]` → `params`.
4. `bind.rs` : à l'émission, écrire `[T]`/`Name[args]` **uniquement** quand il y a des params (sinon inchangé).
5. Fixture de gate « traversée ExternApp » (cf. DoD).

## Definition of Done (vérifiable)
1. `cargo build` vert ; **gate VERT 24/0** — comportement **identique** (pass-through prouvé).
2. **Fixture traversée** : `.iri` `extern type Box2[T] = "…":` + fn rendant `Box2[f64]` ⇒ Rust émis contient
   `…::<f64>` (le canal génériques traverse binder→`.iri`→infer→lower). Ajoutée au gate.
3. Diff `.iri` **vide** pour des crates non génériques régénérés (uuid, zstd).
4. 0 `unsafe` (I2), pipeline I9 intact, codegen `TokenStream` (I10).

## Notes
- C'est la « fondation fige le SCHÉMA + conventions » de la méthodo : on gèle l'ABI `ExternApp` et le schéma
  `.iri` AVANT le fan-out, sinon deux pistes combleraient le trou différemment (intégration cassée).
- Décision d'ABI (variante dédiée additive) tranchée en cadrage `[24000]` — ne pas re-litiger ici.
