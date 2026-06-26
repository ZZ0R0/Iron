# Batterie de tests V7 — 30 projets réels Python ↔ Iron

> **But (phase 4, boucle de vérité).** Pour chaque projet : (1) écrire une version **Python pure** (`.py`),
> la **lancer** (oracle) ; (2) écrire le **clone Iron** (`.irn`, multi-fichiers) ; (3) **compiler vers Rust**
> (`ironc run`) ; (4) **diff** des deux sorties → doit être **identique**. Chaque échec = bug Iron à corriger.
>
> **Échelle.** Vrais projets, **100–200 lignes**, **plusieurs fichiers** par projet (pas des scripts).

## Règles de comparabilité (figées — sinon faux négatifs)

Python et Rust **n'impriment pas pareil**. Donc **toute sortie comparée est déterministe et stable** :
- ✅ **Entiers**, **chaînes explicites**, listes/tuples **d'entiers**, valeurs calculées via f-string.
- ❌ **Interdits en sortie brute** : flottants (Python `1.0` vs Rust `1`), booléens (`True`/`true`),
  listes de **chaînes** (`'a'` vs `"a"`), **dicts** (ordre + quotes). → les reformuler :
  - booléen → `"yes"/"no"` ou `1/0` ; flottant → `int(x * 100)` (centièmes) ou arrondi entier ;
  - collection → **trier puis itérer** et imprimer chaque élément formaté ; dict → `for k in sorted(keys): print(f"{k}={d[k]}")`.
- **Déterminisme** : pas de `random`, pas de `time`, pas d'ordre d'itération de dict non trié.

## Structure d'un projet

```
tests/projects/NN_nom/
  python/   main.py + modules .py        (oracle)
  iron/     main.irn + modules .irn      (import)
  expected.txt                            (sortie de référence, générée par python)
```
Harness : `python3 python/main.py > out_py` ; `ironc run iron/main.irn > out_iron` ; `diff out_py out_iron`.

## Contraintes Iron à respecter dans les clones (limites connues V7)

Args nommés **uniquement** sur construction/`fn` tout-nommé (pas méthode/mixte) → **positionnel** ailleurs.
Pas de `return <valeur>` dans `try` → variable-résultat. HOF param `i64→i64`. Pas d'héritage, générateurs,
décorateurs, `*args`, regex, JSON. Float : éviter en sortie. `import module` pour le multi-fichiers.

---

## Les 30 projets

### A — Structures de données & algorithmes (1–6)

**01. linked_list** — Liste chaînée (via `shared`/`ref`) : push/pop/length/sum/reverse, recherche.
Fichiers : `node.irn` (struct Node) · `list.irn` (opérations) · `main.irn`. Sortie : longueurs, sommes, éléments.

