# planify-4.2 — Stratégie « taille fixe » : `[u8; N]` / `GenericArray`

- Réalise (cadrage) : [24200]
- Version : V12 · Vague : 4 · Piste : 2 · Statut : todo
- Dépend de : planify-3.1, planify-4.1 (octets)

## But
Stratégie de matérialisation pour les **tableaux de taille fixe** (clés/nonces crypto : `[u8; 32]` /
`GenericArray<u8, U32>`) — aes-gcm, chacha20poly1305. Cas particulier de l'`ExternApp` avec un **arg const**.

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition** : `ironc/src/ir.rs` (forme de const-arg si nécessaire), `ironc/src/bridge.rs` (entrée
  `STRATEGIES` : `list[u8]` longueur-connue → `[u8; N]`/`GenericArray::from_slice`), `ironc/src/lower.rs`
  (émission `try_into`/`from_slice`), `ironc/src/bind.rs` (reconnaître `[u8; N]`/`GenericArray<u8, N>`).
- Piste 2 de la vague 4 : **disjointe de 4.1 ?** — non (toutes deux touchent bridge/bind/lower) → en pratique
  **après** 4.1 (séquentielle). [marqué `4*` dans la carte].

## Interface produite / consommée
- **Consomme** : `ExternApp` + `BYTES` (de 4.1).
- **Produit** : `list[u8]` (longueur vérifiée) → `[u8; N]` / `GenericArray`.

## Étapes
1. Figer l'encodage de la taille : `[u8; N]` natif vs `GenericArray<u8, typenum>` (cf. `../24200.md`).
2. `bind.rs` : projeter `&[u8; 32]`/`GenericArray<u8, U32>` en type Iron portant la taille.
3. `lower.rs` : `try_into().unwrap()` (longueur runtime) ou littéral de taille ; `GenericArray::from_slice`.

## Definition of Done (vérifiable)
1. `Aes256Gcm.new(key_bytes)` chiffre/déchiffre un message (run réel), `cargo` vert.
2. **Gate VERT** ; 0 `unsafe` (I2).

## Notes
- Arithmétique sur const-génériques arbitraires = hors périmètre (cf. `../24200.md`).
