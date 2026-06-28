# 64 — Archiveur Zstd
> **Domaine** Crypto & compression · **Crate(s)** `ironc add zstd` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Compresser/décompresser des fichiers en **Zstandard** avec un niveau réglable (1–22), et comparer tailles/temps. Interopérable avec le CLI `zstd`.

## Ce que ça teste dans Iron
- Des **fonctions one-shot très « feuille »** : `encode_all(reader, level)` / `decode_all(reader)` renvoyant `io::Result<Vec<u8>>` — le cas idéal pour le binder auto.
- Paramètre entier (niveau, `i32`), `Vec<u8>`, `Result`, IO `std`.
- Pas de trait ni de générique exotique : un bon **témoin de référence** « tout doit marcher ».

## Approche
Lire le fichier → `encode_all` au niveau choisi → écrire `.zst`. L'inverse avec `decode_all`. En option, l'API streaming (`copy_encode`) pour les gros fichiers, qui réintroduit le motif `Write`. Mesurer ratio et durée selon le niveau.

## Risque / limites attendues
- 🟢 Côté **langage**, risque quasi nul (API plate, types simples).
- **Hors langage** : `zstd` lie une bibliothèque **C** via `zstd-sys` → la *compilation* exige un toolchain C (`cc`). Ce n'est pas une limite d'Iron mais une dépendance de build à documenter.
- L'API streaming (`Write`/`Read`) est un cran au-dessus ; rester sur `encode_all`/`decode_all` garde le 🟢.

## Done
- Aller-retour octet-pour-octet identique ; interop avec le CLI `zstd` du système.
- Le niveau fait varier la taille ; durée mesurée et affichée.
- Build vert malgré la dépendance C (toolchain présent).
