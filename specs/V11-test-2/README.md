# V11-test-2 — 80 projets pour stress-tester Iron (variété maximale)

> **Objectif : la stabilité d'Iron.** Pas « facile à coder » — mais **importer un max de crates** et couvrir
> un **max de domaines** (CLI, données, web, réseau, GUI, TUI, image, audio, jeux, maths/vectoriel, ML,
> crypto, géo, **embarqué/ESP32**, langages…). Chaque projet = un **crate différent** + un **use-case
> différent**. Difficulté **modérée** (le *scope* reste petit), mais le *domaine* peut être exotique.
>
> On les réalisera plus tard, en prenant notre temps. Ici on les **imagine et on les note**.

## Le vrai but : cartographier la frontière d'Iron

Iron = Python→Rust natif, **écosystème = crates.io** (`ironc add <crate>` génère le binding `.iri`). Mais :
- le **binder auto** (V8) gère bien les crates « feuilles » (API simples) ; il **galèrera** sur les API
  **génériques/builders/lazy** (GUI, async lourd, GPU, ML).
- Iron **n'émet jamais d'`unsafe`** (I2) et émet du **`std`** → l'**embarqué `no_std`** (ESP32) est un **mur dur**.
- la **sémantique par valeur** + **pas d'état global mutable** cognent avec les **boucles d'événements** (GUI/jeux).

D'où un **niveau de risque** par projet — c'est *ça* qu'on teste :

> 🟢 **devrait passer** (crate à API simple) · 🟡 **modéré** (builders/génériques, un `.iri` à la main peut suffire)
> · 🔴 **frontière** (boucle d'événements, async lourd, GPU, `no_std`/unsafe — révèle les limites dures d'Iron).

Chaque fiche `NN-nom.md` détaille : **but · ce que ça teste dans Iron · approche · risque/limites attendues · done**.

---

## 1. CLI & fichiers
| # | Projet | `ironc add` | Risque |
|---|--------|-----------|:--:|
| 01 | Gestionnaire de TODO en ligne de commande | `clap` | 🟡 |
| 02 | Hacheur de fichiers (checksums récursifs) | `sha2` `walkdir` | 🟢 |
| 03 | Détecteur de doublons de fichiers | `walkdir` `blake3` | 🟢 |
| 04 | `tree` — arborescence colorée | `walkdir` `colored` | 🟢 |
| 05 | Renommeur en masse par regex | `regex` `walkdir` | 🟢 |

## 2. Données & sérialisation
| # | Projet | `ironc add` | Risque |
|---|--------|-----------|:--:|
| 06 | Convertisseur CSV → JSON | `csv` `serde_json` | 🟢 |
| 07 | Validateur/formateur TOML | `toml` | 🟢 |
| 08 | Processeur YAML | `serde_yaml` | 🟢 |
| 09 | Sérialisation binaire compacte | `bincode` | 🟡 |
| 10 | Parseur XML (extraction de champs) | `quick-xml` | 🟡 |

## 3. Bases de données
| # | Projet | `ironc add` | Risque |
|---|--------|-----------|:--:|
| 11 | Carnet de notes SQLite | `rusqlite` | 🟢 |
| 12 | Magasin clé-valeur embarqué | `sled` | 🟡 |
| 13 | Moteur de requêtes sur DataFrame | `polars` | 🔴 |
| 14 | Client Redis (cache) | `redis` | 🟡 |
| 15 | Mini-ORM + migrations | `rusqlite` | 🟡 |

## 4. Web & réseau (haut niveau)
| # | Projet | `ironc add` | Risque |
|---|--------|-----------|:--:|
| 16 | Client HTTP + parse JSON d'API | `ureq` `serde_json` | 🟢 |
| 17 | Scraper web | `scraper` `ureq` | 🟡 |
| 18 | Serveur HTTP de fichiers statiques | `tiny_http` | 🟢 |
| 19 | API REST (async) | `axum` `tokio` | 🔴 |
| 20 | Lecteur de flux RSS | `rss` `ureq` | 🟢 |

## 5. Réseau bas-niveau & protocoles
| # | Projet | `ironc add` | Risque |
|---|--------|-----------|:--:|
| 21 | Serveur de chat TCP multi-clients | std `net` + threads | 🟡 |
| 22 | Scanner de ports / ping UDP | std `net` | 🟢 |
| 23 | Client WebSocket | `tungstenite` | 🟡 |
| 24 | Résolveur DNS | `hickory-resolver` | 🟡 |
| 25 | Client MQTT (IoT) | `rumqttc` | 🔴 |

## 6. GUI desktop
| # | Projet | `ironc add` | Risque |
|---|--------|-----------|:--:|
| 26 | Calculatrice GUI | `eframe`/`egui` | 🔴 |
| 27 | Éditeur Markdown (aperçu live) | `egui` | 🔴 |
| 28 | Visionneuse d'images | `iced` | 🔴 |
| 29 | Sélecteur de couleurs | `egui` | 🔴 |
| 30 | Dialogues natifs + notifications | `rfd` `notify-rust` | 🟡 |

## 7. TUI terminal
| # | Projet | `ironc add` | Risque |
|---|--------|-----------|:--:|
| 31 | Tableau de bord système (htop-like) | `ratatui` `sysinfo` | 🔴 |
| 32 | Gestionnaire de fichiers TUI | `ratatui` `crossterm` | 🔴 |
| 33 | Barres de progression / spinners | `indicatif` | 🟢 |
| 34 | Lecteur de logs temps réel | `crossterm` | 🟡 |
| 35 | Quiz interactif | `dialoguer` | 🟢 |

