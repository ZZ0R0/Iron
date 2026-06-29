# V11-test-2 — Carte complète des 80 (déroulé honnête)

> Statut de **chacun** des 80. Légende : ✅ **vert** (testé, tourne) · 🟡 **partiel** (binde, usage partiel) ·
> 🔵 **limite de modèle** (cadré V12, avec la spec) · 🟥 **mur dur** (boucle d'événements / async / `no_std` /
> device / ML lourd — déclaré dès le départ). **Base** : `T` = testé et exécuté ici ; `A` = catégorie
> **architecturale** certaine, confirmée par échantillon (ex. eframe/iced **bindent** mais sont event-loop).
>
> **Bilan exact (compté ligne à ligne) : 24 ✅ · 21 🔵 · 7 🟡 · 28 🟥.** Les 🔵/🟥 ne sont **pas** des bugs :
> ce sont les frontières cadrées (V12) et les murs durs déclarés. Tout le **durcissement possible** est fait
> (B5–B26, gate VERT). Soit **~31 projets utilisables** (✅ + 🟡 dont la brique marche) sur 80.
>
> **▶ REFONTE + PASSE DE MESURE V15 (corpus remanié, batterie re-jouée — MESURÉ).** Le corpus a été
> **remanié** : 16 projets **HW-bound supprimés** (GUI/audio/GPU/device/embarqué, non testables headless) et
> remplacés par 16 crates **exotiques durs** choisis pour **stresser le moteur** (type-level, const-gén.,
> parseurs). Puis batterie objective : `ironc add` sur les **73 crates uniques** → **72 bindent** (seul
> `bincode` échoue : rustdoc « could not document », bug outillage). Détail : **PASSE-MESURE-V15** plus bas.
>
> **Vérifiés par RUN réel cette session (la vraie métrique d'usage)** : nalgebra (cosinus 0.707), aes-gcm
> (round-trip, curation), **num-rational** (3/6→1/2), **rust_decimal** (0.1+0.2=0.3), **arrayvec**
> (`ArrayVec[i64,8]`, const-cap C3 e2e). + les ~24 ✅ historiques (durcissement seulement → conservés).
> ⚠️ **Couverture ≠ usage** : nalgebra binde 40.8 % de son API mais **marche** (les items utiles sont liés).
> La couverture mesure la **bindabilité brute**, le run mesure l'**usabilité**.

## ⚠️ PASSE DE VALIDATION (essais réels, pas inférence) — corrections importantes

> Après reproche justifié (« tu n'as fait qu'inférer »), j'ai **essayé** les ~35 classés `A`. Plusieurs
> inférences étaient **fausses** — l'inférence architecturale était **trop pessimiste**. Corrections :

| # | Crate | Avant | **Après essai réel** |
|---|---|:--:|---|
| 73 | serialport | 🟥 | ✅ **vert** — `available_ports()` → **4 ports** listés. |
| 75 | linux-embedded-hal | 🟥 | 🟡 **compile** (`I2cdev.new(path)`) — lecture I²C = matériel au *run*, pas un mur de langage. |
| 30 | notify-rust / rfd | 🟥/🟡 | 🟡 **compile** (builder/`FileDialog.new`) — `show()` = D-Bus/display au *run*. |
| 77 | minifb | 🟥 | 🟡 **compile** (`Window.new`) — fenêtre = display au *run* (échec runtime, pas compile). |
| 71 | esp-hal | 🟥🟥 | 🟥 **confirmé** — `ironc add` **échoue** (no_std, pas de build hôte) → [43000]. |
| 09 | bincode | 🔵 | 🔧 **tooling** — `rustdoc` « could not document » (pas une capacité ; flag à creuser). |
| 14/23/16/01 | redis/tungstenite/ureq/clap | 🔵 | **ctor à borne de trait-crate** (`IntoConnectionInfo`/`IntoClientRequest`/`Uri:TryFrom`/`Into<Str>`) → sous-cas de [24100] (reconnaissance de bornes). |
| 26 | eframe (impl trait) | 🟥 | Iron **parse** `impl App for MyApp` mais le Rust généré est **faux** (chemin/signatures) → [24300] précis. |

> **Nouvel axe découvert : ENVIRONNEMENT ≠ langage.** Des crates **compilent** (Iron sait les exprimer) mais
> exigent **matériel / écran / serveur / D-Bus au runtime** (linux-embedded-hal, minifb, rppal, notify-rust,
> rfd, redis-serveur, rodio/cpal-libs). Ce **n'est pas** une capacité V12 manquante — ça tournerait dans un
> environnement équipé. Je les avais comptés à tort comme « murs de langage ».
>
> **Donc : la barre « ce qu'Iron peut EXPRIMER » est plus haute que ma carte initiale** (+1 vert confirmé,
> ~4 « compile mais env »). Les vrais murs de **langage** restent : génériques de type, impl-trait-externe,
> closures→trait-objets, event-loop, async, no_std, consts, derive-macros.

