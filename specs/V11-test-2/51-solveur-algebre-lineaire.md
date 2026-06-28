# 51 — Solveur d'algèbre linéaire (Ax = b)
> **Domaine** Maths/vectoriel · **Crate(s)** `ironc add nalgebra` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Construire une matrice `A` et un vecteur `b`, résoudre le système linéaire `A·x = b` par décomposition
(LU, QR ou Cholesky), et vérifier la solution. En appui : produit matrice-vecteur, transposée,
déterminant, inverse.

## Ce que ça teste dans Iron
Le binder auto (V8) face à un crate **fortement générique** : `nalgebra` modélise tout par
`Matrix<T, R, C, S>` (scalaire, lignes, colonnes, stockage) avec des **const-génériques** de dimension.
C'est précisément le cas où le binder « feuille » s'arrête. On teste aussi la **surcharge d'opérateurs**
(V11) pour écrire `A * x` et `A + B` à la « numpy » au lieu de `.mul()` / `.add()`.

## Approche
Ne **pas** viser les types statiques (`Matrix3`, `Vector3`, const-génériques) : cibler les **alias
dynamiques** `DMatrix<f64>` / `DVector<f64>`, monomorphes, qui cachent le gros des génériques. Un `.iri`
ciblé expose : constructeurs (`from_row_slice`, `zeros`, `identity`), indexation, `A.lu()` / `A.qr()` puis
`.solve(&b) -> Option<DVector>`, plus `determinant`, `transpose`, `try_inverse`. Mapper `*`/`+`/`-` (V11)
sur les impls d'opérateurs de nalgebra. Cas e2e : un système 3×3 à solution connue.

## Risque / limites attendues
- Les objets de décomposition (`LU`, `QR`, `Cholesky`) sont des **types génériques intermédiaires** : à
  exposer comme **handles opaques**, le binder ne les déduira pas seul.
- `solve` rend une `Option` (matrice singulière) → s'aligne bien sur la sémantique par valeur.
- Les matrices **à taille statique** et les vues (`MatrixView`) restent hors de portée du binder auto.

## Done
`ironc run` résout un système 3×3 et imprime `x` ; on vérifie `A·x ≈ b` ; det/inverse cohérents. Le `.iri`
dynamique compile sans `unsafe` et l'ergonomie opérateurs (`A * x`) fonctionne.
