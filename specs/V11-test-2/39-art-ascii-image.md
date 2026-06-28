# 39 — Art ASCII depuis une image

> **Domaine** Image & graphisme · **Crate(s)** `ironc add image` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Convertir une image en **art ASCII** : redimensionner à une largeur cible, échantillonner la luminance
de chaque cellule, la mapper sur une rampe de caractères (` .:-=+*#%@`), puis imprimer dans le terminal
et/ou écrire un `.txt`.

## Ce que ça teste dans Iron
Même crate `image` que la fiche 36, mais usage encore plus **scalaire**, donc le **meilleur fit** du
lot : `open` → `resize` → `to_luma8()` (niveaux de gris 8 bits) → lire chaque pixel `u8` et indexer la
rampe. Pas de pixel générique (luma8 = `u8` simple), pas de builder, pas d'event loop. On teste surtout :
- les **boucles imbriquées** y/x et l'**indexation de buffer** ;
- la **construction d'une `String`** ligne à ligne ;
- l'enum `FilterType` (qualité de redimensionnement) à la frontière.

## Approche
`open`, `resize_exact(w, h, FilterType::Triangle)` en corrigeant le **ratio caractère ~2:1** (hauteur
divisée par ~2), `to_luma8()` → `GrayImage`. Double boucle : lire la luminance `u8`, normaliser 0–255 →
index dans la rampe, accumuler la ligne, `\n` en fin de ligne. Imprimer ou écrire le fichier.

## Risque / limites attendues
🟢 le plus sûr du lot — bon **témoin positif** que l'interop `image` de base fonctionne. `to_luma8()`
donne un buffer `u8` plat : si l'accès `get_pixel` générique gêne, `as_raw()` (`&Vec<u8>`) est trivial et
full-`std`. Le seul réglage (pas Iron) : corriger l'**aspect** des caractères. `FilterType` est un
**enum** du crate → vérifier qu'il binde, sinon retomber sur `Nearest`. Aucun état mutable global,
aucune closure obligatoire. Attendu : passe sans douleur.

## Done
Image → bloc ASCII reconnaissable (terminal et/ou `.txt`), largeur réglable, ratio corrigé. 0 `unsafe`.
Noter la voie d'accès pixel retenue (`get_pixel` typé vs `as_raw`) et le mapping de la rampe.
