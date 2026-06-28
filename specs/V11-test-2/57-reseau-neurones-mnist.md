# 57 — Réseau de neurones (MNIST)
> **Domaine** ML & IA · **Crate(s)** `ironc add candle-core` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Un petit MLP qui classe les chiffres MNIST : passe avant, perte, rétropropagation, mise à jour des poids.
À défaut d'entraîner : charger des poids pré-entraînés et faire **l'inférence** sur quelques images.

## Ce que ça teste dans Iron
Le **cœur du mur ML**, et pourquoi le binder auto casse :
- **`Tensor` est générique** sur `Device` (CPU/CUDA) et `DType` (F32…), et **chaque opération rend un
  `Result`** → Iron devrait `?` *chaque* op : l'ergonomie s'effondre.
- L'**autograd est un état mutable global** (bande de gradients, `.backward()` → `GradStore`) : viole
  frontalement le **« pas d'état global mutable »** d'Iron.
- La **boucle d'entraînement mute les poids en place** (`optimizer.step`, `VarMap`/`VarBuilder`) → sous
  sémantique par valeur, cloner les tenseurs à chaque pas est **rédhibitoire** (clone-fallback I4).
- Les couches se construisent par **builders/closures** (`linear(in, out, vb)`, trait `Module`) que le
  binder ne sait pas modéliser.

## Approche
**Entraînement : hors de portée** du binder auto — l'assumer comme résultat. **Inférence seule** :
éventuellement une **façade Rust** qui charge un `safetensors`, encapsule le forward dans un objet opaque
`predict(pixels) -> label`, et n'expose **aucun** `Tensor`/`Device`/autograd à Iron. Même là, la
conversion d'E/S tenseur reste lourde.

## Risque / limites attendues
- Tenseur `Result`-partout + Device/DType qui traversent **toutes** les signatures = inexprimable en API
  directe ; autograd mutable = incompatible **par conception** avec Iron.
- Seule une **façade d'inférence** étroite peut passer ; l'entraînement **non**.

## Done
Résultat attendu **honnête** : le binder auto **échoue** sur l'API tenseur/entraînement (documenté). Au
mieux, via façade, `ironc run` classe une image MNIST pré-entraînée. Aucun `unsafe`.
