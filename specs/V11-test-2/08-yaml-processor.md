# 08 — Processeur YAML (lire/transformer/écrire)
> **Domaine** Données & sérialisation · **Crate(s)** `ironc add serde_yaml` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Charge un fichier YAML, applique une transformation (ajouter/renommer une clé, incrémenter une valeur dans une liste de mappings) et le réécrit. Entrée = `.yaml` ; sortie = `.yaml` transformé et re-parsable.

## Ce que ça teste dans Iron
serde_yaml expose un type dynamique `serde_yaml::Value` (avec `Mapping`, une map ordonnée). On teste la **navigation et la mutation d'un arbre de Value** imbriqué — donc enums Iron + accès map + sémantique par valeur (reconstruire vs muter en place). Bon test « document arborescent » sans dépendre d'un type dérivé.

## Approche
- `serde_yaml::from_str::<serde_yaml::Value>(&text)`.
- Naviguer : `value.get("clé")`, descendre dans `Mapping`/séquences ; construire la version transformée (approche **reconstruction** plutôt que mutation en place, alignée sur la sémantique par valeur d'Iron).
- `serde_yaml::to_string(&value)` → fichier.

## Risque / limites attendues
Faible. Le `Value` dynamique évite la derive. À valider : l'accès au `Mapping` (la clé est un `Value`, pas une `String`) et l'indexation. **Note honnête** : serde_yaml est **déprécié/non maintenu** en amont — il compile et marche, mais c'est une dette à signaler. La mutation en place (`get_mut`) cogne avec la sémantique par valeur → préférer la reconstruction. 🟢 fonctionnellement.

## Done
Un YAML `items:` / `  - n: 1`, transformé (incrément de `n`), ressort `items:` / `  - n: 2` et reste un YAML valide re-parsable. Cas de test : ajouter une clé top-level `version: 2` et vérifier sa présence après re-lecture.
