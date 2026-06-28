# 58 — Inférence ONNX (classification d'image)
> **Domaine** ML & IA · **Crate(s)** `ironc add tract-onnx` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Charger un modèle **ONNX** pré-entraîné (ex. MobileNet/MNIST), prétraiter une image en tenseur d'entrée,
lancer l'inférence et imprimer la classe la plus probable. **Pas d'entraînement**, que de l'inférence.

## Ce que ça teste dans Iron
L'autre face du mur ML — un **pipeline type-state** plutôt qu'un autograd :
- Charger → optimiser → rendre exécutable enchaîne des **types successifs différents**
  (`InferenceModel` → `TypedModel` → `RunnableModel` → `SimplePlan`). Ce **type-state** (le type change à
  chaque étape) est exactement ce que le binder auto ne sait **pas** suivre.
- Les entrées passent par la **macro `tvec![ ]`** et par `tract-ndarray` → **macros invisibles** au binder
  + interop ndarray (cf. projet 52).
- Les sorties sont des **`Arc<Tensor>`** (propriété partagée) → cogne avec la sémantique par valeur.

## Approche
Comme pour candle : une **façade Rust** qui **aplatit toute la chaîne** en un objet opaque —
`load(path) -> Model`, `run(model, pixels, shape) -> Vec<f32>` — sans exposer le type-state ni `tvec!`.
**Avantage** sur le 57 : le modèle est **immuable après chargement** et l'inférence est **sans état** → la
sémantique par valeur gêne **moins** (pas de poids à muter). Reste le prétraitement image → tenseur.

## Risque / limites attendues
- **Type-state builder** + **macros `tvec!`/ndarray** = échec du binder auto en API directe.
- `Arc<Tensor>` en sortie : à convertir en `Vec<f32>` dans la façade.
- Plus **traitable** que le 57 (pas d'entraînement, modèle figé), mais **toujours 🔴** sans façade.

## Done
Via façade, `ironc run` classe une image avec un ONNX pré-entraîné et imprime le top-1. Sans façade : on
**documente** que le type-state + macros bloquent le binder auto. Aucun `unsafe`.