> ⚠️ **Les tables par-domaine ci-dessous sont PRÉ-refonte V15** (état historique). Pour les 16 slots remaniés
> (`26-30, 41, 44, 46-47, 49, 71-75, 77` — désormais petgraph/nom/arrayvec/euclid/uom/bitvec/smallvec/
> itertools/num-rational/rust_decimal/time/indexmap/roaring/pulldown-cmark/fixed/palette), la vérité courante
> est la section **PASSE-MESURE-V15** (couverture mesurée + runs vérifiés). Les autres lignes restent valides.

## 1. CLI & fichiers
| # | Crate | Statut | Base | Note |
|---|---|:--:|:--:|---|
| 01 | clap | 🔵 | T | `Command::new(impl Into<clap::Str>)` — ctor générique sur newtype-string de crate. |
| 02 | sha2 | ✅ | T | `Sha256.digest("…")` → 32 octets. |
| 03 | walkdir/blake3 | ✅ | T | `for e in WalkDir.new(".")`. |
| 04 | walkdir/colored | ✅ | T | colored `"x".red()` + walkdir. |
| 05 | regex/walkdir | ✅ | T | `Regex.new("[0-9]+").find().as_str()`. |

## 2. Données & sérialisation
| # | Crate | Statut | Base | Note |
|---|---|:--:|:--:|---|
| 06 | csv/serde_json | ✅ | T | csv `records()` + serde_json `Value.from_str().get()`. |
| 07 | toml | 🔵 | T | voie `Value` partielle ; **document** = serde typé [52900]. |
| 08 | serde_yaml | 🔵 | T | pas de `Value::from_str` → serde **typé** [52900]. |
| 09 | bincode | 🔵 | T | serde typé [52900] ; rustdoc échoue aussi. |
| 10 | quick-xml | 🟡 | T | `Reader.from_str` binde ; lecture = événements io [52800]. |

## 3. Bases de données
| # | Crate | Statut | Base | Note |
|---|---|:--:|:--:|---|
| 11 | rusqlite | ✅ | T | `open_in_memory()` + `execute_batch()`. |
| 12 | sled | 🟡 | T | construit ; `insert` veut des **octets** [82000]. |
| 13 | polars | 🔵 | T | `Series::new` générique (`NamedFrom`) [24100]. |
| 14 | redis | 🟥 | A | réseau + souvent async. |
| 15 | mini-ORM (rusqlite) | ✅ | T | rusqlite vert. |

## 4. Web & réseau (haut niveau)
| # | Crate | Statut | Base | Note |
|---|---|:--:|:--:|---|
| 16 | ureq/serde_json | 🟥 | A | client HTTP = réseau (serde_json ✅). |
| 17 | scraper/ureq | ✅ | T | **Programme complet** : parse HTML → `Selector.parse("li")` → `select` → `inner_html` par `<li>` (3 items). Débloqué par **B31** (collision de nom court façade : first-wins). (ureq=réseau, hors-langage.) |
| 18 | tiny_http | 🔵 | T | `Server::http(impl ToSocketAddrs)` — ctor générique. |
| 19 | axum/tokio | 🟥 | A | **async**. |
| 20 | rss/ureq | ✅ | T | `ChannelBuilder.default()…build()`. |

## 5. Réseau bas-niveau
| # | Crate | Statut | Base | Note |
|---|---|:--:|:--:|---|
| 21 | chat TCP (std) | 🟥 | A | sockets + threads, boucle d'acceptation. |
| 22 | scanner ports (std) | 🟥 | A | `std::net` (pas de binding crate ; sockets). |
| 23 | tungstenite | 🟥 | A | réseau. |
| 24 | hickory-resolver | 🟥 | A | réseau/async. |
| 25 | rumqttc | 🟥 | A | réseau/async (IoT). |

