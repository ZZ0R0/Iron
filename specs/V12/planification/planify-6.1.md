# planify-6.1 — Tableaux à taille fixe (`[u8; N]` / `GenericArray`) — dérivée du pont

- Réalise (cadrage) : [24200]
- Version : V12 · Vague : 6 · Piste : 1 · Statut : todo
- Dépend de : planify-3.1 (fondation close), planify-5.1 (octets)

## But
Fournir un tableau de **taille fixe** : clés/nonces de crypto (`[u8; 32]` / `GenericArray<u8, U32>`) pour
aes-gcm, chacha20poly1305. S'appuie sur l'`ExternApp` (un arg de taille) + la table octets.

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition** : `ironc/src/ir.rs` (si une forme de const-arg est nécessaire au `Type`), `ironc/src/bridge.rs`
  (correspondance `list[u8]` len-connue → `[u8; N]`/`GenericArray::from_slice`), `ironc/src/lower.rs` (émission +
  `try_into`/`from_slice`), `ironc/src/bind.rs` (reconnaître `[u8; N]`/`GenericArray<u8, N>` en signature).
- Séquentielle (vague 6).

## Interface produite / consommée
- **Consomme** : fondation [24000] (ExternApp, table octets de 5.1).
- **Produit** : conversion `list[u8]` (longueur connue/vérifiée) → `[u8; N]` / `GenericArray`.

## Étapes
1. Décision : `[u8; N]` natif vs `GenericArray<u8, typenum>` — figer l'encodage de la taille (cf. questions ouvertes de `24200.md`).
2. `bind.rs` : projeter `&[u8; 32]`/`GenericArray<u8, U32>` en un type Iron portant la taille.
3. `lower.rs` : `list[u8]` → `[u8; N]` via `try_into().unwrap()` (longueur runtime) ou littéral de taille connue ; `GenericArray::from_slice`.

## Definition of Done (vérifiable)
1. `Aes256Gcm.new(key_bytes)` chiffre/déchiffre un message (programme complet, run réel) ; `cargo` vert.
2. **Gate VERT** ; 0 `unsafe` (I2).

## Notes
- Arithmétique sur const-génériques arbitraires = **hors périmètre** (cf. `24200.md`). On vise le cas crypto (taille fixe connue).