**02. graph_search** — Graphe (dict d'adjacence) : BFS, DFS, composantes connexes, plus court chemin (non pondéré).
Fichiers : `graph.irn` · `search.irn` · `main.irn`. Sortie : ordres de visite, distances (entiers).

**03. sorting_suite** — bubble/insertion/merge/quick + **comptage de comparaisons** comme métrique.
Fichiers : `sorts.irn` · `main.irn`. Sortie : listes triées (entiers) + nb de comparaisons.

**04. maze_solver** — Labyrinthe (grille d'entiers) : BFS plus court chemin, longueur, atteignabilité.
Fichiers : `grid.irn` · `solver.irn` · `main.irn`. Sortie : longueur du chemin, nb de cases visitées.

**05. game_of_life** — Jeu de la vie : grille, évolution N générations, comptage de vivants.
Fichiers : `board.irn` · `rules.irn` · `main.irn`. Sortie : population par génération, rendu ASCII (`#`/`.`).

**06. bst** — Arbre binaire de recherche : insert, in-order, hauteur, min/max, contains, count.
Fichiers : `tree.irn` · `main.irn`. Sortie : parcours in-order (entiers), hauteur, présence (yes/no).

### B — Gestion / simulation (7–14)

**07. bank** — Comptes, dépôts/retraits, virements, relevé, intérêts (entiers/centimes).
Fichiers : `account.irn` · `bank.irn` · `main.irn`. Sortie : soldes, historique formaté.

**08. inventory** — Stock : produits, entrées/sorties, ruptures, valorisation, top-N par quantité.
Fichiers : `product.irn` · `store.irn` · `main.irn`. Sortie : quantités, alertes, classement trié.

**09. library** — Bibliothèque : livres, membres, emprunts/retours, retards, dispo.
Fichiers : `models.irn` · `catalog.irn` · `main.irn`. Sortie : états d'emprunt, listes triées.

**10. shopping_cart** — Panier : articles, remises (par seuil/quantité), taxe, total (centimes entiers).
Fichiers : `item.irn` · `cart.irn` · `pricing.irn` · `main.irn`. Sortie : sous-totaux, total.

**11. payroll** — Paie : tranches d'imposition, déductions, net, récap par employé.
Fichiers : `employee.irn` · `tax.irn` · `main.irn`. Sortie : montants (centimes), totaux.

**12. tournament** — Bracket à élimination : seeds, rounds, gagnants, scores.
Fichiers : `bracket.irn` · `main.irn`. Sortie : gagnants par round, champion.

**13. grades** — Notes étudiants : moyennes (entières), mentions, classement, stats par cours.
Fichiers : `student.irn` · `gradebook.irn` · `main.irn`. Sortie : moyennes, rangs, mentions.

**14. hotel** — Réservations : chambres, plages de nuits, conflits, taux d'occupation (entier %).
Fichiers : `room.irn` · `booking.irn` · `main.irn`. Sortie : dispo, occupation %, conflits.

### C — Texte & parsing (15–20)

**15. word_stats** — Statistiques texte : fréquences, top-N mots, longueur moyenne (×100 entier), uniques.
Fichiers : `tokenizer.irn` · `stats.irn` · `main.irn`. Sortie : compte trié, top-N.

**16. rpn_calc** — Calculatrice polonaise inverse : tokenize, évalue (entiers), pile, variables.
Fichiers : `lexer.irn` · `eval.irn` · `main.irn`. Sortie : résultats d'expressions.

**17. expr_eval** — Évaluateur infixe → RPN (shunting-yard) → calcul (entiers, `+ - * / ( )`).
Fichiers : `tokens.irn` · `parser.irn` · `eval.irn` · `main.irn`. Sortie : valeurs d'expressions.

**18. roman** — Chiffres romains ↔ entiers (bidirectionnel), validation.
Fichiers : `roman.irn` · `main.irn`. Sortie : conversions, drapeaux validité (yes/no).

**19. markdown** — Mini-Markdown → HTML : titres `#`, listes `-`, gras `**`, paragraphes.
Fichiers : `parser.irn` · `render.irn` · `main.irn`. Sortie : lignes HTML.

**20. csv_processor** — « CSV » (split) : parse, filtre, group-by, agrégats (somme/moyenne entière).
Fichiers : `parse.irn` · `query.irn` · `main.irn`. Sortie : agrégats triés.

### D — Maths & numérique (21–25)

**21. matrix** — Matrices (listes de listes) : add, mul, transpose, trace, identité, puissance.
Fichiers : `matrix.irn` · `main.irn`. Sortie : matrices (entiers) ligne par ligne.

**22. polynomial** — Polynômes (coeffs entiers) : éval (Horner), add, mul, dérivée.
Fichiers : `poly.irn` · `main.irn`. Sortie : valeurs évaluées, coeffs résultants.

**23. number_theory** — Crible d'Ératosthène, factorisation, gcd/lcm, parfaits, Collatz (longueur).
Fichiers : `primes.irn` · `theory.irn` · `main.irn`. Sortie : listes de premiers, facteurs, longueurs.

**24. calendar** — Logique de date : bissextiles, jours/mois, jour de semaine (Zeller), diff de jours.
Fichiers : `date.irn` · `calc.irn` · `main.irn`. Sortie : jours, indices de semaine.

**25. statistics** — Stats descriptives (entiers/centièmes) : moyenne, médiane, mode, variance, percentiles.
Fichiers : `stats.irn` · `main.irn`. Sortie : valeurs (×100 pour les non-entières).

### E — Jeux & machines à états (26–30)

**26. tic_tac_toe** — Morpion : plateau, détection victoire/nul, IA (minimax simple).
Fichiers : `board.irn` · `ai.irn` · `main.irn`. Sortie : déroulé de partie, résultat.

**27. vending_machine** — Distributeur (machine à états, `enum`/`match`) : insertion, sélection, rendu monnaie.
Fichiers : `state.irn` · `machine.irn` · `main.irn`. Sortie : transitions, monnaie rendue.

**28. mini_vm** — VM à pile : opcodes (`enum`), programme, exécution, registres.
Fichiers : `op.irn` · `vm.irn` · `main.irn`. Sortie : valeurs en sortie du programme.

**29. edit_distance** — Distance de Levenshtein (DP), correcteur (plus proche d'un dictionnaire).
Fichiers : `distance.irn` · `spell.irn` · `main.irn`. Sortie : distances, suggestions.

**30. huffman** — Codage de Huffman simplifié : fréquences, arbre, longueurs de code, taille compressée.
Fichiers : `tree.irn` · `coder.irn` · `main.irn`. Sortie : longueurs de code par symbole, taille totale.

---

## Plan de réalisation

1. **Harnais** : `tests/run_battery.sh` (python → out, ironc → out, diff, rapport pass/fail).
2. **Validation** : 2–3 projets d'abord (valider le diff + la portabilité de sortie), corriger le compilateur si besoin.
3. **Fan-out** : les 30 projets sont **indépendants** → réalisables en parallèle (agents) ou en série.
4. **Boucle de vérité** : chaque diff ≠ vide → bug Iron → corrigé au compilateur (central) → re-run.
5. **Rapport final** : N/30 identiques, bugs trouvés+corrigés, limites restantes.