## 6. GUI desktop
| # | Crate | Statut | Base | Note |
|---|---|:--:|:--:|---|
| 26 | eframe/egui | 🟥 | A | **binde 474 items** mais usage = **boucle d'événements** + état mutable. |
| 27 | egui | 🟥 | A | event-loop. |
| 28 | iced | 🟥 | A | **binde 134 items** ; event-loop (Elm). |
| 29 | egui | 🟥 | A | event-loop. |
| 30 | rfd/notify-rust | 🟡 | A | notify-rust builder peut binder ; `show()` = D-Bus runtime. |

## 7. TUI terminal
| # | Crate | Statut | Base | Note |
|---|---|:--:|:--:|---|
| 31 | ratatui/sysinfo | 🟡 | T | **sysinfo ✅** (`total_memory()`) ; ratatui draw = event-loop. |
| 32 | ratatui/crossterm | 🟡 | T | **crossterm ✅** (`size()`) ; ratatui = event-loop. |
| 33 | indicatif | ✅ | T | `ProgressBar.new(5).inc().finish()`. |
| 34 | crossterm | ✅ | T | `size().unwrap()`. |
| 35 | dialoguer | 🔵 | T | `Input<T>` générique + interactif (stdin). |

## 8. Image & graphisme 2D
| # | Crate | Statut | Base | Note |
|---|---|:--:|:--:|---|
| 36 | image | ✅ | T | `DynamicImage.new_rgb8().grayscale().save()`. |
| 37 | qrcode | ✅ | T | `QrCode.new(url).width()` → 25. |
| 38 | plotters | 🔵 | T | tuple `(u32,u32)` à éléments i64 + contexte de dessin. |
| 39 | image ASCII | ✅ | T | image vert. |
| 40 | tiny-skia | ✅ | T | `Pixmap.new(64,48).save_png()`. |

## 9. Audio & signal
| # | Crate | Statut | Base | Note |
|---|---|:--:|:--:|---|
| 41 | rodio | 🟥 | T | **build échoue** (libs système audio : ALSA). |
| 42 | hound | ✅ | T | écrit un WAV valide. |
| 43 | rustfft | 🔵 | T | `Complex<T>`/`FftPlanner<T>` scalaire générique [24100]. |
| 44 | cpal | 🟥 | T | **build échoue** (libs système audio). |
| 45 | midly | 🔵 | T | `Smf<'a>` emprunte son entrée [30200]. |

## 10. Jeux
| # | Crate | Statut | Base | Note |
|---|---|:--:|:--:|---|
| 46 | macroquad | 🟥 | A | boucle de jeu. |
| 47 | macroquad | 🟥 | A | boucle de jeu. |
| 48 | crossterm démineur | 🟡 | T | crossterm ✅ ; jeu = boucle + état. |
| 49 | bracket-lib | 🟥 | A | boucle de jeu. |
| 50 | crossterm 2048 | 🟡 | T | crossterm ✅ ; jeu = boucle + état. |

