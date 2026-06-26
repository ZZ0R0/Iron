# V8 — Réalisation (tracker)

> **État : cadrage + planification FAITS. Réalisation NON commencée** (attend le feu vert utilisateur — la
> transition planif→réalisation est une décision de l'utilisateur, CLAUDE.md). Ce fichier se remplit au fil
> de la phase 3 (avancement des pistes) et de la phase 4 (écarts/faiblesses → V9).

## Avancement des pistes

| Piste | Spec | Titre | Vague | Statut |
|---|---|---|---|---|
| 1.1 | 50900 | Fondation `.iri` v2 + `Bindings` IR | 1 | ✅ **fait** (split `bind/` non fait : réalisation séquentielle → `bind.rs` monolithique étendu — déviation assumée) |
| 1.2 | 51000 | `map_type` récursif | 1 | ✅ **fait** (scalaires larges, Vec/Option/Result/dict/set/tuple/str/Box/réf externe) |
| 1.3 | 21100·40200 | Squelette e2e minimal | 1 | ✅ **fait** (`use_extern_type.irn` : compile+run) |
| 2.1 | 51100 | Méthodes & constructeurs (`impl`) | 2 | ✅ **fait** (regex 8 types/75 méth., csv 27 types, serde_json 17 types) |
| 2.2 | 51200 | Émission auto des types externes | 2 | ✅ **fait** (fermeture du graphe : types référencés → opaque ; `.iri` auto self-contained) |
| 2.3 | 51300 | Reconnaisseurs d'idiomes | 2 | ✅ **builder/getter/collect** (regex 24 builders, 4 getters) ; **closure** = format+lowering OK, auto-détection rustdoc best-effort (résidu D6) |
| 2.4 | 51400 | Instance-pinning | 2 | ✅ **fait** (`serde_json::from_str::<Vec<i64>>` → e2e vert) |
| 2.5 | 51500 | Namespacing | 2 | ✅ **ctors/types namespacés** (`Regex.new()` résout les collisions) ; `extern mod` non fait (besoin servi par le namespacing de types) |
| 3.1 | 21100 | Inférence complète | 3 | ✅ **fait** (méthodes, ctors namespacés `T.ctor()`, propagation `Assign`) |
| 3.2 | 40200 | Lowering complet + idiomes | 3 | ✅ **fait** (méthode/ctor, glue clone/collect/builder, réinjection `&str`/collections, erreur crate→`IronError`, builders mutables) |
| 4.1 | 90300 | Corpus réel + **gate V8** | 4 | ✅ **fait** (`tests/binding/` : regex, regex_builder, serde_json, csv — **4/4 compile+run+sortie=oracle**) |

> **Gate corpus : VERT en continu** (45/45, 0 `unsafe`, 3 clones) — invariant dur respecté à chaque étape.
> **Gate binding (90300) : VERT** (4/4 vrais crates). Couverture auto-bind mesurée : **regex 92 %** (≥90 % ✅),
> **csv 83 %**, **serde_json 62 %** (le reste = génériques → couche B instance-pinning, prouvée).

## Mesure de référence (avant V8, à battre)

Binder actuel = **0 %** sur regex/csv/serde_json/syn (cross-check `ironc bind regex` = 0 `extern fn`).
Cible V8 (mesurée atteignable) : **≥ 90 %** de l'API publique des 3 crates du gate (auto + assisté + idiomes).
Résidu dur attendu : 0–6 % (regex 0, csv 1, serde_json 0, syn 12) → `rust:`/`.iri` curé.

## Écarts / faiblesses (relevés en phase 3-4)

- **Résultat clé mesuré : `ironc bind regex` = 0 → 76 items** bindés (8 types, 75 méthodes, 24 builders,
  4 getters, 6 sautés). Le bond 0%→réel est **prouvé**.
- **e2e réel prouvé** : programme Iron → `.iri` curé regex → cargo tire le **vrai crate regex** →
  `compile(p)` + `r.is_match(t)` en syntaxe Python → compile (0 `unsafe`) + s'exécute + sortie juste.
- **Collision de noms courts** (rustdoc-json, D5) : `regex::Regex` (str) et `regex::bytes::Regex` (bytes)
  partagent le nom court `Regex` → l'auto-bind les fusionne (API mélangée). Le ré-export public n'est pas
  résolvable simplement (rustdoc donne le chemin interne `regex::regex::string::Regex`). → heuristique
  `<crate>::<Type>` (ré-export racine) ; les sous-modules homonymes restent un trou. **Justifie l'`.iri`
  curé** pour le gate (hybride D5, prévu au cadrage).
- **Ctors = fns libres** (pas encore namespacés `T.new()`) → collision si 2 types ont `new`. → 2.5 (namespacing)
  + 3.1 (résolution `Type.ctor`) à faire pour l'auto-bind direct.
- **Graphe de types non fermé** (2.2 partiel) : un retour `Option[Match]` référence `Match` non auto-déclaré
  → `.iri` auto a des réfs pendantes. À compléter (émettre les types référencés en opaque).
- **Trou I11** : `return <méthode externe bool>` dans une fn `Result[bool]` non auto-emballé en `Ok` →
  contourné par `ok(...)` explicite. À corriger (étendre l'auto-wrap aux retours d'appels externes).
- **Limite try/except confirmée** : variable assignée dans un `try` (hoistée hors IIFE) → capture
  non-initialisée. Contourné en isolant le `?` dans une fn `Result`. (Dette pré-V8.)
- **Builders & mutabilité** : un builder `&mut self` exige `let mut` sur la variable ; non géré (la var n'est
  pas réassignée donc émise immuable). À traiter en 3.2.

## Verdict V8

**Tout le plan V8 est réalisé** (11 pistes : 9 ✅ pleines, 2 ✅ avec déviation assumée — split `bind/`
non nécessaire en séquentiel, `extern mod` servi par le namespacing de types). La chaîne complète tourne :
`ironc bind <crate>` auto-génère un `.iri` riche (types + POO + idiomes, self-contained), consommé par
`infer`/`lower`, et un programme Iron appelle **4 vrais crates** (regex, regex_builder, serde_json, csv) en
**syntaxe Python**, compilé en **Rust sûr (0 `unsafe`)**, sortie = oracle. Couverture auto mesurée 62–92 %
(reste = génériques → instance-pinning assistée, prouvée). **2 gates verts** (corpus 45/45 + binding 4/4).

Déviations honnêtes : `bind.rs` non éclaté en module `bind/` (inutile en réalisation séquentielle) ;
`extern mod` non implémenté (le namespacing par type couvre le besoin) ; auto-détection rustdoc des
closures best-effort (le format `.iri` et le lowering les supportent ; cas complexes → `rust{}`, D6).

## Reporté à V9 (déjà identifié au cadrage)

- **Registre communautaire de bindings** (`.iri` curés mutualisés, `iron add <crate>`, seed top-50).
- **`async`/`await`** + **threading** (`shared_sync` = `Arc<Mutex>`) — chantiers de **langage**, pas de binding.
- **Traits-protocoles auto** (D6 stretch : `Display`/`Iterator`-impl/opérateurs → méthodes Iron).
- Le **0–6 % dur** (itérateurs lazy/infinis, mutation-via-emprunt, frameworks à types associés, `dyn` plugin).
