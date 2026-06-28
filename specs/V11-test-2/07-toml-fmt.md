# 07 — Validateur/formateur de fichiers TOML
> **Domaine** Données & sérialisation · **Crate(s)** `ironc add toml` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Valide un fichier TOML (signale les erreurs de syntaxe avec position) et le reformate canoniquement. Entrée = `.toml` ; sortie = soit un message d'erreur clair, soit le fichier ré-écrit proprement (clés ordonnées, indentation normalisée).

## Ce que ça teste dans Iron
toml est une API de (dé)sérialisation simple autour d'un type dynamique `toml::Value` (enum : table, array, string, int…). On teste le **parsing vers un Value dynamique** (sans struct dérivée), la propagation d'erreur via `Result`, et le round-trip Value→string. C'est le test « enum riche + map » sur un crate stable et courant.

## Approche
- Valider/parser : `toml::from_str::<toml::Value>(&text)`, capturer l'erreur et afficher son message (position incluse).
- Reformater : sérialiser le `Value` avec `toml::to_string_pretty(&value)`.
- CLI : `--check` (valide seulement) vs réécriture du fichier.

## Risque / limites attendues
Faible. En passant par `toml::Value` on évite toute derive. Le binder doit exposer l'enum `Value` et le type table (map). **Limite honnête** : `toml::Value` **perd les commentaires et l'ordre original** — un vrai formateur préservant les commentaires exigerait `toml_edit` (API arbre bien plus lourde, plutôt 🟡). Ici on assume la normalisation destructive. 🟢.

## Done
Un TOML valide est ré-émis sans perte de données (clés/valeurs identiques après re-parse) ; un TOML invalide affiche l'erreur et le numéro de ligne. Cas de test : `a = 1` + `[t]` + `b = 2` re-parse en une table équivalente.
