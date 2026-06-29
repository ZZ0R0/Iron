# planify-1.1 — La garde : runner de corpus + format + intégration `cargo test`

- Réalise (cadrage) : [70400]
- Version : V18 · Vague : 1 · Piste : 1 · Statut : todo
- Dépend de : — (fondation ; protège tout V18)

## But
Poser l'**infrastructure** de la suite de correction : un runner qui, pour chaque cas `<nom>.irn` + attendu,
exécute `ironc` et compare, intégré à `cargo test` (→ `cargo test` re-joue le corpus). Plus un **noyau** de
cas écrits à la main (l'éventail vient en 1.2).

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/tests/corpus_runner.rs` (le runner + l'intégration `cargo test`).
- `ironc/tests/corpus/` (le dossier de cas — noyau initial ici, étendu en 1.2).

## Interface produite / consommée
**Produite (format figé)** : `<nom>.irn` + un fichier-attendu dont l'extension code le type — `.out` (stdout
exact), `.compile` (build réussit), `.err` (échec + diagnostic contient le texte), `.sha` (hash de sortie/fichier,
RNG seedé). Le runner = un `#[test]` paramétré qui balaie `tests/corpus/`.

## Étapes
1. Runner : lister `tests/corpus/*.irn` ; pour chacun, trouver l'attendu (par extension sœur) ; lancer
   `ironc run`/`build`/`check` (binaire release ou via la lib) ; comparer ; collecter pass/fail + **diff**.
2. Intégration `cargo test` : un test paramétré (ou un `#[test]` généré par cas) → échec si un cas régresse,
   avec le nom + le diff.
3. Déterminisme : binding `fastrand` **seedé** pour les cas RNG ; tri des sorties non-ordonnées (hashmap).
4. Noyau de cas : arith, collections, match, struct/enum, io-fichier, un crate simple — ~10 cas verts.

## Definition of Done (vérifiable)
- `cargo test` exécute le noyau et **passe**.
- Une **régression volontaire** (casser un cas) **fait échouer** `cargo test` avec le nom + le diff.
- Le format des 4 types de cas (`.out`/`.compile`/`.err`/`.sha`) fonctionne (un cas de chaque).

## Notes
- C'est la **fondation** : avant les autres axes. Versionné dans le dépôt (garde de référence).
- 1.2 (l'éventail) **étend** `tests/corpus/` — pas de collision (1.1 pose le runner, 1.2 ajoute des cas).
