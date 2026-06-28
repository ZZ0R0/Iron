# 03 — Détecteur de doublons de fichiers (par hash)
> **Domaine** CLI & fichiers · **Crate(s)** `ironc add walkdir` `ironc add blake3` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Parcourt un dossier, hache chaque fichier (BLAKE3) et regroupe ceux qui partagent le même hash : ce sont des doublons exacts. Entrée = un dossier ; sortie = groupes de fichiers identiques (chemins listés sous chaque hash).

## Ce que ça teste dans Iron
blake3 a une API **minimale et idiomatique** (`blake3::hash(&bytes)` en fonction libre, ou `Hasher::new().update().finalize()`), idéale pour valider l'interop sans piège. Le cœur Iron testé : construire et interroger une `HashMap<String, Vec<PathBuf>>` — collections génériques + regroupement, du pur std.

## Approche
- walkdir pour énumérer les fichiers réguliers.
- Pour chaque fichier : lire les octets (std), `blake3::hash(&data)`, `.to_hex().to_string()` comme clé.
- Insérer le chemin dans une `HashMap<String, Vec<PathBuf>>` ; à la fin, n'afficher que les entrées de longueur ≥ 2.

## Risque / limites attendues
Très faible. blake3 est l'archétype du crate « feuille » : fonction libre + struct `Hash` avec `to_hex()`. Seules subtilités côté Iron : le hash incrémental (`update` par blocs) pour les gros fichiers, et la gestion du `HashMap` de `Vec` (emprunt mutable de l'entrée pour `push`). Aucun générique exotique ni builder. 🟢 franc.

## Done
Deux fichiers au contenu identique mais aux noms différents sont signalés dans le même groupe ; un fichier unique n'apparaît pas. Cas de test : copier `a.txt` en `b.txt`, vérifier qu'ils sont listés ensemble et qu'un `c.txt` distinct est absent du rapport.