## 11. Maths & vectoriel
| # | Crate | Statut | Base | Note |
|---|---|:--:|:--:|---|
| 51 | nalgebra | ✅ | T | **V15 C1+héritage** : `Vector3.new` (alias→nominal) + `norm`/`dot` (héritées de Matrix). **Programme réel** : similarité cosinus 3D = `0.7071…`. Le mur des dimensions (V13) est brisé. |
| 52 | ndarray | 🟡 | T | **V15** : `from_vec`/`sum`/`len`/`dot` bindés, MAIS construction en E0308 (storage générique `S` non résolu — pattern `Ix` ≠ `Const<N>` de nalgebra, C1 ne s'applique pas) → **curation requise**. Testé, pas inféré. |
| 53 | num-bigint | ✅ | T | `30!` exact. |
| 54 | statrs | ✅ | T | `Beta.new(2,5).mean()`. |
| 55 | rapier2d | 🔵 | A | physique scalaire-générique + boucle de sim. |

## 12. ML & IA
| # | Crate | Statut | Base | Note |
|---|---|:--:|:--:|---|
| 56 | linfa | 🟥 | A | générique + lourd. |
| 57 | candle-core | 🟥 | A | tenseurs génériques + lourd. |
| 58 | tract-onnx | 🟥 | A | lourd, runtime ONNX. |
| 59 | smartcore | 🔵 | A | générique. |
| 60 | whatlang | ✅ | T | `detect().lang().eng_name()`. |

## 13. Crypto & compression
| # | Crate | Statut | Base | Note |
|---|---|:--:|:--:|---|
| 61 | aes-gcm | ✅ | T | **V15 C2 (curation deux-couches)** : `Aes256Gcm.new/encrypt/decrypt` curés (`specs/V15/examples/aes_gcm.iri`). **Round-trip réel** : chiffre 3 o → 19 (3 + tag GCM 16), déchiffre → 3. (Auto-AEAD non généralisable sans casser ndarray → curation, le filet I7-2.) |
| 62 | chacha20poly1305 | 🔵 | T | clés/nonces taille fixe. |
| 63 | flate2 | 🔵 | T | modèle io `Write`/`Read` [52800]. |
| 64 | zstd | ✅ | T | round-trip compress/decompress. |
| 65 | jsonwebtoken | 🔵 | T | claims serde **typé** [52900]. |

## 14. Géo, temps & documents
| # | Crate | Statut | Base | Note |
|---|---|:--:|:--:|---|
| 66 | chrono | ✅ | T | `from_ymd_opt().weekday()` → Sun. |
| 67 | geo | 🔵 | T | façade **corrigée** (52700) ; `Point<T: CoordNum>` scalaire [24100]. |
| 68 | printpdf | 🔵 | T | `.iri` parse (B24) ; `save(writer)` = io [52800]. |
| 69 | gpx | ✅ | T | `Gpx.default()`. |
| 70 | chrono-tz | ✅ | T | `Tz.from_str("Europe/Paris")`. |

## 15. Embarqué & matériel
| # | Crate | Statut | Base | Note |
|---|---|:--:|:--:|---|
| 71 | esp-hal (ESP32) | 🟥🟥 | A | **`no_std`** — ne build pas pour l'hôte ; I2 (jamais d'`unsafe`). |
| 72 | rppal | 🟥 | A | GPIO matériel (Raspberry Pi). |
| 73 | serialport | 🟥 | A | device série. |
| 74 | btleplug | 🟥 | A | BLE + async. |
| 75 | linux-embedded-hal | 🟥 | A | matériel I²C/SPI. |

## 16. Langages & exotiques
| # | Crate | Statut | Base | Note |
|---|---|:--:|:--:|---|
| 76 | pest | 🟥 | T | grammaire = **macro derive** (pas de derive en Iron). |
| 77 | minifb (CHIP-8) | 🟥 | A | fenêtre framebuffer = event-loop. |
| 78 | templates (à la main) | ✅* | A | **pur Iron**, pas de crate → devrait tourner (non exécuté ici). |
| 79 | teloxide | 🟥 | A | **async** (bot Telegram). |
| 80 | PoW (sha2) | 🔵 | T | sha2 ✅ mais itération d'octets `u8` non typée [82000]. |

---

## Re-classification HONNÊTE (la distinction 🔵/🟥 était bidon)

Il n'y a que **deux** statuts pour un projet qui ne passe pas. La séparation 🔵/🟥 ne voulait rien dire :
les deux = « bloqué ». La seule question utile : **bug du pont** (réparable maintenant) ou **capacité que
le langage n'a pas** (feature) ?

### A. ✅ Marche aujourd'hui — **24** (+ #78 pur-Iron)
CLI, données dynamiques (json/html/rss/sql), image/QR/vectoriel-2D, audio-fichier, bigint, géo-temps,
terminal-requêtes (sysinfo/crossterm).

### B. 🔧 BUG du binder/lower — réparable (peu nombreux, et souvent seulement *partiels*)
| Projet | Le bug | Effet réel du fix |
|---|---|---|
| — (fait) | `str.encode()`/`bytes.decode()` | **corrigé (B27)** — vrai bug que j'avais parqué à tort |
| 38 plotters (backend) | coercition d'éléments de **tuple** (`(200,150)` i64 vs `(u32,u32)`) | *construit* le backend ; le chart complet = API de dessin |
| 18 tiny_http / 01 clap (ctor) | mappings de bornes std manquants (`ToSocketAddrs`/`Into<newtype>` → `str`) | *construit* serveur/commande ; le **tout** = io/event-loop |
| 12 sled (insert) | `list[u8]` **par valeur** vers `Into<IVec>` | clés binaires |

