# V13 — CADRAGE — Extensions dures du système de types / io (le reste « sans matériel »)

> **But.** Débloquer les ~12 projets des 80 qui restent **sans matériel** mais que la V12 n'a pas couverts,
> parce qu'ils exigent trois capacités **dures** que le cadrage V12 avait laissées partielles ou hors champ :
> const-génériques (typenum), durées de vie de retour, io-écriture. **On ne traite QUE ce qui est langage** ;
> les edge-cases d'outillage/API (cf. §5) ne sont pas du ressort de V13.

> **Honnêteté d'entrée (cf. session des 80).** La V12 a livré les génériques **scalaires** (geo, num-complex),
> closures, io-**lecture**, hex, `[u8;N]` littéral, derive, serde-typé, ascription, impl-trait, no_std, consts —
> ~30 crates verts. Les 19 restants « sans matériel » se ramènent à **3 sous-features**. V13 = ces 3, **par
> levier décroissant**. Une (typenum-dimensions) est **vraiment dure** et pourrait rester **partielle** — je le
> dis maintenant, pas à la fin.

## Les 3 capacités V13

| Spec | Capacité | Débloque | Difficulté |
|---|---|---|---|
| **[24500]** | **Const-génériques** (valeur au niveau type : `[u8; N]` générique, `GenericArray<u8, U32>`, `Const<N>`) | aes-gcm, chacha20 (tailles d'octets) ; nalgebra, ndarray, rustfft, rapier2d (dimensions) ; polars/smartcore (partiel) | **gros levier**, **dure** (typenum) |
| **[30200]** | **Durée de vie d'un retour emprunté** tenu (`Smf<'a>`, events, `DelayedFormat<'a>`) | midly, quick-xml (events), chrono (strftime) | moyenne-dure |
| **[52800]w** | **io-écriture** (puits `Write`, out-direction, méthode/fichier) — complète l'io de V12 | flate2, printpdf | moyenne |

## §1 — [24500] const-génériques — décision d'ABI (le cœur)

Le problème : une API encode un **nombre N au niveau type**, sous **deux formes** :
- **const-générique natif** : `[u8; N]`, `Matrix<T, Const<3>, Const<1>>`, `fn f<const N: usize>()`.
- **typenum** (type qui encode N) : `GenericArray<u8, U32>`, vieux nalgebra `Matrix<T, U3, U1>`.

**Constat clé qui simplifie** : dans le cas **octets** (aes-gcm `Key<C> = GenericArray<u8, …>`, chacha20),
on **n'a PAS besoin de connaître N**. La matérialisation `list[u8] → GenericArray<u8, _>` se fait par
`GenericArray::from_slice(&v)` / `(&v[..]).try_into()` — **rustc infère la taille** depuis la signature de
l'appelée (exactement comme `[u8; N]` en V12/[24200], qui marche déjà : blake3). Donc **pas de `Type::ConstN`
nécessaire pour les octets** — il suffit d'**étendre la reconnaissance @arr** de `[u8; N]` à
`GenericArray<u8, _>` / `Key<_>` / `Nonce<_>` (alias d'octets).

**✅ Décision d'ABI** :
- **Pas de nouveau type-niveau-const pour les octets.** On étend la **sentinelle `@arr`** (V12 [24200]) à tout
  **tableau d'octets de taille fixe** : `[u8; N]` (déjà), `GenericArray<u8, …>`, et les **alias** `Key`/`Nonce`/
  `Tag` qui s'expansent (B33) vers `GenericArray<u8, …>`. Matérialisation inchangée : `(&v[..]).try_into()` /
  `from_slice`, taille **inférée par rustc**. → débloque **aes-gcm, chacha20** (clés/nonces).
- **Le type-cipher** (`Aes256Gcm = AesGcm<Aes256, U12>`) : alias → binder le **type appliqué** comme handle
  utilisable (`Aes256Gcm.new(key)`), via l'expansion d'alias (B33) + ExternApp. Les args typenum non-octets
  (`U12`) restent **opaques** (handle `Extern("typenum::U12")`) — on n'a pas besoin de leur valeur.
- **Dimensions de matrices** (nalgebra `Const<3>` / `U3`) : **cas dur, séparé** (§1bis).

### §1bis — Dimensions de matrices (le morceau vraiment dur, peut rester partiel)
`Vector3<T> = Matrix<T, Const<3>, Const<1>, ArrayStorage<T,3,1>>` ; `Vector3::new(x,y,z)` est une **fn associée
par-dimension** (générée par macro). Le binder ne la voit pas comme un `new(T,T,T)` générique. **Honnêtement :
binder les ctors par-dimension (`Vector2/3/4::new`, `Matrix2/3::new`) est une reconnaissance spéciale, pas une
généralité.** Piste : table des **alias-dimension connus** (Vector2..6, Matrix2..6) → ctors `new(x,…)` explicites.
**Risque assumé** : nalgebra/ndarray/rustfft pourraient rester **partiels** (construction OK pour les dims
listées, pas au-delà). Je ne promets pas la généralité ici.

## §2 — [30200] durée de vie d'un retour emprunté

Iron est par valeur ; un retour `&T` **clonable** est déjà géré (idiome `clone`, D4). Les murs :
- **`DelayedFormat<'a>`** (chrono strftime) : implémente `Display` → **`.to_string()` immédiat**. **Tractable** :
  étendre l'idiome `clone` aux retours **`Display`-bornés** → émettre `(expr).to_string()` (chaîne possédée).
- **`Smf<'a>`** (midly) / **events quick-xml** : tiennent un **emprunt de l'entrée** (zéro-copie). Plus dur :
  soit on **garde l'entrée vivante** (la donnée source devient un `own` lié), soit on **clone** la sortie si
  possible. **Cas dur** — peut exiger un suivi de durée de vie partiel.

**✅ Décision** : commencer par le cas **`Display`-borné → `.to_string()`** (débloque chrono strftime, propre).
Le cas **emprunt-zéro-copie tenu** (midly/quick-xml) = prototype : cloner la sortie quand faisable, sinon **noter
comme limite** (zéro-copie vs sémantique-valeur peut ne pas se résoudre proprement).

## §3 — [52800]w io-écriture

`W: Write` collecte une sortie. Deux saveurs :
- **vers fichier** (printpdf `doc.save(w)`) : matérialiser `w` depuis un **chemin** → `BufWriter::new(File::create(p))`.
- **vers buffer** (flate2 compresse → octets) : `Vec::new()` comme puits, écrire, **récupérer** le `Vec`.

**✅ Décision** : sentinelle **`@write`** (parallèle à `@read`). En position de **paramètre de fn libre/ctor**,
le chemin-closure crée le puits (`File::create` pour un chemin, ou `Vec::new()` + retour pour un buffer). En
position de **méthode** (`doc.save(w)`), matérialisation **au site d'appel** dans `lower` (comme un coerce).
Cas buffer-retourné = pattern multi-statement → chemin-closure qui crée, écrit, et rend le `Vec`.

## §4 — Périmètre & non-périmètre

- **Dans V13** : [24500] (octets typenum → aes-gcm/chacha20 ; dimensions → nalgebra/ndarray partiel), [30200]
  (Display→to_string ; zéro-copie en prototype), [52800]w (Write fichier/buffer).
- **Hors V13 (pas du langage)** — les ~7 edge-cases : **bincode** (`rustdoc` échoue — outillage), **pest**
  (macro de grammaire), **sled** (`Into<IVec>` octets-ambigu), **jsonwebtoken/clap** (API de version). On ne
  fait **pas** de cas-par-cas pour eux.

## §5 — Critères d'acceptation (cible)

- aes-gcm : `Aes256Gcm.new(key_bytes)` chiffre/déchiffre un message (run réel) — [24500] octets.
- chrono : `date.format("%Y-%m-%d")` rend une chaîne (run réel) — [30200] Display.
- flate2/printpdf : compresse → octets / sauve un PDF sur disque (run réel) — [52800]w.
- nalgebra : `Vector3.new(1,2,3).norm()` (run réel) **si** la table dimension aboutit ; sinon noté partiel.
- Gate VERT 24/0 à chaque étape ; 0 `unsafe` (I2).

## §6 — Plafond honnête après V13
~27 (V12) + ~**8-12** (V13, selon que nalgebra/ndarray aboutissent) = **~35-39 / 80 sans matériel**. Les ~7
edge-cases (outillage/API) + les ~34 HW restent. **Je ne survends pas la généralité du typenum-dimensions.**

> **Méthode.** Cadrage figé ici (ABI : `@arr` étendu octets ; `@write` ; clone-Display). Prochaine étape =
> **planification** (vagues/pistes) **sur ton feu vert** — je ne démarre pas la réalisation de mon propre chef.
