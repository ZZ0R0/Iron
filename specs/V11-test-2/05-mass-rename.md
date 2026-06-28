# 05 — Renommeur de fichiers en masse par regex
> **Domaine** CLI & fichiers · **Crate(s)** `ironc add regex` `ironc add walkdir` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Renomme en lot les fichiers d'un dossier selon un motif regex et un gabarit de remplacement (avec groupes capturés `$1`). Entrée = dossier + `pattern` + `replacement` ; sortie = fichiers renommés, avec un mode « dry-run » qui affiche sans agir.

## Ce que ça teste dans Iron
regex est un crate **populaire à API propre** : `Regex::new`, `is_match`, `replace_all`. On valide la compilation d'un motif (qui renvoie un `Result`), l'usage des **groupes capturés** dans le remplacement, et le type de retour `Cow<str>`. Combiné à walkdir + `fs::rename` (std), c'est un bon test d'interop string/chemin.

## Approche
- `Regex::new(pattern)?` (gérer l'erreur de motif invalide).
- walkdir pour lister les noms ; pour chacun, `re.replace_all(name, replacement)` → nouveau nom.
- Si différent : afficher `ancien → nouveau` ; hors dry-run, `fs::rename`.

## Risque / limites attendues
Faible. Le piège côté Iron est le retour `Cow<'_, str>` de `replace_all` (enum générique + durée de vie) : le binder doit le ramener à une chaîne (`.to_string()`). Le gabarit `$1` est interprété par regex lui-même, donc rien de spécial à câbler. Risque réel = **logique** (collisions de noms, ordre des renommages) plutôt que binding. 🟢.

## Done
Avec `pattern = (\d+)` et `replacement = n$1`, `12.txt` devient `n12.txt` ; le dry-run liste les changements sans toucher au disque. Cas de test : un motif invalide affiche une erreur propre et n'effectue aucun renommage.
