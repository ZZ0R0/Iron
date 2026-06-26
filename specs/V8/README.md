# Iron — V8 / cadrage (`README.md`)

> **Version « écosystème »** : on fait passer Iron de « langage réel à stdlib bindée » à **langage qui
> réutilise nativement l'écosystème Rust**. Objectif unique : **l'auto-binding des crates** — types natifs,
> POO (`impl`/méthodes/constructeurs), génériques, **et** reconnaissance des idiomes Rust — pour que
> `ironc bind <crate>` produise une interface Iron **style Python, utilisable**, sans armée de devs.
> Parent : `specs/generated.md` (V8 ajoutée). Contrat de contenu : `../generator.md`.
> Statut : **🔶 cadrage** (architecture + décisions structurantes posées ; specs détaillées + planif ci-jointes).
>
> V8 **réalise et dépasse** les drafts V7 jamais implémentés (`50500` bind complet, `50700` génériques/traits) :
> elle ajoute ce qui leur manquait — **la POO (méthodes) et les reconnaisseurs d'idiomes**, mesurés comme
> le vrai levier. `async`/`await`, threading, LSP, registre communautaire de bindings = **V9+** (cf. hors-V8).

## Pourquoi cette version (la preuve chiffrée)

On a **mesuré** le binder actuel sur 5 vrais crates (rustdoc-json → classification A/B/C). Faits, pas intuitions :

| crate | binder actuel | + types récursifs & méthodes | + instance-pinning | + idiomes | **shim humain irréductible** |
|---|---:|---:|---:|---:|---:|
| regex | **0 %** | 59 % | 65 % | **100 %** | 0 fn |
| csv | **0 %** | 36 % | 56 % | **99 %** | 1 fn |
| serde_json | **0 %** | 55 % | 79 % | **100 %** | 0 fn |
| proc_macro2 | **0 %** | 98 % | 100 % | **100 %** | 0 fn |
| syn | **0 %** | 56 % | 76 % | **94 %** | 12 fn |

