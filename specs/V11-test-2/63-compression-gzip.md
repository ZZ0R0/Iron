# 63 — Compression Gzip (compress/décompress)
> **Domaine** Crypto & compression · **Crate(s)** `ironc add flate2` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Compresser et décompresser des fichiers au format **gzip**, avec un niveau réglable, et afficher le ratio. Interopérable avec le `gzip` du système.

## Ce que ça teste dans Iron
- Le motif **adaptateur sur `Write`/`Read`** : `GzEncoder` enveloppe un écrivain, `GzDecoder` un lecteur — Iron doit chaîner ses flux `std` avec un type générique du crate (`GzEncoder<W>`).
- L'enum **`Compression`** (niveau 0–9) mappée 1:1 depuis Rust.
- `Result`/`io::Result` sur l'IO et le `finish()` qui restitue le flux sous-jacent.

## Approche
Ouvrir l'entrée, envelopper le fichier de sortie dans un `GzEncoder` au niveau choisi, **copier** les octets, puis `finish()`. La décompression lit au travers d'un `GzDecoder`. Variante simple en mémoire possible (`compress`/`decompress` sur `Vec<u8>`). Vérifier l'interop : `gzip -d` doit ouvrir la sortie d'Iron, et l'inverse.

## Risque / limites attendues
- 🟢 API « feuille » bien dégrossie ; le seul point d'attention est le **paramètre générique `GzEncoder<W>`** — Iron doit résoudre le type de l'écrivain enveloppé (un fichier, un `Vec`). Devrait passer.
- La sémantique par valeur cogne un peu avec « l'encodeur **possède** l'écrivain puis le rend via `finish()` » : à surveiller, mais c'est un *move* propre.
- `flate2` choisit un backend (miniz_oxide pur Rust par défaut) : rester sur ce défaut évite toute dépendance C.

## Done
- Aller-retour d'un fichier texte → identique ; ratio affiché.
- Interop système vérifiée (`gzip`/`gunzip`).
- Le niveau change effectivement la taille de sortie.
