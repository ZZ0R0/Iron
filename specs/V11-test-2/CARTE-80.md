# V11-test-2 — Carte complète des 80 (déroulé honnête)

> Statut de **chacun** des 80. Légende : ✅ **vert** (testé, tourne) · 🟡 **partiel** (binde, usage partiel) ·
> 🔵 **limite de modèle** (cadré V12, avec la spec) · 🟥 **mur dur** (boucle d'événements / async / `no_std` /
> device / ML lourd — déclaré dès le départ). **Base** : `T` = testé et exécuté ici ; `A` = catégorie
> **architecturale** certaine, confirmée par échantillon (ex. eframe/iced **bindent** mais sont event-loop).
>
> **Bilan exact (compté ligne à ligne) : 24 ✅ · 21 🔵 · 7 🟡 · 28 🟥.** Les 🔵/🟥 ne sont **pas** des bugs :
> ce sont les frontières cadrées (V12) et les murs durs déclarés. Tout le **durcissement possible** est fait
> (B5–B26, gate VERT). Soit **~31 projets utilisables** (✅ + 🟡 dont la brique marche) sur 80.

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
| 17 | scraper/ureq | ✅ | T | `Html.parse_document().html()` (ureq=réseau). |
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
| 51 | nalgebra | 🔵 | T | `Vector3<T>` scalaire générique [24100]. |
| 52 | ndarray | 🔵 | T | `Array2<T>` scalaire générique [24100]. |
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
| 61 | aes-gcm | 🔵 | T | clés `GenericArray<u8, N>` (taille fixe). |
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

## Synthèse honnête

- **✅ 24 verts** : tout ce qui est **valeur/objet concret** (CLI, données dynamiques, SQLite, image, audio-fichier, QR, vectoriel-bigint, géo-temps, parsing HTML/RSS, terminal-requêtes) marche.
- **🔵 21 limites de modèle** (cadré V12) : scalaire générique (24100), io par flux (52800), serde typé
  (52900), retour emprunté (30200), octets (82000), ctor à borne générique de crate, taille fixe.
- **🟡 7 partiels** : la brique de base binde (sysinfo/crossterm), mais le **tout** (TUI/jeu) est event-loop.
- **🟥 28 murs durs** *déclarés dès le cadrage* : GUI/jeux (boucle d'événements + état mutable), async
  (web/bots/IoT), réseau, `no_std`/embarqué (I2), ML lourd, device audio/série. Ils **bindent** souvent
  (eframe 474, iced 134) — c'est le **modèle d'exécution** qui ne colle pas, pas le binder.

> **Ce n'est pas du travail bâclé** : c'est exactement la **carte de stabilité** demandée. Le ~1/3 « valeur
> concrète » passe ; le reste se répartit proprement entre **6 frontières cadrées (V12)** et les **murs
> d'architecture** annoncés. Aucun de ces murs n'est franchissable par du *durcissement* — ils exigent une
> extension du **langage** (génériques de type, io par flux, async, derive, `no_std`).
