# V12 — PLANIFICATION (phase 2) — Plan d'ensemble & fondation-refactor

> **But de la V12 (rappel, élargi sur directive)** : non pas « ajouter 14 features », mais **tarir la source
> des crate-bugs** — présents ET futurs. La passe des 80 a montré que les bugs viennent d'**un** endroit (la
> frontière de types, encodée 3×). Donc V12 = **un refactor-fondation** qui supprime la dette + **les capacités
> qui en dérivent**, plutôt que du cas-par-cas. Solutions **stables et pérennes** d'abord.

## §0 — Constat de dette (census de la frontière Iron↔Rust)

La correspondance Iron↔Rust est **réimplémentée 3 fois** (détails : `../24000.md`) :

| Couche | Fichier | Fonctions à cas-par-cas | Exemple de dette |
|---|---|---|---|
| Projection | `bind.rs` | `map_type`, `map_named`, `map_prim`, `bound_to_iron`, `try_expand_alias` | `Foo<T>` → `Extern("Foo")` (jette `T`) |
| Inférence | `infer/local.rs`, `infer/mod.rs` | `builtin_method_ret`, `extern_iter_elem`, `extern_method_ret` | type d'élément d'itérateur (B30) |
| Coercition | `lower.rs` | `extern_arg_to`, `coerce_to`, `coerce_sized_binop`, `num_kind_str`, `try_builtin_method`, `try_*_method` | `i64`→`u8`/`usize` éparpillé (B10/B22/B26/B29) |

**10 tables « match-sur-nom »** recensées (numérique, conteneurs, bornes-traits, octets, méthodes built-in,
mots-clés…) — autant de points où *projection* et *coercition* peuvent **diverger** silencieusement.

## §1 — La grande solution : le pont (`[24000]`)

Un module **pont** = **source unique** de la correspondance, 2 faces (projection / coercition) + inférence,
dérivées des **mêmes tables de données**. Levier décisif : **suivre les paramètres de type** → [24100]
devient l'expression directe de la fondation, ET le problème des **bornes de trait-crate** disparaît pour le
cas dominant (constructeur scalaire-générique : on **instancie par l'argument** au lieu de **whitelister** la
borne). Voir `../24000.md` pour l'architecture et la **décision d'ABI** (`Type::ExternApp`, additive).

## §2 — Carte des VAGUES de TOUTE la V12

> Règle de lecture : **vague = rang séquentiel** (toute la vague N avant la N+1) ; **piste = parallèle**
> (ownership disjoint) au sein d'une vague. Le squelette/fondation EST la vague 1.

### VAGUE 1 — Fondation `[24000]` (séquentiel, squelette)
Fige le **schéma** : module pont + `Type::ExternApp` + `.iri` `Name[args]` + `ExternType.params`, câblés en
**pass-through vert** (comportement quasi inchangé, e2e passe). C'est le contrat gelé. → `24000.1.md`.

### VAGUE 2 — Remplissage de la fondation = **[24100] génériques** (fan-out, ownership par fichier)
| Piste | Fichier possédé | Contenu | Unité |
|---|---|---|---|
| 2.A | `bind.rs` | projection produit les args génériques ; tables pilotées-donnée (R3) | `24000.2.md` |
| 2.B | `lower.rs` | coercition unifiée (R1 numérique) ; monomorphisation `path::<args>` | `24000.3.md` |
| 2.C | `infer/` | substitution des params (`Point[f64]`.méthode rendant `T` → `f64`) | `24000.4.md` |

### VAGUE 3 — Unification + validation [24100] (séquentiel)
Greffe, e2e **réel** sur geo/nalgebra/ndarray/polars/rustfft + rusqlite-lecture (FromSql). Gate VERT. → `24000.4.md` (§unif).

