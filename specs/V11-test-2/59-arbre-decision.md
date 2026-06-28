# 59 — Arbre de décision (classification)
> **Domaine** ML & IA · **Crate(s)** `ironc add smartcore` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Entraîner un **arbre de décision** sur un petit jeu tabulaire (ex. Iris réduit), prédire les classes d'un
jeu de test et imprimer l'exactitude. En appui : random forest, k-NN (même API).

## Ce que ça teste dans Iron
`smartcore` est du ML **100 % Rust** avec son **propre type matrice** `DenseMatrix<f64>` — **pas de
dépendance ndarray**, ce qui **évite** les soucis des projets 52/56. C'est ce qui le rend 🟡 et non 🔴.
On teste : le `DenseMatrix` concret, le `fit`/`predict` (génériques sur matrice + cible, à monomorphiser),
les **paramètres en struct** (`DecisionTreeClassifierParameters`), et le `Result` de `fit`. Le modèle
entraîné est **immuable** → bon accord avec la sémantique par valeur.

## Approche
`.iri` exposant `DenseMatrix::from_2d_vec(...)`, puis
`DecisionTreeClassifier::fit(&x, &y, params) -> Result<Model>` et `model.predict(&x) -> Result<Vec>`,
figés sur `(DenseMatrix<f64>, Vec<i32>)`. Cibles et features en `Vec` natifs Iron. Cas e2e : split
train/test sur un petit CSV, accuracy au-dessus d'un seuil.

## Risque / limites attendues
- smartcore est **plus générique qu'il n'y paraît** (traits `Matrix`, `RealNumber`, type de cible
  paramétré) : le binder doit **épingler l'instanciation** monomorphe, sinon trous d'inférence.
- `fit`/`predict` rendent des `Result` : gestion d'erreur à mapper.
- Pas d'état mutable ni de macros : nettement plus simple que candle/tract → 🟡 tenable.

## Done
`ironc run` entraîne l'arbre, prédit le jeu de test et imprime une accuracy plausible (> 0.8 sur Iris).
Compile sans `unsafe`, sans clone-fallback sur le modèle.