## 8. Image & graphisme 2D
| # | Projet | `ironc add` | Risque |
|---|--------|-----------|:--:|
| 36 | Filtres d'image (flou, niveaux de gris) | `image` | 🟢 |
| 37 | Générateur de QR codes | `qrcode` `image` | 🟢 |
| 38 | Graphiques/courbes en PNG | `plotters` | 🟡 |
| 39 | Art ASCII depuis une image | `image` | 🟢 |
| 40 | Dessin vectoriel 2D | `tiny-skia` | 🟡 |

## 9. Audio & signal
| # | Projet | `ironc add` | Risque |
|---|--------|-----------|:--:|
| 41 | Lecteur audio (MP3/WAV) | `rodio` | 🟡 |
| 42 | Générateur de sons / synthé | `hound` | 🟢 |
| 43 | Visualiseur de spectre (FFT) | `rustfft` `hound` | 🟡 |
| 44 | Enregistreur micro | `cpal` | 🔴 |
| 45 | Lecteur/parseur MIDI | `midly` | 🟡 |

## 10. Jeux
| # | Projet | `ironc add` | Risque |
|---|--------|-----------|:--:|
| 46 | Snake | `macroquad` | 🔴 |
| 47 | Tetris | `macroquad` | 🔴 |
| 48 | Démineur (TUI) | `crossterm` | 🟡 |
| 49 | Roguelike (grille texte) | `bracket-lib` | 🔴 |
| 50 | 2048 (TUI) | `crossterm` | 🟢 |

## 11. Maths & calcul scientifique (vectoriel)
| # | Projet | `ironc add` | Risque |
|---|--------|-----------|:--:|
| 51 | Solveur d'algèbre linéaire (Ax=b) | `nalgebra` | 🟡 |
| 52 | Tableaux N-dim & statistiques | `ndarray` | 🟡 |
| 53 | Grands entiers (factorielles énormes) | `num-bigint` | 🟢 |
| 54 | Lois de probabilité & stats | `statrs` | 🟢 |
| 55 | Simulation physique (collisions) | `rapier2d` | 🔴 |

## 12. ML & IA
| # | Projet | `ironc add` | Risque |
|---|--------|-----------|:--:|
| 56 | Régression linéaire / k-means | `linfa` | 🟡 |
| 57 | Réseau de neurones (MNIST) | `candle-core` | 🔴 |
| 58 | Inférence ONNX (classif image) | `tract-onnx` | 🔴 |
| 59 | Arbre de décision | `smartcore` | 🟡 |
| 60 | Détection de langue + stemming | `whatlang` `rust-stemmers` | 🟢 |

## 13. Crypto & compression
| # | Projet | `ironc add` | Risque |
|---|--------|-----------|:--:|
| 61 | Gestionnaire de mots de passe (AES) | `aes-gcm` `argon2` | 🟡 |
| 62 | Chiffreur de fichiers | `chacha20poly1305` | 🟡 |
| 63 | Compression Gzip | `flate2` | 🟢 |
| 64 | Archiveur Zstd | `zstd` | 🟢 |
| 65 | JWT (émission/vérification) | `jsonwebtoken` | 🟢 |

## 14. Géo, temps & documents
| # | Projet | `ironc add` | Risque |
|---|--------|-----------|:--:|
| 66 | Calendrier / planificateur | `chrono` | 🟢 |
| 67 | Distances géo & itinéraires | `geo` | 🟡 |
| 68 | Générateur de PDF (factures) | `printpdf` | 🟡 |
| 69 | Lecteur de traces GPS (GPX) | `gpx` | 🟢 |
| 70 | Convertisseur de fuseaux horaires | `chrono-tz` | 🟢 |

## 15. Embarqué & matériel *(la frontière dure)*
| # | Projet | `ironc add` | Risque |
|---|--------|-----------|:--:|
| 71 | Blink LED sur **ESP32** (`no_std`) | `esp-hal` | 🔴🔴 |
| 72 | GPIO Raspberry Pi (capteur) | `rppal` | 🔴 |
| 73 | Communication série (Arduino) | `serialport` | 🟡 |
| 74 | Scan Bluetooth LE | `btleplug` | 🔴 |
| 75 | Lecture capteur I²C/SPI (Linux) | `linux-embedded-hal` | 🔴 |

## 16. Langages, parsing & exotiques
| # | Projet | `ironc add` | Risque |
|---|--------|-----------|:--:|
| 76 | Interpréteur d'un mini-langage | `pest` (ou à la main) | 🟡 |
| 77 | Émulateur CHIP-8 | `minifb` | 🟡 |
| 78 | Moteur de templates (type Jinja) | — (à la main) | 🟢 |
| 79 | Bot Telegram | `teloxide` `tokio` | 🔴 |
| 80 | Mineur de crypto jouet (preuve de travail) | `sha2` | 🟢 |

---

**Bilan attendu** : les 🟢/🟡 valident le binder + l'interop sur ~60 domaines ; les 🔴 **cartographient les
murs** (boucles d'événements GUI/jeux, async lourd, GPU/ML, et surtout l'**embarqué `no_std`/unsafe** — là où
l'invariant « jamais d'`unsafe` » d'Iron rencontre sa limite). C'est *exactement* ce qu'un test de stabilité
doit révéler.
