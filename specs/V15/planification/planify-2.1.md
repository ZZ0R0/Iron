# planify-2.1 — Binder modularisé & à portée connue (Option → Result<Sig, SkipDiagnostic>)

- Réalise (cadrage) : [50500] (portée connue, zéro drop silencieux)
- Version : V15 · Vague : 2 · Piste : 1 · Statut : todo
- Dépend de : planify-1.1 (schéma figé)

## But
Transformer le binder d'un générateur qui **abandonne en silence** en un binder qui **sait et nomme** ce
qu'il ne peut pas binder, **et** le découper en sous-modules à ownership disjoint pour rendre le fan-out
(vague 4) parallèle. Cœur : `project()` et les collecteurs passent de `Option<Sig>` à
`Result<Sig, SkipDiagnostic>` ; chaque ancien `None`/drop devient une **raison nommée** ; on compte la
**couverture** et on asserte `seen == bound + skipped`.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/bind.rs` → éclaté en `ironc/src/bind/mod.rs` + `bind/mapty.rs` + `bind/project.rs` +
  `bind/impls.rs` + `bind/render.rs`. **Cette piste possède tout l'arbre `bind/`.**

## Interface produite / consommée
**Consommée** : `SkipReason`/`SkipDiagnostic`/`Coverage`/`ItemKind` (1.1).
**Produite** (figée pour le fan-out) :
- `bind/mapty.rs` : `map_type(...) -> Result<Type, SkipDiagnostic>` ; `bound_to_iron(...)` ; ALIAS_MAP +
  `try_expand_alias` ; **`alias_for_monomorphization(&Json) -> Option<String>`** (reverse-alias, **consommée
  par C1/4.1** qui ne modifiera donc PAS `mapty.rs`).
- `bind/project.rs` : `project(...) -> Result<Sig, SkipDiagnostic>` ; hooks de sentinelles `@arr`/`@read`/
  `@write` exposés (consommés par C2/5.1 et C4/6.1).
- `bind/impls.rs` : la boucle d'`impl` (collecte ctors/méthodes par type) — **point d'extension C1 (4.1) et
  C2 (5.1)**.
- `bind/mod.rs` : orchestrateur `emit(...)` qui **accumule** `Coverage` + le `Vec<SkipDiagnostic>` et les
  **retourne** (le rendu du rapport est 3.1).

## Étapes
1. Découper `bind.rs` en `bind/{mod,mapty,project,impls,render}.rs` — **déplacement mécanique d'abord**,
   `cargo build` vert avant tout changement de signature.
2. `project()` + collecteurs : `Option<Sig>` → `Result<Sig, SkipDiagnostic>`. Chaque site de `None`/drop
   actuel → un `Err(SkipDiagnostic { reason: … , hint: … })` avec la bonne variante de taxonomie.
3. `emit()` : compter `seen` (items publics vus), `bound` (Ok), `skipped` (Err) ; **assert** `seen == bound + skipped` ;
   retourner `(iri, Coverage, Vec<SkipDiagnostic>)`.
4. Exposer `alias_for_monomorphization` (reverse de ALIAS_MAP) — squelette suffisant (peut renvoyer `None`,
   C1 l'enrichira en **consommation**, pas en édition de `mapty.rs`).
5. Adapter les appelants (`driver.rs`) au nouveau tuple de retour **a minima** (le rapport réel = 3.1).

## Definition of Done (vérifiable)
- `cargo build -p ironc` + `cargo test -p ironc` verts.
- **Aucune régression de binding** : `ironc add` sur ≥2 crates du corpus V12/V13 produit un `.iri`
  fonctionnellement équivalent (un programme témoin compile+run comme avant).
- **Invariant** : sur ≥3 crates, `seen == bound + skipped` (assertion ne panique pas).
- Le gate du projet reste VERT (24/0) ; 0 `unsafe` (I2).

## Notes
- **Refactor à risque** (éclatement d'un gros fichier couplé) : faire le **déplacement mécanique** + build
  vert AVANT de toucher aux signatures. Commits intermédiaires si besoin.
- Ne PAS implémenter C1–C4 ici. On crée les **coutures** et la **portée connue**, rien d'autre.
- I9 (pipeline figé) : on ne change pas l'ordre des passes, on réorganise le code du binder.