### VAGUE 4 — Capacités **dérivées** de la fondation (fan-out, petites)
Chacune ≈ **une entrée de donnée** ou un petit mécanisme sur le pont déjà principié :
| Spec | Capacité | Dérive de | Taille |
|---|---|---|---|
| [24100b] | bornes de ctor convertibles (`Into<X>`/`TryFrom`) | table bornes→type (R3) | petite |
| [82000] | ergonomie octets (`bytes`, hex) | table octets + ExternApp | petite |
| [24200] | tableaux taille fixe `[u8; N]`/`GenericArray` | ExternApp + const-arg | moyenne |
| [50500] | constantes publiques (`pub const`) | binder (orthogonal, trivial) | triviale |
| [30200] | retour emprunté lié à l'entrée (`<'a>`) | face coercition (durées de vie) | moyenne |

### VAGUE 5+ — **Clusters séparés** (autres fondations ; NE dérivent PAS du pont)
Chacun est un cycle 4-phases distinct, planifié **à son tour** (cf. §4 prototype) :
| Spec | Cluster | Nature | Tension invariants |
|---|---|---|---|
| [52800] | io par flux (Write/Read) | modèle puits/source | — |
| [40300] | closures→trait-objets | modèle HOF/closures | — |
| [40400] | derive-macros | émission de macros | — |
| [24300] | impl trait externe sur type Iron | émission d'impl | — |
| [52900] | serde typé | dépend de [40400]+[24100] | — |
| [35000] | event-loop & état mutable partagé | **modèle d'exécution** | **vs sémantique par valeur** |
| [53000] | runtime async | exécuteur tokio | — (lourd) |
| [43000] | émission no_std + HAL | mode d'émission | **vs I2 (unsafe)** |

## §3 — Graphe de dépendances (qui débloque quoi)

```
[24000] FONDATION (pont + ExternApp + numérique unifié + tables-donnée)
   └─► [24100] génériques  ──► [24100b] bornes ctor
                           ──► [24200] arrays fixes ──► (aes-gcm, chacha20)
                           ──► [82000] octets/hex
                           ──► [52900] serde typé (AUSSI besoin de [40400])
   [50500] consts        (orthogonal, se branche n'importe quand)
   [30200] retours empruntés (face coercition ; après vague 1)

CLUSTERS INDÉPENDANTS (fondations propres) :
[52800] io-flux · [40300] closures→dyn · [40400] derive · [24300] impl-extern
[35000] event-loop  ·  [53000] async  ·  [43000] no_std   ← à PROTOTYPER (cf. §4)
```

## §4 — Plannable maintenant vs à PROTOTYPER (honnêteté)

On ne planifie **atomiquement** que ce dont la faisabilité est connue. Sinon on planifie le **prototype**.

- **Plannable atomiquement maintenant** : `[24000]` (ci-dessous, unités 24000.1–.4) et, par héritage, `[24100]`.
  Les dérivées vague 4 ([24100b]/[82000]/[24200]/[50500]/[30200]) se planifieront en **début de leur cycle**,
  une fois la fondation verte (elles sont petites et bien cernées).
- **À prototyper AVANT planif atomique** (faisabilité ou compatibilité-invariants incertaine) :
  - **[35000] event-loop** — tension **frontale** avec la sémantique par valeur (pas d'état global mutable).
    Prototype requis : peut-on exprimer `loop { handle(&mut state) }` sans violer le modèle ? **Difficile vs incompatible** : à trancher par l'essai.
  - **[43000] no_std** — tension avec **I2** (le HAL encapsule de l'`unsafe`). Prototype : mode d'émission `no_std` + crate-HAL bénie, sans qu'Iron **émette** d'`unsafe`.
  - **[53000] async** — pas de tension d'invariant mais **lourd** (exécuteur) : prototype d'intégration tokio + codegen `await`.
  - **[24300] impl-trait-externe** — Iron *parse* `impl App for MyApp` mais émet un Rust faux : prototype du **mapping chemins/signatures** avant de planifier en grand.
- **Clusters plannables sans prototype mais hors-fondation** : [52800], [40300], [40400], [52900] — cycles 4-phases classiques, à lancer après la fondation+[24100].

## §5 — Séquencement recommandé & gates

1. **Vague 1 [24000] squelette** → gate VERT (pass-through). **Gate de fondation** : `Type::ExternApp` traverse
   binder→`.iri`→infer→lower sur un cas trivial, e2e vert.
2. **Vagues 2–3 [24100]** → gate VERT + **geo/nalgebra construits et calculent**. **Gate [24100]**.
3. **Vague 4** (dérivées) → chacune ajoute **une entrée**, gate VERT à chaque greffe.
4. **Clusters** (vague 5+) un par un, dans l'ordre de rentabilité ([52800] d'abord — débloque flate2/quick-xml/gpx).
5. **Prototypes** ([35000]/[43000]/[53000]/[24300]) **avant** d'en faire des cycles complets ; le prototype **tranche** difficile vs incompatible.

> **Invariant méthodo** : à aucun moment la colonne ne cesse de **passer l'e2e**. Refactor = squelette
> pass-through d'abord, fan-out par fichier (ownership disjoint), unification, e2e réel — jamais de big-bang.

## §6 — Ce que ce plan garantit (et ce qu'il ne garantit pas)

- **Garantit** : la dette des crate-bugs se **tarit à la source** — un motif nouveau = **une entrée de donnée**
  consultée par les 3 couches, plus un correctif éparpillé. Génériques suivis ⇒ la classe entière
  « scalaire-générique » (la plus grosse des 80) se débloque d'un coup, et les bornes de trait-crate cessent
  d'exiger une whitelist.
- **Ne garantit pas** : que [35000]/[43000] se résolvent **proprement** — ils **tensionnent** avec les
  invariants ; le **prototype** dira difficile vs incompatible. On ne les a pas cachés : ils sont cadrés et
  **marqués à prototyper**, pas planifiés à l'aveugle.

## Unités atomiques de cette vague (fondation)
- `24000.1.md` — VAGUE 1 squelette : module pont + `Type::ExternApp` + `.iri` `Name[args]` + `ExternType.params` (contrat gelé).
- `24000.2.md` — VAGUE 2 piste A (`bind.rs`) : projection des args + tables pilotées-donnée.
- `24000.3.md` — VAGUE 2 piste B (`lower.rs`) : numérique unifié + monomorphisation.
- `24000.4.md` — VAGUE 2 piste C (`infer/`) + VAGUE 3 unification & validation [24100].
