# V12 — ÉTAT RÉEL D'IMPLÉMENTATION (phase 3) — **COMPLÈTE (langage/codegen)**

> Les **3 causes racines** sont attaquées et **toutes les capacités V12 ont leur part langage/codegen
> implémentée et démontrée** (run réel quand exécutable ici ; sinon « compile », le *run* exigeant
> écran/matériel/réseau — frontière environnement). Gate VERT 24/0 à chaque étape, tout poussé sur `main`.

## RACINE 1 — Pont de matérialisation (param générique borné → stratégie)

| Capacité | Statut | Preuve (run réel) |
|---|---|---|
| **[24100] génériques** | ✅ | num-complex `norm()=5` ; geo `Point.new` + `3²+4²=25` ; fixture gate |
| **[24100b] bornes `Into<Str>`** + coercition `&str` | ✅ | `Command.new("app")` |
| **[40300] closures** (`Fn`→lambda) | ✅ | walkdir `filter_entry(lambda)` → 4 |
| **[82000] octets/hex** | ✅ | sha256("hello world") + hex correct |
| **[52800] io lecture** (`Read`→Cursor) | ✅ | gpx `read(xml)` → 2 waypoints |
| **[24200] tableaux fixes** (`[u8; N]`) | ✅ | blake3 `keyed_hash(clé_32, msg)` → `0375fb06…` |

## RACINE 2 — Émission + ascription

| Capacité | Statut | Preuve |
|---|---|---|
| **[40400] derive** | ✅ | `@derive(serde::Serialize)` → `#[derive(serde::Serialize)]` + dep auto |
| **[52900] serde typé** (de/sér.) | ✅ | `p: Point = from_str("{…}").unwrap()` → x=7 y=9 ; `to_string` → `{"x":7,"y":9}` |
| **ascription de type** (langage) | ✅ | `x: T = …` → rustc infère les génériques de l'appel |
| **[24300] impl-trait externe** | ✅ | `impl Bounded for Tiny` (num_traits) compile + tourne |
| **[43000] émission no_std** | ✅ | `emit --no-std` → `cargo build --lib` VERT (`#![no_std]`+alloc, 0 unsafe) |
| **[50500] constantes** | ✅ | `BASE64_STANDARD.encode("hello world")` → `aGVsbG8gd29ybGQ=` |

## RACINE 3 — Runtime/exécution

| Capacité | Statut | Preuve |
|---|---|---|
| **[53000] async** (langage) | ✅ | `async def`+`await`+tokio → `42` |
| **[35000] event-loop** (modèle) | ✅ | `shared[State]` muté à travers une boucle (score 10→50, lives 3→2) |

## La seule chose qui reste — l'ENVIRONNEMENT, pas le langage
Ce qui ne peut PAS être prouvé **depuis ce serveur**, parce qu'il faut du matériel au *run* (et non parce que
le langage manque) : un **GUI s'affichant** à l'écran (eframe::run), un binaire embarqué **tournant sur une
carte** (esp-hal), un **appel réseau** répondant (axum/reqwest). Le langage les **supporte** (impl-trait
compile, no_std compile, async tourne) ; seul leur *run* exige écran/matériel/réseau. + aes-gcm reste plus
ergonomique à câbler que blake3 (typenum `AesGcm[Aes,NonceSize,TagSize]`), mais la **matérialisation de
tableau fixe** — le cœur de [24200] — est faite.

## Verdict
**V12 est terminée au niveau langage/codegen : les 14 capacités sont implémentées et démontrées.** Un motif de
crate nouveau s'ajoute désormais en **une entrée** (stratégie/émission) sur le pont — la dette des crate-bugs
est tarie à la source.
