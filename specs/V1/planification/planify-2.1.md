# [2.1] Escape analysis (resolve réel)

- Spec parente : 30200 · V1 · **Vague 2** (parallèle) · Statut : todo
- Dépend de : 1.1 (contrat gelé)

## But
`resolve.rs` réel : calculer `ParamModes` + `OwnershipMap` par escape analysis **conservatrice** (I2).

## Fichiers possédés
`src/resolve.rs` (+ `tests/resolve_*.rs`).

## Interface
Consomme `&Ast, &TypeMap`. Produit `(OwnershipMap, ParamModes)` (types gelés).

## Étapes
- **ParamModes** : un param est `ByRef` ssi, dans le corps, il est **uniquement lu** (accès champ/index,
  comparaison, `print`) et **ne s'échappe pas** (jamais retourné, stocké, ni passé en valeur à un appel
  utilisateur). Sinon `ByValue`. *(Passe 1-shot conservatrice : passé à un appel utilisateur ⇒ s'échappe.)*
- **OwnershipMap** : lecture = **dernière utilisation** sur tous chemins → `Move` ; lecture non-dernière dont
  le consommateur accepte `&` → `Borrow` ; sinon `Clone`. `Copy` → ignoré (pas de clone).

## Critère de done
Cas couverts par tests : param read-only → `ByRef` ; param retourné/stocké → `ByValue` ; local dernière-utilisation
→ `Move` ; au doute → `Clone`. Build vert.

## Notes
Conservateur > agressif (préserver I1). Raffinement point-fixe inter-signatures = dette ultérieure.
