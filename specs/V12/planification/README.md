# V12 — PLANIFICATION (phase 2) — attaque des **3 causes racines**

> **Découpage d'EXÉCUTION** (vagues × pistes). Cadrage (le *quoi*) dans `../*.md`. Nommage
> `planify-<vague>.<piste>.md` (vague = rang séquentiel, piste = track parallèle à ownership disjoint ; **PAS**
> un n° de spec — chaque planif **cite** sa/ses spec(s)). Cadrage ≠ planification.

## Le principe (corrigé) : 3 causes racines, pas 14 capacités

Les ~40 projets encore bloqués après une V12 « génériques seuls » ne sont **pas** 14 problèmes indépendants —
ils remontent à **3 causes** (analyse complète : `../24000.md`, `../44000.md`) :

- **RACINE 1 — `[24000]` Le pont de matérialisation** : un paramètre générique borné par un trait que le binder
  ne sait pas matérialiser. UNE table « borne → stratégie » : numérique→argument, `Into<str>`→chaîne,
  `Read`→Cursor/File, `Write`→buffer, `Fn`→lambda, `Serialize`→struct. **Couvre [24100]/[24100b]/[52800]/[82000]/
  [24200]/[40300] + le param de [52900].** La dominante : débloque la plus grosse part des 40.
- **RACINE 2 — `[44000]` La couche d'émission** : émettre un item/attribut qu'Iron ne produit pas encore.
  `#[derive(X)]` ([40400]→[52900]), `impl ExternTrait for T` ([24300]), `#![no_std]` ([43000]).
- **RACINE 3 — Runtime/exécution** : `[53000]` async (tokio déjà injecté ; reste binder+await), `[35000]`
  event-loop (= **callback racine 1 + impl-trait racine 2 + état partagé `shared`/`shared_sync` qui existe déjà**).

> **Deux « incompatibilités » que j'avais annoncées TOMBENT** : [43000] no_std ne viole pas I2 (l'`unsafe` est
> dans le HAL, Iron en émet 0) ; [35000] event-loop ne viole pas la sémantique-valeur (Iron a `shared`/
> `shared_sync`/`inout`). Ce sont des chantiers, pas des murs.

## La SEULE limite vraiment permanente (et ce n'est PAS un bug)
**L'ENVIRONNEMENT** : rodio/cpal (libs audio système), minifb (fenêtre), esp/rppal/btleplug (matériel),
serveur/réseau au run. Ces projets **compilent** (Iron les exprime) mais exigent hardware/libs/écran **à
l'exécution**. **Aucune** architecture de langage ne répare ça (~5-8 projets). Ce n'est pas une dette V12.
+ **friction résiduelle** sur les cas les plus durs (Write-stream continu, event-loop à état complexe, traits
async lourds type `tower::Service`) : premier jet **partiel**, à compléter au besoin.

## Carte des vagues × pistes (toute la V12, 3 racines)

| Planif | Vague | Réalise | Racine | Rôle | Possède |
|---|:--:|---|:--:|---|---|
| `planify-1.1` | 1 | [24000] | 1 | **Squelette du pont** : table `STRATEGIES` + `ExternApp` + `.iri` (contrat gelé, pass-through vert) | bridge.rs(création), ir.rs, lower.rs(ExternApp), iri.rs, bind.rs(émission) |
| `planify-2.1` | 2 | [24000][24100] | 1 | Projection : stratégie scalaire + tables-donnée | **bind.rs** |
| `planify-2.2` | 2 | [24000][24100] | 1 | Coercition : numérique unifié + monomorphisation | **lower.rs** |
| `planify-2.3` | 2 | [24000][24100] | 1 | Inférence : substitution des params | **infer/** |
| `planify-3.1` | 3 | [24100] | 1 | Unification + e2e réel (geo/nalgebra) → **Gate [24100]** | jonctions + tests |
| `planify-4.1` | 4 | [24100b][82000] | 1 | Stratégies « valeur » : `Into<str>`/bornes convertibles + octets/hex | bridge.rs, bind.rs, lower.rs |
| `planify-4.2` | 4* | [24200] | 1 | Stratégie « taille fixe » `[u8;N]`/`GenericArray` | bridge.rs, lower.rs, ir.rs |
| `planify-5.1` | 5 | [52800] | 1 | Stratégie **io** : `Read`→Cursor/File, `Write`→buffer rendu | bridge.rs, lower.rs |
| `planify-6.1` | 6 | [40300] | 1 | Stratégie **closure** : `Fn`-borné → lambda Iron | bridge.rs, lower.rs, infer/ |
| `planify-7.1` | 7 | [50500] | — | Constantes publiques (`pub const`), orthogonale | bind.rs, iri.rs, infer/, lower.rs |
| `planify-8.1` | 8 | [40400] | 2 | Émission de **derives** (`#[derive(X)]`) | lower.rs, ir.rs, bind.rs |
| `planify-8.2` | 8* | [52900] | 1+2 | serde **typé** = param `Deserialize` (r1) + derive émis (r2) | lower.rs, infer/ |
| `planify-9.1` | 9 | [24300] | 2 | Émission **`impl ExternTrait for T`** (signatures via bindings) | lower.rs, bind.rs |
| `planify-10.1` | 10 | [43000] | 2 | Mode d'émission **`#![no_std]`** + HAL béni (0 `unsafe` côté Iron) | lower.rs, driver.rs |
| `planify-11.1` | 11 | [53000] | 3 | **async** : binder+await les fns async (tokio déjà injecté) — ⚠️ **gate prototype** | infer/, lower.rs |
| `planify-12.1` | 12 | [35000] | 3 | **event-loop** = callback(6)+impl-trait(9)+`shared` — ⚠️ **gate prototype** | lower.rs (intégration) |

`*` = piste parallèle de la même vague **si** ownership disjoint ; sinon séquentielle.

## Gates de prototype (honnêteté : on ne planifie pas l'aveugle)
Avant `planify-11.1`/`12.1`, un **prototype** tranche la faisabilité **fine** :
- **async** : intégrer une fn async d'une vraie crate (ex. `reqwest::get`) + `.await` → tourne ? Les traits lourds
  (`tower::Service`, `axum::Handler`) peuvent rester partiels — à mesurer, pas à promettre.
- **event-loop** : exprimer `eframe::run(|ctx| { … })` capturant un `shared[State]` muté chaque frame → compile
  **et** affiche (dans un env avec écran). Si l'ergonomie de l'état partagé est trop lourde → noter, itérer.
Le prototype **tranche** ; il ne se contourne pas.

## Ordre & invariant
1. Vagues 1-3 (RACINE 1 cœur) → Gate [24100]. 2. Vagues 4-6 (stratégies r1, une entrée chacune). 3. Vague 7
(consts). 4. Vagues 8-10 (RACINE 2 émission). 5. Vagues 11-12 (RACINE 3, après prototype). **La colonne passe
l'e2e à chaque greffe** (skeleton pass-through → fan-out par fichier → unification). `.iri` du gate commités ⇒ filet permanent.

## Ce que V12 garantit (honnête)
- **Tarit les bugs racine** : un motif de crate = **une entrée** (stratégie ou émission), plus jamais 3 correctifs dispersés.
- Débloque **la grande majorité** des 40 (math, io, closures, serde, GUI-à-trait, embarqué, web simple).
- **Ne résout PAS** : l'environnement (hardware/libs/écran — hors langage, ~5-8 projets) ni la friction des cas
  async/event-loop les plus lourds (premier jet partiel). Et un refactor de cette taille porte un **risque de
  régression** maîtrisé par le squelette-d'abord + gate-vert-continu.
