# V12 — Binder avancé & frontières de modèle

> **Cadrage** (phase 1). V12 regroupe les **limites de fond** d'Iron révélées par le stress-test V11-test-2
> (80 crates). Ce ne sont **ni des bugs ni des bindings ratés** : ce sont les deux **frontières dures** du
> modèle, infranchissables sous la contrainte « **correction globale, zéro cas-par-cas** » sans étendre le
> langage lui-même. Le durcissement V11-test-2 a corrigé tout le reste (B5–B19 : 10 crates verts, 4/5 murs).

## Contexte — ce qui est déjà fait (V11-test-2, hors V12)

Tous corrigés en **règles globales** (aucun nom de crate dans le compilo), gate VERT à chaque pas :
- B16 **traits blanket matérialisés** · B17 **struct/enum externes par littéral** · B18 **args par valeur** ·
  B19 **méthodes de trait sur primitifs + méthodes fournies** · 52700 **binding multi-crate des façades**.

## Les deux chantiers V12

| Spec | Titre | Sous-système | Révélé par |
|---|---|---|---|
| [24100] | Suivi des paramètres de type génériques + monomorphisation des scalaires | 2 (types/inférence) | `geo`, `nalgebra`, `rustfft` |
| [52800] | Modèle d'io par flux (`Write`/`Read`/`BufRead`, puits/sources d'octets) | 5 (interop) | `flate2` (`GzEncoder<W: Write>`) |
| [52900] | Interop **serde typé** (derive + de/serialize générique) — *note*, pas encore spec'é | 5 (interop) | `toml` (doc), `jsonwebtoken`, `bincode` |
| [50500] | Binder des **constantes** publiques d'un crate (`pub const X`) — *gap mineur* | 5 (interop) | `base64` (`STANDARD`) |
| [30200] | **Retour emprunté lié à l'entrée** (parseurs zéro-copie : `Smf::parse(&buf) -> Smf<'a>`) — la valeur emprunte son entrée ; durée de vie à suivre. *note* | 3 (ownership) | `midly` |
| [82000] | **Ergonomie des octets** : type `bytes`, `str.encode()/.decode()`, type d'élément d'un itérateur externe (`for b in digest` → `b: u8`), hex. *note* | 8 (stdlib) | `sha2` (hex/PoW), `flate2` |

## Pourquoi ce sont des frontières (et pas du durcissement)

- **24100 — généricité de type.** Iron ne **suit pas** les paramètres de type d'un type externe : `Point<f64>`
  et `Point<i32>` sont le même `Extern("Point")`. Les API génériques sur le **scalaire** (`new(x: T) -> Self`,
  `distance() -> T`) ne se bindent pas : `T` n'est mappable que si on reconnaît sa borne comme numérique —
  or `CoordNum`/`Num` sont des **traits de crate** (interdits). C'est une extension du **système de types**.
- **52800 — io par flux.** L'écosystème io (`flate2`, `image`, lecteurs/écrivains…) repose sur `Write`/`Read`
  consommant des **puits/sources d'octets en streaming**, avec out-params `&mut Vec`. Iron est **par valeur** :
  pas d'état mutable partagé en flux. C'est une extension du **modèle d'exécution/interop**.
- **52900 — serde typé.** `from_str::<T>`/`to_string(&T)` exigent un `T` **dérivant** `Serialize`/`Deserialize`
  ; Iron n'a pas de **macros derive** ni de génériques de type suivis. La voie **dynamique** (`Value` :
  `serde_json` ✅, `toml::Value` partiel) marche déjà — c'est la voie **typée** qui est la frontière. Lié à
  [24100] (génériques de type). À spec'er si un besoin concret le justifie.

> **Note de méthode.** On n'ajoute ici **que** des frontières de modèle confirmées par tentative (pas des
> suppositions). Si le déroulé restant des 80 en révèle d'autres de même nature, on les **ajoute à ce
> cadrage** (nouvelle spec `Vn/XXXXX.md`), on ne les patche pas en cas-par-cas.

## Murs « durs du langage » déjà connus (non-objectifs V12, sauf décision)

- **`no_std` / embarqué (ESP32)** — Iron émet du `std` et jamais d'`unsafe` (I2). Mur déclaré dès V11-test-2.
- **GUI / jeux / async lourd** — boucles d'événements + état mutable global vs sémantique par valeur.
