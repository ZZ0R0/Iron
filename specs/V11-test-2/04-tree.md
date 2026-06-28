# 04 — `tree` : arborescence de dossier colorée
> **Domaine** CLI & fichiers · **Crate(s)** `ironc add walkdir` `ironc add colored` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Reproduit la commande `tree` : affiche l'arborescence d'un dossier avec indentation `├──`/`└──` et couleurs (dossiers en bleu, exécutables en vert, etc.). Entrée = un chemin ; sortie = arbre coloré sur stdout.

## Ce que ça teste dans Iron
colored fonctionne par **trait d'extension** (`Colorize`) ajouté à `str`/`String` : `"nom".blue().bold()`. On teste donc la capacité d'Iron à appeler des **méthodes de trait sur un type primitif** et à chaîner des transformations renvoyant `ColoredString` (qui implémente `Display`). walkdir fournit la profondeur (`entry.depth()`) pour l'indentation.

## Approche
- walkdir : `WalkDir::new(dir)`, lire `entry.depth()` et `entry.file_name()`.
- Construire le préfixe d'indentation selon la profondeur et la position (dernier enfant ou non).
- Colorer : `name.blue()` pour un dossier, `.green()` pour un exécutable, sinon brut ; tout passe par `Display`.

## Risque / limites attendues
Faible, avec **un** point à valider : l'**extension-trait-sur-primitive**. Si le binder expose bien l'impl `Colorize for str`, l'appel `"x".blue()` marche ; sinon il faut un wrapper Iron explicite. Le `ColoredString` doit s'imprimer via `Display`. Détecter « dernier enfant » impose un petit look-ahead que walkdir ne donne pas directement (bufferiser une fratrie). Couleurs désactivées hors TTY. 🟢, modulo le trait sur `str`.

## Done
Sur un dossier à sous-dossiers, l'arbre s'affiche avec les bons connecteurs et les dossiers colorés. Cas de test : `dir/{a.txt, sub/b.txt}` produit `a.txt` puis `sub/` (coloré) avec `b.txt` indenté en dessous.
