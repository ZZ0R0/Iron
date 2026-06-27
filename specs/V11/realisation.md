# V11 — Réalisation (tracker)

> **État : cadrage + planification FAITS. Réalisation NON commencée** (attend le feu vert). 9 pistes · 7
> vagues · combler **tous** les trous de l'audit + finir le moteur. Gate final = comparaison-test utilisateur.

## Avancement des pistes

| Piste | Spec | Décisions | Titre | Vague | Statut |
|---|---|---|---|---|---|
| 1.1 | (toutes) | formes IR | Fondation : `Pattern::Or/Range` + traits de classe | 1 | ☐ todo |
| 2.1 | 11000 | DB-1·2·3 | Patterns : or-patterns + range patterns | 2 | ☐ todo |
| 3.1 | 11100 | DD-1·2·3·bonus | Surface : fn imbriquées · while/for-else · chaînée · const | 3 | ☐ todo |
| 4.1 | 23400 | DC-1·2·3 | Traits user : `class(Trait)` + bornes + méthodes par défaut | 4 | ☐ todo |
| 5.1 | 23500 | DC-4 | Surcharge d'opérateurs (`__dunder__` → `std::ops`) | 5 | ☐ todo |
| 6.1 | 23000 | DA-1·2 | Inférence : enum-construction + scrutateur + anti-unification | 6 | ☐ todo |
| 6.2 | 23200 | DA-4·5 | Flux Option/Result en contraintes + retrait `flow.rs` + couverture | 6 | ☐ todo |
| 6.3 | 23100 | DA-3 | Diagnostics de type riches (champ/méthode/arité/borne) | 6 ∥ | ☐ todo |
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

## Écarts / faiblesses (à remplir en réalisation)

- (rien encore)
