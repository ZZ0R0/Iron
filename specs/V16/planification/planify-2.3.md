# planify-2.3 — Inférence whole-program : propagation inter-appels + annotation de retour

- Réalise (cadrage) : [21800]
- Version : V16 · Vague : 2 · Piste : 3 · Statut : todo
- Dépend de : planify-1.1 figée (pour le call graph de la propagation (b) ; (a)/(c) indépendants)

## But
Boucher les trois trous d'inférence révélés par le ray tracer, qui ont forcé des annotations artificielles :
(a) **membre jamais appelé** → type libre `__T<n>` qui fuit vers `rustc` ; (b) **type concret non propagé** le
long d'une chaîne d'appels (`HitRecord` à travers `ray_color→shade→shade_lambert`) ; (c) **annotation de retour
ignorée** quand un `Option` est deviné (`-> Vec3` réécrit en `-> Option<Vec3>`).

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/infer/` (moteur de contraintes + `engine/flux.rs`). Disjoint de resolve/driver/lower → **parallèle**.

## Interface produite / consommée
**Consommée** : le moteur de contraintes I15 ; le call graph (réutilisé de 1.1 si exposé, sinon reconstruit
localement — la construction est bon marché). **Produite** : `TypeMap` plus complète (types propagés inter-appels)
+ diagnostics I8 ((a) « param non inférable », (c) « fall-through non couvert »).

## Étapes
1. **(b)** propagation : faire circuler un **type d'argument concret** connu chez l'appelant dans la signature
   de l'appelé, **sur plusieurs niveaux** (point fixe sur les arêtes d'appel).
2. **(a)** membre sans site d'appel : inférer depuis le **corps** (usage des champs/arith) ; à défaut, **diagnostic
   I8 ciblé** (« annote ce param ») plutôt que laisser fuir `__T<n>` jusqu'à `rustc`.
3. **(c)** annotation de retour **prioritaire** : `def f() -> T:` impose `T` ; le moteur de flux prouve
   l'exhaustivité du `return` (`while True` sans `break`, `match` exhaustif, `if/else` couvrant) ⇒ pas d'`Option` ;
   sinon **diagnostic** (jamais de réécriture silencieuse du type de retour).

## Definition of Done (vérifiable)
- Le raytracer compile **sans** les annotations ajoutées en contournement : `cross(self, o: Vec3)`,
  `rec: HitRecord`, `scene: Scene`, `r: Ray` (ou, pour (a), avec un **diagnostic clair** si le repli ne suffit pas).
- `random_in_unit_sphere() -> Vec3` reste `Vec3` (plus d'`Option` injecté).
- Corpus 80 vert (aucune inférence existante cassée — surtout l'inférence Option/Result **légitime** I11 préservée).

## Notes
- **Ne pas** casser l'inférence Option/Result depuis le flux (I11) : seul le cas où elle **écrase une annotation
  explicite** (c) change. Conservateur sur (a) : un diagnostic net vaut mieux qu'une devinette fragile.
