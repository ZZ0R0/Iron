# planify-5.1 — Ergonomie des octets (`bytes`, hex) — dérivée du pont

- Réalise (cadrage) : [82000] (sous-cas de [24400] pour `u8`)
- Version : V12 · Vague : 5 · Piste : 1 · Statut : todo
- Dépend de : planify-3.1 (fondation close)

## But
Rendre fluide le travail sur octets : `list[u8]` ⟷ `&[u8]`/`Vec<u8>`/`GenericArray<u8>` + **hex** (encode/decode).
Débloque sha2 (digest→hex), PoW (octets). Une **entrée de la table octets** du pont + un helper hex.

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition** : `ironc/src/bridge.rs` (table `BYTES` : équivalences `list[u8]` ↔ formes Rust d'octets),
  `ironc/src/lower.rs` (hex : `to_hex`/`from_hex` sur `list[u8]`/`str`).
- Séquentielle (vague 5).

## Interface produite / consommée
- **Consomme** : fondation [24000] (table octets, coercition table-pilotée de 2.2).
- **Produit** : `digest.to_hex()` (ou équivalent) → `str` ; `bytes`-arg accepté là où Rust veut `&[u8]`/`Vec<u8>`.

## Étapes
1. Compléter `BYTES` : `list[u8]` ↔ `&[u8]`, `Vec<u8>`, `[u8; N]` (renvoyer vers [24200] pour la taille fixe).
2. Hex : helper `bridge`/lowering — `list[u8] → str` (encode), `str → list[u8]` (decode), sans crate tierce (`format!("{:02x}")` / parse).
3. Brancher dans la coercition d'arg (2.2) : un param Rust `&[u8]` accepte un `list[u8]` Iron sans cas-par-cas.

## Definition of Done (vérifiable)
1. sha2 : `Sha256.digest(s)` → octets → **hex** affichable (programme complet vert, run réel).
2. **Gate VERT** ; logique octets centralisée dans `bridge` (pas re-dispersée).

## Notes
- `GenericArray<u8, N>`/`[u8; N]` (taille fixe) = [24200] (planify-6.1) ; ici on couvre le `Vec`/slice dynamique + hex.
