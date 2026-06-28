# V12 — Capacités de langage révélées par le stress-test des 80

> **Cadrage** (phase 1). Le stress-test V11-test-2 (80 crates) a tranché : Iron passe le **tiers “valeur
> concrète”** du catalogue, et le reste bute sur des **capacités que le langage n'a pas encore**. Ce ne sont
> **ni des bugs ni des bindings ratés** (tout ça a été corrigé : B5–B29, gate VERT) — ce sont de **vraies
> extensions** du langage. V12 = la liste **complète** de ces capacités, chacune reliée aux projets qu'elle
> débloque.
>
> **Axe unique et honnête** (l'ancienne distinction « 🔵 limite / 🟥 mur » ne voulait rien dire) : un projet
> qui ne passe pas relève soit d'un **bug** (réparé), soit d'une **capacité** (ici).

## Bilan des 80 (cf. `../V11-test-2/CARTE-80.md`)

- **~24 ✅ marchent** · **~4 🔧 bugs réparés** (B27–B29 : `encode/decode`, bornes `where`, coercition tuple) ·
  **~52 🧱 capacités manquantes** (ci-dessous).

## Les capacités V12 (complètes)

| Spec | Capacité | Sous-sys. | Projets bloqués | État spec |
|---|---|:--:|---|---|
| **[24000]** | **RACINE 1 — Le pont de matérialisation** (borne de trait → stratégie). Couvre [24100]/[24100b]/[52800]/[82000]/[24200]/[40300] + param [52900]. Tarit la dette B5→B33. | 2 | *(transverse — la grande majorité des 40)* | **cadré + planifié** (`24000.md`, `planification/`) |
| **[44000]** | **RACINE 2 — La couche d'émission** (`#[derive]`, `impl ExternTrait for T`, `#![no_std]`). Couvre [40400]/[24300]/[43000]. | 4 | serde typé, GUI-à-trait, embarqué | **cadré + planifié** (`44000.md`, `planification/`) |
| *(RACINE 3)* | **Runtime/exécution** : [53000] async (tokio injecté), [35000] event-loop (= callback+impl+`shared`). **À prototyper** (faisabilité fine). | 3/5 | web/bots/IoT, GUI/jeux/TUI | planifié (vagues 11-12, gate prototype) |
| [24100] | **Génériques de type** suivis — **stratégie n°1 de [24000]** (vagues 2-3) | 2 | geo, nalgebra, ndarray, polars, rustfft, dialoguer, rapier2d, **ML** | **planifié via [24000]** |
| [52800] | **io par flux** (`Write`/`Read`/`BufRead`, puits/sources, out-params) | 5 | flate2, printpdf(save), quick-xml, image(save-stream) | spec'é |
| [52900] | **serde typé** (derive + de/serialize générique) | 5 | toml(doc), serde_yaml, bincode, jsonwebtoken | cadré |
| [24300] | **Implémenter un trait EXTERNE** sur un type Iron (`impl eframe::App for MyApp`) | 2 | GUI (eframe/iced), tout framework à trait-utilisateur | **confirmé par essai** (Iron parse `impl App for MyApp` mais émet un Rust faux : chemin/signatures) |
| [40300] | **Closures rendant un trait-objet** (`Box<dyn FnOnce -> Box<dyn App>>`), HOF avancées | 4 | GUI, callbacks de crates | confirmé (eframe `app_creator`) |
| [40400] | **Macros derive** (`#[derive(Parser)]`/`#[derive(Serialize)]`) émises sur un type Iron | 4 | pest (grammaire), serde typé (cf. 52900) | **découvert (essai pest)** |
| [24100b] | Reconnaissance de **bornes de ctor convertibles depuis chaîne** (`Into<X>`/`X: TryFrom<T>`/trait-crate) | 2 | redis, tungstenite, ureq, clap (construction bloquée) | **découvert (essais)** — sous-cas [24100] |
| [35000] | **Boucle d'événements & état mutable partagé** | 3 | **GUI** (26-30), **TUI complet** (31-32), **jeux** (46-50) | **cadré (nouveau)** |
| [53000] | **Runtime async** (`tokio`/`async fn`/`.await` exécutés) | 5 | axum, teloxide, redis, tungstenite, hickory, rumqttc, web (16/19) | **cadré (nouveau)** |
| [43000] | **Émission `no_std` + HAL encapsulé** (embarqué, sans violer I2) | 4 | esp-hal (ESP32), rppal, serialport, btleplug, linux-embedded-hal | **cadré (nouveau)** |
| [30200] | **Retour emprunté lié à l'entrée** (parseurs zéro-copie, `<'a>`) | 3 | midly, plotters(backend) | cadré |
| [24200] | **Tableaux à taille fixe** (const-génériques, `[u8; 32]`/`GenericArray`) | 2 | aes-gcm, chacha20poly1305 | cadré |
| [24400] | **Type de l'élément d'un itérateur externe** (`for x in records()` → `x: Result[StringRecord]`) | 2 | csv, walkdir (locaux) · sha2-octets (foreign) | **🔧 CORRIGÉ (B30)** pour itérateurs **locaux** (CSV→JSON complet ✅) ; reste **capacité** pour `Item` **foreign** (`GenericArray<u8>`) |
| [82000] | **Ergonomie des octets** (`bytes`, hex) — sous-cas de [24400] pour `u8` | 8 | sha2(hex), PoW(80) | partiel (encode/decode faits B27) |
| [50500] | **Constantes** publiques d'un crate (`pub const`) | 5 | base64 | gap mineur |

> Hors-langage (environnement, pas une spec) : `rodio`/`cpal`/`minifb` exigent des **libs système / une
> fenêtre** au build/run ; le ML est aussi **lourd** (dépendances). Ils relèvent de [24100]/[35000] pour le
> *langage*, mais ne tourneront de toute façon que dans un environnement équipé.
>
> Hors-V12 (tooling) : `bincode` → `rustdoc` « could not document » (flag rustdoc à creuser, pas une capacité).

## Anticipé pendant la passe des « 25 complets » — les 3 points TRANCHÉS (aucun n'est un mur)

Les 3 risques anticipés ont été **confirmés par l'essai** ; **aucun** n'est une nouvelle capacité bloquante :
- ~~**[83000] opérations fichier**~~ → **🔧 RÉSOLU (B32)** : `rename`/`copy_file`/`remove_file`/`create_dir`/
  `create_dir_all`/`remove_dir_all`/`path_exists` ajoutés **globalement** à `std.iri`. Renommeur #05 VERT.
  Listage de répertoire déjà couvert par **walkdir** (B30). Restant mineur : `std::fs::metadata` (taille/mtime)
  pour la dédup #03 — à ajouter si un projet le force.
- ~~**[52600b] construction d'un `Value` dynamique**~~ → **non bloquant** : CSV→JSON ✅ assemble le JSON par
  chaînes (`serde_json::json!{}` reste une ergonomie, pas un mur).
- ~~**dépaquetage d'un retour tuple** d'une fn externe~~ → **MARCHE** : `q, r = a.div_rem(b)` (num-bigint)
  donne `14 reste 2`. Pas un bug.

### Trouvés en écrivant les programmes complets
- **🔧 [24400] corrigé (B30)** — type d'élément d'itérateur externe local → **CSV→JSON complet ✅**.
- **🔧 collision de nom court via façade (52700) CORRIGÉE (B31)** — scraper avait **deux** `Selector`
  (`scraper::Selector` *avec `parse`* ET `selectors::parser::Selector`, fusionné par le multi-crate) ;
  au chargement le 2ᵉ **écrasait** le 1ᵉʳ → `parse` perdu → **scraper bloqué**. Fix : **first-wins** au
  chargement du `.iri` (`iri.rs`) — le crate **principal** est émis avant les sources de façade, on garde
  donc son type. Sûr : aucun homonyme dans le gate ; geo est bloqué par les génériques scalaires de toute
  façon (pas par ce choix). **scraper select complet ✅** (`Selector.parse("li")` → `scraper::Selector::parse`,
  parse HTML → select → `inner_html` sur chaque `<li>`).
- **🔧 [83000] opérations fichier RÉSOLU (B32)** — std fs élargie (`rename`/`copy_file`/`remove_file`/
  `create_dir`/`create_dir_all`/`remove_dir_all`/`path_exists`), **globale**. **Renommeur #05 VERT** :
  `IMG_001.JPG`→`photo_001.jpg`, `note.txt` ignoré (regex `captures`), vérifié sur disque.
- **🔧 expansion des alias de type CORRIGÉE (B33)** — `pub type ImageResult<T> = Result<T, ImageError>`
  (et `ParseResult<T>`, tout `XResult<T>`/`XBytes = Vec<u8>`…) était bindé en **handle opaque** → `.unwrap()`
  impossible. Le binder recense désormais les `type_alias` et les **expanse** (substitution des params),
  strictement non régressif (retombe sur le handle si l'expansion ne mappe pas). Débloque **image::open**
  (load→resize→grayscale→save, 400×300→200×150 ✅) et **chrono `parse_from_str`** (parsing de dates ✅).
- **rusqlite #20 — partiel, bloqué par capacités existantes (pas un bug)** : `open_in_memory` +
  `execute_batch` (CREATE+INSERT) + `total_changes`/`last_insert_rowid` ✅ ; mais `execute(sql, params)` et
  `query_row(sql, params, |row| …)` sont **droppés au binding** (génériques `P: Params` = trait de crate
  → [24100b] ; closure `|row| row.get::<T>(0)` → [40300]/[24100]). La **lecture** de lignes est donc bloquée
  par **[24100]+[40300] déjà cadrés** — rien de neuf.
- **gpx #?? — bloqué [52800]** : `gpx::read(reader: impl Read)` droppé au binding (entrée par flux). Le
  parse RSS, lui, marche (`Channel.from_str(str)` ✅) — la différence est *str vs `impl Read`*.

### Bilan HONNÊTE de la passe « programmes complets » (essais réels, end-to-end)
**Programmes complets VERTS** (≥ 1 programme multi-étapes exécuté, sortie vérifiée) : CSV→JSON, statrs,
walkdir, **scraper** (B31), **renommeur/regex** (B32), **num-bigint** (factorial 30!), **tiny-skia** (dessin→PNG),
**image** (B33, load→process→save), **rss** (feed→items), **whatlang**, **zstd** (round-trip), **uuid**,
**sysinfo**, **chrono-tz**, **chrono** (B33, parse+arith ; seul `.format()` strftime = [30200]), serde_json
(tableaux), + terminal/headless (colored, indicatif, crossterm, serialport). → **≈ 18-19 réellement complets.**
**Partiels / bloqués, TOUS sur des capacités DÉJÀ cadrées** (zéro nouvelle capacité non prévue) : rusqlite
(lecture [24100]+[40300]), sha2 (hex [82000]), gpx ([52800]), qr ([24100]).
**Conclusion** : écrire les **programmes entiers** (vs bouts) a révélé **4 bugs de binder** — B30 (itérateur),
B31 (collision façade), B32 (fs std), B33 (alias de type) — **tous corrigés**. **Aucune** nouvelle capacité de
langage non déjà listée dans ce cadrage n'a émergé. Le « ~25 qui marchent » est honnêtement **~18-19 complets**,
le reste butant sur les capacités V12 ci-dessus.

## Pourquoi ce sont des capacités (pas du durcissement)

- **[24100] généricité de type.** Iron ne suit pas `T` dans `Foo<T>` : `Point<f64>` ≡ `Point<i32>` ≡
  `Extern("Point")`. Les API scalaires (`new(x: T)`, `-> T`) ne bindent que si on reconnaît la borne comme
  numérique — or `CoordNum`/`Scalar` sont des **traits de crate** (interdits). Extension du **système de types**.
- **[35000] event-loop & état mutable.** GUI/jeux tournent sur `loop { handle(&mut state) }` : un **état
  mutable partagé** muté à chaque frame. Iron est **par valeur, sans état global mutable** (I-sémantique).
  C'est la frontière la plus structurante — elle touche le **modèle d'exécution**, pas le binder.
- **[53000] async.** Iron a `await` en surface mais **pas d'exécuteur** : pas de `tokio`, pas de `Future`
  piloté. Web/bots/IoT en dépendent. Extension **runtime + codegen**.
- **[43000] no_std.** Iron émet du `std` et **jamais d'`unsafe`** (I2) ; l'embarqué exige `no_std` + un HAL
  qui *encapsule* l'`unsafe`. Piste : un mode d'émission `no_std` + une crate-HAL bénie.
- **[52900] serde typé / [24100].** `from_str::<T>` exige un `T` **dérivé** ; pas de macros derive en Iron.
  La voie **dynamique** `Value` marche déjà (serde_json ✅) — c'est le **typé** qui manque.

## Architecture V12 — **3 causes racines**, pas 14 capacités

> **Décision de planification (phase 2), corrigée** : les 14 « capacités » sont des **symptômes** de **3 causes
> racines**. V12 attaque les **racines** (architecture cohérente), pas les symptômes un par un. Détail complet et
> carte des vagues : `planification/README.md`. Les leaf-specs ci-dessous restent les **contrats** ; elles se
> rangent sous une racine.

- **RACINE 1 — `[24000]` Le pont de matérialisation** (param générique borné par un trait → table « borne →
  stratégie »). Couvre **[24100]** (numérique), **[24100b]** (`Into<str>`), **[52800]** (`Read`/`Write`),
  **[82000]/[24200]** (octets/arrays), **[40300]** (`Fn`), + le param de **[52900]** (`Deserialize`). La dominante.
- **RACINE 2 — `[44000]` La couche d'émission** (`#[derive]`, `impl ExternTrait for T`, `#![no_std]`). Couvre
  **[40400]** (derive→[52900]), **[24300]** (impl-trait), **[43000]** (no_std).
- **RACINE 3 — Runtime/exécution** : **[53000]** async (tokio déjà injecté), **[35000]** event-loop (= callback
  racine 1 + impl racine 2 + `shared` existant).

> **Deux « incompatibilités invariants » que j'avais annoncées TOMBENT** : [43000] no_std n'émet pas d'`unsafe`
> (il est dans le HAL — I2 intact) ; [35000] event-loop a ses échappatoires (`shared`/`shared_sync`/`inout`).
> Restent à **prototyper** pour la *faisabilité fine* (pas la compat) : async lourd, ergonomie event-loop.

**La seule limite permanente (pas un bug) = l'ENVIRONNEMENT** : libs système (rodio/cpal), écran (minifb),
matériel (esp/rppal), réseau au run. Ces projets **compilent** mais exigent du hardware/des libs **à
l'exécution** — aucune racine ne les « répare » (~5-8 projets). Hors périmètre de toute V.

> **Méthode.** Racine 1 d'abord (vagues 1-6, débloque le plus), puis racine 2 (vagues 8-10), puis racine 3
> (vagues 11-12, **après prototype**). [50500] consts = orthogonale (vague 7). Squelette→fan-out→unification,
> e2e vert en continu. Un motif nouveau = **une entrée** (stratégie ou émission), jamais du cas-par-cas.

## ✅ Passe de validation FAITE — ce cadrage reste une hypothèse, mais affinée

J'ai **essayé** (compilation réelle) les ~35 classés par architecture. Résultats (détail :
`../V11-test-2/CARTE-80.md` § Passe de validation) :
- **Découvert 4 capacités non vues** : [24300] impl-trait-externe, [40300] closures→trait-objets, [40400]
  derive-macros (pest), [24100b] bornes de ctor convertibles (redis/tungstenite/ureq/clap).
- **Corrigé des inférences fausses** : serialport est **vert** ; linux-embedded-hal / rfd / notify-rust /
  minifb **compilent** (Iron les exprime) — leur blocage est l'**environnement** (matériel/écran/D-Bus au
  *run*), **pas le langage**. Donc moins de murs de langage que je ne l'avais dit.
- **Nouvel axe : ENVIRONNEMENT ≠ langage.** Plusieurs « murs » étaient des besoins runtime (device/display/
  serveur/libs système), pas des capacités V12. À ne **pas** mettre dans V12.

**Réponse à « les 80 passeront-ils avec V12 ? » → toujours NON garanti**, mais maintenant **mesuré** : les
capacités de langage restantes sont [24100]/[24300]/[40300]/[40400] (génériques, trait-externe, closures,
derive), [35000] (event-loop), [53000] (async), [43000] (no_std) — et **deux d'entre elles tensionnent avec
les invariants** : [35000] vs sémantique par valeur, [43000] vs I2. Celles-là pourraient **ne pas se
résoudre proprement** ; il faudra les **prototyper** pour trancher (difficile vs incompatible).
