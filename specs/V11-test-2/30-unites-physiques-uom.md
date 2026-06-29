# 30 — Calculateur d'unités physiques (analyse dimensionnelle)
> **Domaine** Calcul scientifique · **Crate(s)** `ironc add uom` · **Risque Iron** 🟥 · **Difficulté** très élevée

## But
Manipuler des grandeurs physiques typées (longueur, temps, vitesse) avec **vérification dimensionnelle à la
compilation** : `distance / temps = vitesse`, conversions d'unités.

## Ce que ça teste dans Iron
`uom` est de la **programmation au niveau type** poussée : les dimensions sont encodées en types (typenum),
les quantités sont `Quantity<Dimension, Units, Value>`. C'est l'un des crates les **plus durs** à binder
(aucune valeur runtime pour la dimension). Test limite du moteur.

## Approche
Mesurer d'abord la **couverture** de `ironc add uom` + rapport. Si tractable, cibler les constructeurs
d'unités SI concrets (`Length::new::<meter>(x)`) via curation ; sinon documenter le mur précis.

## Done
Au minimum : couverture mesurée + rapport des sautés. Idéalement : une vitesse calculée et convertie.
