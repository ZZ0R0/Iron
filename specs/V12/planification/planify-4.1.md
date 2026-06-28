# planify-4.1 — Stratégies « valeur » : bornes convertibles + octets/hex

- Réalise (cadrage) : [24100b], [82000]
- Version : V12 · Vague : 4 · Piste : 1 · Statut : todo
- Dépend de : planify-3.1 (pont + stratégie scalaire en place)

## But
Ajouter au pont (RACINE 1) deux **stratégies de matérialisation** : (a) borne « convertible depuis chaîne »
(`Into<String>`/`AsRef<Path>`/newtype-string) → chaîne Iron ; (b) octets `list[u8]` ↔ `&[u8]`/`Vec<u8>` + hex.
Chacune = **une entrée** dans `STRATEGIES`/`BYTES`, pas un mécanisme neuf.

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition** : `ironc/src/bridge.rs` (entrées `STRATEGIES`+`BYTES`), `ironc/src/bind.rs` (ne plus dropper le
  ctor convertible), `ironc/src/lower.rs` (hex : `list[u8]`→`str`, `str`→`list[u8]`).
- Séquentielle (vague 4).

## Interface produite / consommée
- **Consomme** : `bridge::{accepts, materialize, STRATEGIES, BYTES}` (gelé en 1.1).
- **Produit** : ctors convertibles bindés (`Command.new("x")`) ; `digest.to_hex()` → `str`.

## Étapes
1. `STRATEGIES` : borne `Into<X>`/`TryFrom<&str>` où `X` se réduit à `str`/numérique → `accepts = str`,
   `materialize = (expr).into()`/`as_ref()`.
2. `BYTES` : `list[u8]` ↔ `&[u8]`/`Vec<u8>` (déjà partiel) + hex sans crate tierce (`{:02x}` / parse).
3. `bind.rs` : fallback d'un `T` à borne convertible = garder le param (plus de drop).

## Definition of Done (vérifiable)
1. clap/redis/ureq : ctor-clé **apparaît** et un appel trivial **compile**.
2. sha2 : `digest` → **hex** affichable (programme complet, run réel).
3. **Gate VERT** ; logique centralisée dans `bridge` (rien de re-dispersé).

## Notes
- `GenericArray<u8, N>` (taille fixe) = planify-4.2. Conversion exigeant une closure → planify-6.1 (Fn).
