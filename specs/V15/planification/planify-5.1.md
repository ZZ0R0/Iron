# planify-5.1 — C2 : méthodes de trait projetées sur le type implémenteur (AEAD & co)

- Réalise (cadrage) : [50600] C2
- Version : V15 · Vague : 5 · Piste : 1 · Statut : todo
- Dépend de : planify-4.1 (partage `bind/impls.rs` — séquencé après C1)

## But
Faire qu'une **méthode de trait** portée par un type concret via `impl Trait for Type` devienne une **méthode
du type implémenteur**, ses paramètres génériques matérialisés par les sentinelles existantes (`@arr`/`@read`/
`@write`). Règle **générale** = « projeter les méthodes d'un impl-trait sur le type qui l'implémente », **pas**
« connaître `Aead` ». Résorbe `GenericTraitMethod`.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/bind/impls.rs` (branche `impl Trait for Type` — la collecte des ctors/méthodes inhérents y est
  déjà ; on ajoute la **projection des méthodes de trait**).
- `ironc/src/bind/project.rs` (matérialisation des génériques de méthode de trait via les hooks sentinelles).

## Interface produite / consommée
**Consommée** : hooks `@arr`/`@read`/`@write` de `project.rs` (figés en 2.1) ; la mécanique d'`impl` (4.1 mergée).
**Produite** : pour un `impl Trait for T`, les méthodes du trait sont émises comme **méthodes de `T`**
(`cipher.encrypt(nonce, pt)` → `aead::Aead::encrypt(&cipher, …)`), génériques matérialisés.

## Étapes
1. `impls.rs` : au traitement d'un bloc `impl Trait for T` (pas seulement inhérent), **projeter** les méthodes
   du trait sur `T` (chemin Rust `<Trait>::method` qualifié, receveur `&T`/`&mut T`/`T`).
2. `project.rs` : pour les params génériques de ces méthodes, réutiliser les sentinelles (`list[u8]`→`@arr`,
   source io→`@read`, puits→`@write`). Un générique hors-sentinelle → `SkipReason::GenericTraitMethod` (classé,
   pas droppé).
3. Cas `Self`-retour (chiffrement renvoyant le type) → cohérent avec C4 (6.1), ne pas empiéter sur `project.rs`
   self-return (laissé à 6.1).

## Definition of Done (vérifiable)
- **Run réel** : aes-gcm **chiffre puis déchiffre** un message (round-trip correct). chacha20 idem si trivial.
- Le rapport montre `GenericTraitMethod` **en baisse** pour les méthodes AEAD.
- e2e binding VERT ; corpus non régressé ; 0 `unsafe`.

## Notes
- **Séquencé après 4.1** car partage `impls.rs` (honnêteté anti-collision : pas de fausse ‖).
- `project.rs` est **partagé** avec C4 (6.1) → 6.1 séquencé après cette piste.
- La règle ne nomme **aucun** trait : elle s'applique à tout `impl Trait for T` projetable. AEAD est juste la
  première famille qui en bénéficie.
