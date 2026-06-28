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
| [24100] | **Génériques de type** suivis (`Point<T>`, `Vec<T>`, tenseurs) + monomorphisation scalaire | 2 | geo, nalgebra, ndarray, polars, rustfft, dialoguer, rapier2d, **ML** (candle/linfa/tract/smartcore) | spec'é |
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

## À SURVEILLER pendant la passe des « 25 complets » (anticipé, non confirmé)

Les fiches complètes (pas les bouts) toucheront sans doute **plus** que les bouts. À confirmer/ajouter si ça
bloque vraiment :
- ~~**[83000] opérations fichier**~~ → **🔧 RÉSOLU (B32)** : `rename`/`copy_file`/`remove_file`/`create_dir`/
  `create_dir_all`/`remove_dir_all`/`path_exists` ajoutés **globalement** à `std.iri` (mêmes signatures que
  `read_file`/`write_file`). Le **renommeur #05 est VERT** (regex `captures` → `rename` → `path_exists`,
  vérifié sur disque). Le listage de répertoire passe déjà par **walkdir** (B30, vert). Restant mineur
  (non bloquant) : `std::fs::metadata` (taille/mtime) pour la dédup #03 — à ajouter si un projet le force.
- **[52600b] construction d'un `Value` dynamique** (et pas que le parser) : `serde_json::json!{}` /
  `Value::Object.insert` pour le **CSV→JSON** (#06) — **non bloquant en fait** : CSV→JSON ✅ construit le
  JSON par assemblage de chaînes (la voie `json!{}` reste une ergonomie, pas un mur).
- **dépaquetage d'un retour tuple** d'une fn externe (`(c, r) = size()`) — possiblement un **bug** réparable,
  pas une capacité ; à vérifier.

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
- **rusqlite #20 — partiel, bloqué par capacités existantes (pas un bug)** : `open_in_memory` +
  `execute_batch` (CREATE+INSERT) + `total_changes`/`last_insert_rowid` ✅ ; mais `execute(sql, params)` et
  `query_row(sql, params, |row| …)` sont **droppés au binding** (génériques `P: Params` = trait de crate
  → [24100b] ; closure `|row| row.get::<T>(0)` → [40300]/[24100]). La **lecture** de lignes est donc bloquée
  par **[24100]+[40300] déjà cadrés** — rien de neuf.

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

## Priorisation suggérée (rentabilité)

1. **[24100]** — débloque le plus de projets d'un coup (geo, nalgebra, ndarray, polars, rustfft + base ML) et
   conditionne [52900]/[24200]. **Le meilleur premier pas.**
2. **[52800]** io par flux — débloque flate2/printpdf/quick-xml, et complète [82000].
3. **[35000]** event-loop — gros, structurant (tout GUI/jeux/TUI), mais c'est un **changement de modèle**.
4. **[53000]** async, **[43000]** no_std — gros chantiers ciblés (web ; embarqué).

> **Méthode.** Chaque capacité = un cycle 4-phases à part entière. On en planifie **une** à la fois (commencer
> par [24100]). Tout nouveau mur **de même nature** s'ajoute ici, jamais en cas-par-cas.

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
