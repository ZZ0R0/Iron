# État mesuré & frontière

> Ce que Iron sait faire **aujourd'hui**, mesuré sur un corpus de **80 projets** réels. Mesuré, **pas inféré**.
> Détail crate-par-crate : [`../specs/V11-test-2/CARTE-80.md`](../specs/V11-test-2/CARTE-80.md), section
> **PASSE-MESURE-V15**.

## Le langage : complet pour son périmètre

Le **langage** (V1–V11) est complet et stable pour son sous-ensemble : front-end, inférence par contraintes
(quasi zéro annotation), sémantique par valeur + ownership, lowering `TokenStream`, source-maps, CLI. Aucune
capacité de langage non prévue n'a émergé sur le corpus.

## Le binder : couverture mesurée 16–92 %

Batterie `ironc add` sur les **73 crates uniques** du corpus → **72 bindent** (1 échec outillage : `bincode`,
rustdoc). Couverture moyenne **~62 %**.

- **🟢 Haut (≥75 %)** — sysinfo, whatlang, walkdir, num-bigint, chrono, csv, statrs, tungstenite, indicatif,
  printpdf, qrcode, tiny-skia, num-rational, rustfft, euclid, rust_decimal, sha2, hound, time, quick-xml,
  ureq, blake3, tiny_http, jsonwebtoken, fixed, pest.
- **🟡 Moyen (50–75 %)** — axum, colored, dialoguer, arrayvec, zstd, roaring, pulldown-cmark, scraper, gpx,
  linfa, rapier2d, serde_json, regex, palette, polars, image, flate2, candle, clap, ndarray, bitvec, indexmap,
  smartcore, petgraph, chacha20poly1305…
- **🔴 Bas (<50 %)** — nalgebra (41, mais **marche**), aes-gcm, geo, rusqlite, itertools, **uom (26)**, **nom (16)**.

> ⚠️ **Couverture ≠ usabilité.** nalgebra binde 41 % mais fait de la vraie géométrie (les items utiles sont
> liés). Lis toujours le **rapport** d'un crate pour savoir ce qui te manque vraiment.

## Capacités du binder (V12–V15)

- **Fondation à portée connue (V15)** : `ironc add` **dit** ce qu'il ne sait pas binder (rapport + couverture
  calculée, **zéro drop silencieux**). Système **deux couches** (gen/curé).
- **Génériques scalaires** (V12) : `Point<T>`, `Complex<T>`, `Ratio<T>`.
- **C1 — dimensions (V15)** : `Vector3.new(x,y,z)` + **héritage des méthodes de base** (`norm`/`dot`).
  Le « mur des dimensions » est **brisé** (nalgebra opérable).
- **C3 — const-génériques (V15)** : `v: ArrayVec[i64, 8] = ArrayVec.new()` (vérifié e2e).
- **Octets, serde typé, io-lecture/écriture, no_std, hex, consts, traits, idiomes** (V12–V13).
- **Curation deux-couches** : aes-gcm round-trip via `.iri` curé (vérifié).

## Vérifiés par RUN (usables, pas juste bindés)

CLI & fichiers, données (json/csv/html/rss/sql), image/QR/vectoriel-2D, audio-fichier (hound), bigint, stats
(statrs), géo-temps (chrono/chrono-tz), terminal (sysinfo/crossterm), **nalgebra** (cosinus 3D), **aes-gcm**
(round-trip), **num-rational** (fractions), **rust_decimal** (décimal exact), **arrayvec** (const-cap),
**roaring** (ensembles). ≈ **~30 confirmés usables** + nombreux haute-couverture probables.

## La frontière (honnête)

Au-delà de C1, **peu de généralités auto restent** — la frontière est du **type-level / architecture** :

| Famille | Exemples | Issue |
|---|---|---|
| Type-level lourd | uom, palette, nom | curation partielle / `rust{}` |
| Phantom types | euclid `Point2D<T,Unit>` | annoter l'unité / curer |
| Storage générique | ndarray | curer |
| AEAD / trait générique | aes-gcm | **curation** (gabarit fourni) |
| Flux / builder à état | flate2 | chaîner à la main |
| Event-loop / async / no_std / ML lourd | GUI, axum, esp-hal, candle | hors-périmètre **langage** (architecture/environnement) |

## La voie pour augmenter la couverture

1. **Curation guidée** (fiable) : le rapport dit quoi écrire → un `.iri` curé = 1 crate de plus.
   Voir [curation.md](curation.md).
2. **Plus de moteur** : ROI décroissant (C1 était le gros levier). Restent quelques polissages d'ergonomie.

> Le corpus a été **remanié en V15** : les 16 projets *matériel* (GUI/audio/GPIO/embarqué, non testables
> headless) ont été remplacés par 16 crates **exotiques durs** (type-level, const-gén., parseurs) — pour que
> le corpus **mesure le moteur**, pas l'environnement.
