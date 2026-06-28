# 01 — Gestionnaire de TODO en ligne de commande
> **Domaine** CLI & fichiers · **Crate(s)** `ironc add clap` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Un binaire `todo` qui gère une liste de tâches persistée dans un fichier (JSON ou lignes). Sous-commandes `add <texte>`, `list`, `done <id>`, `rm <id>`. Entrée = arguments CLI + fichier d'état ; sortie = liste affichée et fichier mis à jour.

## Ce que ça teste dans Iron
clap est le crate de parsing d'arguments de référence, mais c'est un **builder lourd**, doublé d'une API derive à base de macros d'attributs. On stresse la capacité du binder à exposer une **chaîne de builders** (`Command` → `Arg` → `ArgMatches`) et sa cohabitation avec de la persistance fichier std. C'est le test « le builder pattern survit-il à la sémantique par valeur d'Iron ».

## Approche
- API **builder uniquement** (la derive `#[derive(Parser)]` repose sur des macros d'attributs hors de portée du binder) : `Command::new("todo")`, `.subcommand(Command::new("add").arg(Arg::new("text")))`, `.get_matches()`.
- Dispatch par `match` sur `matches.subcommand()`.
- État : (dé)sérialisé en JSON via std, ou format simple `id|done|texte` lu/écrit ligne à ligne.

## Risque / limites attendues
La derive est **inaccessible** (macros) → builder obligatoire, plus verbeux. Les chaînes `.arg().arg()` renvoient `Command` par valeur : compatible Iron, mais le binder peut **rater les méthodes génériques** (`value_parser`, `ArgAction`) et l'accès typé `get_one::<String>()`. Repli : rester sur des valeurs string. Un `.iri` à la main peut être nécessaire pour les helpers génériques. 🟡 = builder gérable, génériques douloureux.

## Done
`todo add "acheter du pain"` puis `todo list` affiche la tâche avec un id ; `todo done 1` la marque faite ; l'état survit entre deux exécutions. Cas de test : ajouter 2 tâches, en clore une, vérifier que `list` montre bien `[x]` et `[ ]`.
