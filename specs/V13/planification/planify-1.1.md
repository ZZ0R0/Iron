# planify-1.1 — Octets typenum (`GenericArray<u8, N>`/`Key`/`Nonce`) → aes-gcm, chacha20

- Réalise (cadrage) : [24500] sous-cas A
- Version : V13 · Vague : 1 · Piste : 1 · Statut : todo
- Dépend de : — (réutilise `@arr`/`wraps` de V12 [24200])

## But
Étendre la reconnaissance des **tableaux d'octets de taille fixe** de `[u8; N]` (V12) à `GenericArray<u8, …>`
et aux alias `Key<C>`/`Nonce<C>`/`Tag<C>` (qui s'expansent vers `GenericArray<u8, …>`). La matérialisation
(`(&v[..]).try_into()`/`from_slice`, **taille inférée par rustc**) est **déjà** en place (`wraps`/render).
Débloque la **construction de clés/nonces** → aes-gcm, chacha20.

## Fichiers POSSÉDÉS
- `ironc/src/bind.rs` : `map_type` — reconnaître `resolved_path` `GenericArray`/`Key`/`Nonce`/`Tag` à élément
  `u8` → sentinelle `@arr` (comme la branche `array`). Vérifier que l'expansion d'alias (B33) amène `Key<C>`
  vers `GenericArray<u8, …>` avant cette reconnaissance.
- (lecture seule : `wraps`/render déjà faits en [24200].)

## Interface produite / consommée
- **Consomme** : `@arr` + `wraps` (V12 [24200]) ; B33 (expansion d'alias).
- **Produit** : params octets-fixes (clé/nonce) acceptant un `list[u8]` Iron, matérialisé (taille inférée).

## Étapes
1. `map_type` : ajouter, à côté de la branche `array`, la reconnaissance d'un `resolved_path` dont le nom court
   ∈ {`GenericArray`,`Key`,`Nonce`,`Tag`} ET dont l'élément/1ᵉʳ arg est `u8` → `@arr`.
2. Vérifier le **type-cipher** `Aes256Gcm` (alias `AesGcm<Aes256, U12>`) : binder le type appliqué comme handle
   utilisable (`Aes256Gcm.new(key)`) ; args typenum non-octets (`U12`) restent opaques.
3. Régénérer aes-gcm/chacha20, écrire un programme chiffre→déchiffre, exécuter.

## Definition of Done (vérifiable)
1. aes-gcm : `Aes256Gcm.new(key32)` + `encrypt(nonce, msg)` → chiffré ≠ clair ; `decrypt(nonce, chiffré)` → clair
   (run réel, round-trip). chacha20 idem.
2. Gate VERT 24/0 ; 0 `unsafe`.

## Notes
- Si le **type-cipher** (AesGcm à 3 params) résiste à l'expansion/au binding, noter précisément où (l'octet/clé
  est l'objectif ; si seul le cipher bloque, c'est un cas typenum-type distinct à escalader).
