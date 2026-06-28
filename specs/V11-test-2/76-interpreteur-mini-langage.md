# 76 — Interpréteur d'un mini-langage
> **Domaine** Langages & exotiques · **Crate(s)** `ironc add pest` *(ou parser à la main)* · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Un interpréteur tree-walking d'un mini-langage : variables, arithmétique, affectations, éventuellement
`if`/`while`. Lire une source, l'évaluer, afficher les résultats. Pure logique : ni matériel, ni async,
ni réseau.

## Ce que ça teste dans Iron
Le cœur d'Iron sur du « langage » :
- **enums récursifs** pour l'AST (`Expr::Add(Box, Box)`) → Iron doit gérer l'**indirection** que Rust exige
  pour les types récursifs (`Box`) — auto ou explicite ?
- **pattern matching** profond sur l'AST et **récursion** d'évaluation ;
- un **environnement** `HashMap<String, Value>` confronté à la **sémantique par valeur** (les `Value`
  circulent par clone) ;
Côté `pest` : un **proc-macro `derive(Parser)`** lisant une grammaire `.pest` externe et exposant
`Pairs`/`Pair` + l'enum `Rule` — le binder peut buter sur ce derive génératif piloté par fichier.

## Approche
Deux voies. **À la main** (recommandée) : lexer + descente récursive ou Pratt, 100 % Iron, aucun crate —
exerce directement enums/récursion/`Result`. **Via `pest`** : tester si Iron sait piloter un derive de
proc-macro externe ; en cas d'échec, repli assumé sur le parseur manuel, sans rien changer à l'évaluateur.

## Risque / limites attendues
🟡 — la voie manuelle est solide ; le vrai test Iron est l'**AST récursif (boxing automatique ?)** et la
circulation des `Value` sous sémantique par valeur. Le risque réel est concentré sur `pest` (derive +
API `Pairs`), d'où le repli explicite prévu dès le départ.

## Done
Interpréteur qui lit un programme du mini-langage (assignations + expressions arithmétiques au minimum),
l'évalue correctement sur quelques cas, et signale proprement les erreurs (variable inconnue, parse invalide).