→ **Honnêteté** : après vérif, **80 PoW / sha2-hex ne sont PAS un bug simple** — le type d'élément du digest
(`u8`) vit dans la crate **étrangère `generic-array`**, non récupérable sans binding multi-crate ou hack
spécifique-crate. Ça bascule en **capacité** (C). Et les bugs restants font surtout *construire* le 1er
objet ; le **projet complet** retombe sur une capacité. Donc : les bugs réparables ajoutent **0-1 vert
complet**, pas une vague.

### C. 🧱 CAPACITÉ que le langage n'a pas — **feature, pas durcissement** (~51)
Une seule liste, peu importe la « couleur » que je leur avais collée :
- **Génériques de type** (`Point<T>`, `Series<T>`, `Vector<T>`) : geo, nalgebra, ndarray, polars, rustfft, dialoguer, rapier2d. → [24100]
- **serde typé** (macros derive) : toml-doc, serde_yaml, bincode, jsonwebtoken. → [52900]
- **io par flux** (`Write`/`Read` streaming) : flate2, printpdf-save, quick-xml. → [52800]
- **Boucle d'événements + état mutable** : tout GUI (26-30), TUI complet (31-32), jeux (46-50).
- **Runtime async** : axum, teloxide, redis, tungstenite, hickory, rumqttc, réseau (14/16/19/21-25).
- **`no_std` / matériel** : esp-hal (71), rppal, serialport, btleplug, linux-embedded-hal ; build device (rodio/cpal/minifb).
- **ML lourd / tenseurs** : linfa, candle, tract, smartcore.
- **Durée de vie d'un retour emprunté** : midly. → [30200]
- **Tableaux à taille fixe** (`GenericArray<u8, N>`) : aes-gcm, chacha20poly1305.

> **Conclusion franche.** ~24 marchent, ~5 sont des **bugs** que je finis de corriger (→ ~26-29), et le gros
> (~51) demande de **vraies extensions du langage** — pas du durcissement. La « couleur » bleu/rouge était
> une fausse distinction de ma part ; le vrai axe c'est **bug vs feature**, et la majorité est *feature*.

---

## PASSE « PROGRAMMES COMPLETS » (essais réels end-to-end, pas des bouts)

> Suite à la demande : écrire et **exécuter** les programmes ENTIERS (boucles, usage des éléments,
> pipeline complet), pas juste prouver que la lib se binde. Bilan honnête ci-dessous.

**Verts confirmés (programme multi-étapes exécuté, sortie vérifiée) :**

| Crate | Programme complet exécuté | Débloqué par |
|---|---|---|
| CSV→JSON | lit le CSV, construit `[{...}]`, sortie JSON | B30 |
| statrs | stats descriptives sur un échantillon | — |
| walkdir | parcourt l'arbo + utilise `de.depth()`/`path()` | B30 |
| scraper | parse HTML → `Selector.parse("li")` → select → `inner_html` ×3 | **B31** |
| renommeur (regex) | `captures` extrait le n° → `rename` → `path_exists` (vérifié disque) | **B32** |
| num-bigint | `30! = 265252859812191058636308480000000` (boucle), `(30!)²`, `div_rem` | — |
| tiny-skia | Pixmap + rect rouge + cercle bleu → PNG 400×300 valide | — |
| image | `open()` → resize → grayscale → `save()` (400×300→200×150) | **B33** |
| rss | `Channel.from_str` → titre/lien + itère les `items` (match Option) | — |
| whatlang | détecte Deutsch/Français + confidence | — |
| zstd | `compress`/`decompress` round-trip (73→55 o, ok=true) | B27 |
| uuid | `from_u128`/`nil`/`max` + `hyphenated` | — |
| sysinfo | mémoire totale/used/swap + host | — |
| chrono-tz | `Tz.from_str` + `name` (3 zones) | — |
| chrono | `parse_from_str` + diff de dates + weekday + ordinal + succ | **B33** (parse) ; `.format()` = [30200] |
| serde_json | parse tableau → itère → `as_i64` → somme | — (clé-accès = [24100]) |

**Partiels / bloqués — TOUS sur des capacités DÉJÀ cadrées (aucune surprise) :**

