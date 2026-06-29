# planify-6.1 — C4 : modèle builder/flux (chaînage de méthodes) + unification du moteur

- Réalise (cadrage) : [50600] C4
- Version : V15 · Vague : 6 · Piste : 1 · Statut : todo
- Dépend de : planify-5.1 (partage `bind/project.rs`), planify-4.2 (partage `lower.rs`)

## But
Reconnaître le pattern **builder consommant/rendant `self`** (`Encoder::new(w) → .write_all(..) → .finish()`)
et l'exposer comme **chaînage de méthodes** Iron, au lieu d'exiger une fn one-shot à puits. Résorbe
`StreamBuilderApi`. **Dernière capacité** → cette piste **unifie** aussi le moteur (greffe C1–C4, recompute la
couverture globale).

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/bind/project.rs` (détection des méthodes self-consommant/self-rendant).
- `ironc/src/lower.rs` (émission du chaînage).

## Interface produite / consommée
**Consommée** : `project.rs` (post-C2, mergé), `lower.rs` (post-C3, mergé).
**Produite** : un builder `T` dont les méthodes prennent/rendent `self` est exposé en chaînage
(`Encoder.new(sink).write_all(data).finish()` → la chaîne Rust équivalente, avec puits matérialisé via `@write`).

## Étapes
1. `project.rs` : détecter les méthodes `fn(self, …) -> Self` / `fn(self) -> Out` (consomment le receveur) →
   marquer comme **chaînables** (receveur `own self`).
2. `lower.rs` : émettre le chaînage en respectant la consommation de `self` (pas d'emprunt là où Rust prend
   `self`).
3. Matérialiser le puits initial via `@write` ([52800]) quand le builder écrit vers un sink.
4. Cas builder non réductible (état multi-objets) → `SkipReason::StreamBuilderApi` (classé honnêtement).
5. **Unification** : rebuild complet, re-run l'e2e sur les familles débloquées par C1–C4, **recompute la
   couverture globale** et la consigner.

## Definition of Done (vérifiable)
- **Run réel** : flate2 **compresse** un message → octets, puis **décompresse** → message d'origine (round-trip).
- Le rapport montre `StreamBuilderApi` **en baisse** pour le pattern Encoder/Decoder.
- **Gate e2e global** : tout le corpus vert re-validé ; couverture globale **mesurée et consignée** (pas estimée).
- 0 `unsafe` ; pipeline figé respecté (I9/I10).

## Notes
- **Séquencé en dernier** parmi les capacités (partage `project.rs` avec C2 et `lower.rs` avec C3).
- C4 est la capacité **la plus structurelle** : si l'état porté dépasse la projection, rester **partiel** et
  laisser 50500 le **classer** — surtout ne pas prétendre l'avoir couvert.
- Fin de cette piste = **moteur++ unifié** ; reste le corpus (vague 7).
