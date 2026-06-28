# 37 — Générateur de QR codes (texte → PNG)

> **Domaine** Image & graphisme · **Crate(s)** `ironc add qrcode image` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Prendre une chaîne (URL, texte) en argument, générer le QR code correspondant et l'exporter en **PNG**
(modules noirs/blancs), avec une échelle/taille de module réglable.

## Ce que ça teste dans Iron
`qrcode` a une **entrée simple** (`QrCode::new(bytes)`) ; la difficulté est la **sortie image**, et
surtout l'**interop entre deux crates** : `qrcode` produit, `image` enregistre. Deux voies :
- voie haute : `code.render::<Luma<u8>>().min_dimensions(...).build()` → un **builder paramétré par un
  type de pixel générique**, exactement le genre de tour que le binder V8 peut rater ;
- voie basse : lire la **matrice de modules** (`code.to_colors()` → liste clair/foncé) et **peindre
  soi-même** un buffer `image`. C'est cette voie qui teste le passage d'un buffer de pixels à `image`.

## Approche
`QrCode::new(text.as_bytes())`. **Plan A** : `.render::<Luma<u8>>().min_dimensions(s).build()` →
`ImageBuffer`, `.save("out.png")`. **Plan B** (si le builder générique coince) : `to_colors()` → boucle
Iron qui remplit chaque module en blocs S×S pixels dans une `ImageBuffer` construite à la main, puis `save`.

## Risque / limites attendues
🟢 sous réserve du builder. `render::<Luma<u8>>()` cumule **paramètre de type explicite + chaînage** →
candidat sérieux à l'échec du binder auto. D'où le **plan B quasi garanti** : `to_colors()` rend une
liste simple → boucle + remplissage manuel, ami du `std`. Interop 2 crates : vérifier que `ImageBuffer`/
`Luma` traverse proprement la frontière ; sinon, passer par un `Vec<u8>` brut + `save_buffer`. Pas
d'event loop ni d'état global. Attendu : passe, plan B si le `render` générique résiste.

## Done
`texte → out.png` scannable par un téléphone, échelle de module réglable. 0 `unsafe`. Noter si le
builder `render` a binté ou s'il a fallu **peindre la matrice à la main** (plan B), et comment
l'interop `qrcode`↔`image` s'est faite (type partagé vs buffer brut).
