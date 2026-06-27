# V11 — Réalisation (tracker)

> **État : réalisation EN COURS** (vagues 1-6.2 faites ; reste 6.3 + gate 7.1). 9 pistes · 7
> vagues · combler **tous** les trous de l'audit + finir le moteur. Gate final = comparaison-test utilisateur.

## Avancement des pistes

| Piste | Spec | Décisions | Titre | Vague | Statut |
|---|---|---|---|---|---|
| 1.1 | (toutes) | formes IR | Fondation : `Pattern::Or/Range` + traits de classe | 1 | ✅ fait |
| 2.1 | 11000 | DB-1·2·3 | Patterns : or-patterns + range patterns | 2 | ✅ fait |
| 3.1 | 11100 | DD-1·2·3·bonus | Surface : fn imbriquées · while/for-else · chaînée · const | 3 | ✅ fait |
| 4.1 | 23400 | DC-1·2·3 | Traits user : `class(Trait)` + bornes + méthodes par défaut | 4 | ✅ fait |
| 5.1 | 23500 | DC-4 | Surcharge d'opérateurs (`__dunder__` → `std::ops`) | 5 | ✅ fait |
| 6.1 | 23000 | DA-1·2 | Inférence : enum-construction + scrutateur + anti-unification | 6 | ✅ fait |
| 6.2 | 23200 | DA-4·5 | Flux Option/Result dans le moteur + retrait `flow.rs` + couverture | 6 | ✅ fait |
| 6.3 | 23100 | DA-3 | Diagnostics de type riches (champ/méthode/arité) | 6 ∥ | ✅ fait |
| **7.1** | 90900 | — | **Gate V11** — fixtures + comparaison-test + verdict | 7 | ☐ todo |

## Couverture des décisions (TOUT planifié)

DA-1→6.1 · DA-2→6.1 · DA-3→6.3 · DA-4→6.2 · DA-5→6.2 · DB-1→2.1 · DB-2→2.1 · DB-3→2.1(rustc) · DC-1→4.1 ·
DC-2→4.1 · DC-3→4.1 · DC-4→5.1 · DD-1→3.1 · DD-2→3.1 · DD-3→3.1 · DD-bonus→3.1 · hors-périmètre→7.1(doc).

## Invariant dur de réalisation

À chaque vague la colonne passe l'e2e (corpus 50 + binding 30 + vitrine + zero-annot). **Chaque feature
complète** (parse+lower+infer+fixture compile+run) avant de clore — **rien d'à moitié fini**. `flow.rs` retiré
(6.2) seulement prouvé couvert ; sinon mesuré + documenté.

## Hors-périmètre (justifié, noté)

État global **mutable** (I2 → `static mut`/`unsafe`) · dispatch dynamique (`dyn Trait`, I6) · exhaustivité
maison (rustc filet) · HM-global pur (le « 0.1 % »). Constantes globales **immuables** : dans le périmètre.

## Écarts / faiblesses (remplis en réalisation)

- **DA-5 — couverture moteur & reliquat `local.rs` (mesuré + documenté).** Le moteur résout **100 %**
  des **signatures** (params + retours) non annotées sur le corpus, le binding et les 7 fixtures zero-annot
  (mesure `ironc build --audit` → « couverture moteur d'inférence : N/N (100 %) »). **MAIS `local.rs` n'est
  PAS retiré** : il produit la `TypeMap` **par expression** (type de *chaque* nœud) que `lower` consomme,
  là où le moteur ne calcule que les **signatures**. Couches **complémentaires**, pas redondantes → retrait
  refusé (honnêteté DA-5, pas de retrait à l'aveugle). Retrait de `local.rs` = chantier futur (le moteur
  devrait émettre une `TypeMap` complète).
- **DA-4 — `flow.rs` retiré, mais pas « en contraintes ».** La classification de retour effectif (chute →
  `Option`, `?` → `Result`) est par **nature** une analyse de **flux de contrôle**, pas une contrainte
  d'unification : la coder en `Eq` serait artificiel. Choix honnête : la logique est **déplacée dans le
  moteur** (`infer/engine/flux.rs`, source unique du système de types) ; le fichier `flow.rs` est retiré.
- **Trou de langage corrigé (hors plan, vague 4).** `return self.champ` non-`Copy` levait `E0507` (move
  hors `&self`) → clone-fallback (I4). Vaut pour toute méthode renvoyant un champ par valeur.
- **Dunders : `__getitem__` différé.** Retour `&Output` (`Index`) incompatible avec la sémantique par
  valeur sans analyse de durée de vie. Les 7 autres dunders bénis livrés. Opérateurs **homogènes** (Rhs=Self)
  en v0 ; hétérogènes (`Vec2 * i64`) post-v0.
- **Traits : classe multi-base avec méthodes réparties = limite v0.** `class C(A, B)` met les méthodes dans
  le 1ᵉʳ trait ; les autres bases reçoivent un impl vide (OK si méthodes par défaut). `rustc` filet sinon.
- **DA-3 — borne de trait non signalée nativement.** Des 4 catégories du spec, 3 sont livrées (champ inconnu,
  méthode inconnue, arité) avec **zéro faux-positif** (n'agissent que sur un `Struct` Iron résolu). La 4ᵉ
  (« borne de trait non satisfaite ») est laissée au filet `rustc` : la trancher sans faux-positif exige une
  résolution de trait complète (risque > bénéfice) — choix conservateur assumé (priorité zéro-faux-positif).
- **DA-3 — diagnostics live dans VSCode.** Le LSP (`ironc lsp`) lance désormais les passes de types
  (best-effort, bindings vides) en plus de lex+parse → erreurs de type **soulignées live**. `ironc check`
  rend le span en `fichier:ligne:col:` (regex `checkOnSave`). Les deux chemins du checker bénéficient de DA-3.
