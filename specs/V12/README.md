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
| [24300] | **Implémenter un trait EXTERNE** sur un type Iron (`impl eframe::App for MyApp`) | 2 | GUI (eframe/iced), tout framework à trait-utilisateur | **découvert tardivement** |
| [40300] | **Closures rendant un trait-objet** (`Box<dyn FnOnce -> Box<dyn App>>`), HOF avancées | 4 | GUI, callbacks de crates | **découvert tardivement** |
| [35000] | **Boucle d'événements & état mutable partagé** | 3 | **GUI** (26-30), **TUI complet** (31-32), **jeux** (46-50) | **cadré (nouveau)** |
| [53000] | **Runtime async** (`tokio`/`async fn`/`.await` exécutés) | 5 | axum, teloxide, redis, tungstenite, hickory, rumqttc, web (16/19) | **cadré (nouveau)** |
| [43000] | **Émission `no_std` + HAL encapsulé** (embarqué, sans violer I2) | 4 | esp-hal (ESP32), rppal, serialport, btleplug, linux-embedded-hal | **cadré (nouveau)** |
| [30200] | **Retour emprunté lié à l'entrée** (parseurs zéro-copie, `<'a>`) | 3 | midly, plotters(backend) | cadré |
| [24200] | **Tableaux à taille fixe** (const-génériques, `[u8; 32]`/`GenericArray`) | 2 | aes-gcm, chacha20poly1305 | cadré |
| [82000] | **Ergonomie des octets** (`bytes`, type d'élément d'itérateur externe `u8`, hex) | 8 | sha2(hex), PoW(80) | partiel (encode/decode faits B27) |
| [50500] | **Constantes** publiques d'un crate (`pub const`) | 5 | base64 | gap mineur |

> Hors-langage (environnement, pas une spec) : `rodio`/`cpal`/`minifb` exigent des **libs système / une
> fenêtre** au build/run ; le ML est aussi **lourd** (dépendances). Ils relèvent de [24100]/[35000] pour le
> *langage*, mais ne tourneront de toute façon que dans un environnement équipé.

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

## ⚠️ Ce cadrage est une HYPOTHÈSE, pas une garantie

**Honnêteté requise.** Je n'ai **exécuté que ~45 des 80** ; les ~35 autres sont classés par **architecture**,
pas par essai. Et chaque capacité ci-dessus est **draft** : la *construire* est une vraie recherche qui
**révélera des sous-problèmes**. Preuve : en sondant eframe au-delà du binding, j'ai découvert **[24300]
impl-trait-externe** et **[40300] closures→trait-objets** que je n'avais **pas** vus — un seul projet GUI a
fait apparaître **2 capacités manquantes**.

Donc, à la question « avec tout V12, les 80 passeront-ils ? » → **NON, je ne peux pas le garantir** :
- Les ~35 non-essayés cachent probablement **d'autres** capacités (comme eframe en a caché 2).
- Certaines capacités **tensionnent avec les invariants d'Iron** et pourraient **ne pas se résoudre
  proprement** : [35000] (état mutable partagé) vs la sémantique **par valeur** ; [43000] (`no_std`) vs **I2
  (jamais d'`unsafe`)**. Ce sont des **arbitrages d'identité du langage**, pas de simples features.

**Pour le SAVOIR vraiment**, il faudrait une *passe de validation* : essayer (au moins à la compilation)
chaque projet non-testé pour en extraire les capacités réelles, puis prototyper la plus risquée ([35000])
pour vérifier qu'elle est seulement *difficile* et pas *incompatible*. Tant que ce n'est pas fait, ce
cadrage est la **meilleure carte connue**, à corriger au fil des essais — pas une promesse.
