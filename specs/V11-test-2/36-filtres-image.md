# 36 — Filtres d'image (flou, niveaux de gris, sépia)

> **Domaine** Image & graphisme · **Crate(s)** `ironc add image` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Charger une image (PNG/JPEG), lui appliquer trois filtres — **flou gaussien**, **niveaux de gris**,
**sépia** — et réenregistrer chacun en PNG. Une entrée, trois sorties.

## Ce que ça teste dans Iron
Le crate `image` côté **lecture/écriture** (`image::open`, `img.save(...)`) est simple → bon fit. Flou
(`img.blur(sigma)`) et gris (`img.grayscale()`) sont des **méthodes directes** sur `DynamicImage`. Le
sépia n'est pas fourni : il faut **itérer sur les pixels** (manipuler les canaux RGBA, *clamp* à 255),
ce qui teste :
- l'accès pixel et l'arithmétique sur `u8` ;
- le seul point délicat d'un projet sinon 🟢 : `Rgba<u8>` est **générique sur le type de canal** (le
  trait `Pixel`), terrain glissant pour le binder auto V8.

## Approche
`open` → `DynamicImage`. Flou : `.blur(2.0)`. Gris : `.grayscale()`. Sépia : convertir en `RgbaImage`,
boucler `enumerate_pixels_mut`, recalculer les composantes selon la matrice sépia (avec *clamp*), puis
`save`. Trois fichiers de sortie.

## Risque / limites attendues
🟢 avec un astérisque. `open`/`save`/`blur`/`grayscale` = appels simples, binding direct attendu. Le mur
potentiel : `DynamicImage` est un **enum** de buffers et `Rgba<u8>` un type **générique** ; `get_pixel`/
`put_pixel` renvoient/prennent un `Rgba<u8>` qu'Iron doit savoir construire/déstructurer — là, le binder
générique peut tousser. **Plan B robuste** : indexer le buffer brut (`as_raw()` → `Vec<u8>`, traiter par
groupes de 4 octets), full-`std`, sans générique. La sémantique par valeur peut **cloner** l'image à
chaque filtre (warning perf I4) — acceptable ici. Attendu : passe, sépia via buffer brut si besoin.

## Done
Entrée → 3 PNG (flou, gris, sépia) corrects à l'œil. 0 `unsafe`. Noter la voie pixel retenue (`Rgba`
typé vs buffer brut) et tout clone coûteux signalé par `--audit`.
