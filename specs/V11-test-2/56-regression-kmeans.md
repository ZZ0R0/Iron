# 56 — Régression linéaire / k-means
> **Domaine** ML & IA · **Crate(s)** `ironc add linfa` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Entraîner une **régression linéaire** (puis prédire) et un **k-means** sur un petit jeu de données 2D,
imprimer coefficients/centroïdes et l'erreur. La « scikit-learn de Rust » en version minimale.

## Ce que ça teste dans Iron
À mi-chemin du mur ML. `linfa` s'articule autour d'un type **générique `Dataset<R, T>`** adossé à
**`ndarray`** (`Array2<f64>`) — il **hérite donc des problèmes du projet 52** (dimension générique, macro
`s![ ]`). Le `fit`/`predict` passe par des **traits** (`Fit`, `Predict`). Mais les modèles
(`LinearRegression`, `KMeans`) sont des entrées **assez concrètes**, et un modèle entraîné est **immuable**
→ ça s'aligne bien sur la sémantique par valeur (entraîner une fois, prédire ensuite).

## Approche
Réutiliser le `.iri` ndarray (projet 52) comme **dépendance**. Exposer `Dataset::new(records, targets)`
monomorphe en `f64`, puis `LinearRegression::default().fit(&ds) -> Result<Model>` et
`model.predict(&records)`. Idem `KMeans::params(k).fit(..)`. Lier explicitement les méthodes de trait
`fit`/`predict`. Cas e2e : régresser `y = 2x + 1` bruité, vérifier la pente ≈ 2 ; k-means à 2 clusters
nettement séparés.

## Risque / limites attendues
- **`Dataset<R, T>` générique + backing ndarray** : le binder doit figer l'instanciation `f64` ; sans le
  `.iri` ndarray, ça ne décolle pas. Plus proche du 🔴 que les projets 53/54.
- `fit`/`predict` sont des **méthodes de trait** (bound `Fit`) → à surfacer manuellement.
- Builders de paramètres (`KMeans::params`) : structs gérables, mais chaînés.

## Done
`ironc run` entraîne une régression et un k-means, imprime coefficients/centroïdes plausibles. Le modèle
entraîné, immuable, se prédit sans clone-fallback. Compile sans `unsafe`.
