# 35 — Quiz interactif en ligne de commande

> **Domaine** TUI terminal · **Crate(s)** `ironc add dialoguer` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Dérouler une série de questions — QCM (menu de sélection), saisie texte libre, confirmation oui/non —
lire les réponses au clavier, comparer aux bonnes réponses et afficher le score final.

## Ce que ça teste dans Iron
`dialoguer` offre des prompts **bloquants et séquentiels** : chaque `Select`/`Input`/`Confirm`
**bloque jusqu'à la réponse** puis **rend une valeur** — exactement le modèle « appel → valeur retour »
qui colle à Iron (pas de boucle d'événements à gérer soi-même). On teste surtout :
- la tolérance du binder aux **builders légers** : `Select::new().with_prompt(...).items(&[...]).interact()` ;
- le passage d'un **slice d'items** Iron (`&[&str]`) vers le crate ;
- la gestion du **`Result`** rendu par `.interact()` (via `?`/match côté Iron).

## Approche
Pour chaque question, monter le prompt adéquat (`Select` pour un QCM, `Input` pour une saisie, `Confirm`
pour un oui/non), appeler `.interact()`, comparer l'index/la valeur à l'attendu, incrémenter le score.
Boucle séquentielle simple, état = un compteur. Afficher le score à la fin.

## Risque / limites attendues
🟢 devrait passer — bon **témoin « builder léger »**. Risque résiduel : le **chaînage**
`new().with_prompt().items().interact()` que le binder doit reconstituer ; les builders à retour `Self`
passent souvent, c'est précisément ce qu'on mesure. `.items()` attend un `&[T]` → vérifier la conversion
d'une liste Iron de chaînes. Le *theming* (`ColorfulTheme`) est optionnel → l'éviter si le binder bronche.
Pas d'état global mutable ni d'event loop : très bon fit.

## Done
Un quiz d'au moins 5 questions mêlant `Select`/`Input`/`Confirm` tourne, lit les réponses, calcule et
affiche le score. 0 `unsafe`. Noter si les builders `dialoguer` ont binté tels quels ou ont demandé un
coup de pouce `.iri`.