Trois lectures qui **fondent** V8 :
1. **Le binder actuel = 0 %** sur tout vrai crate (cross-check `ironc bind regex` = 0 `extern fn`). Il ne mappe que `i64/f64/bool` et ignore **toutes** les méthodes.
2. **La valeur est dans la POO** : ~99 % du gain « types récursifs + méthodes » sont des **méthodes** (l'`impl` des crates). C'est le cœur de V8.
3. **Le résidu « dur » est minuscule** : le « C » qu'on croyait shim-humain est à **73–100 % des idiomes mécaniques** (builder/getter/iterator/closure). Le *vraiment* irréductible = **0–6 %**. Donc **pas d'armée** : on apprend **~4 idiomes au compilateur, une fois**, et ça vaut pour **tous** les crates.

## La stratégie (3 leviers compilateur + 1 levier écosystème)

> Coût de l'écosystème = **O(idiomes) + O(top-N libs partagées)**, jamais O(devs × libs).

| Levier | Quoi | Effet mesuré | Où (V8 ?) |
|---|---|---|---|
| **1. Auto-binder** | types récursifs + méthodes/ctors + émission des types | 0 % → ~56 % | **V8** (51000/51100/51200) |
| **2. Instance-pinning** | monomorphiser un générique à la demande (1 ligne `.iri`) | → ~75 % | **V8** (51400) |
| **3. Reconnaisseurs d'idiomes** | builder→mute, getter→clone, iter→collect, closure→lambda | → **~95 %** | **V8** (51300 + 40200) |
| 4. Registre partagé | `.iri` curés mutualisés (modèle @types/nimble), `iron add` | l'irréductible 0–6 % | **V9** (hors-V8) |

L'ironie féconde, à exploiter : **la sémantique-valeur d'Iron (I1) — celle qui rendait le binding brut
impossible — est exactement ce qui rend la normalisation des idiomes triviale.** Un getter `-> &T` devient
`-> T` par `.clone()` : *c'est Iron qui fait du Iron* (I1 + warning perf I4).

## Carte des specs (10 specs, 1 chantier : l'auto-binding)

### Fondation — format & projection (interop `5`)
| Spec | Contenu |
|---|---|
| `50900` | **Format `.iri` v2 (gelé)** : types externes à méthodes/ctors, types riches, **tags d'idiomes**, modules ; `Bindings` IR ; pipeline `emit` modulaire (`bind/`) |
| `51000` | **`map_type` récursif** : scalaires larges, `Vec`→`list`/`Option`/`Result`/`HashMap`→`dict`/`HashSet`→`set`/tuple/`str`/`String`/`Box`, **réfs de types externes** (la convention de projection partagée) |

### POO & types (interop `5`)
| Spec | Contenu |
|---|---|
| `51100` | **Extraction POO** : méthodes (`self`) + constructeurs (assoc. fns) depuis les `impl` inhérents → API du type externe |
| `51200` | **Émission auto des types externes** : tout struct/enum référencé par une signature bindée → `extern type` opaque, **fermeture transitive** du graphe |

### Génériques, idiomes, modules (interop `5`)
| Spec | Contenu |
|---|---|
| `51300` | **Reconnaisseurs d'idiomes** : détecte builder/getter/iterator/closure → signature Iron normalisée + tag |
| `51400` | **Instance-pinning** : `bind --instantiate "from_str::<Point>"` / `.iri` curé → monomorphise un générique (couche assistée) |
| `51500` | **Namespacing** : `extern mod` + chemins de module → `regex.escape(...)` côté Iron |

### Consommateurs — rendre le `.iri` utilisable (inférence `2`, lowering `4`)
| Spec | Contenu |
|---|---|
| `21100` | **Inférence des appels externes** : types-handles, types de retour des méthodes/ctors, instances génériques connues du `TypeMap` |
| `40200` | **Lowering des appels externes + glue des idiomes** : appel de méthode, `[clone]`→`.clone()`, `[collect]`→`.collect()`, `[builder]`→mute+jette, `[closure]`→closure Rust (I8, I10, I2) |

### Validation (méta `9`)
| Spec | Contenu |
|---|---|
| `90300` | **Corpus de binding réel** : `bind` de `regex`/`serde_json`/`csv` + programmes Iron qui les utilisent (POO) → compile + s'exécute + sortie juste. **Gate V8.** |

## Décisions structurantes (à FIGER en phase 1)

> Tout ce qui est **partagé entre `bind` (producteur) et `infer`/`lower` (consommateurs)** se gèle ici,
> sinon deux pistes le comblent différemment en phase 3 (CLAUDE.md). Design = `✅` ; fond = `🔶`.

- **✅ D0 — Fondation `.iri` v2 (1ʳᵉ vague, gelée).** Le **format de binding** + le **`Bindings` IR** sont LE
  contrat producteur↔consommateurs. Gelés avant tout fan-out (comme D0 de V6/V7). Repr IR :
  `ExternType { iron_name, rust_path, ctors, methods }`,
  `ExternFn { iron_name, params: Vec<(String,Type)>, ret: Type, rust_target, recv: Option<Recv>, idioms: Vec<Idiom> }`,
  `Recv = ByValue|ByRef|ByRefMut`, `Idiom = Builder|Clone|Collect|Closure(idx)`.
- **✅ D1 — Projection récursive figée (la convention partagée).** `map_type` projette un **ensemble fermé**
  de constructeurs : `Vec→list`, `Option/Result` idem, `HashMap/BTreeMap→dict`, `HashSet/BTreeSet→set`,
  `Box/Rc/Arc→intérieur`, `(A,B)→tuple`, `&str/String→str`, scalaires larges (I6), réf de type externe →
  son `iron_name`. **Mapping exact**, jamais de widening (un `i32`↦`i64` ferait un binding faux). Non
  mappable → **sauté + compté** (D5, dégradation propre). Geler ce mapping = anti-collision n°1.
- **✅ D2 — Type externe = handle opaque + API.** Un type externe est une **valeur Rust possédée** qu'Iron
  détient et sur laquelle il appelle des méthodes ; **pas d'accès aux champs**. Le graphe se ferme en
  émettant un `extern type` opaque pour **chaque** struct/enum référencé. Respecte I6 (projection propre),
  I5 (pas de runtime : valeur possédée, pas de GC).
- **✅ D3 — Idiomes : détectés au `bind`, glue au `lower`.** Le `.iri` reste **déclaratif** (signature Iron
  normalisée + tag). Les 4 tags et leur glue : `[builder]` (recv `ByRefMut`, retour `unit`, on jette le
  `&mut Self`) · `[clone]` (retour emprunté `&T` → `.clone()`/`.cloned()`) · `[collect]` (retour itérateur →
  `.collect::<Vec<_>>()` → `list[T]`) · `[closure]` (param `Fn`-borné → lambda Iron). Aucun runtime nouveau (I5).
- **✅ D4 — Clone-au-retour pour les getters (cohérent I1/I4).** Un getter `-> &T` est projeté `-> T` possédé
  via clone. **C'est la sémantique-valeur d'Iron**, pas un hack ; le clone émet le warning perf `--audit` (I4).
  Tradeoff assumé, pas caché.
- **✅ D5 — Instance-pinning = couche assistée, PAS automatique.** Les génériques ne s'auto-bindent pas. Une
  directive curée (`.iri` ou `bind --instantiate`) épingle une **instance concrète** (`from_str::<Point>`).
  Honnête : la couche B coûte **1 ligne humaine**, pas zéro. (Prolonge D5 de V7 : hybride assumé.)
- **🔶 D6 — Périmètre des idiomes = 4 reconnaisseurs.** builder/getter/iterator/closure (les 73–100 % mesurés
  du résidu). **Hors V8** (le 0–6 % dur) : itérateurs lazy/infinis, mutation-via-emprunt, frameworks à types
  associés, `dyn Trait` en plugin → `rust:` (niveau 1) ou `.iri` curé. *Stretch possible (à arbitrer) :
  traits-protocoles std `Display`/`Iterator`-impl/opérateurs → méthodes Iron mécaniques. Proposé reporté V9.*

## Priorité / ordre des chantiers (vagues séquentielles)

1. **Fondation D0/D1** (`50900` format + `51000` projection) + **squelette e2e** : un `.iri` écrit à la main
   (1 type externe + 1 méthode + idiome `clone`) qu'un `.irn` utilise → compile + s'exécute. Fige le contrat.
2. **Producteurs `bind`** (fan-out parallèle, fichiers `bind/*` disjoints) : méthodes (`51100`), émission des
   types (`51200`), idiomes (`51300`), instance-pinning (`51400`), modules (`51500`).
3. **Consommateurs** (unification séquentielle, surfaces partagées `infer`/`lower`) : inférence complète
   (`21100`), lowering complet + glue idiomes (`40200`). On re-valide l'e2e **réel**.
4. **Gate V8** : corpus de binding réel (`90300`).

## e2e / gate V8

Au-delà du corpus existant : **`ironc bind regex` (et `serde_json`, `csv`) produit un `.iri` riche**
(types + méthodes + idiomes) ; un **programme Iron multi-fichiers** construit un `Regex`, appelle
`is_match`/`find`/`replace_all` en **style Python**, compile + s'exécute, **sortie identique** à l'oracle ;
couverture mesurée ≥ **90 %** de l'API publique des 3 crates (auto + assisté + idiomes) ; **0 `unsafe`**
(I2) ; clones de getters comptés par `--audit` (I4) ; remontée I8 préservée. Faiblesses → `realisation.md` (et V9).

## Hors V8 (→ V9+, honnête)

**Registre communautaire de bindings** (`.iri` curés mutualisés, `iron add <crate>`, seed du top-50 par
l'équipe cœur) ; **`async`/`await`** et **threading** (`shared_sync` = `Arc<Mutex>`) = chantiers de
**langage**, pas de binding ; traits-protocoles auto (D6 stretch) ; LSP / auto-hébergement ;
le **0–6 % dur** restera du `rust:` ou du `.iri` curé — prix irréductible, minuscule, assumé.

## Note

Specs au gabarit `generator §4`. Planification (vagues/pistes) : `planification/`. Le format `.iri` v2 (D0)
et la projection (D1) **se figent avant tout fan-out** — c'est la fondation, au sens strict de la méthodo.