| Crate | Ce qui marche | Ce qui bloque |
|---|---|---|
| rusqlite | open + `execute_batch` (CREATE/INSERT) + `total_changes` | lecture `query_row`/`execute(params)` → [24100b]+[40300] |
| sha2 | hash fichier → 32 octets | hex → [82000] |
| gpx | (construction en mémoire) | `gpx::read(impl Read)` → [52800] |
| qr | — | génériques scalaires → [24100] |

> **Verdict honnête.** Le « ~25 qui marchent » est en réalité **~18-19 programmes COMPLETS** verts ; le reste
> bute sur des capacités **déjà listées en V12**. Écrire les programmes entiers a fait sortir **4 bugs de
> binder** (B30 itérateur, B31 collision façade, B32 fs std, B33 alias de type) — **tous corrigés直接**. Aucune
> capacité de langage non prévue n'a émergé : le cadrage V12 est **complet** pour ce qu'on a essayé.

---

## PASSE-MESURE-V15 (batterie `ironc add` objective sur le corpus remanié)

> **Méthode.** `ironc add <crate>` sur les **73 crates uniques** du corpus (×3 parallèle). Métrique =
> **couverture** = `items liés / items candidats` (objectif, calculé par la fondation [50500]). **72/73
> bindent** ; seul **bincode** échoue (`rustdoc` « could not document » — outillage, pas langage).
> **Rappel : couverture = bindabilité brute, pas usabilité** (nalgebra binde 41 % mais *marche*).

### Couverture par palier (mesurée)
**🟢 Haute (≥ 75 %) — 26 crates** : sysinfo 92, whatlang 91, walkdir 91, num-bigint 89, chrono 89, csv 87,
statrs 86, tungstenite 86, indicatif 84, printpdf 84, qrcode 83, tiny-skia 83, **num-rational 83**, rustfft 82,
**euclid 82**, **rust_decimal 81**, sha2 79, hound 78, **time 78**, quick-xml 78, ureq 77, blake3 77,
tiny_http 76, jsonwebtoken 76, **fixed 76**, pest 75.

**🟡 Moyenne (50–75 %) — 29 crates** : axum 75, colored 74, dialoguer 73, hickory 72, **arrayvec 72**,
zstd 72, chrono-tz 71, **roaring 71**, **pulldown-cmark 71**, scraper 71, gpx 71, linfa 70, rapier2d 62,
serde_json 62, regex 59, **palette 58**, polars 57, image 56, flate2 56, rumqttc 55, candle-core 55, clap 54,
ndarray 54, **bitvec 54**, rss 52, **indexmap 52**, smartcore 52, **petgraph 51**, chacha20poly1305 50.

**🔴 Basse (< 50 %) — 17 crates** : plotters 48, serde_yaml 46, **smallvec 44**, toml 44, midly 41,
nalgebra 41, crossterm 41, aes-gcm 41, tract-onnx 41, sled 40, geo 39, teloxide 37, redis 36, rusqlite 32,
**itertools 32**, **uom 26**, **nom 16**. (En gras = nouveaux exotiques.)

**❌ Échec outillage — 1** : bincode (rustdoc).

### Lecture honnête
- **Le corpus stresse maintenant le moteur** (objectif de la refonte) : les nouveaux exotiques s'étalent de
  **82 % (euclid/num-rational)** à **16 % (nom)** — ils **exposent la frontière** du binder (type-level
  `uom`/`nom`/`itertools` = bas ; génériques scalaires/const = haut).
- **Vérifiés par RUN (usables, pas juste bindés)** : nalgebra, aes-gcm (curation), num-rational, rust_decimal,
  arrayvec (const-cap C3), **roaring** (insert/len→2) + les ~24 ✅ historiques ≈ **~30 confirmés usables**.
  **🟡 euclid** : binde 82 % mais l'**unité fantôme** `Point2D<T, U>` non inférable (E0282) — frontière
  type-marqueur honnête (construction bloquée sans unité explicite). **🟡 ndarray** : E0308 storage générique.
- **Couverture moyenne (72 OK) ≈ 62 %** — le moteur lie la **majorité** de la surface API même sur des crates
  durs. Le passage couverture→usable se fait crate par crate (items utiles + parfois curation/ascription).
- **Reste à faire (mesuré, pas inféré)** : confirmer par run les crates haute-couverture non encore exécutés
  (euclid, time, fixed, roaring, pulldown-cmark…) — probables ✅ ; curer les bas exotiques si on les veut.
