# 46 — Statistiques par groupes (itertools)
> **Domaine** Données · **Crate(s)** `ironc add itertools` · **Risque Iron** 🟡 · **Difficulté** élevée

## But
Grouper une liste de valeurs, calculer des agrégats par groupe (compte, somme), combiner des itérateurs
(`zip`, `chunks`, `unique`).

## Ce que ça teste dans Iron
`itertools` est **itérateur/trait-lourd** : des adaptateurs génériques sur `Iterator` renvoyant des types
itérateurs imbriqués. Teste le mappage d'API à base de traits d'itération et de closures vers les
compréhensions/boucles Iron.

## Approche
Cibler des fns concrètes rendant des `Vec` : `sorted`, `unique`, `chunks` recollectés. Probable idiome
`collect`. Mesurer la couverture si les adaptateurs paresseux ne se projettent pas.

## Done
`ironc run` imprime un agrégat correct (ex. nombre de valeurs uniques), ou couverture mesurée + rapport.
